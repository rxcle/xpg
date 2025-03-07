use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

pub fn to_lpcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().collect()
}
