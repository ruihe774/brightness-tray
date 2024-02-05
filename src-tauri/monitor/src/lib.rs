#![allow(clippy::uninit_vec)]

use std::collections::BTreeMap;
use std::ffi::{c_void, OsString};
use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::mem::{size_of, take, transmute, MaybeUninit};
use std::num::NonZeroUsize;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::os::windows::io::IntoRawHandle;
use std::ptr;

use once_cell::race::OnceNonZeroUsize;
use wide::L;
use windows::core::Interface as _;
pub use windows::core::{Error, Result};
use windows::core::{BSTR, PCWSTR};
use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitor, GetNumberOfPhysicalMonitorsFromHMONITOR,
    GetPhysicalMonitorsFromHMONITOR, GetVCPFeatureAndVCPFeatureReply, SetVCPFeature,
    DISPLAYPOLICY_AC, DISPLAYPOLICY_DC, DISPLAY_BRIGHTNESS, IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS,
    IOCTL_VIDEO_QUERY_SUPPORTED_BRIGHTNESS, IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS, PHYSICAL_MONITOR,
};
use windows::Win32::Foundation::{CloseHandle, BOOL, ERROR_NOT_SUPPORTED, HANDLE, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, DISPLAY_DEVICEW,
    DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER, HDC, HMONITOR,
    MONITORINFOEXW,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayUnaccessData,
};
use windows::Win32::System::Variant::{VariantClear, VARIANT, VT_ARRAY};
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator, WBEM_FLAG_CONNECT_USE_MAX_WAIT,
    WBEM_FLAG_FORWARD_ONLY,
};
use windows::Win32::System::IO::DeviceIoControl;

#[derive(Debug)]
pub struct Monitor {
    pub id: OsString,
    hphysical: HANDLE,
    hdevice: HANDLE,
}

impl Drop for Monitor {
    fn drop(&mut self) {
        if self.hphysical.0 != -1 {
            unsafe { DestroyPhysicalMonitor(self.hphysical) }.unwrap();
        }
        if self.hdevice.0 != -1 {
            unsafe { CloseHandle(self.hdevice) }.unwrap();
        }
    }
}

fn pcwstr_to_osstring(s: &[u16]) -> OsString {
    let len = s.iter().position(|&ch| ch == 0).unwrap_or(s.len());
    OsString::from_wide(&s[..len])
}

fn get_monitor_ids() -> BTreeMap<OsString, OsString> {
    fn get_display_device(
        device_name: Option<&[u16]>,
        device_num: u32,
        flags: u32,
    ) -> Option<DISPLAY_DEVICEW> {
        let mut device = MaybeUninit::<DISPLAY_DEVICEW>::uninit();
        unsafe { device.assume_init_mut() }.cb = size_of::<DISPLAY_DEVICEW>() as u32;
        unsafe {
            EnumDisplayDevicesW(
                PCWSTR::from_raw(device_name.map_or(ptr::null(), |name| name.as_ptr())),
                device_num,
                device.as_mut_ptr(),
                flags,
            )
        }
        .as_bool()
        .then(|| unsafe { device.assume_init() })
    }

    let mut monitor_ids = BTreeMap::new();
    for adapter_idx in 0.. {
        let Some(adapter) = get_display_device(None, adapter_idx, 0) else {
            break;
        };
        for monitor_idx in 0.. {
            let Some(monitor) = get_display_device(
                Some(&adapter.DeviceName),
                monitor_idx,
                /* EDD_GET_DEVICE_INTERFACE_NAME */ 1,
            ) else {
                break;
            };
            if (monitor.StateFlags
                & (DISPLAY_DEVICE_ATTACHED_TO_DESKTOP | DISPLAY_DEVICE_MIRRORING_DRIVER))
                == DISPLAY_DEVICE_ATTACHED_TO_DESKTOP
            {
                let name = pcwstr_to_osstring(&monitor.DeviceName);
                let id = pcwstr_to_osstring(&monitor.DeviceID);
                monitor_ids.insert(name, id);
            }
        }
    }

    monitor_ids
}

