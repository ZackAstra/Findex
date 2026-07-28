#![allow(non_snake_case)]
#![allow(unused_unsafe)]
#![allow(dead_code)]

/// Findex - Graphical User Interface
/// Windows native UI with egui rendering.

mod win32;
mod config;
mod egui_win32;
mod egui_windows;

use win32::*;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);

/// Global searcher instance, shared between main window and search overlay.
pub static SEARCHER: Mutex<Option<findex_engine::Searcher>> = Mutex::new(None);

/// Main hidden window handle for hotkey registration.
static MAIN_HWND: AtomicUsize = AtomicUsize::new(0);

/// Custom message for tray icon notifications.
const TRAY_CALLBACK_MSG: UINT = win32::WM_APP + 1;
const HOTKEY_ID_SEARCH: i32 = 1;
const HOTKEY_ID_SETTINGS: i32 = 2;

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
        config::load_config();
        {
            let cfg = config::get_config();
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

        MAIN_HWND.store(main_hwnd as usize, Ordering::Relaxed);

        // Register global hotkeys from config
        register_hotkeys(main_hwnd);

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

/// Register hotkeys from the current config.
unsafe fn register_hotkeys(hwnd: HWND) {
    let cfg = config::get_config();

    // Unregister old hotkeys first
    UnregisterHotKey(hwnd, HOTKEY_ID_SEARCH);
    UnregisterHotKey(hwnd, HOTKEY_ID_SETTINGS);

    // Register search hotkey
    if let Some((mods, vk)) = config::parse_hotkey(&cfg.hotkey_search) {
        RegisterHotKey(hwnd, HOTKEY_ID_SEARCH, mods, vk);
        eprintln!("Search hotkey: {}", cfg.hotkey_search);
    }

    // Register settings hotkey
    if let Some((mods, vk)) = config::parse_hotkey(&cfg.hotkey_settings) {
        RegisterHotKey(hwnd, HOTKEY_ID_SETTINGS, mods, vk);
        eprintln!("Settings hotkey: {}", cfg.hotkey_settings);
    }
}

/// Re-register hotkeys (called after config save).
pub fn re_register_hotkeys() {
    unsafe {
        let hwnd = MAIN_HWND.load(Ordering::Relaxed) as HWND;
        if !hwnd.is_null() {
            register_hotkeys(hwnd);
        }
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
                HOTKEY_ID_SEARCH => {
                    let hinstance = GetModuleHandleW(std::ptr::null());
                    egui_windows::run_search_overlay(hinstance);
                }
                HOTKEY_ID_SETTINGS => {
                    run_settings();
                }
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            let id = loword(wparam as DWORD) as usize;
            match id {
                MENU_SHOW_SETTINGS => {
                    run_settings();
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
                    run_settings();
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

/// Run the egui settings window (blocking).
fn run_settings() {
    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        egui_windows::run_settings_window(hinstance);
        // Re-register hotkeys in case they were changed in settings
        register_hotkeys(MAIN_HWND.load(Ordering::Relaxed) as HWND);
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