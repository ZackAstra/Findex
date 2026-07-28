#![allow(non_snake_case)]
#![allow(unused_unsafe)]
#![allow(dead_code)]

/// Findex - Graphical User Interface
/// Windows native UI with zero external dependencies.

mod win32;
mod config;
mod settings;
mod search_overlay;

use win32::*;
use settings::SettingsWindow;
use search_overlay::SearchOverlay;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);

/// Global searcher instance, shared between main window and search overlay.
pub static SEARCHER: Mutex<Option<findex_engine::Searcher>> = Mutex::new(None);

/// HWND of the search overlay window (for toggling from hotkey handler).
pub static SEARCH_OVERLAY_HWND: AtomicUsize = AtomicUsize::new(0);

/// HWND of the settings window (for toggling from hotkey handler).
pub static SETTINGS_HWND: AtomicUsize = AtomicUsize::new(0);

/// Custom message for tray icon notifications.
const TRAY_CALLBACK_MSG: UINT = win32::WM_APP + 1;

/// Menu IDs for tray context menu
const MENU_SHOW_SETTINGS: usize = 1001;
const MENU_QUIT: usize = 1002;

fn main() {
    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        if hinstance.is_null() {
            eprintln!("Failed to get module handle");
            return;
        }

        // Load the index from standard locations
        settings::load_config();
        {
            let cfg = settings::get_config();
            load_index(&cfg);
        }

        // Register main hidden window class
        let main_class = to_wstring("FindexMainClass");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0,
            lpfnWndProc: Some(main_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: main_class.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wc);

        let main_hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW, main_class.as_ptr(), to_wstring("Findex").as_ptr(),
            WS_POPUP, 0, 0, 0, 0,
            std::ptr::null_mut(), std::ptr::null_mut(), hinstance, std::ptr::null_mut(),
        );

        // Register global hotkeys
        RegisterHotKey(main_hwnd, 1, MOD_CONTROL, VK_SPACE as UINT);
        RegisterHotKey(main_hwnd, 2, MOD_CONTROL | MOD_SHIFT, 'F' as UINT);

        // Create settings window (hidden by default)
        let settings = SettingsWindow::new(hinstance);

        // Create search overlay
        let mut search = SearchOverlay::new(hinstance);
        search.create();

        // Store HWNDs in globals for hotkey access
        SEARCH_OVERLAY_HWND.store(search.hwnd() as usize, Ordering::Relaxed);
        SETTINGS_HWND.store(settings.hwnd() as usize, Ordering::Relaxed);

        // Create system tray icon
        create_tray_icon(main_hwnd, hinstance);

        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        while !SHOULD_QUIT.load(Ordering::Relaxed) {
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 { break; }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Remove tray icon on exit
        remove_tray_icon(main_hwnd);
    }
}

