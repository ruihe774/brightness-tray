// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod colors;
mod monitors;
mod process;
mod tray;
mod util;
mod wm;

use tauri::{Manager, SystemTrayEvent, WindowEvent};

fn main() {
    process::hook_panic();
    process::ensure_windows_version();
    process::ensure_singleton();
    process::init_com().expect("failed to initialize COM");

    let tray_manager = tray::TrayManager::new();
    let monitor_manager = monitors::MonitorManager::new();
    tauri::Builder::default()
        .system_tray(tray_manager.make_system_tray())
        .manage(tray_manager)
        .manage(monitor_manager)
        .setup(move |app| {
            let panel = app.get_window("panel").unwrap();
            let app = panel.app_handle();

            wm::enable_mica(&panel)?;

            app.state::<tray::TrayManager>()
                .set_theme(&app.tray_handle(), panel.theme().ok());
            panel.on_window_event(move |event| {
                if let WindowEvent::ThemeChanged(theme) = event {
                    app.state::<tray::TrayManager>()
                        .set_theme(&app.tray_handle(), Some(*theme));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            monitors::refresh_monitors,
            monitors::get_monitors,
            monitors::get_monitor_user_friendly_name,
            monitors::get_monitor_feature,
            monitors::set_monitor_feature,
            colors::get_accent_colors,
        ])
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { position, .. } => {
                let window = app.get_window("panel").unwrap();
                if window.is_visible().unwrap_or_default() {
                    window.hide().unwrap()
                } else {
                    wm::locate_panel(&window, &position);
                    window.show().unwrap();
                    wm::enable_mica(&window).unwrap();
                    window.set_focus().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } if id == "quit" => {
                app.exit(0);
            }
            _ => (),
        })
        .on_window_event(|e| {
            if let WindowEvent::ThemeChanged(_) = e.event() {
                wm::enable_mica(e.window()).unwrap();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
