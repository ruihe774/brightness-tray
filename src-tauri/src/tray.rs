use serde::{Deserialize, Serialize};
use tauri::{Icon, Manager, Window};

use crate::util::JSResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayIcon {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

#[tauri::command]
pub fn set_tray_icon(window: Window, icon: TrayIcon) -> JSResult<()> {
    let tray = window.app_handle().tray_handle();
    let icon = Icon::Rgba {
        rgba: icon.rgba,
        width: icon.width,
        height: icon.height,
    };
    tray.set_icon(icon)?;
    Ok(())
}
