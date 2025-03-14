#![allow(unused_must_use)]

use std::{collections::HashMap, ffi::c_void};

use windows::{
    core::{w, Result, HSTRING, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::Gdi::{
            BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW,
            CreateSolidBrush, DeleteDC, DeleteObject, EndPaint, FillRect, GetDC, GetDeviceCaps,
            GetTextExtentPoint32W, InvalidateRect, ReleaseDC, SelectObject, SetBkMode,
            SetTextColor, TextOutW, UpdateWindow, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
            DEFAULT_QUALITY, FW_MEDIUM, HBRUSH, HDC, HFONT, HGDIOBJ, LOGPIXELSY,
            OUT_DEFAULT_PRECIS, PAINTSTRUCT, SRCCOPY, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};

use crate::{
    helpers::{determine_key_pressed, hiword, loword, mul_div_round, to_lpcwstr},
    keys::{Keychain, ScanCode, SC_BACK, SC_ESCAPE},
};

const WINDOW_CLASS_NAME: PCWSTR = w!("rxcle.skproto.wc");
const COLOR_KEY: COLORREF = COLORREF(0x00FF00FF);

const WIN_WIDTH: i32 = 200;
const WIN_HEIGHT: i32 = 25;

pub struct Window {
    handle: HWND,
    font: HFONT,
    fgbrush: HBRUSH,
    fgactive_brush: HBRUSH,
    fgstopped_brush: HBRUSH,
    transparent_brush: HBRUSH,
    window_active: bool,
    size: SIZE,
    keychain: Keychain,
    key_render_sizes: HashMap<ScanCode, SIZE>,
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
                transparent_brush: HBRUSH::default(),
                window_active: false,
                size: SIZE {
                    cx: WIN_WIDTH,
                    cy: WIN_HEIGHT,
                },
                keychain: Keychain::new(),
                key_render_sizes: HashMap::new(),
            });

            let hinstance: HINSTANCE = instance.into();
            let handle = CreateWindowExW(
                WS_EX_APPWINDOW | WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT,
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

            SetLayeredWindowAttributes(handle, COLOR_KEY, 220, LWA_ALPHA | LWA_COLORKEY);

            window.reset();

            Ok(window)
        }
    }

    unsafe fn init_window(&mut self, window: HWND) {
        self.handle = window;
        let dc = GetDC(Some(window));
        let font_size = -mul_div_round(12, GetDeviceCaps(Some(dc), LOGPIXELSY), 72);
        ReleaseDC(Some(window), dc);
        self.font = CreateFontW(
            font_size,
            0,
            0,
            0,
            FW_MEDIUM.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            DEFAULT_QUALITY,
            0,
            w!("Consolas"),
        );
        self.fgbrush = CreateSolidBrush(COLORREF(0x00FFFFFF));
        self.fgactive_brush = CreateSolidBrush(COLORREF(0x00D7792B));
        self.fgstopped_brush = CreateSolidBrush(COLORREF(0x002B31D7));
        self.transparent_brush = CreateSolidBrush(COLOR_KEY);
    }

    fn destroy_window(&mut self) {
        unsafe {
            PostQuitMessage(0);
            DeleteObject(HGDIOBJ::from(self.font));
            DeleteObject(HGDIOBJ::from(self.fgbrush));
            DeleteObject(HGDIOBJ::from(self.fgactive_brush));
            DeleteObject(HGDIOBJ::from(self.transparent_brush));
        }
        self.handle = HWND::default();
        self.font = HFONT::default();
        self.fgbrush = HBRUSH::default();
        self.fgactive_brush = HBRUSH::default();
        self.transparent_brush = HBRUSH::default();
    }

    unsafe fn paint(&mut self, hdc: HDC) {
        let width = self.size.cx;
        let height = self.size.cy;

        let mem_dc = CreateCompatibleDC(Some(hdc));
        let mem_bitmap = CreateCompatibleBitmap(hdc, width, height);
        let old_bitmap = SelectObject(mem_dc, mem_bitmap.into());

        let (bg, fg) = (self.fgactive_brush, COLORREF(0x00FFFFFF));

        let rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };

        FillRect(mem_dc, &rect, self.transparent_brush);

        SelectObject(mem_dc, HGDIOBJ::from(self.font));
        SetTextColor(mem_dc, fg);
        SetBkMode(mem_dc, TRANSPARENT);

        let mut x = 0;
        for key in &self.keychain.keys {
            self.keychain
                .key_infos
                .get(&key.scan_code)
                .map_or((), |key_info| {
                    let text_size = self
                        .key_render_sizes
                        .get(&key.scan_code)
                        .map_or_else(|| self.measure_text(&key_info.name), |text_size| *text_size);

                    let rect = RECT {
                        left: x,
                        top: 0,
                        right: x + text_size.cx + 10,
                        bottom: self.size.cy,
                    };

                    FillRect(mem_dc, &rect, bg);
                    TextOutW(mem_dc, x + 5, 0, &to_lpcwstr(&key_info.name));
                    x += text_size.cx + 15;
                });
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
            0, // window_rect.bottom - WIN_HEIGHT - 5,
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
            let required_size = self.determine_required_size();

            let mut screen_rect = RECT::default();
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                Some(&mut screen_rect as *mut _ as *mut c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );

            SetWindowPos(
                self.handle,
                None,
                screen_rect.left + ((screen_rect.right - screen_rect.left) - required_size.cx) / 2,
                0,
                required_size.cx,
                required_size.cy,
                SWP_NOZORDER,
            );
            InvalidateRect(Some(self.handle), None, false);
            UpdateWindow(self.handle);
        }
    }

    fn determine_required_size(&mut self) -> SIZE {
        SIZE { cx: 300, cy: 200 }
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

    fn handle_key(&mut self, scan_code: ScanCode) {
        println!("{:04X}", &scan_code.0);
        if scan_code == SC_ESCAPE {
            self.keychain.clear();
        } else if scan_code == SC_BACK {
            self.keychain.back();
        } else {
            self.keychain.add(scan_code);
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
            WM_SIZE => {
                self.size = SIZE {
                    cx: loword(lparam.0),
                    cy: hiword(lparam.0),
                };
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
                if let Some(scan_code) = determine_key_pressed(wparam, lparam) {
                    self.handle_key(scan_code);
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
}
