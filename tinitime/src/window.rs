#![allow(unused_must_use)]

use std::{ffi::c_void, ptr::null_mut};

use windows::{
    core::{w, Result, HSTRING, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::{
            Gdi::{
                BeginPaint, CreateCompatibleBitmap, CreateCompatibleDC, CreateFontW,
                CreateSolidBrush, DeleteDC, DeleteObject, DrawTextW, EndPaint, FillRect,
                GetStockObject, RedrawWindow, SelectObject, SetBkMode, SetTextColor,
                CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_QUALITY, DT_CENTER, DT_SINGLELINE,
                DT_VCENTER, HBRUSH, HDC, HFONT, HGDIOBJ, NULL_BRUSH, OUT_DEFAULT_PRECIS,
                PAINTSTRUCT, RDW_INVALIDATE, RDW_UPDATENOW, SRCCOPY, TRANSPARENT,
            },
            GdiPlus::{
                GdipCreateFromHDC, GdipCreatePen1, GdipDeleteGraphics, GdipDeletePen,
                GdipDrawRectangleI, GpGraphics, GpPen, UnitPixel,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{RegisterHotKey, MOD_CONTROL, VK_F12},
            WindowsAndMessaging::*,
        },
    },
};

const WINDOW_CLASS_NAME: PCWSTR = w!("rxcle.tinitime.wc");
const IDT_TIMER: usize = 1;
const IDH_HOTKEY: i32 = 100;

const DEF_TIME: i32 = 1500;

const WIN_WIDTH: i32 = 68;
const WIN_HEIGHT: i32 = 25;

pub struct Window {
    handle: HWND,
    font: HFONT,
    fgbrush: HBRUSH,
    fgactive_brush: HBRUSH,
    fgstopped_brush: HBRUSH,
    time_left: i32,
    timer_active: bool,
    window_active: bool,
    client_rect: RECT,
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
                time_left: DEF_TIME,
                timer_active: false,
                window_active: false,
                client_rect: RECT {
                    left: 0,
                    top: 0,
                    right: WIN_WIDTH,
                    bottom: WIN_HEIGHT,
                },
            });

            let hinstance: HINSTANCE = instance.into();
            let handle = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_PALETTEWINDOW | WS_EX_LAYERED,
                WINDOW_CLASS_NAME,
                &HSTRING::from(title),
                WS_POPUP | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                WIN_WIDTH,
                WIN_HEIGHT,
                None,
                None,
                Some(hinstance),
                Some(window.as_mut() as *mut _ as _),
            )?;

            SetLayeredWindowAttributes(handle, COLORREF::default(), 230, LWA_ALPHA);

            window.init_window(handle);
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

        RegisterHotKey(Some(self.handle), IDH_HOTKEY, MOD_CONTROL, VK_F12.0 as u32);
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

    unsafe fn paint(&mut self, ps: PAINTSTRUCT, hdc: HDC) {
        let width = self.client_rect.right - self.client_rect.left;
        let height = self.client_rect.bottom - self.client_rect.top;

        let mem_dc = CreateCompatibleDC(Some(hdc));
        let mem_bitmap = CreateCompatibleBitmap(hdc, width, height);
        let old_bitmap = SelectObject(mem_dc, mem_bitmap.into());

        let (bg, fg) = if self.window_active {
            (self.fgactive_brush, COLORREF(0x00FFFFFF))
        } else if self.timer_active {
            (self.fgbrush, COLORREF(0x00000000))
        } else {
            (self.fgstopped_brush, COLORREF(0x00FFFFFF))
        };
        FillRect(mem_dc, &ps.rcPaint, bg);

        SelectObject(mem_dc, HGDIOBJ::from(self.font));
        SetTextColor(mem_dc, fg);
        SetBkMode(mem_dc, TRANSPARENT);

        SelectObject(mem_dc, GetStockObject(NULL_BRUSH));

        let minutes = self.time_left / 60;
        let seconds = self.time_left % 60;
        let mut time_left_str: Vec<u16> = format!("{:0}:{:02}", minutes, seconds)
            .encode_utf16()
            .collect();

        let mut rtime = RECT {
            left: self.client_rect.left + 15,
            top: self.client_rect.top,
            right: self.client_rect.right,
            bottom: self.client_rect.bottom,
        };

        DrawTextW(
            mem_dc,
            time_left_str.as_mut_slice(),
            &mut rtime,
            DT_SINGLELINE | DT_VCENTER | DT_CENTER,
        );

        let state_str = if self.timer_active {
            "\u{E102}"
        } else {
            "\u{E103}"
        };
        let mut state_symbol: Vec<u16> = state_str.encode_utf16().collect();

        let mut ricon = RECT {
            left: self.client_rect.left,
            top: self.client_rect.top,
            right: 15,
            bottom: self.client_rect.bottom,
        };

        DrawTextW(
            mem_dc,
            state_symbol.as_mut_slice(),
            &mut ricon,
            DT_SINGLELINE | DT_VCENTER,
        );

        let mut graphics: *mut GpGraphics = null_mut();
        GdipCreateFromHDC(mem_dc, &mut graphics);

        let grey: u32 = (0x30 << 24) | (0x00 << 16) | (0x00 << 8) | 0x00;
        let mut pen: *mut GpPen = null_mut();
        GdipCreatePen1(grey, 1.0, UnitPixel, &mut pen);

        // Draw rectangle at (10,10) with width 100 and height 50.
        GdipDrawRectangleI(
            graphics,
            pen,
            0,
            0,
            self.client_rect.right - 1,
            self.client_rect.bottom - 1,
        );

        windows::Win32::Graphics::Gdi::BitBlt(
            hdc,
            0,
            0,
            width,
            height,
            Some(mem_dc),
            0,
            0,
            SRCCOPY,
        );

        // Cleanup
        GdipDeletePen(pen);
        GdipDeleteGraphics(graphics);
        SelectObject(mem_dc, old_bitmap);
        DeleteObject(mem_bitmap.into());
        DeleteDC(mem_dc);
    }

    unsafe fn reset(&mut self) {
        self.stop_timer();
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
            window_rect.bottom - WIN_HEIGHT - 5,
            0,
            0,
            SWP_NOSIZE,
        );
    }

    unsafe fn activate_window(&mut self, activate: bool) {
        self.window_active = activate;
        self.refresh();
    }

    unsafe fn start_timer(&mut self) {
        if self.timer_active {
            self.stop_timer();
        }
        self.timer_active = true;
        SetTimer(Some(self.handle), IDT_TIMER, 1000, None);
        self.update_timer(DEF_TIME);
    }

    unsafe fn stop_timer(&mut self) {
        KillTimer(Some(self.handle), IDT_TIMER);
        self.timer_active = false;
        self.update_timer(DEF_TIME);
    }

    unsafe fn toggle_timer(&mut self) {
        if self.timer_active {
            self.stop_timer();
        } else {
            self.start_timer();
        }
    }

    unsafe fn update_timer(&mut self, new_time: i32) {
        if new_time < 0 {
            self.stop_timer();
        } else {
            self.time_left = new_time;
            self.refresh();
        }
    }

    unsafe fn refresh(&mut self) {
        RedrawWindow(
            Some(self.handle),
            None,
            None,
            RDW_INVALIDATE | RDW_UPDATENOW,
        );
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
            WM_HOTKEY => {
                self.toggle_timer();
                LRESULT(0)
            }
            WM_TIMER => {
                self.update_timer(self.time_left - 1);
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let psp = &mut ps as *mut PAINTSTRUCT;
                let hdc = BeginPaint(self.handle, psp);
                self.paint(ps, hdc);
                EndPaint(self.handle, &ps);
                LRESULT(0)
            }
            WM_ERASEBKGND => LRESULT(1),
            WM_NCHITTEST => {
                let result = DefWindowProcW(self.handle, message, wparam, lparam);
                if result.0 == HTCLIENT as isize {
                    LRESULT(HTCAPTION as isize)
                } else {
                    result
                }
            }
            WM_NCRBUTTONDOWN => {
                self.reset_pos();
                LRESULT(0)
            }
            WM_NCLBUTTONDBLCLK => {
                self.toggle_timer();
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
            let cs: *const CREATESTRUCTW = lparam.0 as *const CREATESTRUCTW;
            let this = (*cs).lpCreateParams as *mut Self;
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
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
}
