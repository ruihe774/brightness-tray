use std::io::{self, Write};

use monitor::{get_monitors, init_com};

fn main() {
    init_com().unwrap();
    let monitors = get_monitors();
    let mut stdout = io::stdout().lock();
    for monitor in monitors {
        let name = monitor.get_user_friendly_name().unwrap();
        writeln!(stdout, "{name:?}: {monitor:?}").unwrap();
    }
}
