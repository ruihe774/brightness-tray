use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};
use tauri::{Icon, Theme};

#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JSError(pub String);

impl fmt::Display for JSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for JSError {}

impl From<windows::core::Error> for JSError {
    fn from(value: windows::core::Error) -> Self {
        JSError(error_to_message(value))
    }
}

impl From<tauri::Error> for JSError {
    fn from(value: tauri::Error) -> Self {
        JSError(error_to_message(value))
    }
}

impl From<String> for JSError {
    fn from(value: String) -> Self {
        JSError(value)
    }
}

fn error_to_message<E: Error>(e: E) -> String {
    #[cfg(debug_assertions)]
    return format!("{e:?}");
    #[cfg(not(debug_assertions))]
    return format!("{e}");
}

pub type JSResult<T> = Result<T, JSError>;

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
