#![allow(dead_code)]
/// Findex Settings Window
/// Configuration UI with save/load, path management, and index triggering.

use crate::win32::*;
use crate::config::Config;
use crate::SEARCHER;

use std::sync::Mutex;

/// Global config instance shared between windows.
static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

/// Load the config from disk into the global static.
pub fn load_config() {
    let cfg = Config::load();
    *CONFIG.lock().unwrap() = Some(cfg);
}

/// Get a clone of the current config.
pub fn get_config() -> Config {
    CONFIG.lock().unwrap().clone().unwrap_or_default()
}

pub struct SettingsWindow {
    hwnd: HWND,
    hinstance: HINSTANCE,
}

impl SettingsWindow {
    pub fn new(hinstance: HINSTANCE) -> Self {
        SettingsWindow { hwnd: std::ptr::null_mut(), hinstance }
    }

    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn create(&mut self) -> HWND {
        // Load config
        load_config();

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
                WS_OVERLAPPEDWINDOW,
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

    /// Refresh the index path listbox from the current config.
    unsafe fn refresh_path_list(hwnd: HWND) {
        let list = GetDlgItem(hwnd, 203);
        SendMessageW(list, LB_RESETCONTENT, 0, 0);
        let cfg = CONFIG.lock().unwrap();
        if let Some(ref config) = *cfg {
            for path in &config.index_paths {
                let w = to_wstring(path);
                SendMessageW(list, LB_ADDSTRING, 0, w.as_ptr() as LPARAM);
            }
        }
    }

    /// Update the status bar with current index stats.
    unsafe fn update_status(hwnd: HWND) {
        let status = GetDlgItem(hwnd, 400);
        let (file_count, dir_count) = {
            let searcher = SEARCHER.lock().unwrap();
            match searcher.as_ref() {
                Some(s) => {
                    let status = s.status();
                    (status.total_files, status.total_folders)
                }
                None => (0, 0)
            }
        };
        let text = format!("状态: 就绪 | 索引: {} 个文件 | {} 个目录", file_count, dir_count);
        let w = to_wstring(&text);
        SetWindowTextW(status, w.as_ptr());
    }

    /// Browse for a folder using SHBrowseForFolderW.
    unsafe fn browse_folder(hwnd: HWND) -> Option<String> {
        let mut display_name = [0u16; 260];
        let title = to_wstring("选择要索引的文件夹");
        let mut bi = BROWSEINFOW {
            hwndOwner: hwnd,
            pidlRoot: std::ptr::null_mut(),
            pszDisplayName: display_name.as_mut_ptr(),
            lpszTitle: title.as_ptr(),
            ulFlags: BIF_USENEWUI | BIF_RETURNONLYFSDIRS,
            lpfn: std::ptr::null_mut(),
            lParam: 0,
            iImage: 0,
        };

        let pidl = SHBrowseForFolderW(&mut bi);
        if pidl.is_null() {
            return None;
        }

        let mut path_buf = vec![0u16; 260];
        let success = SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr());
        CoTaskMemFree(pidl);

        if success == 0 {
            return None;
        }

        let path = from_wstring(path_buf.as_ptr());
        if path.is_empty() { None } else { Some(path) }
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_CREATE => {
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

                Self::add_control(hwnd, "Edit", "", ES_LEFT | WS_BORDER | ES_AUTOHSCROLL, 15, 68, 350, 24, 200);
                Self::add_control(hwnd, "Button", "浏览...", BS_PUSHBUTTON, 375, 68, 60, 24, 201);
                Self::add_control(hwnd, "Button", "添加", BS_PUSHBUTTON, 440, 68, 50, 24, 202);

                // Indexed paths list
                let listbox = Self::add_control(hwnd, "ListBox", "", LBS_NOTIFY | WS_VSCROLL | WS_BORDER, 15, 98, 475, 80, 203);
                SendMessageW(listbox, WM_SETFONT, normal_font as WPARAM, 1);
                Self::add_control(hwnd, "Button", "删除", BS_PUSHBUTTON, 440, 182, 50, 24, 206);

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

                // Refresh list
                Self::refresh_path_list(hwnd);
                Self::update_status(hwnd);

                0
            }
            WM_SHOWWINDOW => {
                if wparam != 0 {
                    // Window is being shown — refresh data
                    Self::refresh_path_list(hwnd);
                    Self::update_status(hwnd);
                }
                0
            }
            WM_COMMAND => {
                let id = loword(wparam as DWORD) as i32;
                let _code = hiword(wparam as DWORD) as UINT;
                match id {
                    201 => { // Browse
                        if let Some(path) = Self::browse_folder(hwnd) {
                            let edit = GetDlgItem(hwnd, 200);
                            let w = to_wstring(&path);
                            SetWindowTextW(edit, w.as_ptr());
                        }
                    }
                    202 => { // Add
                        let edit = GetDlgItem(hwnd, 200);
                        let len = GetWindowTextLengthW(edit);
                        if len > 0 {
                            let mut buf = vec![0u16; (len + 1) as usize];
                            GetWindowTextW(edit, buf.as_mut_ptr(), len + 1);
                            let path = from_wstring(buf.as_ptr());
                            if !path.is_empty() {
                                let mut cfg = CONFIG.lock().unwrap();
                                if let Some(ref mut config) = *cfg {
                                    if !config.index_paths.contains(&path) {
                                        config.index_paths.push(path);
                                    }
                                }
                                drop(cfg);
                                Self::refresh_path_list(hwnd);
                                // Clear the edit
                                SetWindowTextW(edit, to_wstring("").as_ptr());
                            }
                        }
                    }
                    206 => { // Delete selected path
                        let list = GetDlgItem(hwnd, 203);
                        let sel = SendMessageW(list, LB_GETCURSEL, 0, 0) as i32;
                        if sel >= 0 {
                            let mut cfg = CONFIG.lock().unwrap();
                            if let Some(ref mut config) = *cfg {
                                if (sel as usize) < config.index_paths.len() {
                                    config.index_paths.remove(sel as usize);
                                }
                            }
                            drop(cfg);
                            Self::refresh_path_list(hwnd);
                        }
                    }
                    300 => { // Pinyin toggle
                        let btn = GetDlgItem(hwnd, 300);
                        let checked = SendMessageW(btn, BM_GETCHECK, 0, 0);
                        let mut cfg = CONFIG.lock().unwrap();
                        if let Some(ref mut config) = *cfg {
                            config.enable_pinyin = checked == 1;
                        }
                    }
                    301 => { // Fuzzy toggle
                        let btn = GetDlgItem(hwnd, 301);
                        let checked = SendMessageW(btn, BM_GETCHECK, 0, 0);
                        let mut cfg = CONFIG.lock().unwrap();
                        if let Some(ref mut config) = *cfg {
                            config.enable_fuzzy = checked == 1;
                        }
                    }
                    302 => { // Hidden toggle
                        let btn = GetDlgItem(hwnd, 302);
                        let checked = SendMessageW(btn, BM_GETCHECK, 0, 0);
                        let mut cfg = CONFIG.lock().unwrap();
                        if let Some(ref mut config) = *cfg {
                            config.show_hidden = checked == 1;
                        }
                    }
                    500 => { // Index Now
                        let status = GetDlgItem(hwnd, 400);
                        let text = to_wstring("状态: 正在索引...");
                        SetWindowTextW(status, text.as_ptr());

                        let cfg = CONFIG.lock().unwrap();
                        let paths: Vec<String> = cfg.as_ref()
                            .map(|c| c.index_paths.clone())
                            .unwrap_or_default();
                        drop(cfg);

                        // Index each path
                        let mut all_entries = Vec::new();
                        for path_str in &paths {
                            let path = std::path::Path::new(path_str);
                            if path.exists() {
                                if let Ok(entries) = findex_engine::FsWalker::walk(path, 0) {
                                    all_entries.extend(entries);
                                }
                            }
                        }

                        // Build the search index
                        if !all_entries.is_empty() {
                            // Deduplicate by path
                            let mut seen = std::collections::HashSet::new();
                            all_entries.retain(|e| seen.insert(e.path.clone()));

                            let mut index = findex_engine::TrieIndex::new();
                            index.load_entries(all_entries);
                            let searcher = findex_engine::Searcher::new(index);
                            *SEARCHER.lock().unwrap() = Some(searcher);
                        }

                        Self::update_status(hwnd);
                    }
                    501 => { // Save
                        let cfg = CONFIG.lock().unwrap();
                        if let Some(ref config) = *cfg {
                            if config.save().is_ok() {
                                let status = GetDlgItem(hwnd, 400);
                                let text = to_wstring("状态: 已保存");
                                SetWindowTextW(status, text.as_ptr());
                            }
                        }
                    }
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
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}
