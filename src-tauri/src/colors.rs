use serde::{Deserialize, Serialize};
use windows::UI;
use windows::UI::ViewManagement::{UIColorType, UISettings};

use crate::util::JSResult;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct AccentColors {
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
pub fn get_accent_colors() -> JSResult<AccentColors> {
    let settings = UISettings::new()?;

    let get_color = |color_type| -> JSResult<Color> {
        let UI::Color { R, G, B, A } = settings.GetColorValue(color_type)?;
        debug_assert_eq!(A, 255);
        Ok(Color { r: R, g: G, b: B })
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
