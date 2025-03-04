use std::collections::HashMap;

use std::ffi::CStr;
use std::hash::Hash;

use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyNameTextA;

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct KeyRef(pub u16);

#[derive(Clone, Copy)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[derive(Clone)]
pub struct Key {
    pub vk: KeyRef,
    pub name: String,
    pub text_size: Size,
}

pub struct KeyInfo {
    pub vk: KeyRef,
    pub name: String,
    pub text_size: Size,
    pub is_dirty: bool,
}

pub struct Keychain {
    pub keys: Vec<Key>,
    pub key_info: HashMap<KeyRef, KeyInfo>,
}

impl Keychain {
    pub fn new() -> Self {
        Self {
            keys: vec![],
            key_info: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: Key) {
        let key_info = self.key_info.get(&key.vk);
        // if key_info.is_none() {
        //     self.key_info.insert(
        //         key.vk,
        //         KeyInfo {
        //             vk: key.vk,
        //             name: key.name,
        //             text_size: key.text_size,
        //             is_dirty: false,
        //         },
        //     );
        // }
        self.keys.push(Key {
            name: key.name,
            vk: key.vk,
            text_size: Size {
                width: key.text_size.width,
                height: key.text_size.height,
            },
        });
    }

    pub fn back(&mut self) {
        self.keys.pop();
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }

    pub fn get_key_name(scan_code: u32) -> String {
        let lparam_for_key_name = (scan_code << 16) as i32;
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
