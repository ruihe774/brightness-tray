use std::collections::BTreeMap;
use std::ffi::{c_void, OsString};
use std::fmt::Write as _;
use std::mem;
use std::num::NonZeroUsize;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr;

use once_cell::race::OnceNonZeroUsize;
use wide::L;
pub use windows::core::{Error, Result};
use windows::core::{Interface, BSTR, PCWSTR};
use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitor, GetNumberOfPhysicalMonitorsFromHMONITOR,
    GetPhysicalMonitorsFromHMONITOR, GetVCPFeatureAndVCPFeatureReply, SetVCPFeature,
    PHYSICAL_MONITOR,
};
use windows::Win32::Foundation::{BOOL, HANDLE, LPARAM, RECT, RPC_E_TOO_LATE, S_FALSE};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, DISPLAY_DEVICEW,
    DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER, HDC, HMONITOR,
    MONITORINFOEXW,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CLSCTX_INPROC_SERVER,
    COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_DEFAULT, RPC_C_IMP_LEVEL_IMPERSONATE,
};
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayUnaccessData,
};
use windows::Win32::System::Variant::{VariantClear, VARIANT, VT_ARRAY};
use windows::Win32::System::Wmi::{
    IWbemClassObject, IWbemLocator, IWbemServices, WbemLocator, WBEM_FLAG_CONNECT_USE_MAX_WAIT,
    WBEM_FLAG_FORWARD_ONLY,
};

#[derive(Debug)]
pub struct Monitor {
    pub id: OsString,
    pub handle: HANDLE,
}

impl Drop for Monitor {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe { DestroyPhysicalMonitor(self.handle) }.unwrap();
        }
    }
}

fn pcwstr_to_osstring(s: &[u16]) -> OsString {
    let len = s.iter().position(|ch| *ch == 0).unwrap_or(s.len());
    OsString::from_wide(&s[..len])
}

fn get_monitor_ids() -> BTreeMap<String, OsString> {
    fn get_display_device(
        device_name: Option<&[u16]>,
        device_num: u32,
        flags: u32,
    ) -> Option<DISPLAY_DEVICEW> {
        let mut device: mem::MaybeUninit<DISPLAY_DEVICEW> = mem::MaybeUninit::uninit();
        unsafe { device.assume_init_mut() }.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;
        unsafe {
            EnumDisplayDevicesW(
                PCWSTR::from_raw(device_name.map_or(ptr::null(), |name| name.as_ptr())),
                device_num,
                device.as_mut_ptr(),
                flags,
            )
        }
        .as_bool()
        .then_some(unsafe { device.assume_init() })
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
                let Ok(name) = pcwstr_to_osstring(&monitor.DeviceName).into_string() else {
                    // XXX: we cannot handle this
                    continue;
                };
                let id = pcwstr_to_osstring(&monitor.DeviceID);
                monitor_ids.insert(name, id);
            }
        }
    }

    monitor_ids
}

fn get_monitors_ddcci(monitors: &mut Vec<Monitor>, monitor_ids: &mut BTreeMap<String, OsString>) {
    fn get_monitor_info(hmonitor: HMONITOR) -> Option<MONITORINFOEXW> {
        let mut info: mem::MaybeUninit<MONITORINFOEXW> = mem::MaybeUninit::uninit();
        unsafe { info.assume_init_mut() }.monitorInfo.cbSize =
            mem::size_of::<MONITORINFOEXW>() as u32;
        unsafe { GetMonitorInfoW(hmonitor, mem::transmute(info.as_mut_ptr())) }
            .as_bool()
            .then_some(unsafe { info.assume_init() })
    }

    fn get_physical_monitors_from_hmonitor(hmonitor: HMONITOR) -> Vec<PHYSICAL_MONITOR> {
        let mut num = 0;
        let _ = unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(hmonitor, &mut num) };
        if num == 0 {
            Vec::new()
        } else {
            let num = num as usize;
            let mut v = Vec::with_capacity(num);
            unsafe { v.set_len(num) };
            unsafe { GetPhysicalMonitorsFromHMONITOR(hmonitor, v.as_mut_slice()) }
                .map_or(Vec::new(), |()| v)
        }
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
        let Ok(mut name) = pcwstr_to_osstring(&info.szDevice).into_string() else {
            // XXX: we cannot handle this
            continue;
        };
        let physical_monitors = get_physical_monitors_from_hmonitor(hmonitor);
        for (i, phy) in physical_monitors.into_iter().enumerate() {
            let orig_len = name.len();
            write!(name, "\\Monitor{i}").unwrap();
            let id = monitor_ids.remove(&name);
            name.truncate(orig_len);
            let Some(id) = id else {
                debug_assert!(false, "the device of '{name}\\Monitor{i}' not found");
                continue;
            };
            monitors.push(Monitor {
                id,
                handle: phy.hPhysicalMonitor,
            });
        }
    }
}

