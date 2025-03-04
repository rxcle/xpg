#![allow(unused_must_use)]

use std::ffi::{c_void, CStr, OsStr};

use windows::{
    core::{w, Result, HSTRING, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::{
            Dwm::{
                DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_SYSTEMBACKDROP_TYPE,
                DWMWA_USE_HOSTBACKDROPBRUSH, DWMWA_VISIBLE_FRAME_BORDER_THICKNESS,
                DWMWA_WINDOW_CORNER_PREFERENCE, DWMWINDOWATTRIBUTE, DWM_WINDOW_CORNER_PREFERENCE,
            },
            Gdi::{
                BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW,
                CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect, GetDC,
                GetTextExtentPoint32A, GetTextExtentPoint32W, InvalidateRect, RedrawWindow,
                ReleaseDC, SelectObject, SetBkMode, SetTextColor, TextOutW, UpdateWindow,
                CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY, DT_CENTER, DT_SINGLELINE,
                DT_VCENTER, HBRUSH, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
                RDW_INVALIDATE, RDW_UPDATENOW, SRCCOPY, TRANSPARENT,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{
                GetKeyNameTextA, GetKeyboardLayout, MapVirtualKeyExW, MAPVK_VSC_TO_VK_EX,
                VIRTUAL_KEY, VK_BACK, VK_ESCAPE, VK_F12,
            },
            WindowsAndMessaging::*,
        },
    },
};

use crate::{
    helpers::{scancode_to_vk, to_lpcwstr},
    keys::{Key, KeyRef, Keychain, Size},
};

const WINDOW_CLASS_NAME: PCWSTR = w!("rxcle.skproto.wc");

const WIN_WIDTH: i32 = 200;
const WIN_HEIGHT: i32 = 25;

pub struct Window {
    handle: HWND,
    font: HFONT,
    fgbrush: HBRUSH,
    fgactive_brush: HBRUSH,
    fgstopped_brush: HBRUSH,
    window_active: bool,
    client_rect: RECT,
    keychain: Keychain,
}

impl Window {
    pub fn new(title: &str) -> Result<Box<Self>> {
        unsafe {
            let instance = GetModuleHandleW(None)?;

            let wc = WNDCLASSW {
                hCursor: LoadCursorW(None, IDC_ARROW).ok().unwrap(),
                hInstance: instance.into(),
                lpszClassName: WINDOW_CLASS_NAME,
                style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
                lpfnWndProc: Some(Self::wnd_proc),
                ..Default::default()
            };
            let atom = RegisterClassW(&wc);
            debug_assert!(atom != 0);

            let mut window = Box::new(Self {
                handle: HWND::default(),
                font: HFONT::default(),
                fgbrush: HBRUSH::default(),
                fgactive_brush: HBRUSH::default(),
                fgstopped_brush: HBRUSH::default(),
                window_active: false,
                client_rect: RECT {
                    left: 0,
                    top: 0,
                    right: WIN_WIDTH,
                    bottom: WIN_HEIGHT,
                },
                keychain: Keychain::new(),
            });

            let hinstance: HINSTANCE = instance.into();
            let handle = CreateWindowExW(
                WS_EX_APPWINDOW | WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_COMPOSITED,
                WINDOW_CLASS_NAME,
                &HSTRING::from(title),
                WS_VISIBLE | WS_POPUP,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                WIN_WIDTH,
                WIN_HEIGHT,
                None,
                None,
                Some(hinstance),
                Some(window.as_mut() as *mut _ as _),
            )?;

            SetLayeredWindowAttributes(handle, COLORREF::default(), 220, LWA_ALPHA);

            let preference = DWM_WINDOW_CORNER_PREFERENCE(3);
            DwmSetWindowAttribute(
                handle,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const _ as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            let enable = 1;
            DwmSetWindowAttribute(
                handle,
                DWMWA_USE_HOSTBACKDROPBRUSH,
                &enable as *const _ as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            window.reset();

            Ok(window)
        }
    }

    unsafe fn init_window(&mut self, window: HWND) {
        self.handle = window;
        self.font = CreateFontW(
            20,
            0,
            0,
            0,
            700i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            0,
            w!("Segoe UI Symbol"),
        );
        self.fgbrush = CreateSolidBrush(COLORREF(0x00FFFFFF));
        self.fgactive_brush = CreateSolidBrush(COLORREF(0x00D7792B));
        self.fgstopped_brush = CreateSolidBrush(COLORREF(0x002B31D7));
    }

    unsafe fn destroy_window(&mut self) {
        PostQuitMessage(0);
        self.handle = HWND::default();
        DeleteObject(HGDIOBJ::from(self.font));
        self.font = HFONT::default();
        DeleteObject(HGDIOBJ::from(self.fgbrush));
        self.fgbrush = HBRUSH::default();
        DeleteObject(HGDIOBJ::from(self.fgactive_brush));
        self.fgactive_brush = HBRUSH::default();
    }

    unsafe fn paint(&mut self, hdc: HDC) {
        let width = self.client_rect.right - self.client_rect.left;
        let height = self.client_rect.bottom - self.client_rect.top;

        let mem_dc = CreateCompatibleDC(Some(hdc));
        let mem_bitmap = CreateCompatibleBitmap(hdc, width, height);
        let old_bitmap = SelectObject(mem_dc, mem_bitmap.into());

        let (bg, fg) = (self.fgactive_brush, COLORREF(0x00FFFFFF));

        let rect = RECT {
            left: self.client_rect.left,
            top: self.client_rect.top,
            right: self.client_rect.right,
            bottom: self.client_rect.bottom,
        };

        FillRect(mem_dc, &rect, bg);

        SelectObject(mem_dc, HGDIOBJ::from(self.font));
        SetTextColor(mem_dc, fg);
        SetBkMode(mem_dc, TRANSPARENT);

        let mut x = 0;
        for key in &self.keychain.keys {
            TextOutW(mem_dc, x, 0, &to_lpcwstr(&key.name));
            x += key.text_size.width + 5;
        }

        BitBlt(hdc, 0, 0, width, height, Some(mem_dc), 0, 0, SRCCOPY);

        // Cleanup
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(mem_bitmap.into());
        DeleteDC(mem_dc);
    }

    unsafe fn reset(&mut self) {
        self.reset_pos();
        ShowWindow(self.handle, SW_SHOW);
    }

    unsafe fn reset_pos(&mut self) {
        let mut window_rect = RECT::default();
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut window_rect as *mut _ as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );

        SetWindowPos(
            self.handle,
            None,
            window_rect.right - WIN_WIDTH - 5,
            50, // window_rect.bottom - WIN_HEIGHT - 5,
            0,
            0,
            SWP_NOSIZE,
        );
    }

    unsafe fn activate_window(&mut self, activate: bool) {
        self.window_active = activate;
        self.refresh();
    }

    fn refresh(&mut self) {
        unsafe {
            InvalidateRect(Some(self.handle), None, false);
            UpdateWindow(self.handle);
        }
    }

    fn measure_text(&self, text: &str) -> SIZE {
        unsafe {
            let mut size = SIZE::default();
            let dc = GetDC(Some(self.handle));
            let org_obj = SelectObject(dc, HGDIOBJ::from(self.font));
            GetTextExtentPoint32W(dc, &to_lpcwstr(text), &mut size);
            SelectObject(dc, org_obj);
            ReleaseDC(Some(self.handle), dc);
            size
        }
    }

    fn handle_key(&mut self, scan_code: u32, vk_code: VIRTUAL_KEY) {
        if vk_code == VK_ESCAPE {
            self.keychain.clear();
        } else if vk_code == VK_BACK {
            self.keychain.back();
        } else {
            let key_name = Keychain::get_key_name(scan_code);
            let size = self.measure_text(&key_name);
            self.keychain.add(Key {
                name: key_name,
                vk: KeyRef(vk_code.0),
                text_size: Size {
                    width: size.cx,
                    height: size.cy,
                },
            });
        }
        self.refresh();
    }

    unsafe fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_QUERYENDSESSION => {
                self.destroy_window();
                LRESULT(1)
            }
            WM_DESTROY => {
                self.destroy_window();
                LRESULT(0)
            }
            WM_ACTIVATE => {
                self.activate_window(wparam.0 > 0);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let psp = &mut ps as *mut PAINTSTRUCT;
                let hdc = BeginPaint(self.handle, psp);
                self.paint(hdc);
                EndPaint(self.handle, &ps);
                LRESULT(0)
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                let is_repeat = ((lparam.0 >> 30) & 1) != 0;
                if is_repeat {
                    return LRESULT(0);
                }

                let mut scan_code = ((lparam.0 >> 16) & 0xFF) as u32;
                let is_extended = lparam.0 & (1 << 24) != 0;
                if is_extended {
                    scan_code |= 0x100;
                }
                let vk_code = scancode_to_vk(scan_code, VIRTUAL_KEY(wparam.0 as u16), is_extended);

                // GetKeyNameText expects the scan code in the upper 16 bits.
                self.handle_key(scan_code, vk_code);
                LRESULT(0)
            }
            _ => DefWindowProcW(self.handle, message, wparam, lparam),
        }
    }

    unsafe extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCCREATE {
            let cs = lparam.0 as *const CREATESTRUCTW;
            let this = (*cs).lpCreateParams as *mut Self;
            (*this).init_window(window);
            SetWindowLongPtrW(window, GWLP_USERDATA, this as _);
        } else {
            let this = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut Self;
            if let Some(this) = this.as_mut() {
                return this.message_handler(message, wparam, lparam);
            }
        }
        DefWindowProcW(window, message, wparam, lparam)
    }
}