/// Load the index from standard locations.
fn load_index(config: &config::Config) {
    let search_paths: Vec<String> = if !config.index_paths.is_empty() {
        config.index_paths.iter()
            .map(|p| format!("{}\\findex.db", p))
            .collect()
    } else {
        vec![
            "findex.db".to_string(),
            {
                let appdata = std::env::var("APPDATA").unwrap_or_default();
                if !appdata.is_empty() {
                    format!("{}\\Findex\\index.db", appdata)
                } else {
                    String::new()
                }
            },
        ]
    };

    for path in &search_paths {
        if path.is_empty() { continue; }
        if let Ok(storage) = findex_engine::Storage::open(path) {
            if let Ok(entries) = storage.load_entries() {
                if !entries.is_empty() {
                    let mut index = findex_engine::TrieIndex::new();
                    index.load_entries(entries);
                    let searcher = findex_engine::Searcher::new(index);
                    *SEARCHER.lock().unwrap() = Some(searcher);
                    eprintln!("Loaded index from {}: {} entries", path, storage.entry_count().unwrap_or(0));
                    return;
                }
            }
        }
    }
    eprintln!("No index found. Use 'findex index <directory>' to create one.");
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let id = wparam as i32;
            match id {
                1 => { // Ctrl+Space: toggle search overlay
                    let overlay_hwnd = SEARCH_OVERLAY_HWND.load(Ordering::Relaxed) as HWND;
                    if !overlay_hwnd.is_null() {
                        if IsWindowVisible(overlay_hwnd) != 0 {
                            ShowWindow(overlay_hwnd, SW_HIDE);
                        } else {
                            let screen_w = GetSystemMetrics(0);
                            let screen_h = GetSystemMetrics(1);
                            SetWindowPos(overlay_hwnd, std::ptr::null_mut(),
                                (screen_w - 500) / 2, (screen_h - 400) / 3,
                                500, 400, SWP_SHOWWINDOW | SWP_NOZORDER);
                            SetForegroundWindow(overlay_hwnd);
                            let edit = GetWindowLongPtrW(overlay_hwnd, 0) as HWND;
                            if !edit.is_null() {
                                SetFocus(edit);
                            }
                        }
                    }
                }
                2 => { // Ctrl+Shift+F: toggle settings window
                    let settings_hwnd = SETTINGS_HWND.load(Ordering::Relaxed) as HWND;
                    if !settings_hwnd.is_null() {
                        if IsWindowVisible(settings_hwnd) != 0 {
                            ShowWindow(settings_hwnd, SW_HIDE);
                        } else {
                            ShowWindow(settings_hwnd, SW_SHOWNORMAL);
                            SetForegroundWindow(settings_hwnd);
                        }
                    }
                }
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            let id = loword(wparam as DWORD) as usize;
            match id {
                MENU_SHOW_SETTINGS => {
                    let settings_hwnd = SETTINGS_HWND.load(Ordering::Relaxed) as HWND;
                    if !settings_hwnd.is_null() {
                        ShowWindow(settings_hwnd, SW_SHOWNORMAL);
                        SetForegroundWindow(settings_hwnd);
                    }
                }
                MENU_QUIT => {
                    SHOULD_QUIT.store(true, Ordering::Relaxed);
                    PostQuitMessage(0);
                }
                _ => {}
            }
            0
        }
        TRAY_CALLBACK_MSG => {
            let event = lparam as u32;
            match event {
                0x0203 => { // WM_LBUTTONDBLCLK
                    let settings_hwnd = SETTINGS_HWND.load(Ordering::Relaxed) as HWND;
                    if !settings_hwnd.is_null() {
                        ShowWindow(settings_hwnd, SW_SHOWNORMAL);
                        SetForegroundWindow(settings_hwnd);
                    }
                }
                0x0205 => { // WM_RBUTTONUP
                    show_tray_menu(hwnd);
                }
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            remove_tray_icon(hwnd);
            SHOULD_QUIT.store(true, Ordering::Relaxed);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Create the system tray icon.
unsafe fn create_tray_icon(hwnd: HWND, hinstance: HINSTANCE) {
    let hicon = LoadIconW(hinstance, IDI_APPLICATION);
    if hicon.is_null() {
        let _ = LoadIconW(std::ptr::null_mut(), IDI_APPLICATION);
    }

    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as DWORD,
        hWnd: hwnd,
        uID: 1,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: TRAY_CALLBACK_MSG,
        hIcon: hicon,
        szTip: [0u16; 128],
        dwState: 0,
        dwStateMask: 0,
        szInfo: [0u16; 256],
        uVersion: 0,
        szInfoTitle: [0u16; 64],
        dwInfoFlags: 0,
        guidItem: [0u8; 16],
        hBalloonIcon: std::ptr::null_mut(),
    };

    let tip = to_wstring("Findex - 文件搜索工具");
    let tip_len = tip.len().min(127);
    for (i, &ch) in tip[..tip_len].iter().enumerate() {
        nid.szTip[i] = ch;
    }
    nid.szTip[tip_len] = 0;

    Shell_NotifyIconW(NIM_ADD, &mut nid);
}

/// Remove the system tray icon.
unsafe fn remove_tray_icon(hwnd: HWND) {
    let mut nid = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as DWORD,
        hWnd: hwnd,
        uID: 1,
        uFlags: 0,
        uCallbackMessage: 0,
        hIcon: std::ptr::null_mut(),
        szTip: [0u16; 128],
        dwState: 0,
        dwStateMask: 0,
        szInfo: [0u16; 256],
        uVersion: 0,
        szInfoTitle: [0u16; 64],
        dwInfoFlags: 0,
        guidItem: [0u8; 16],
        hBalloonIcon: std::ptr::null_mut(),
    };
    Shell_NotifyIconW(NIM_DELETE, &mut nid);
}

/// Show the tray icon context menu.
unsafe fn show_tray_menu(hwnd: HWND) {
    let hmenu = CreatePopupMenu();
    if hmenu.is_null() { return; }

    AppendMenuW(hmenu, MF_STRING, MENU_SHOW_SETTINGS, to_wstring("显示设置").as_ptr());
    AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(hmenu, MF_STRING, MENU_QUIT, to_wstring("退出").as_ptr());

    let mut pt = POINT { x: 0, y: 0 };
    GetCursorPos(&mut pt);

    SetForegroundWindow(hwnd);
    TrackPopupMenu(hmenu, TPM_LEFTALIGN | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, std::ptr::null_mut());
    DestroyMenu(hmenu);
}
