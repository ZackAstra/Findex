/// Egui-based windows for Findex.
/// Search overlay and settings window using egui UI.

use crate::win32::*;
use crate::egui_win32::EguiRenderer;
use crate::SEARCHER;

use std::sync::Mutex;

/// The current search results stored for the search overlay.
static CURRENT_RESULTS: Mutex<Vec<findex_engine::SearchResult>> = Mutex::new(Vec::new());

/// Create and run the search overlay window.
/// Blocks until the window is closed.
pub fn run_search_overlay(hinstance: HINSTANCE) {
    unsafe {
        let class_name = to_wstring("FindexEguiSearchClass");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0,
            lpfnWndProc: Some(search_wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 32, // Space for renderer pointer and context
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(), // We handle painting ourselves
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        };
        RegisterClassExW(&wc);

        let screen_w = GetSystemMetrics(0);
        let screen_h = GetSystemMetrics(1);
        let win_w = 520;
        let win_h = 400;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            to_wstring("Findex Search").as_ptr(),
            WS_POPUP | WS_BORDER | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            (screen_w - win_w) / 2, (screen_h - win_h) / 3,
            win_w, win_h,
            std::ptr::null_mut(), std::ptr::null_mut(), hinstance, std::ptr::null_mut(),
        );

        if hwnd.is_null() { return; }

        // Initialize egui context and renderer
        let ctx = egui::Context::default();
        let renderer = EguiRenderer::new();
        let state = EguiWindowState { ctx, renderer, query: String::new() };

        // Store state in window extra bytes
        let state_ptr = Box::into_raw(Box::new(state));
        SetWindowLongPtrW(hwnd, 0, state_ptr as isize);

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        // Focus the search input by setting a flag
        let focused_ptr = Box::into_raw(Box::new(true));
        SetWindowLongPtrW(hwnd, 8, focused_ptr as isize);

        // Message loop
        let mut msg: MSG = std::mem::zeroed();
        let mut running = true;
        while running {
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    running = false;
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if !running { break; }

            // Redraw the window
            InvalidateRect(hwnd, std::ptr::null(), 0);
            UpdateWindow(hwnd);
        }

        // Cleanup
        let state_ptr = GetWindowLongPtrW(hwnd, 0) as *mut EguiWindowState;
        if !state_ptr.is_null() {
            let _ = Box::from_raw(state_ptr);
        }
        let focused_ptr = GetWindowLongPtrW(hwnd, 8) as *mut bool;
        if !focused_ptr.is_null() {
            let _ = Box::from_raw(focused_ptr);
        }
        DestroyWindow(hwnd);
    }
}

/// Internal state for an egui window.
struct EguiWindowState {
    ctx: egui::Context,
    renderer: EguiRenderer,
    query: String,
}

unsafe extern "system" fn search_wnd_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let state_ptr = GetWindowLongPtrW(hwnd, 0) as *mut EguiWindowState;
            if state_ptr.is_null() {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            let state = &mut *state_ptr;

            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rect: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut rect);
            let width = (rect.right - rect.left) as usize;
            let height = (rect.bottom - rect.top) as usize;

            if width > 0 && height > 0 {
                state.renderer.resize(width, height);

                // Build egui input
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::pos2(0.0, 0.0),
                        egui::vec2(width as f32, height as f32),
                    )),
                    ..Default::default()
                };

                // Run the egui UI
                let output = state.ctx.run(input, |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.heading("Findex 搜索");
                        ui.separator();

                        // Search input
                        let mut query = state.query.clone();
                        let resp = ui.text_edit_singleline(&mut query);
                        if resp.changed() {
                            state.query = query.clone();
                        }

                        // Perform search
                        let results = if !query.is_empty() {
                            SEARCHER.lock().unwrap().as_ref().map(|s| {
                                s.search(&findex_engine::SearchQuery {
                                    query: query.clone(),
                                    scope: findex_engine::SearchScope::Global,
                                    context_path: None,
                                    max_results: 100,
                                    offset: 0,
                                    sort_by: findex_engine::SortBy::Relevance,
                                })
                            }).unwrap_or_default()
                        } else {
                            Vec::new()
                        };

                        // Store results for later lookup
                        *CURRENT_RESULTS.lock().unwrap() = results.clone();

                        // Show results
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for result in &results {
                                let entry = &result.entry;
                                let label = if entry.is_dir {
                                    format!("📁 {}  ({})", entry.name, entry.path)
                                } else {
                                    format!("📄 {}  ({})", entry.name, entry.path)
                                };
                                if ui.selectable_label(false, &label).clicked() {
                                    // Open the file
                                    open_file(&entry.path);
                                }
                            }
                        });

                        if results.is_empty() && !query.is_empty() {
                            ui.label("未找到匹配结果");
                        }
                    });
                });

                // Render to pixel buffer
                state.renderer.render(
                    output.shapes,
                    &output.textures_delta,
                    output.pixels_per_point,
                    state.ctx.style().visuals.window_fill(),
                );

                // Blit to window
                let bgra = state.renderer.as_bgra();
                let bmi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as DWORD,
                        biWidth: width as i32,
                        biHeight: -(height as i32), // top-down
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB,
                        biSizeImage: 0,
                        biXPelsPerMeter: 0,
                        biYPelsPerMeter: 0,
                        biClrUsed: 0,
                        biClrImportant: 0,
                    },
                    bmiColors: [0; 1],
                };

                StretchDIBits(
                    hdc,
                    0, 0, width as i32, height as i32,
                    0, 0, width as i32, height as i32,
                    bgra.as_ptr() as LPCVOID,
                    &bmi,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }

            EndPaint(hwnd, &ps);
            0
        }
        WM_KEYDOWN => {
            if wparam as i32 == VK_ESCAPE {
                PostQuitMessage(0);
            }
            0
        }
        WM_CLOSE => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Open a file using ShellExecuteW.
fn open_file(path: &str) {
    let wpath = to_wstring(path);
    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            to_wstring("open").as_ptr(),
            wpath.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
}
