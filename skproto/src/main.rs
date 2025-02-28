#![windows_subsystem = "windows"]

mod helpers;
mod window;

use window::Window;

use windows::core::Result;
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    unsafe {
        _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
    println!("skproto v{}", version);

    let result = run();
    if let Err(error) = result {
        error.code().unwrap();
    }
}

fn run() -> Result<()> {
    _ = Window::new("skproto")?;
    Window::run_message_loop();
    Ok(())
}
