// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(debug_assertions, allow(unreachable_code))]
#![cfg_attr(debug_assertions, allow(unused_imports))]

use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::ffi::c_void;
use std::mem;

use monitor::{Feature, Monitor};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::Mutex;
use tauri::{State, Window};

#[derive(Debug)]
struct Monitors(Mutex<BTreeMap<String, Monitor>>);

impl Monitors {
    const fn new() -> Monitors {
        Monitors(Mutex::const_new(BTreeMap::new()))
    }
}

type JSResult<T> = Result<T, String>;

#[tauri::command]
async fn refresh_monitors(monitors: State<'_, Monitors>) -> JSResult<()> {
    let mut monitors = monitors.0.lock().await;
    monitors.clear();
    for monitor in monitor::get_monitors() {
        let pv = monitors.insert(monitor.id.to_string_lossy().into_owned(), monitor);
        debug_assert!(pv.is_none())
    }
    Ok(())
}

#[tauri::command]
async fn get_monitors(monitors: State<'_, Monitors>) -> JSResult<Vec<String>> {
    let monitors = monitors.0.lock().await;
    Ok(monitors.keys().map(String::clone).collect())
}

fn get_monitor_by_id<'a>(
    monitors: &'a BTreeMap<String, Monitor>,
    id: &'_ String,
) -> JSResult<&'a Monitor> {
    monitors
        .get(id)
        .ok_or_else(|| format!("no such monitor: '{id}'"))
}

#[tauri::command]
async fn get_monitor_user_friendly_name(
    monitors: State<'_, Monitors>,
    id: String,
) -> JSResult<Option<String>> {
    let monitors = monitors.0.lock().await;
    let monitor = get_monitor_by_id(&monitors, &id)?;
    Ok(monitor
        .get_user_friendly_name()
        .ok()
        .flatten()
        .map(|s| s.to_string_lossy().into_owned()))
}

fn error_to_string<E: StdError>(e: E) -> String {
    #[cfg(debug_assertions)]
    return format!("{e:?}");
    #[cfg(not(debug_assertions))]
    return format!("{e}");
}

fn feature_from_string(mut feature_name: String) -> JSResult<Feature> {
    feature_name.make_ascii_lowercase();
    Ok(match feature_name.as_str() {
        "luminance" => Feature::Luminance,
        "contrast" => Feature::Contrast,
        "brightness" => Feature::Brightness,
        "volume" => Feature::Volume,
        "powerstate" => Feature::PowerState,
        _ => return Err(format!("invalid feature name: '{feature_name}'")),
    })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Reply {
    current: u32,
    maximum: u32,
}

#[tauri::command]
async fn get_monitor_feature(
    monitors: State<'_, Monitors>,
    id: String,
    feature: String,
) -> JSResult<Reply> {
    let monitors = monitors.0.lock().await;
    let monitor = get_monitor_by_id(&monitors, &id)?;
    let feature = feature_from_string(feature)?;
    match monitor.get_feature(feature) {
        Ok(monitor::Reply { current, maximum }) => Ok(Reply { current, maximum }),
        Err(e) => Err(error_to_string(e)),
    }
}

#[tauri::command]
async fn set_monitor_feature(
    monitors: State<'_, Monitors>,
    id: String,
    feature: String,
    value: u32,
) -> JSResult<()> {
    let monitors = monitors.0.lock().await;
    let monitor = get_monitor_by_id(&monitors, &id)?;
    let feature = feature_from_string(feature)?;
    monitor.set_feature(feature, value).map_err(error_to_string)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct AccentColors {
    accent: Color,
    accentDark1: Color,
    accentDark2: Color,
    accentDark3: Color,
    accentLight1: Color,
    accentLight2: Color,
    accentLight3: Color,
    background: Color,
    foreground: Color,
}

#[tauri::command]
fn get_accent_colors() -> JSResult<AccentColors> {
    use windows::UI;
    use windows::UI::ViewManagement::{UIColorType, UISettings};

    let settings = UISettings::new().map_err(error_to_string)?;

    let get_color = |color_type| match settings.GetColorValue(color_type) {
        Ok(UI::Color { R, G, B, .. }) => Ok(Color { r: R, g: G, b: B }),
        Err(e) => Err(error_to_string(e)),
    };

    Ok(AccentColors {
        accent: get_color(UIColorType::Accent)?,
        accentDark1: get_color(UIColorType::AccentDark1)?,
        accentDark2: get_color(UIColorType::AccentDark2)?,
        accentDark3: get_color(UIColorType::AccentDark3)?,
        accentLight1: get_color(UIColorType::AccentLight1)?,
        accentLight2: get_color(UIColorType::AccentLight2)?,
        accentLight3: get_color(UIColorType::AccentLight3)?,
        background: get_color(UIColorType::Background)?,
        foreground: get_color(UIColorType::Foreground)?,
    })
}

fn enable_mica(window: &Window) -> windows::core::Result<()> {
    use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{BOOL, HWND};
    use windows::Win32::Graphics::Dwm::{
        DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMSBT_MAINWINDOW,
        DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE, DWM_SYSTEMBACKDROP_TYPE,
    };
    use windows::Win32::UI::Controls::MARGINS;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetWindowLongW, SetWindowLongW, GWL_STYLE, WS_SYSMENU,
    };

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
            &(window.theme().unwrap() == tauri::Theme::Dark).into() as *const BOOL as *const c_void,
            mem::size_of::<BOOL>() as u32,
        )
    }?;
    unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &DWMSBT_MAINWINDOW as *const DWM_SYSTEMBACKDROP_TYPE as *const c_void,
            mem::size_of::<DWM_SYSTEMBACKDROP_TYPE>() as u32,
        )
    }?;
    Ok(())
}

