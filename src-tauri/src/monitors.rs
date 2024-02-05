use std::ffi::OsStr;

use monitor::{Feature, Interface, Monitor};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::{Mutex, RwLock};
use tauri::State;
use tokio::time::{sleep, sleep_until, Duration, Instant};

use crate::util::JSResult;

#[derive(Debug)]
pub struct Monitors(RwLock<Vec<(Monitor, Mutex<Instant>)>>);

impl Monitors {
    pub const fn new() -> Monitors {
        Monitors(RwLock::const_new(Vec::new()))
    }
}

#[tauri::command]
pub async fn refresh_monitors(monitors: State<'_, Monitors>) -> JSResult<()> {
    let mut monitors = monitors.0.write().await;
    monitors.clear();
    let stub_instant = Instant::now();
    for monitor in monitor::get_monitors() {
        monitors.push((monitor, Mutex::const_new(stub_instant)));
    }
    Ok(())
}

#[tauri::command]
pub async fn get_monitors(monitors: State<'_, Monitors>) -> JSResult<Vec<String>> {
    let monitors = monitors.0.read().await;
    Ok(monitors
        .iter()
        .map(|(monitor, _)| monitor.id.to_string_lossy().into_owned())
        .collect())
}

fn get_monitor_by_id<'a>(
    monitors: &'a [(Monitor, Mutex<Instant>)],
    id: &'_ str,
) -> JSResult<&'a (Monitor, Mutex<Instant>)> {
    let id_os: &OsStr = id.as_ref();
    monitors
        .iter()
        .find(|(monitor, _)| monitor.id == id_os)
        .ok_or_else(|| format!("no such monitor: '{id}'").into())
}

#[tauri::command]
pub async fn get_monitor_user_friendly_name(
    monitors: State<'_, Monitors>,
    id: String,
) -> JSResult<Option<String>> {
    let monitors = monitors.0.read().await;
    let monitor = &get_monitor_by_id(&monitors, &id)?.0;
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
    source: &'static str,
}

// TODO: move it into JS
const UPDATE_INTERVAL: Duration = Duration::from_millis(200);

#[tauri::command]
pub async fn get_monitor_feature(
    monitors: State<'_, Monitors>,
    id: String,
    feature: String,
) -> JSResult<Reply> {
    let monitors = monitors.0.read().await;
    let (monitor, instant) = get_monitor_by_id(&monitors, &id)?;
    let feature = feature_from_string(feature)?;

    let mut instant = instant.lock().await;
    sleep_until(*instant).await;
    let monitor::Reply {
        current,
        maximum,
        source,
    } = monitor.get_feature(feature)?;
    *instant = Instant::now() + UPDATE_INTERVAL;

    Ok(Reply {
        current,
        maximum,
        source: match source {
            Interface::DDCCI => "ddcci",
            Interface::IOCTL => "ioctl",
        },
    })
}

#[tauri::command]
pub async fn set_monitor_feature(
    monitors: State<'_, Monitors>,
    id: String,
    feature: String,
    value: u32,
) -> JSResult<Reply> {
    let monitors = monitors.0.read().await;
    let (monitor, instant) = get_monitor_by_id(&monitors, &id)?;
    let feature = feature_from_string(feature)?;

    let mut instant = instant.lock().await;
    sleep_until(*instant).await;
    monitor.set_feature(feature, value)?;

    sleep(UPDATE_INTERVAL).await;
    let monitor::Reply {
        current,
        maximum,
        source,
    } = monitor.get_feature(feature)?;
    *instant = Instant::now() + UPDATE_INTERVAL;

    Ok(Reply {
        current,
        maximum,
        source: match source {
            Interface::DDCCI => "ddcci",
            Interface::IOCTL => "ioctl",
        },
    })
}
