#![windows_subsystem = "windows"]

mod helpers;
mod keys;
mod program;
mod window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    program::Program::new()?.run();
    Ok(())
}