fn locate_panel(window: &Window, pos: &tauri::PhysicalPosition<f64>) {
    use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
    use windows::Win32::Foundation::{HWND, POINT, RECT};
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetWindowRect;

    let handle = window.raw_window_handle();
    let RawWindowHandle::Win32(handle) = handle else {
        panic!("failed to get HWND");
    };
    let hwnd = HWND(handle.hwnd as isize);

    let hmonitor = unsafe {
        MonitorFromPoint(
            POINT {
                x: pos.x as i32,
                y: pos.y as i32,
            },
            MONITOR_DEFAULTTOPRIMARY,
        )
    };
    let mut info = MONITORINFO::default();
    info.cbSize = mem::size_of::<MONITORINFO>() as u32;
    unsafe { GetMonitorInfoW(hmonitor, &mut info) };
    let mrect = info.rcWork;

    let mut wrect = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut wrect) }.unwrap();
    let w = wrect.right - wrect.left;
    let h = wrect.bottom - wrect.top;
    let x = mrect.right - w - 16;
    let y = mrect.bottom - h - 16;

    window
        .set_position(tauri::PhysicalPosition { x, y })
        .unwrap()
}

fn hook_panic() {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR};
    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(|info| {
        let text = format!("The program {info}\0");
        let wtext: Vec<_> = text.encode_utf16().collect();
        let caption = "Brightness Tray\0";
        let wcaption: Vec<_> = caption.encode_utf16().collect();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR::from_raw(wtext.as_ptr()),
                PCWSTR::from_raw(wcaption.as_ptr()),
                MB_ICONERROR,
            )
        };
    }));
}

fn ensure_singleton() {
    use std::env;
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    use std::process::exit;
    use windows::core::PCSTR;
    use windows::Win32::UI::WindowsAndMessaging::{
        MessageBoxA, IDABORT, IDIGNORE, IDRETRY, MB_ABORTRETRYIGNORE, MB_ICONWARNING,
    };

    let mut lock_file = env::temp_dir();
    lock_file.push("BrightnessTray.lock");
    mem::forget(
        match OpenOptions::new()
            .write(true)
            .create(true)
            .share_mode(0)
            .open(lock_file)
        {
            Ok(f) => f,
            Err(e) if e.raw_os_error() == Some(32) => {
                #[cfg(debug_assertions)]
                panic!("another instance is running");
                let text = b"Another instance is running.\0";
                let caption = b"Brightness Tray\0";
                let r = unsafe {
                    MessageBoxA(
                        None,
                        PCSTR::from_raw(text.as_ptr()),
                        PCSTR::from_raw(caption.as_ptr()),
                        MB_ABORTRETRYIGNORE | MB_ICONWARNING,
                    )
                };
                return match r {
                    IDABORT => exit(0),
                    IDRETRY => ensure_singleton(),
                    IDIGNORE => (),
                    _ => unreachable!(),
                };
            }
            r @ Err(_) => r.expect("failed to create singleton lock"),
        },
    )
}

fn main() {
    use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};
    hook_panic();
    ensure_singleton();
    monitor::init_com().expect("failed to initialize COM");
    tauri::Builder::default()
        .manage(Monitors::new())
        .invoke_handler(tauri::generate_handler![
            refresh_monitors,
            get_monitors,
            get_monitor_user_friendly_name,
            get_monitor_feature,
            set_monitor_feature,
            get_accent_colors,
        ])
        .setup(|app| {
            for (_, window) in app.windows() {
                enable_mica(&window)?;
            }
            Ok(())
        })
        .system_tray(SystemTray::new().with_menu(
            SystemTrayMenu::new().add_item(CustomMenuItem::new("quit".to_owned(), "Quit")),
        ))
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { position, .. } => {
                let window = app.get_window("panel").unwrap();
                locate_panel(&window, &position);
                window.show().unwrap();
                enable_mica(&window).unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } if id == "quit" => {
                app.exit(0);
            }
            _ => (),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
