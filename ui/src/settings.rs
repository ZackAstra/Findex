/// Findex Settings Window
use crate::win32::*;

pub struct SettingsWindow {
    hwnd: HWND,
    hinstance: HINSTANCE,
}

impl SettingsWindow {
    pub fn new(hinstance: HINSTANCE) -> Self {
        SettingsWindow { hwnd: std::ptr::null_mut(), hinstance }
    }

    pub fn create(&mut self) -> HWND {
        let class_name = to_wstring("FindexSettingsClass");

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

        let window_name = to_wstring("Findex 设置");
        let hwnd = unsafe {
            CreateWindowExW(
                0, class_name.as_ptr(), window_name.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                200, 200, 520, 420,
                std::ptr::null_mut(), std::ptr::null_mut(), self.hinstance, std::ptr::null_mut()
            )
        };

        self.hwnd = hwnd;
        hwnd
    }

    pub fn show(&self) {
        if !self.hwnd.is_null() {
            unsafe {
                ShowWindow(self.hwnd, SW_SHOWNORMAL);
                SetForegroundWindow(self.hwnd);
            }
        }
    }

    fn add_control(
        parent: HWND, class: &str, text: &str,
        style: DWORD, x: i32, y: i32, w: i32, h: i32, id: i32,
    ) -> HWND {
        let class_w = to_wstring(class);
        let text_w = to_wstring(text);
        unsafe {
            CreateWindowExW(
                0, class_w.as_ptr(), text_w.as_ptr(),
                WS_CHILD | WS_VISIBLE | style,
                x, y, w, h,
                parent, id as HMENU,
                GetModuleHandleW(std::ptr::null()),
                std::ptr::null_mut(),
            )
        }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
                // Title
                let title_font = CreateFontW(
                    -18, 0, 0, 0, 700, 0, 0, 0, 0, 0, 0, 0, 0,
                    to_wstring("Segoe UI").as_ptr(),
                );
                let normal_font = CreateFontW(
                    -13, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0,
                    to_wstring("Segoe UI").as_ptr(),
                );

                // Title
                let title = Self::add_control(hwnd, "Static", "Findex 设置", SS_LEFT, 15, 10, 470, 25, 100);
                SendMessageW(title, WM_SETFONT, title_font as WPARAM, 1);

                // Index section
                let section = Self::add_control(hwnd, "Static", "索引目录", SS_LEFT, 15, 45, 470, 20, 101);
                SendMessageW(section, WM_SETFONT, normal_font as WPARAM, 1);

                Self::add_control(hwnd, "Edit", "D:\\Findows", ES_LEFT | WS_BORDER | ES_AUTOHSCROLL, 15, 68, 350, 24, 200);
                Self::add_control(hwnd, "Button", "浏览...", BS_PUSHBUTTON, 375, 68, 60, 24, 201);
                Self::add_control(hwnd, "Button", "添加", BS_PUSHBUTTON, 440, 68, 50, 24, 202);

                // Indexed paths list
                let listbox = Self::add_control(hwnd, "ListBox", "", LBS_NOTIFY | WS_VSCROLL | WS_BORDER, 15, 98, 475, 80, 203);
                SendMessageW(listbox, WM_SETFONT, normal_font as WPARAM, 1);

                // Hotkey section
                let section2 = Self::add_control(hwnd, "Static", "快捷键", SS_LEFT, 15, 190, 470, 20, 102);
                SendMessageW(section2, WM_SETFONT, normal_font as WPARAM, 1);

                Self::add_control(hwnd, "Static", "搜索浮窗:", SS_LEFT, 15, 215, 80, 22, 103);
                Self::add_control(hwnd, "Edit", "Ctrl + Space", ES_LEFT | WS_BORDER | ES_READONLY, 100, 215, 120, 24, 204);

                Self::add_control(hwnd, "Static", "设置窗口:", SS_LEFT, 240, 215, 80, 22, 104);
                Self::add_control(hwnd, "Edit", "Ctrl + Shift + F", ES_LEFT | WS_BORDER | ES_READONLY, 320, 215, 120, 24, 205);

                // Search options section
                let section3 = Self::add_control(hwnd, "Static", "搜索选项", SS_LEFT, 15, 250, 470, 20, 105);
                SendMessageW(section3, WM_SETFONT, normal_font as WPARAM, 1);

                Self::add_control(hwnd, "Button", "启用拼音搜索", BS_AUTOCHECKBOX, 15, 275, 150, 22, 300);
                Self::add_control(hwnd, "Button", "启用模糊匹配", BS_AUTOCHECKBOX, 180, 275, 150, 22, 301);
                Self::add_control(hwnd, "Button", "显示隐藏文件", BS_AUTOCHECKBOX, 345, 275, 150, 22, 302);

                // Status bar
                let status = Self::add_control(hwnd, "Static",
                    "状态: 就绪 | 索引: 0 个文件 | 0 个目录",
                    SS_LEFT | WS_BORDER, 15, 310, 475, 28, 400);
                SendMessageW(status, WM_SETFONT, normal_font as WPARAM, 1);

                // Buttons
                Self::add_control(hwnd, "Button", "立即索引", BS_PUSHBUTTON, 15, 345, 90, 28, 500);
                Self::add_control(hwnd, "Button", "保存", BS_PUSHBUTTON | BS_DEFPUSHBUTTON, 320, 345, 70, 28, 501);
                Self::add_control(hwnd, "Button", "取消", BS_PUSHBUTTON, 400, 345, 70, 28, 502);

                0
            }
            WM_COMMAND => {
                let id = loword(wparam as DWORD) as i32;
                match id {
                    502 => { // Cancel
                        DestroyWindow(hwnd);
                    }
                    _ => {}
                }
                0
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                // Don't PostQuitMessage - only the main window should do that
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}


