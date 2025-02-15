#![windows_subsystem = "windows"]

mod window;

use window::Window;

use windows::core::Result;
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};

fn main() {
    let version = env!("CARGO_PKG_VERSION");
    unsafe {
        _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
    println!("tinitime v{}", version);

    // if Window::is_running() {
    //     println!("already active, switching");
    //     return;
    // }

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
