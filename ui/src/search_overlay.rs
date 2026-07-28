/// Findex Search Floating Overlay Window
/// A borderless, always-on-top search window that appears on hotkey.

use crate::win32::*;

pub struct SearchOverlay {
    hwnd: HWND,
    hinstance: HINSTANCE,
    edit_hwnd: HWND,
    list_hwnd: HWND,
    visible: bool,
}

impl SearchOverlay {
    pub fn new(hinstance: HINSTANCE) -> Self {
        SearchOverlay {
            hwnd: std::ptr::null_mut(),
            hinstance,
            edit_hwnd: std::ptr::null_mut(),
            list_hwnd: std::ptr::null_mut(),
            visible: false,
        }
    }

    pub fn create(&mut self) -> HWND {
        let class_name = to_wstring("FindexSearchOverlayClass");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0,
            lpfnWndProc: Some(Self::wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: self.hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: unsafe { LoadCursorW(std::ptr::null_mut(), IDC_ARROW) },
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };

        unsafe { RegisterClassExW(&wc); }

        // Get screen dimensions
        let screen_w = unsafe { GetSystemMetrics(0) }; // SM_CXSCREEN
        let screen_h = unsafe { GetSystemMetrics(1) }; // SM_CYSCREEN
        let win_w = 500;
        let win_h = 400;
        let x = (screen_w - win_w) / 2;
        let y = (screen_h - win_h) / 3;

        let window_name = to_wstring("Findex Search");
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_NOACTIVATE,
                class_name.as_ptr(), window_name.as_ptr(),
                WS_POPUP | WS_BORDER | WS_CLIPCHILDREN,
                x, y, win_w, win_h,
                std::ptr::null_mut(), std::ptr::null_mut(), self.hinstance, std::ptr::null_mut()
            )
        };

        // Set opacity
        unsafe { SetWindowLongW(hwnd, -20, (GetWindowLongW(hwnd, -20) | WS_EX_LAYERED as i32) as i32); } // GWL_EXSTYLE

        self.hwnd = hwnd;
        hwnd
    }

    pub fn toggle(&mut self) {
        if self.visible {
            self.hide();
        } else {
            self.show();
        }
    }

    pub fn show(&mut self) {
        if !self.hwnd.is_null() {
            unsafe {
                // Center on screen
                let screen_w = GetSystemMetrics(0);
                let screen_h = GetSystemMetrics(1);
                let x = (screen_w - 500) / 2;
                let y = (screen_h - 400) / 3;
                SetWindowPos(self.hwnd, std::ptr::null_mut(), x, y, 500, 400, SWP_NOZORDER | SWP_SHOWWINDOW);
                SetForegroundWindow(self.hwnd);
                SetFocus(self.edit_hwnd);
            }
            self.visible = true;
        }
    }

    pub fn hide(&mut self) {
        if !self.hwnd.is_null() {
            unsafe {
                ShowWindow(self.hwnd, SW_HIDE);
            }
            self.visible = false;
        }
    }

    pub fn is_visible(&self) -> bool { self.visible }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                // Search input
                let edit = CreateWindowExW(
                    0, to_wstring("Edit").as_ptr(), to_wstring("").as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                    8, 8, 474, 28, hwnd, 1000 as HMENU,
                    GetModuleHandleW(std::ptr::null()), std::ptr::null_mut(),
                );
                let font = CreateFontW(-16, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0, to_wstring("Segoe UI").as_ptr());
                SendMessageW(edit, WM_SETFONT, font as WPARAM, 1);

                // Results list
                let list = CreateWindowExW(
                    0, to_wstring("ListBox").as_ptr(), to_wstring("").as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER | LBS_NOTIFY | LBS_NOINTEGRALHEIGHT,
                    8, 42, 484, 348, hwnd, 1001 as HMENU,
                    GetModuleHandleW(std::ptr::null()), std::ptr::null_mut(),
                );
                let list_font = CreateFontW(-14, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0, to_wstring("Consolas").as_ptr());
                SendMessageW(list, WM_SETFONT, list_font as WPARAM, 1);

                // Store handles
                SetWindowLongPtrW(hwnd, 0, edit as isize);
                SetWindowLongPtrW(hwnd, 8, list as isize);

                // Set focus to edit
                SetFocus(edit);
                0
            }
            WM_SETFOCUS => {
                let edit = GetWindowLongPtrW(hwnd, 0) as HWND;
                if !edit.is_null() {
                    SetFocus(edit);
                }
                0
            }
            WM_COMMAND => {
                let id = loword(wparam as DWORD) as i32;
                let code = hiword(wparam as DWORD) as UINT;
                match id {
                    1000 => { // Edit control
                        if code == EN_CHANGE {
                            // Get text and search
                            let len = GetWindowTextLengthW(lparam as HWND);
                            if len > 0 {
                                let mut buf = vec![0u16; (len + 1) as usize];
                                GetWindowTextW(lparam as HWND, buf.as_mut_ptr(), len + 1);
                                let query = from_wstring(buf.as_ptr());
                                // TODO: Perform search using engine
                                let list = GetWindowLongPtrW(hwnd, 8) as HWND;
                                SendMessageW(list, LB_RESETCONTENT, 0, 0);
                                let placeholder = format!("搜索: {}", query);
                                let w = to_wstring(&placeholder);
                                SendMessageW(list, LB_ADDSTRING, 0, w.as_ptr() as LPARAM);
                            }
                        }
                    }
                    1001 => { // ListBox
                        if code == LBN_DBLCLK {
                            // TODO: Open selected file
                            DestroyWindow(hwnd);
                        }
                    }
                    _ => {}
                }
                0
            }
            WM_KEYDOWN => {
                if wparam == VK_ESCAPE as usize {
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_CHAR => {
                if wparam == VK_ESCAPE as usize {
                    DestroyWindow(hwnd);
                }
                0
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                // Don't quit app - just hide
                0
            }
            WM_NCHITTEST => {
                // Allow dragging by caption area
                let result = DefWindowProcW(hwnd, msg, wparam, lparam);
                if result == 0 {
                    // Return HTCAPTION to allow dragging
                    return 2; // HTCAPTION
                }
                result
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}


