use std::collections::BTreeMap;

use monitor::{Feature, Monitor};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::RwLock;
use tauri::State;

use crate::util::JSResult;

#[derive(Debug)]
pub struct MonitorManager(RwLock<BTreeMap<String, Monitor>>);

impl MonitorManager {
    pub const fn new() -> MonitorManager {
        MonitorManager(RwLock::const_new(BTreeMap::new()))
    }
}

#[tauri::command]
pub async fn refresh_monitors(monitors: State<'_, MonitorManager>) -> JSResult<()> {
    let mut monitors = monitors.0.write().await;
    monitors.clear();
    for monitor in monitor::get_monitors() {
        let pv = monitors.insert(monitor.id.to_string_lossy().into_owned(), monitor);
        debug_assert!(pv.is_none())
    }
    Ok(())
}

#[tauri::command]
pub async fn get_monitors(monitors: State<'_, MonitorManager>) -> JSResult<Vec<String>> {
    let monitors = monitors.0.read().await;
    Ok(monitors.keys().map(String::clone).collect())
}

fn get_monitor_by_id<'a>(
    monitors: &'a BTreeMap<String, Monitor>,
    id: &'_ String,
) -> JSResult<&'a Monitor> {
    monitors
        .get(id)
        .ok_or_else(|| format!("no such monitor: '{id}'").into())
}

#[tauri::command]
pub async fn get_monitor_user_friendly_name(
    monitors: State<'_, MonitorManager>,
    id: String,
) -> JSResult<Option<String>> {
    let monitors = monitors.0.read().await;
    let monitor = get_monitor_by_id(&monitors, &id)?;
    Ok(monitor
        .get_user_friendly_name()?
        .map(|s| s.to_string_lossy().into_owned()))
}

fn feature_from_string(mut feature_name: String) -> JSResult<Feature> {
    feature_name.make_ascii_lowercase();
    Ok(match feature_name.as_str() {
        "luminance" => Feature::Luminance,
        "contrast" => Feature::Contrast,
        "brightness" => Feature::Brightness,
        "volume" => Feature::Volume,
        "powerstate" => Feature::PowerState,
        _ => return Err(format!("invalid feature name: '{feature_name}'").into()),
    })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reply {
    current: u32,
    maximum: u32,
}

#[tauri::command]
pub async fn get_monitor_feature(
    monitors: State<'_, MonitorManager>,
    id: String,
    feature: String,
) -> JSResult<Reply> {
    let monitors = monitors.0.read().await;
    let monitor = get_monitor_by_id(&monitors, &id)?;
    let feature = feature_from_string(feature)?;
    let monitor::Reply { current, maximum } = monitor.get_feature(feature)?;
    Ok(Reply { current, maximum })
}

#[tauri::command]
pub async fn set_monitor_feature(
    monitors: State<'_, MonitorManager>,
    id: String,
    feature: String,
    value: u32,
) -> JSResult<()> {
    let monitors = monitors.0.read().await;
    let monitor = get_monitor_by_id(&monitors, &id)?;
    let feature = feature_from_string(feature)?;
    Ok(monitor.set_feature(feature, value)?)
}
