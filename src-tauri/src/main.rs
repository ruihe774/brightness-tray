// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod colors;
mod monitors;
mod process;
mod tray;
mod util;
mod wm;

use tauri::{CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu};

fn main() {
    process::hook_panic();
    process::ensure_windows_version();
    process::ensure_singleton();
    process::init_com().expect("failed to initialize COM");

    tauri::Builder::default()
        .system_tray(SystemTray::new().with_menu(
            SystemTrayMenu::new().add_item(CustomMenuItem::new("quit".to_owned(), "Quit")),
        ))
        .manage(monitors::MonitorManager::new())
        .invoke_handler(tauri::generate_handler![
            monitors::refresh_monitors,
            monitors::get_monitors,
            monitors::get_monitor_user_friendly_name,
            monitors::get_monitor_feature,
            monitors::set_monitor_feature,
            colors::get_accent_colors,
            wm::refresh_panel_style,
            wm::get_workarea_corner,
            tray::set_tray_icon,
        ])
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick { position, .. } => {
                app.emit_all("tray-icon-click", position).unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } if id == "quit" => {
                app.exit(0);
            }
            _ => (),
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
