use std::error::Error;

use tauri::{Icon, Theme};

pub type JSResult<T> = Result<T, String>;

pub fn error_to_string<E: Error>(e: E) -> String {
    #[cfg(debug_assertions)]
    return format!("{e:?}");
    #[cfg(not(debug_assertions))]
    return format!("{e}");
}

#[derive(Debug)]
pub struct ThemableIcon {
    pub dark: Icon,
    pub light: Icon,
}

impl ThemableIcon {
    pub fn choose(&self, theme: Option<Theme>) -> &Icon {
        match theme {
            Some(Theme::Dark) => &self.dark,
            _ => &self.light,
        }
    }
}
