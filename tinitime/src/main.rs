#![windows_subsystem = "windows"]

mod window;

use std::ptr::null_mut;

use window::Window;
use windows::core::Result;
use windows::Win32::Graphics::GdiPlus;
use windows::Win32::Graphics::GdiPlus::GdiplusShutdown;
use windows::Win32::Graphics::GdiPlus::GdiplusStartup;
use windows::Win32::Graphics::GdiPlus::GdiplusStartupInput;

fn main() {
    let mut gdiplus_token = 0;

    let status = unsafe {
        GdiplusStartup(
            &mut gdiplus_token,
            &GdiplusStartupInput {
                GdiplusVersion: 1,
                ..Default::default()
            },
            null_mut(),
        )
    };

    assert_eq!(status, GdiPlus::Ok);

    let result = run();
    if let Err(error) = result {
        error.code().unwrap();
    }

    unsafe {
        GdiplusShutdown(gdiplus_token);
    }
}

fn run() -> Result<()> {
    _ = Window::new("tinitime")?;
    Window::run_message_loop();
    Ok(())
}
