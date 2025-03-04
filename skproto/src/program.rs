use windows::core::Result;
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

use crate::window::Window;

pub struct Program {
    _window: Box<Window>,
}

impl Program {
    pub fn new() -> Result<Self> {
        Self::attach_console()?;
        Ok(Self {
            _window: Window::new("skproto")?,
        })
    }

    fn attach_console() -> Result<()> {
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS)?;
        }
        let version = env!("CARGO_PKG_VERSION");
        println!("skproto v{}", version);
        Ok(())
    }

    pub fn run(&self) {
        let mut message = MSG::default();
        unsafe {
            while GetMessageW(&mut message, None, 0, 0).into() {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        println!("bye bye");
    }
}
