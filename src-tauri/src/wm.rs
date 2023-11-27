use std::ffi::c_void;
use std::mem::{size_of, MaybeUninit};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use tauri::{LogicalPosition, PhysicalPosition, Theme, Window};
use windows::core::Result;
use windows::Win32::Foundation::{BOOL, HWND, POINT};
use windows::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMSBT_MAINWINDOW,
    DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE, DWM_SYSTEMBACKDROP_TYPE,
};
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
};
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongW, SetWindowLongW, GWL_STYLE, WS_SYSMENU,
};

pub fn enable_mica(window: &Window) -> Result<()> {
    let handle = window.raw_window_handle();
    let RawWindowHandle::Win32(handle) = handle else {
        panic!("failed to get HWND");
    };
    let hwnd = HWND(handle.hwnd as isize);

    let mut style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32;
    style &= !WS_SYSMENU.0;
    unsafe { SetWindowLongW(hwnd, GWL_STYLE, style as i32) };

    unsafe {
        DwmExtendFrameIntoClientArea(
            hwnd,
            &MARGINS {
                cxLeftWidth: -1,
                cxRightWidth: -1,
                cyBottomHeight: -1,
                cyTopHeight: -1,
            } as *const MARGINS,
        )
    }?;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &(window
                .theme()
                .map(|theme| theme == Theme::Dark)
                .unwrap_or_default()
                .into()) as *const BOOL as *const c_void,
            size_of::<BOOL>() as u32,
        )
    }?;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &DWMSBT_MAINWINDOW as *const DWM_SYSTEMBACKDROP_TYPE as *const c_void,
            size_of::<DWM_SYSTEMBACKDROP_TYPE>() as u32,
        )
    }?;
    Ok(())
}

pub fn locate_panel(window: &Window, pos: &PhysicalPosition<f64>) {
    let hmonitor = unsafe {
        MonitorFromPoint(
            POINT {
                x: pos.x as i32,
                y: pos.y as i32,
            },
            MONITOR_DEFAULTTOPRIMARY,
        )
    };
    let mut info: MaybeUninit<MONITORINFO> = MaybeUninit::uninit();
    unsafe { info.assume_init_mut() }.cbSize = size_of::<MONITORINFO>() as u32;
    if !unsafe { GetMonitorInfoW(hmonitor, info.as_mut_ptr()) }.as_bool() {
        return;
    }
    let mrect = unsafe { info.assume_init() }.rcWork;

    let Ok(wsize) = window.inner_size() else {
        return;
    };
    let npos = PhysicalPosition {
        x: mrect.right as u32 - wsize.width,
        y: mrect.bottom as u32 - wsize.height,
    };
    let mut npos = LogicalPosition::<f64>::from_physical(npos, window.scale_factor().unwrap_or(1.));
    npos.x -= 12.;
    npos.y -= 12.;
    let _ = window.set_position(npos);
}
