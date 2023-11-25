use monitor::{get_monitors, Feature};
use std::env;

fn main() {
    let mut args = env::args_os();
    let id = args.nth(1).expect("expected monitor id");
    let feature_name = args.next().expect("expected feature name");
    let mut feature_name = feature_name.into_string().expect("invalid feature name");
    feature_name.make_ascii_lowercase();
    let feature = match feature_name.as_str() {
        "luminance" => Feature::Luminance,
        "contrast" => Feature::Contrast,
        "brightness" => Feature::Brightness,
        "volume" => Feature::Volume,
        "powerstate" => Feature::PowerState,
        _ => panic!("invalid feature name"),
    };
    let value = args.next().map(|value| {
        let value = value.into_string().expect("invalid value");
        value.parse().expect("invalid value")
    });
    let monitors = get_monitors();
    let monitor = monitors
        .into_iter()
        .find(|monitor| monitor.id == id)
        .expect("monitor not found");
    if let Some(value) = value {
        monitor
            .set_feature(feature, value)
            .expect("failed to set feature");
    } else {
        println!(
            "{:?}",
            monitor.get_feature(feature).expect("failed to get feature")
        );
    }
}
