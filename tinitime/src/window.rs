use std::{ffi::c_void, sync::Once};

use windows::{
    core::{w, Result, HSTRING, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::Gdi::{
            BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextA, EndPaint, FillRect,
            RedrawWindow, SelectObject, SetBkMode, SetTextColor, CLIP_DEFAULT_PRECIS,
            DEFAULT_CHARSET, DEFAULT_QUALITY, DT_CENTER, DT_SINGLELINE, DT_VCENTER, HBRUSH, HDC,
            HFONT, HGDIOBJ, OUT_DEFAULT_PRECIS, PAINTSTRUCT, RDW_INVALIDATE, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{RegisterHotKey, MOD_CONTROL, VK_F12},
            WindowsAndMessaging::*,
        },
    },
};

static REGISTER_WINDOW_CLASS: Once = Once::new();
const WINDOW_CLASS_NAME: PCWSTR = w!("rxcle-tinitime");
const IDT_TIMER: usize = 1;
const IDH_HOTKEY: i32 = 100;

const WIN_WIDTH: i32 = 80;
const WIN_HEIGHT: i32 = 25;

pub struct Window {
    handle: HWND,
    font: HFONT,
    fgbrush: HBRUSH,
    time_left: u32,
    active: bool,
}

impl Window {
    pub fn new(title: &str) -> Result<Box<Self>> {
        unsafe {
            let instance = GetModuleHandleW(None)?;

            REGISTER_WINDOW_CLASS.call_once(|| {
                let wc = WNDCLASSW {
                    hCursor: LoadCursorW(None, IDC_ARROW).ok().unwrap(),
                    hInstance: instance.into(),
                    lpszClassName: WINDOW_CLASS_NAME,
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(Self::wnd_proc),
                    ..Default::default()
                };
                let atom = RegisterClassW(&wc);
                debug_assert!(atom != 0);
            });

            let mut result = Box::new(Self {
                handle: HWND::default(),
                font: HFONT::default(),
                fgbrush: HBRUSH::default(),
                time_left: 1500,
                active: false,
            });

            let hinstance: HINSTANCE = instance.into();
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_PALETTEWINDOW,
                WINDOW_CLASS_NAME,
                &HSTRING::from(title),
                WS_POPUP | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                80,
                25,
                None,
                None,
                Some(hinstance),
                Some(result.as_mut() as *mut _ as _),
            )?;

            let _ = SetLayeredWindowAttributes(hwnd, COLORREF::default(), 180, LWA_ALPHA);

            let mut window_rect = RECT::default();
            let _ = SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                Some(&mut window_rect as *mut _ as *mut c_void),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );

            let _ = SetWindowPos(
                hwnd,
                None,
                window_rect.right - WIN_WIDTH,
                window_rect.bottom - WIN_HEIGHT,
                0,
                0,
                SWP_NOSIZE,
            );

            _ = ShowWindow(hwnd, SW_SHOW);

            Ok(result)
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
            w!("Consolas"),
        );
        self.fgbrush = CreateSolidBrush(COLORREF(0x00FFFFFF));

        self.start_timer();

        //let result = RegisterHotKey(Some(self.handle), IDH_HOTKEY, MOD_CONTROL, VK_F12.0 as u32);
        //println!("RegisterHotKey result: {:?}", result);
    }

    unsafe fn destroy_window(&mut self) {
        PostQuitMessage(0);
        self.handle = HWND::default();
        _ = DeleteObject(HGDIOBJ::from(self.font));
        self.font = HFONT::default();
        _ = DeleteObject(HGDIOBJ::from(self.fgbrush));
        self.fgbrush = HBRUSH::default();
    }

    unsafe fn paint(&mut self, ps: PAINTSTRUCT, rp: *mut RECT, hdc: HDC) {
        FillRect(hdc, &ps.rcPaint, self.fgbrush);

        SelectObject(hdc, HGDIOBJ::from(self.font));
        SetTextColor(hdc, COLORREF(0x00000000));
        SetBkMode(hdc, TRANSPARENT);

        let minutes = self.time_left / 60;
        let seconds = self.time_left % 60;
        let mut time_left_str = format!("{:02}:{:02}", minutes, seconds);

        DrawTextA(
            hdc,
            time_left_str.as_bytes_mut(),
            rp,
            DT_SINGLELINE | DT_CENTER | DT_VCENTER,
        );
    }

    unsafe fn start_timer(&mut self) {
        let _ = SetTimer(Some(self.handle), IDT_TIMER, 1000, None);
    }

    unsafe fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_DESTROY => {
                self.destroy_window();
                LRESULT(0)
            }
            WM_HOTKEY => {
                println!("Hotkey pressed!");
                self.start_timer();
                LRESULT(0)
            }
            WM_TIMER => {
                self.time_left = self.time_left - 1;
                _ = RedrawWindow(Some(self.handle), None, None, RDW_INVALIDATE);
                LRESULT(0)
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let psp = &mut ps as *mut PAINTSTRUCT;
                let rp = &mut ps.rcPaint as *mut RECT;
                let hdc = BeginPaint(self.handle, psp);
                self.paint(ps, rp, hdc);
                _ = EndPaint(self.handle, &ps);
                LRESULT(0)
            }
            WM_NCHITTEST => {
                let result = DefWindowProcW(self.handle, message, wparam, lparam);
                return if result.0 == HTCLIENT as isize {
                    LRESULT(HTCAPTION as isize)
                } else {
                    result
                };
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
