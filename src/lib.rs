use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt::Write as _;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::ptr;

use windows::core::PCWSTR;
pub use windows::core::{Error, Result};
use windows::Win32::Devices::Display::{
    DestroyPhysicalMonitor, GetNumberOfPhysicalMonitorsFromHMONITOR,
    GetPhysicalMonitorsFromHMONITOR, GetVCPFeatureAndVCPFeatureReply, SetVCPFeature,
    PHYSICAL_MONITOR,
};
use windows::Win32::Foundation::{BOOL, HANDLE, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, DISPLAY_DEVICEW,
    DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER, HDC, HMONITOR,
    MONITORINFOEXW,
};

#[derive(Debug)]
pub enum Interface {
    DDCCI(HANDLE),
    WMI,
}

#[derive(Debug)]
pub struct Monitor {
    pub id: OsString,
    pub interface: Interface,
}

impl Drop for Monitor {
    fn drop(&mut self) {
        if let Interface::DDCCI(handle) = self.interface {
            unsafe { DestroyPhysicalMonitor(handle) }.unwrap();
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
                interface: Interface::DDCCI(phy.hPhysicalMonitor),
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
        match self.interface {
            Interface::DDCCI(handle) => get_vcp(handle, feature.vcp_code()),
            Interface::WMI => unimplemented!(),
        }
    }

    pub fn set_feature(&self, feature: Feature, value: u32) -> Result<()> {
        match self.interface {
            Interface::DDCCI(handle) => set_vcp(handle, feature.vcp_code(), value),
            Interface::WMI => unimplemented!(),
        }
    }
}
