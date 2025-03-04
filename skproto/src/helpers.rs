use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

pub fn to_lpcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().collect()
}

use windows::Win32::UI::Input::KeyboardAndMouse::*;

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
