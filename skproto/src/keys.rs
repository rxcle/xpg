use std::collections::HashMap;

use std::ffi::CStr;

use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyNameTextA;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct ScanCode(pub i32);
pub const SC_ESCAPE: ScanCode = ScanCode(0x01);
pub const SC_BACK: ScanCode = ScanCode(0x0E);

pub struct KeyInfo {
    pub name: String,
}

pub struct Keychain {
    pub keys: Vec<Key>,
    pub key_infos: HashMap<ScanCode, KeyInfo>,
}

pub struct Key {
    pub scan_code: ScanCode,
}

impl Keychain {
    pub fn new() -> Self {
        Self {
            keys: vec![],
            key_infos: HashMap::new(),
        }
    }

    pub fn add(&mut self, scan_code: ScanCode) {
        self.key_infos.entry(scan_code).or_insert_with(|| KeyInfo {
            name: Keychain::get_key_name(&scan_code),
        });
        self.keys.push(Key { scan_code });
    }

    pub fn back(&mut self) {
        self.keys.pop();
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }

    pub fn get_key_name(scan_code: &ScanCode) -> String {
        let lparam_for_key_name = (scan_code.0 << 16) as i32;
        let mut key_name_buf = [0u8; 128];
        let ret = unsafe { GetKeyNameTextA(lparam_for_key_name, &mut key_name_buf) };
        if ret > 0 {
            if let Ok(cstr) = CStr::from_bytes_with_nul(&key_name_buf[..ret as usize + 1]) {
                if let Ok(key_name) = cstr.to_str() {
                    return key_name.to_string();
                }
            }
        }
        return String::default();
    }
}
