use std::ffi::{c_void, CStr};

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
                BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint,
                FillRect, InvalidateRect, RedrawWindow, SelectObject, SetBkMode, SetTextColor,
                UpdateWindow, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY, DT_CENTER,
                DT_SINGLELINE, DT_VCENTER, HBRUSH, HDC, HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS,
                PAINTSTRUCT, RDW_INVALIDATE, RDW_UPDATENOW, TRANSPARENT,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{GetKeyNameTextA, VIRTUAL_KEY, VK_F12},
            WindowsAndMessaging::*,
        },
    },
};

const WINDOW_CLASS_NAME: PCWSTR = w!("rxcle.skproto.wc");

const WIN_WIDTH: i32 = 100;
const WIN_HEIGHT: i32 = 25;

pub struct Window {
    handle: HWND,
    font: HFONT,
    fgbrush: HBRUSH,
    fgactive_brush: HBRUSH,
    fgstopped_brush: HBRUSH,
    window_active: bool,
    client_rect: RECT,
    keys: Vec<String>,
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
                keys: vec![],
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

            _ = SetLayeredWindowAttributes(handle, COLORREF::default(), 220, LWA_ALPHA);

            let preference = DWM_WINDOW_CORNER_PREFERENCE(3);
            _ = DwmSetWindowAttribute(
                handle,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const _ as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            let enable = 1;
            _ = DwmSetWindowAttribute(
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
        _ = DeleteObject(HGDIOBJ::from(self.font));
        self.font = HFONT::default();
        _ = DeleteObject(HGDIOBJ::from(self.fgbrush));
        self.fgbrush = HBRUSH::default();
        _ = DeleteObject(HGDIOBJ::from(self.fgactive_brush));
        self.fgactive_brush = HBRUSH::default();
    }

    unsafe fn paint(&mut self, ps: PAINTSTRUCT, hdc: HDC) {
        let (bg, fg) = if self.window_active {
            (self.fgactive_brush, COLORREF(0x00FFFFFF))
        } else {
            (self.fgstopped_brush, COLORREF(0x00FFFFFF))
        };
        FillRect(hdc, &ps.rcPaint, bg);

        SelectObject(hdc, HGDIOBJ::from(self.font));
        SetTextColor(hdc, fg);
        SetBkMode(hdc, TRANSPARENT);

        let mut time_left_str: Vec<u16> = "Hello".encode_utf16().collect();
        let last_key = self.keys.last();
        if let Some(a) = last_key {
            time_left_str = a.encode_utf16().collect();
        }

        let mut rtime = RECT {
            left: self.client_rect.left,
            top: self.client_rect.top,
            right: self.client_rect.right,
            bottom: self.client_rect.bottom,
        };

        DrawTextW(
            hdc,
            &mut time_left_str,
            &mut rtime,
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );
    }

    unsafe fn reset(&mut self) {
        self.reset_pos();
        _ = ShowWindow(self.handle, SW_SHOW);
    }

    unsafe fn reset_pos(&mut self) {
        let mut window_rect = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut window_rect as *mut _ as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );

        let _ = SetWindowPos(
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
            _ = InvalidateRect(Some(self.handle), None, false);
            _ = UpdateWindow(self.handle);
        }
    }

    fn handle_key(&mut self, scan_code: u32, name: &str) {
        println!("Key pressed: {}, {}", scan_code, name);
        self.keys.push(String::from(name));
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
                self.paint(ps, hdc);
                _ = EndPaint(self.handle, &ps);
                LRESULT(0)
            }
            WM_KEYDOWN | WM_SYSKEYDOWN => {
                // Check bit 30: if set, it's a repeat.
                if ((lparam.0 >> 30) & 1) != 0 {
                    return LRESULT(0);
                }
                // Extract the virtual key code (wParam)
                let _vk_code = wparam.0 as u32;
                // Extract the scan code from lParam
                let mut scan_code = ((lparam.0 >> 16) & 0xFF) as u32;
                // If the key is extended, set the extended flag.
                if (lparam.0 & (1 << 24)) != 0 {
                    scan_code |= 0x100;
                }
                // GetKeyNameText expects the scan code in the upper 16 bits.
                let lparam_for_key_name = (scan_code << 16) as i32;
                let mut key_name_buf = [0u8; 128];
                let ret = GetKeyNameTextA(lparam_for_key_name, &mut key_name_buf);
                if ret > 0 {
                    // Convert the returned C-string to a Rust string.
                    if let Ok(cstr) = CStr::from_bytes_with_nul(&key_name_buf[..ret as usize + 1]) {
                        if let Ok(key_name) = cstr.to_str() {
                            self.handle_key(scan_code, key_name);
                        }
                    }
                }
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

    pub fn run_message_loop() {
        let mut message = MSG::default();
        unsafe {
            while GetMessageW(&mut message, None, 0, 0).into() {
                _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
}
