#![windows_subsystem = "windows"]

mod window;

use window::Window;

use windows::core::Result;
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

fn main() {
    unsafe {
        _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
    println!("tinitime started!");
    let result = run();
    if let Err(error) = result {
        error.code().unwrap();
    }
}

fn run() -> Result<()> {
    _ = Window::new("tinitime")?;
    Window::run_message_loop();
    Ok(())
}