fn get_monitors_wmi(_monitors: &mut Vec<Monitor>, monitor_ids: &mut BTreeMap<String, OsString>) {
    debug_assert!(monitor_ids.is_empty(), "WMI support is not implemented yet");
}

pub fn get_monitors() -> Vec<Monitor> {
    let mut monitors = Vec::new();
    let mut monitor_ids = get_monitor_ids();
    get_monitors_ddcci(&mut monitors, &mut monitor_ids);
    get_monitors_wmi(&mut monitors, &mut monitor_ids);
    debug_assert!(
        monitor_ids.is_empty(),
        "cannot get interfaces for some display devices: {monitor_ids:?}"
    );
    monitors
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Reply {
    current: u32,
    maximum: u32,
}

fn get_vcp(handle: HANDLE, code: u8) -> Result<Reply> {
    let mut reply = Reply {
        current: 0,
        maximum: 0,
    };
    if unsafe {
        GetVCPFeatureAndVCPFeatureReply(
            handle,
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

fn set_vcp(handle: HANDLE, code: u8, value: u32) -> Result<()> {
    if unsafe { SetVCPFeature(handle, code, value) } == 0 {
        Err(Error::from_win32())
    } else {
        Ok(())
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
    pub fn get_feature(&self, feature: Feature) -> Result<Reply> {
        if self.handle.is_invalid() {
            unimplemented!()
        }
        get_vcp(self.handle, feature.vcp_code())
    }

    pub fn set_feature(&self, feature: Feature, value: u32) -> Result<()> {
        if self.handle.is_invalid() {
            unimplemented!()
        }
        set_vcp(self.handle, feature.vcp_code(), value)
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

    fn get_wmi_instance(&self, class: &str) -> Result<Option<IWbemClassObject>> {
        let query = format!("SELECT * FROM {class} WHERE InstanceName=\"");
        let mut query: Vec<_> = query.encode_utf16().collect();
        let instance_name = self.get_wmi_instance_name();
        query.extend(instance_name.into_iter().flat_map(|ch| match ch {
            L!('\\') => [ch, ch].into_iter().take(2),
            L!('"') => [L!('\\'), ch].into_iter().take(2),
            ch => [ch, 0].into_iter().take(1),
        }));
        query.push(L!('"'));
        query_wmi(&query)
    }

    fn get_user_friendly_name_inner(&self) -> Result<Option<OsString>> {
        let Some(instance) = self.get_wmi_instance("WmiMonitorID")? else {
            return Ok(None);
        };
        let mut variant: mem::MaybeUninit<VARIANT> = mem::MaybeUninit::uninit();
        let user_friendly_name = PCWSTR::from_raw(L!("UserFriendlyName").as_ptr());
        unsafe { instance.Get(user_friendly_name, 0, variant.as_mut_ptr(), None, None) }?;
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
                    .take_while(|ch| **ch != 0)
                    .map(|ch| *ch as u16)
                    .collect();
                let s = OsString::from_wide(&buf);
                unsafe { SafeArrayUnaccessData(array) }?;
                Ok(s)
            })
            .transpose();
        unsafe { VariantClear(&mut variant) }?;
        s
    }

    pub fn get_user_friendly_name(&self) -> Option<OsString> {
        self.get_user_friendly_name_inner().ok().flatten()
    }
}

fn init_com() -> Result<()> {
    match unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) } {
        Ok(()) => (),
        Err(e) if e.code() == S_FALSE => (),
        Err(e) => return Err(e),
    }
    match unsafe {
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
    } {
        Ok(()) => Ok(()),
        Err(e) if e.code() == RPC_E_TOO_LATE => Ok(()),
        Err(e) => Err(e),
    }
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
            init_com()?;
            let services = create_wmi_services()?;
            let ptr = services.into_raw() as usize;
            Ok(NonZeroUsize::try_from(ptr).unwrap())
        })?
        .get() as *mut c_void;
    Ok((*mem::ManuallyDrop::new(unsafe { IWbemServices::from_raw(services) })).clone())
}

fn query_wmi(query: &[u16]) -> Result<Option<IWbemClassObject>> {
    let services = get_wmi_services()?;
    let wql = BSTR::from_wide(&L!("WQL"))?;
    let query = BSTR::from_wide(query)?;
    let enumerator = unsafe { services.ExecQuery(&wql, &query, WBEM_FLAG_FORWARD_ONLY, None) }?;
    let mut objects = [None; 1];
    let mut returned = 0;
    let _ = unsafe { enumerator.Next(1000, &mut objects, &mut returned) };
    Ok(mem::take(&mut objects[0]))
}
