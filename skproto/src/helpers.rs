use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

pub fn to_lpcwstr(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().collect()
}

pub fn mul_div_round(a: i32, b: i32, c: i32) -> i32 {
    ((a as i64 * b as i64 + c as i64 / 2) / c as i64) as i32
}
