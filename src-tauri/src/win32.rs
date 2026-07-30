/// Win32 API FFI for global hotkeys and system tray.
/// Runs in a dedicated thread with its own message loop.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tauri::Manager;

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);
static MAIN_HWND: AtomicUsize = AtomicUsize::new(0);
static mut APP_HANDLE: Option<tauri::AppHandle> = None;

const HOTKEY_ID_SEARCH: i32 = 1;
const HOTKEY_ID_SETTINGS: i32 = 2;

// ===== Win32 FFI =====

type HWND = *mut std::ffi::c_void;
type HINSTANCE = *mut std::ffi::c_void;
type HMENU = *mut std::ffi::c_void;
type HICON = *mut std::ffi::c_void;
type HBRUSH = *mut std::ffi::c_void;
type HCURSOR = *mut std::ffi::c_void;
type HGDIOBJ = *mut std::ffi::c_void;
type LPCWSTR = *const u16;
type UINT = u32;
type WPARAM = usize;
type LPARAM = isize;
type LRESULT = isize;
type DWORD = u32;
type LONG = i32;
type BOOL = i32;
type BYTE = u8;
type WORD = u16;
type ATOM = u16;

#[repr(C)]
struct WNDCLASSEXW {
    cbSize: UINT,
    style: UINT,
    lpfnWndProc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName: LPCWSTR,
    lpszClassName: LPCWSTR,
    hIconSm: HICON,
}

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    time: DWORD,
    pt: POINT,
}

#[repr(C)]
struct POINT { x: LONG, y: LONG }

#[repr(C)]
struct NOTIFYICONDATAW {
    cbSize: DWORD,
    hWnd: HWND,
    uID: UINT,
    uFlags: UINT,
    uCallbackMessage: UINT,
    hIcon: HICON,
    szTip: [u16; 128],
    dwState: DWORD,
    dwStateMask: DWORD,
    szInfo: [u16; 256],
    uVersion: UINT,
    szInfoTitle: [u16; 64],
    dwInfoFlags: DWORD,
    guidItem: [u8; 16],
    hBalloonIcon: HICON,
}

const WM_APP: UINT = 0x8000;
const WM_HOTKEY: UINT = 0x0312;
const WM_COMMAND: UINT = 0x0111;
const WM_DESTROY: UINT = 0x0002;
const WS_POPUP: UINT = 0x80000000;
const WS_EX_TOOLWINDOW: UINT = 0x00000080;
const IDC_ARROW: LPCWSTR = 32512 as LPCWSTR;
const IDI_APPLICATION: LPCWSTR = 32512 as LPCWSTR;
const NIF_MESSAGE: UINT = 0x00000001;
const NIF_ICON: UINT = 0x00000002;
const NIF_TIP: UINT = 0x00000004;
const NIM_ADD: UINT = 0x00000000;
const NIM_DELETE: UINT = 0x00000002;
const MF_STRING: UINT = 0x00000000;
const MF_SEPARATOR: UINT = 0x00000800;
const TPM_LEFTALIGN: UINT = 0x00000000;
const TPM_RIGHTBUTTON: UINT = 0x00000002;
const COLOR_WINDOW: UINT = 5;

const TRAY_CALLBACK_MSG: UINT = WM_APP + 1;
const MENU_SHOW_SETTINGS: usize = 1001;
const MENU_QUIT: usize = 1002;

extern "system" {
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HINSTANCE;
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM;
    fn CreateWindowExW(
        dwExStyle: UINT, lpClassName: LPCWSTR, lpWindowName: LPCWSTR,
        dwStyle: UINT, X: i32, Y: i32, nWidth: i32, nHeight: i32,
        hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: *mut std::ffi::c_void,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    fn PostQuitMessage(nExitCode: i32);
    fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: UINT, vk: UINT) -> BOOL;
    fn UnregisterHotKey(hWnd: HWND, id: i32) -> BOOL;
    fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> HCURSOR;
    fn Shell_NotifyIconW(dwMessage: DWORD, lpdata: *mut NOTIFYICONDATAW) -> BOOL;
    fn CreatePopupMenu() -> HMENU;
    fn DestroyMenu(hMenu: HMENU) -> BOOL;
    fn AppendMenuW(hMenu: HMENU, uFlags: UINT, uIDNewItem: usize, lpNewItem: LPCWSTR) -> BOOL;
    fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    fn TrackPopupMenu(hMenu: HMENU, uFlags: UINT, x: i32, y: i32, nReserved: i32, hWnd: HWND, prcRect: *const std::ffi::c_void) -> BOOL;
    fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
}

fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Show the Tauri settings window.
fn show_settings_window() {
    unsafe {
        if let Some(ref handle) = APP_HANDLE {
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}

/// Run the hotkey message loop in a dedicated thread.
pub fn run_hotkey_loop(app_handle: tauri::AppHandle) {
    unsafe {
        APP_HANDLE = Some(app_handle);

        let hinstance = GetModuleHandleW(std::ptr::null());
        if hinstance.is_null() { return; }

        let class_name = to_wstring("FindexTauriHotkeyClass");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0,
            lpfnWndProc: Some(hotkey_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wc);

        let main_hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW, class_name.as_ptr(), to_wstring("Findex").as_ptr(),
            WS_POPUP, 0, 0, 0, 0,
            std::ptr::null_mut(), std::ptr::null_mut(), hinstance, std::ptr::null_mut(),
        );
        MAIN_HWND.store(main_hwnd as usize, Ordering::Relaxed);

        register_hotkeys(main_hwnd);
        create_tray_icon(main_hwnd, hinstance);

        let mut msg: MSG = std::mem::zeroed();
        while !SHOULD_QUIT.load(Ordering::Relaxed) {
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 { break; }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        remove_tray_icon(main_hwnd);
    }
}

fn register_hotkeys(hwnd: HWND) {
    unsafe {
        UnregisterHotKey(hwnd, HOTKEY_ID_SEARCH);
        UnregisterHotKey(hwnd, HOTKEY_ID_SETTINGS);
    }

    let config = crate::CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    let config = config.clone().unwrap_or_default();
    let search_hotkey = config.search_hotkey.clone();
    let settings_hotkey = config.settings_hotkey.clone();
    drop(config);

    if let Some((mods, vk)) = crate::config::parse_hotkey(&search_hotkey) {
        unsafe {
            RegisterHotKey(hwnd, HOTKEY_ID_SEARCH, mods, vk);
        }
    }
    if let Some((mods, vk)) = crate::config::parse_hotkey(&settings_hotkey) {
        unsafe {
            RegisterHotKey(hwnd, HOTKEY_ID_SETTINGS, mods, vk);
        }
    }
}

unsafe extern "system" fn hotkey_wnd_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let id = wparam as i32;
            match id {
                HOTKEY_ID_SEARCH => {
                    let hinstance = GetModuleHandleW(std::ptr::null());
                    // Use a raw pointer to pass across threads
                    let ptr = hinstance as usize;
                    std::thread::spawn(move || {
                        let hinst = ptr as *mut std::ffi::c_void;
                        crate::egui_overlay::run_search_overlay(hinst);
                    });
                }
                HOTKEY_ID_SETTINGS => {
                    show_settings_window();
                }
                _ => {}
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as usize;
            match id {
                MENU_SHOW_SETTINGS => {
                    show_settings_window();
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
                0x0203 => { show_settings_window(); }
                0x0205 => { show_tray_menu(hwnd); }
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

unsafe fn create_tray_icon(hwnd: HWND, hinstance: HINSTANCE) {
    let hicon = LoadIconW(hinstance, IDI_APPLICATION);

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
