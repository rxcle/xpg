#![windows_subsystem = "windows"]

mod window;

use window::Window;

use windows::core::Result;

fn main() {
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