fn get_monitors_gdi(monitors: &mut Vec<Monitor>, monitor_ids: &mut BTreeMap<OsString, OsString>) {
    fn get_monitor_info(hmonitor: HMONITOR) -> Option<MONITORINFOEXW> {
        let mut info = MaybeUninit::<MONITORINFOEXW>::uninit();
        unsafe { info.assume_init_mut() }.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        unsafe { GetMonitorInfoW(hmonitor, transmute(info.as_mut_ptr())) }
            .as_bool()
            .then(|| unsafe { info.assume_init() })
    }

    fn get_physical_monitors_from_hmonitor(hmonitor: HMONITOR) -> Vec<PHYSICAL_MONITOR> {
        let mut num = 0;
        let _ = unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut num) };
        (num != 0)
            .then(|| {
                let num = num as usize;
                let mut v = Vec::with_capacity(num);
                unsafe { v.set_len(num) };
                unsafe { GetPhysicalMonitorsFromHMONITOR(hmonitor, v.as_mut_slice()) }.map(|()| v)
            })
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default()
    }

    unsafe extern "system" fn monitor_enum_proc(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let handles = &mut *(lparam.0 as *mut Vec<HMONITOR>);
        handles.push(hmonitor);
        true.into()
    }

    let mut handles = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut handles as *mut Vec<HMONITOR> as isize),
        )
    };

    for hmonitor in handles {
        let Some(info) = get_monitor_info(hmonitor) else {
            continue;
        };
        let display_name = pcwstr_to_osstring(&info.szDevice);
        let physical_monitors = get_physical_monitors_from_hmonitor(hmonitor);
        for (i, phy) in physical_monitors.into_iter().enumerate() {
            let mut monitor_name = display_name.clone();
            write!(monitor_name, "\\Monitor{i}").unwrap();
            let id = monitor_ids.remove(&monitor_name);
            let Some(id) = id else {
                debug_assert!(
                    false,
                    "the device of '{}' not found",
                    monitor_name.to_string_lossy()
                );
                continue;
            };
            let hphysical = phy.hPhysicalMonitor;
            let hdevice = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&id)
                .map_or_else(
                    |e| {
                        debug_assert!(false, "failed to open device file handle: {e:?}");
                        HANDLE(-1)
                    },
                    |f| HANDLE(f.into_raw_handle() as isize),
                );
            monitors.push(Monitor {
                id,
                hphysical,
                hdevice,
            });
        }
    }
}

