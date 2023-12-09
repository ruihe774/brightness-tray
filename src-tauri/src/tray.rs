use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use tauri::{Icon, Manager, Window};

use crate::util::JSResult;

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrayIcon {
    #[serde_as(as = "Base64")]
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
