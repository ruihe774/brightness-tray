use tauri::{CustomMenuItem, Icon, SystemTray, SystemTrayHandle, SystemTrayMenu, Theme};

use crate::util::ThemableIcon;

#[derive(Debug)]
pub struct TrayManager {
    icon: ThemableIcon,
}

impl TrayManager {
    pub fn new() -> TrayManager {
        TrayManager {
            icon: ThemableIcon {
                light: Icon::Raw(Vec::from(include_bytes!("../icons/light.ico"))),
                dark: Icon::Raw(Vec::from(include_bytes!("../icons/dark.ico"))),
            },
        }
    }

    pub fn make_system_tray(&self) -> SystemTray {
        SystemTray::new()
            .with_menu(
                SystemTrayMenu::new().add_item(CustomMenuItem::new("quit".to_owned(), "Quit")),
            )
            .with_icon(self.icon.choose(None).clone())
    }

    pub fn set_theme(&self, tray: &SystemTrayHandle, theme: Option<Theme>) {
        tray.set_icon(self.icon.choose(theme).clone()).unwrap();
    }
}
