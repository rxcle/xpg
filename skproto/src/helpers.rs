use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    UI::Input::KeyboardAndMouse::{MapVirtualKeyW, MAPVK_VK_TO_VSC_EX},
};

use crate::keys::ScanCode;

pub const fn loword(value: isize) -> i32 {
    (value & 0xFFFF) as i32
}

pub const fn hiword(value: isize) -> i32 {
    ((value >> 16) & 0xFFFF) as i32
}

pub fn to_lpcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().collect()
}

pub fn mul_div_round(a: i32, b: i32, c: i32) -> i32 {
    ((a as i64 * b as i64 + c as i64 / 2) / c as i64) as i32
}

pub fn determine_key_pressed(wparam: WPARAM, lparam: LPARAM) -> Option<ScanCode> {
    let is_repeat = ((lparam.0 >> 30) & 1) != 0;
    if is_repeat {
        return None;
    }
    let raw_scan_code = ((lparam.0 >> 16) & 0xFF) as i32;
    let scan_code_value = if raw_scan_code == 0 {
        // Media keys only generate a VK, not a scan code
        let mapped_scan_code =
            unsafe { MapVirtualKeyW(wparam.0 as u32, MAPVK_VK_TO_VSC_EX) as i32 };
        if mapped_scan_code == 0 {
            return None;
        }
        mapped_scan_code
    } else if lparam.0 & (1 << 24) != 0 {
        // Extended key (Right Alt, Right Ctrl, ...)
        raw_scan_code | 0x100
    } else {
        raw_scan_code
    };
    Some(ScanCode(scan_code_value))
}
