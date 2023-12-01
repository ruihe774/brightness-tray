use std::ffi::c_void;
use std::mem::{size_of, MaybeUninit};

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use tauri::{PhysicalPosition, Theme, Window};
use windows::core::{Error, PCSTR};
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
    FindWindowA, GetWindowLongW, SetWindowLongW, SetWindowPos, GWL_STYLE, SWP_ASYNCWINDOWPOS,
    SWP_NOMOVE, SWP_NOSIZE, WS_SYSMENU,
};

use crate::util::JSResult;

#[tauri::command]
pub fn refresh_panel_style(window: Window) -> JSResult<()> {
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

    let tray_wnd = unsafe { FindWindowA(PCSTR::from_raw(b"Shell_TrayWnd\0".as_ptr()), None) };
    unsafe {
        SetWindowPos(
            hwnd,
            tray_wnd,
            0,
            0,
            0,
            0,
            SWP_ASYNCWINDOWPOS | SWP_NOMOVE | SWP_NOSIZE,
        )
    }?;

    Ok(())
}

#[tauri::command]
pub fn get_workarea_corner(position: PhysicalPosition<i32>) -> JSResult<PhysicalPosition<i32>> {
    let hmonitor = unsafe {
        MonitorFromPoint(
            POINT {
                x: position.x,
                y: position.y,
            },
            MONITOR_DEFAULTTOPRIMARY,
        )
    };
    let mut info: MaybeUninit<MONITORINFO> = MaybeUninit::uninit();
    unsafe { info.assume_init_mut() }.cbSize = size_of::<MONITORINFO>() as u32;
    if !unsafe { GetMonitorInfoW(hmonitor, info.as_mut_ptr()) }.as_bool() {
        return Err(Error::from_win32().into());
    }
    let mrect = unsafe { info.assume_init() }.rcWork;
    Ok(PhysicalPosition {
        x: mrect.right,
        y: mrect.bottom,
    })
}
