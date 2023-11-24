use std::io::{self, Write};

use monitor::get_monitors;

fn main() {
    let monitors = get_monitors();
    let mut stdout = io::stdout().lock();
    for monitor in monitors {
        writeln!(stdout, "{monitor:?}").unwrap();
    }
}
