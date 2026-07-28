/// Findex Search Floating Overlay Window
/// A borderless, always-on-top search window that appears on hotkey.
/// Connected to the global engine Searcher for real-time file search.

use crate::win32::*;
use crate::SEARCHER;

use std::sync::Mutex;

/// Stores the current search results so the list box items can be mapped to file paths.
static CURRENT_RESULTS: Mutex<Vec<findex_engine::SearchResult>> = Mutex::new(Vec::new());

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

    pub fn hwnd(&self) -> HWND {
        self.hwnd
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

        let screen_w = unsafe { GetSystemMetrics(0) };
        let screen_h = unsafe { GetSystemMetrics(1) };
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

        self.hwnd = hwnd;
        self.visible = false;
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
                let screen_w = GetSystemMetrics(0);
                let screen_h = GetSystemMetrics(1);
                let x = (screen_w - 500) / 2;
                let y = (screen_h - 400) / 3;
                SetWindowPos(self.hwnd, std::ptr::null_mut(), x, y, 500, 400, SWP_NOZORDER | SWP_SHOWWINDOW);
                SetForegroundWindow(self.hwnd);
                if !self.edit_hwnd.is_null() {
                    SetFocus(self.edit_hwnd);
                }
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

    /// Open the file at the given list box index via ShellExecuteW.
    unsafe fn open_item_at_index(hwnd: HWND, index: i32) {
        if index < 0 { return; }
        let results = CURRENT_RESULTS.lock().unwrap();
        if (index as usize) < results.len() {
            let entry = &results[index as usize].entry;
            let path = to_wstring(&entry.path);
            ShellExecuteW(
                std::ptr::null_mut(),
                to_wstring("open").as_ptr(),
                path.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
            ShowWindow(hwnd, SW_HIDE);
        }
    }

    /// Perform a search and populate the list box.
    unsafe fn perform_search(hwnd: HWND, query: &str) {
        let list = GetWindowLongPtrW(hwnd, 8) as HWND;
        SendMessageW(list, LB_RESETCONTENT, 0, 0);

        if query.is_empty() {
            CURRENT_RESULTS.lock().unwrap().clear();
            return;
        }

        let results = SEARCHER.lock().unwrap().as_ref().map(|searcher| {
            searcher.search(&findex_engine::SearchQuery {
                query: query.to_string(),
                scope: findex_engine::SearchScope::Global,
                context_path: None,
                max_results: 100,
                offset: 0,
                sort_by: findex_engine::SortBy::Relevance,
            })
        }).unwrap_or_default();

        *CURRENT_RESULTS.lock().unwrap() = results.clone();

        for result in &results {
            let entry = &result.entry;
            let display = if entry.is_dir {
                format!("[DIR] {}  ({})", entry.name, entry.path)
            } else {
                format!("{}  ({})", entry.name, entry.path)
            };
            let w = to_wstring(&display);
            SendMessageW(list, LB_ADDSTRING, 0, w.as_ptr() as LPARAM);
        }

        if !results.is_empty() {
            SendMessageW(list, LB_SETCURSEL, 0, 0);
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                let edit = CreateWindowExW(
                    0, to_wstring("Edit").as_ptr(), to_wstring("").as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                    8, 8, 474, 28, hwnd, 1000 as HMENU,
                    GetModuleHandleW(std::ptr::null()), std::ptr::null_mut(),
                );
                let font = CreateFontW(-16, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0, to_wstring("Segoe UI").as_ptr());
                SendMessageW(edit, WM_SETFONT, font as WPARAM, 1);

                // Subclass edit control for keyboard navigation
                let original_edit_proc = SetWindowLongPtrW(edit, GWLP_WNDPROC, edit_subclass_proc as *const () as isize);
                SetWindowLongPtrW(edit, GWLP_USERDATA, original_edit_proc);

                let list = CreateWindowExW(
                    0, to_wstring("ListBox").as_ptr(), to_wstring("").as_ptr(),
                    WS_CHILD | WS_VISIBLE | WS_VSCROLL | WS_BORDER | LBS_NOTIFY | LBS_NOINTEGRALHEIGHT,
                    8, 42, 484, 348, hwnd, 1001 as HMENU,
                    GetModuleHandleW(std::ptr::null()), std::ptr::null_mut(),
                );
                let list_font = CreateFontW(-14, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0, to_wstring("Consolas").as_ptr());
                SendMessageW(list, WM_SETFONT, list_font as WPARAM, 1);

                // Subclass list box for keyboard navigation
                let original_list_proc = SetWindowLongPtrW(list, GWLP_WNDPROC, list_subclass_proc as *const () as isize);
                SetWindowLongPtrW(list, GWLP_USERDATA, original_list_proc);

                SetWindowLongPtrW(hwnd, 0, edit as isize);
                SetWindowLongPtrW(hwnd, 8, list as isize);

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
                    1000 => {
                        if code == EN_CHANGE {
                            let len = GetWindowTextLengthW(lparam as HWND);
                            if len > 0 {
                                let mut buf = vec![0u16; (len + 1) as usize];
                                GetWindowTextW(lparam as HWND, buf.as_mut_ptr(), len + 1);
                                let query = from_wstring(buf.as_ptr());
                                Self::perform_search(hwnd, &query);
                            } else {
                                let list = GetWindowLongPtrW(hwnd, 8) as HWND;
                                SendMessageW(list, LB_RESETCONTENT, 0, 0);
                                CURRENT_RESULTS.lock().unwrap().clear();
                            }
                        }
                    }
                    1001 => {
                        if code == LBN_DBLCLK {
                            let sel = SendMessageW(lparam as HWND, LB_GETCURSEL, 0, 0) as i32;
                            Self::open_item_at_index(hwnd, sel);
                        }
                    }
                    _ => {}
                }
                0
            }
            WM_CLOSE => {
                ShowWindow(hwnd, SW_HIDE);
                0
            }
            WM_DESTROY => {
                0
            }
            WM_NCHITTEST => {
                let result = DefWindowProcW(hwnd, msg, wparam, lparam);
                if result == 0 {
                    return 2;
                }
                result
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Subclass procedure for the Edit control.
/// Enter = open first result, Down = move to list, Esc = hide overlay.
unsafe extern "system" fn edit_subclass_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    let original_proc = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as LPVOID;

    match msg {
        WM_KEYDOWN => {
            match wparam as i32 {
                VK_RETURN => {
                    let parent = GetParent(hwnd);
                    if !parent.is_null() {
                        SearchOverlay::open_item_at_index(parent, 0);
                    }
                    return 0;
                }
                VK_DOWN => {
                    let parent = GetParent(hwnd);
                    if !parent.is_null() {
                        let list = GetWindowLongPtrW(parent, 8) as HWND;
                        if !list.is_null() {
                            SendMessageW(list, LB_SETCURSEL, 0, 0);
                            SetFocus(list);
                        }
                    }
                    return 0;
                }
                VK_ESCAPE => {
                    let parent = GetParent(hwnd);
                    if !parent.is_null() {
                        ShowWindow(parent, SW_HIDE);
                    }
                    return 0;
                }
                _ => {}
            }
        }
        _ => {}
    }

    CallWindowProcW(original_proc, hwnd, msg, wparam, lparam)
}

/// Subclass procedure for the ListBox control.
/// Enter = open selected, Up at top = move to edit, Esc = hide overlay.
unsafe extern "system" fn list_subclass_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    let original_proc = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as LPVOID;

    match msg {
        WM_KEYDOWN => {
            match wparam as i32 {
                VK_RETURN => {
                    let sel = SendMessageW(hwnd, LB_GETCURSEL, 0, 0) as i32;
                    let parent = GetParent(hwnd);
                    if !parent.is_null() {
                        SearchOverlay::open_item_at_index(parent, sel);
                    }
                    return 0;
                }
                VK_UP => {
                    let sel = SendMessageW(hwnd, LB_GETCURSEL, 0, 0) as i32;
                    if sel <= 0 {
                        let parent = GetParent(hwnd);
                        if !parent.is_null() {
                            let edit = GetWindowLongPtrW(parent, 0) as HWND;
                            if !edit.is_null() {
                                SetFocus(edit);
                            }
                        }
                        return 0;
                    }
                }
                VK_ESCAPE => {
                    let parent = GetParent(hwnd);
                    if !parent.is_null() {
                        ShowWindow(parent, SW_HIDE);
                    }
                    return 0;
                }
                _ => {}
            }
        }
        _ => {}
    }

    CallWindowProcW(original_proc, hwnd, msg, wparam, lparam)
}
