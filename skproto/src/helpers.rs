use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

pub fn to_lpcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().collect()
}

use windows::Win32::{Foundation::SIZE, UI::Input::KeyboardAndMouse::*};

pub struct Key {
    pub vk: VIRTUAL_KEY,
    pub name: String,
    pub text_size: SIZE,
}

pub struct Keychain {
    pub(crate) keys: Vec<Key>,
}

impl Keychain {
    pub fn new() -> Self {
        Self { keys: vec![] }
    }

    pub fn add(&mut self, key: Key) {
        self.keys.push(key);
    }

    pub fn back(&mut self) {
        self.keys.pop();
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }
}

pub fn scancode_to_vk(scancode: u32, input_vk: VIRTUAL_KEY, is_extended: bool) -> VIRTUAL_KEY {
    let vk = unsafe { MapVirtualKeyW(scancode, MAPVK_VSC_TO_VK_EX) as u16 };
    if vk > 0 {
        VIRTUAL_KEY(vk)
    } else {
        map_extended_key(input_vk, is_extended)
    }
}

pub fn map_extended_key(vk: VIRTUAL_KEY, is_extended: bool) -> VIRTUAL_KEY {
    match vk {
        VK_SHIFT => {
            if is_extended {
                VK_RSHIFT
            } else {
                VK_LSHIFT
            }
        }
        VK_CONTROL => {
            if is_extended {
                VK_RCONTROL
            } else {
                VK_LCONTROL
            }
        }
        VK_MENU => {
            if is_extended {
                VK_RMENU
            } else {
                VK_LMENU
            }
        }
        _ => vk,
    }
}
