#![allow(non_snake_case)]
#![allow(unused_unsafe)]

/// Findex - Graphical User Interface
/// Windows native UI with zero external dependencies.

mod win32;
mod settings;
mod search_overlay;

use win32::*;
use settings::SettingsWindow;
use search_overlay::SearchOverlay;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);

struct AppState {
    settings_created: bool,
    search_created: bool,
    main_hwnd: win32::HWND,
}

unsafe impl Send for AppState {}

static APP_STATE: Mutex<Option<AppState>> = Mutex::new(None);

fn main() {
    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        if hinstance.is_null() {
            eprintln!("Failed to get module handle");
            return;
        }

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

        RegisterHotKey(main_hwnd, 1, MOD_CONTROL, VK_SPACE as UINT);
        RegisterHotKey(main_hwnd, 2, MOD_CONTROL | MOD_SHIFT, 'F' as UINT);

        let mut settings = SettingsWindow::new(hinstance);
        settings.create();
        settings.show();

        let mut search = SearchOverlay::new(hinstance);
        search.create();

        {
            let mut state = APP_STATE.lock().unwrap();
            *state = Some(AppState {
                settings_created: true,
                search_created: true,
                main_hwnd,
            });
        }

        let mut msg: MSG = std::mem::zeroed();
        while !SHOULD_QUIT.load(Ordering::Relaxed) {
            let ret = GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0);
            if ret == 0 { break; }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            let id = wparam as i32;
            if id == 1 {
                // Toggle search - TODO
            } else if id == 2 {
                // Show settings - TODO
            }
            0
        }
        WM_DESTROY => {
            SHOULD_QUIT.store(true, Ordering::Relaxed);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
