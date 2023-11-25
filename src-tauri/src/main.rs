// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::BTreeMap;
use std::error::Error as StdError;

use monitor::{Feature, Monitor};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::Mutex;
use tauri::State;
use uuid::Uuid;

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

#[tauri::command]
fn uuid4() -> String {
    let uuid = Uuid::new_v4();
    uuid.as_hyphenated().to_string()
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct OsVersion {
    pub major: u32,
    pub minor: u32,
    pub pack: u32,
    pub build: u32,
}

#[tauri::command]
fn windows_version() -> OsVersion {
    let ver = windows_version::OsVersion::current();
    OsVersion { major: ver.major, minor: ver.minor, pack: ver.pack, build: ver.build }
}

fn main() {
    monitor::init_com().expect("failed to initialize COM");
    tauri::Builder::default()
        .manage(Monitors::new())
        .invoke_handler(tauri::generate_handler![
            refresh_monitors,
            get_monitors,
            get_monitor_user_friendly_name,
            get_monitor_feature,
            set_monitor_feature,
            uuid4,
            windows_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
