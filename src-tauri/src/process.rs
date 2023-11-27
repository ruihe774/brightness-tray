#![cfg_attr(debug_assertions, allow(unreachable_code))]
#![cfg_attr(debug_assertions, allow(unused_imports))]

pub use monitor::init_com;
use std::env;
use std::fs::OpenOptions;
use std::mem::forget;
use std::os::windows::fs::OpenOptionsExt;
use std::process::exit;
use windows::core::PCSTR;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{
    MessageBoxA, IDABORT, IDIGNORE, IDRETRY, MB_ABORTRETRYIGNORE, MB_ICONWARNING,
};
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR};
use windows_version::OsVersion;

const MESSAGE_CAPTION: &str = "Brightness Tray\0";

pub fn hook_panic() {
    #[cfg(not(debug_assertions))]
    std::panic::set_hook(Box::new(|info| {
        let text = format!("The program {info}\0");
        let wtext: Vec<_> = text.encode_utf16().collect();
        let caption = MESSAGE_CAPTION;
        let wcaption: Vec<_> = caption.encode_utf16().collect();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR::from_raw(wtext.as_ptr()),
                PCWSTR::from_raw(wcaption.as_ptr()),
                MB_ICONERROR,
            )
        };
    }));
}

pub fn ensure_singleton() {
    let mut lock_file = env::temp_dir();
    lock_file.push("BrightnessTray.lock");
    forget(
        match OpenOptions::new()
            .write(true)
            .create(true)
            .share_mode(0)
            .open(lock_file)
        {
            Ok(f) => f,
            Err(e) if e.raw_os_error() == Some(32) => {
                #[cfg(debug_assertions)]
                panic!("another instance is running");
                let text = b"Another instance is running.\0";
                let caption = MESSAGE_CAPTION.as_bytes();
                let r = unsafe {
                    MessageBoxA(
                        None,
                        PCSTR::from_raw(text.as_ptr()),
                        PCSTR::from_raw(caption.as_ptr()),
                        MB_ABORTRETRYIGNORE | MB_ICONWARNING,
                    )
                };
                return match r {
                    IDABORT => exit(0),
                    IDRETRY => ensure_singleton(),
                    IDIGNORE => (),
                    _ => unreachable!(),
                };
            }
            r @ Err(_) => r.expect("failed to create singleton lock"),
        },
    )
}

pub fn ensure_windows_version() {
    let version = OsVersion::current();
    if version.build < 22621 {
        #[cfg(debug_assertions)]
        panic!("unsupported Windows version");
        let text = b"Unsupported Windows version. Please upgrade your system to Windows 11 22H2 or later.\0";
        let caption = MESSAGE_CAPTION.as_bytes();
        let r = unsafe {
            MessageBoxA(
                None,
                PCSTR::from_raw(text.as_ptr()),
                PCSTR::from_raw(caption.as_ptr()),
                MB_ABORTRETRYIGNORE | MB_ICONWARNING,
            )
        };
        return match r {
            IDABORT => exit(0),
            IDRETRY => ensure_windows_version(),
            IDIGNORE => (),
            _ => unreachable!(),
        };
    }
}