pub fn get_monitors() -> Vec<Monitor> {
    let mut monitors = Vec::new();
    let mut monitor_ids = get_monitor_ids();
    get_monitors_gdi(&mut monitors, &mut monitor_ids);
    debug_assert!(
        monitor_ids.is_empty(),
        "cannot get interfaces for some display devices: {monitor_ids:?}"
    );
    monitors
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Interface {
    DDCCI,
    WMI,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Reply {
    pub current: u32,
    pub maximum: u32,
    pub source: Interface,
}

fn ddcci_get_vcp(hphysical: HANDLE, code: u8) -> Result<Reply> {
    let mut reply = Reply {
        current: 0,
        maximum: 0,
        source: Interface::DDCCI,
    };
    if unsafe {
        GetVCPFeatureAndVCPFeatureReply(
            hphysical,
            code,
            None,
            &mut reply.current,
            Some(&mut reply.maximum),
        )
    } == 0
    {
        Err(Error::from_win32())
    } else {
        Ok(reply)
    }
}

fn ddcci_set_vcp(hpysical: HANDLE, code: u8, value: u32) -> Result<()> {
    if unsafe { SetVCPFeature(hpysical, code, value) } == 0 {
        Err(Error::from_win32())
    } else {
        Ok(())
    }
}

// ioctl functions are copied from the "brightness" crate

fn ioctl_query_supported_brightness(hdevice: HANDLE) -> Result<Vec<u8>> {
    let mut bytes_returned = 0;
    let mut out_buffer = Vec::<u8>::with_capacity(256);
    unsafe {
        DeviceIoControl(
            hdevice,
            IOCTL_VIDEO_QUERY_SUPPORTED_BRIGHTNESS,
            None,
            0,
            Some(out_buffer.as_mut_ptr() as *mut c_void),
            out_buffer.capacity() as u32,
            Some(&mut bytes_returned),
            None,
        )
    }?;
    unsafe { out_buffer.set_len(bytes_returned as usize) };
    Ok(out_buffer)
}

fn ioctl_query_display_brightness(hdevice: HANDLE) -> Result<u8> {
    let mut display_brightness = MaybeUninit::<DISPLAY_BRIGHTNESS>::uninit();
    unsafe {
        DeviceIoControl(
            hdevice,
            IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS,
            None,
            0,
            Some(display_brightness.as_mut_ptr() as *mut c_void),
            size_of::<DISPLAY_BRIGHTNESS>() as u32,
            None,
            None,
        )
    }?;
    let display_brightness = unsafe { display_brightness.assume_init() };
    Ok(match display_brightness.ucDisplayPolicy as u32 {
        DISPLAYPOLICY_AC => display_brightness.ucACBrightness,
        DISPLAYPOLICY_DC => display_brightness.ucDCBrightness,
        _ => unreachable!(),
    })
}

fn ioctl_set_display_brightness(hdevice: HANDLE, value: u8) -> Result<()> {
    let mut display_brightness = DISPLAY_BRIGHTNESS {
        ucACBrightness: value,
        ucDCBrightness: value,
        ucDisplayPolicy: 3, // DISPLAYPOLICY_BOTH
    };
    unsafe {
        DeviceIoControl(
            hdevice,
            IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS,
            Some(&mut display_brightness as *mut DISPLAY_BRIGHTNESS as *mut c_void),
            size_of::<DISPLAY_BRIGHTNESS>() as u32,
            None,
            0,
            None,
            None,
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Feature {
    Luminance,
    Contrast,
    Brightness,
    Volume,
    PowerState,
}

impl Feature {
    fn vcp_code(&self) -> u8 {
        match self {
            Feature::Luminance => 0x10,
            Feature::Contrast => 0x12,
            Feature::Brightness => 0x13,
            Feature::Volume => 0x62,
            Feature::PowerState => 0xD6,
        }
    }
}

impl Monitor {
    fn is_builtin(&self) -> bool {
        self.id.as_encoded_bytes().starts_with(b"\\\\?\\LCD")
    }

    pub fn get_feature(&self, feature: Feature) -> Result<Reply> {
        if self.is_builtin() {
            if feature == Feature::Luminance {
                ioctl_query_display_brightness(self.hdevice).map(|value| Reply {
                    current: value as u32,
                    maximum: 100,
                    source: Interface::WMI,
                })
            } else {
                Err(ERROR_NOT_SUPPORTED.into())
            }
        } else {
            ddcci_get_vcp(self.hphysical, feature.vcp_code())
        }
    }

    pub fn set_feature(&self, feature: Feature, value: u32) -> Result<()> {
        if self.is_builtin() {
            if feature == Feature::Luminance {
                ioctl_query_supported_brightness(self.hdevice).and_then(|levels| {
                    let value = levels
                        .iter()
                        .min_by_key(|&level| value.abs_diff(*level as u32))
                        .copied()
                        .unwrap_or(value as u8);
                    ioctl_set_display_brightness(self.hdevice, value)
                })
            } else {
                Err(ERROR_NOT_SUPPORTED.into())
            }
        } else {
            ddcci_set_vcp(self.hphysical, feature.vcp_code(), value)
        }
    }
}

impl Monitor {
    fn get_wmi_instance_name(&self) -> Vec<u16> {
        let mut id: Vec<u16> = self.id.encode_wide().collect();
        if id.strip_prefix(&L!("\\\\?\\")).is_some() {
            id.drain(..4);
        } else {
            debug_assert!(false, "id not starts with '\\\\?\\'");
        }
        let mut last_hash = 0;
        for (i, ch) in id.iter_mut().enumerate() {
            if *ch == L!('#') {
                *ch = L!('\\');
                last_hash = i;
            }
        }
        debug_assert_ne!(last_hash, 0);
        debug_assert_eq!(id.len() - last_hash, 39);
        id.truncate(last_hash);
        id.push(L!('_'));
        id.push(L!('0'));
        id
    }

    fn get_wmi_instance(&self, class: &[u16]) -> Result<Option<IWbemClassObject>> {
        let mut query = Vec::from(L!("SELECT * FROM "));
        query.extend_from_slice(class);
        query.extend(L!(" WHERE InstanceName=\""));
        let instance_name = self.get_wmi_instance_name();
        query.extend(instance_name.into_iter().flat_map(|ch| match ch {
            L!('\\') => [ch, ch].into_iter().take(2),
            L!('"') => [L!('\\'), ch].into_iter().take(2),
            ch => [ch, 0].into_iter().take(1),
        }));
        query.push(L!('"'));
        query_wmi(&query)
    }

    pub fn get_user_friendly_name(&self) -> Result<Option<OsString>> {
        let Some(instance) = self.get_wmi_instance(&L!("WmiMonitorID"))? else {
            return Ok(None);
        };
        let mut variant = MaybeUninit::<VARIANT>::uninit();
        unsafe {
            instance.Get(
                PCWSTR::from_raw(L!("UserFriendlyName\0").as_ptr()),
                0,
                variant.as_mut_ptr(),
                None,
                None,
            )
        }?;
        let mut variant = unsafe { variant.assume_init() };
        let s = ((unsafe { &variant.Anonymous.Anonymous }.vt.0 & VT_ARRAY.0) != 0)
            .then(|| {
                let array = unsafe { variant.Anonymous.Anonymous.Anonymous.parray };
                let mut data = ptr::null_mut();
                let l = unsafe { SafeArrayGetLBound(array, 1) }?;
                let r = unsafe { SafeArrayGetUBound(array, 1) }?;
                unsafe { SafeArrayAccessData(array, &mut data) }?;
                let buf =
                    &unsafe { std::slice::from_raw_parts(data as *const u32, r as usize + 1) }
                        [l as usize..];
                let buf: Vec<_> = buf
                    .iter()
                    .take_while(|&ch| *ch != 0)
                    .map(|&ch| ch as u16)
                    .collect();
                let s = OsString::from_wide(&buf);
                unsafe { SafeArrayUnaccessData(array) }?;
                Ok(s)
            })
            .transpose();
        unsafe { VariantClear(&mut variant) }?;
        s
    }
}

#[doc(hidden)]
pub fn init_com() -> Result<()> {
    use windows::Win32::System::Com::{
        CoInitializeSecurity, EOAC_NONE, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE,
    };
    use windows::Win32::System::Ole::OleInitialize;
    unsafe { OleInitialize(None) }?;
    unsafe {
        CoInitializeSecurity(
            None,
            -1,
            None,
            None,
            RPC_C_AUTHN_LEVEL_DEFAULT,
            RPC_C_IMP_LEVEL_IMPERSONATE,
            None,
            EOAC_NONE,
            None,
        )
    }?;
    Ok(())
}

static WMI_SERVICES: OnceNonZeroUsize = OnceNonZeroUsize::new();

fn create_wmi_services() -> Result<IWbemServices> {
    let locator: IWbemLocator =
        unsafe { CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER) }?;
    let resource = BSTR::from_wide(&L!("root\\WMI"))?;
    unsafe {
        locator.ConnectServer(
            &resource,
            None,
            None,
            None,
            WBEM_FLAG_CONNECT_USE_MAX_WAIT.0,
            None,
            None,
        )
    }
}

fn get_wmi_services() -> Result<IWbemServices> {
    let services = WMI_SERVICES
        .get_or_try_init(|| -> Result<NonZeroUsize> {
            let services = create_wmi_services()?;
            let ptr = services.into_raw() as usize;
            Ok(NonZeroUsize::try_from(ptr).unwrap())
        })?
        .get() as *mut c_void;
    Ok(unsafe { IWbemServices::from_raw_borrowed(&services) }
        .unwrap()
        .clone())
}

fn query_wmi(query: &[u16]) -> Result<Option<IWbemClassObject>> {
    let services = get_wmi_services()?;
    let enumerator = unsafe {
        services.ExecQuery(
            &BSTR::from_wide(&L!("WQL"))?,
            &BSTR::from_wide(query)?,
            WBEM_FLAG_FORWARD_ONLY,
            None,
        )
    }?;
    let mut objects = [None; 1];
    let mut returned = 0;
    let _ = unsafe { enumerator.Next(1000, &mut objects, &mut returned) };
    Ok(take(&mut objects[0]))
}
