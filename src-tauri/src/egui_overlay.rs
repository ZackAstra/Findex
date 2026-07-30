/// egui-based search overlay for Findex.
/// Renders via software rasterization to a Win32 window.
/// Launched on hotkey press, runs its own message loop.

use std::sync::Mutex;

static CURRENT_RESULTS: Mutex<Vec<findex_engine::SearchResult>> = Mutex::new(Vec::new());

// ===== Win32 Types for egui overlay =====

type HWND = *mut std::ffi::c_void;
type HINSTANCE = *mut std::ffi::c_void;
type HDC = *mut std::ffi::c_void;
type HBITMAP = *mut std::ffi::c_void;
type HGDIOBJ = *mut std::ffi::c_void;
type LPCVOID = *const std::ffi::c_void;
type UINT = u32;
type WPARAM = usize;
type LPARAM = isize;
type LRESULT = isize;
type DWORD = u32;
type LONG = i32;
type BOOL = i32;
type WORD = u16;
type ATOM = u16;
type HRGN = *mut std::ffi::c_void;
type LPCWSTR = *const u16;

#[repr(C)]
struct WNDCLASSEXW {
    cbSize: UINT, style: UINT,
    lpfnWndProc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    cbClsExtra: i32, cbWndExtra: i32,
    hInstance: HINSTANCE, hIcon: *mut std::ffi::c_void,
    hCursor: *mut std::ffi::c_void, hbrBackground: *mut std::ffi::c_void,
    lpszMenuName: LPCWSTR, lpszClassName: LPCWSTR, hIconSm: *mut std::ffi::c_void,
}

#[repr(C)]
struct MSG { hwnd: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM, time: DWORD, pt: POINT }

#[repr(C)]
struct POINT { x: LONG, y: LONG }

#[repr(C)]
struct PAINTSTRUCT { hdc: HDC, fErase: BOOL, rcPaint: RECT, fRestore: BOOL, fIncUpdate: BOOL, rgbReserved: [u8; 32] }

#[repr(C)]
struct RECT { left: LONG, top: LONG, right: LONG, bottom: LONG }

#[repr(C)]
struct BITMAPINFOHEADER {
    biSize: DWORD, biWidth: LONG, biHeight: LONG,
    biPlanes: WORD, biBitCount: WORD, biCompression: DWORD,
    biSizeImage: DWORD, biXPelsPerMeter: LONG, biYPelsPerMeter: LONG,
    biClrUsed: DWORD, biClrImportant: DWORD,
}

#[repr(C)]
struct BITMAPINFO { bmiHeader: BITMAPINFOHEADER, bmiColors: [DWORD; 1] }

const WS_POPUP: UINT = 0x80000000;
const WS_CLIPCHILDREN: UINT = 0x04000000;
const WS_CLIPSIBLINGS: UINT = 0x04000000;
const WS_EX_TOPMOST: UINT = 0x00000008;
const WS_EX_TOOLWINDOW: UINT = 0x00000080;
const WS_EX_NOACTIVATE: UINT = 0x08000000;
const WS_EX_LAYERED: UINT = 0x00080000;
const WM_PAINT: UINT = 0x000F;
const WM_CLOSE: UINT = 0x0010;
const WM_ERASEBKGND: UINT = 0x0014;
const WM_QUIT: UINT = 0x0012;
const WM_KEYDOWN: UINT = 0x0100;
const WM_CHAR: UINT = 0x0102;
const WM_ACTIVATE: UINT = 0x0006;
const SW_SHOW: i32 = 5;
const PM_REMOVE: UINT = 1;
const BI_RGB: DWORD = 0;
const DIB_RGB_COLORS: DWORD = 0;
const SRCCOPY: DWORD = 0x00CC0020;
const IDC_ARROW: LPCWSTR = 32512 as LPCWSTR;
const VK_ESCAPE: u32 = 0x1B;
const VK_RETURN: u32 = 0x0D;
const VK_UP: u32 = 0x26;
const VK_DOWN: u32 = 0x28;
const VK_BACK: u32 = 0x08;
const VK_DELETE: u32 = 0x2E;

extern "system" {
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HINSTANCE;
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM;
    fn CreateWindowExW(dwExStyle: UINT, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: UINT, X: i32, Y: i32, nWidth: i32, nHeight: i32, hWndParent: HWND, hMenu: *mut std::ffi::c_void, hInstance: HINSTANCE, lpParam: *mut std::ffi::c_void) -> HWND;
    fn DefWindowProcW(hWnd: HWND, msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PeekMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: UINT, wMsgFilterMax: UINT, wRemoveMsg: UINT) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    fn PostQuitMessage(nExitCode: i32);
    fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    fn InvalidateRect(hWnd: HWND, lpRect: *const std::ffi::c_void, bErase: BOOL) -> BOOL;
    fn UpdateWindow(hWnd: HWND) -> BOOL;
    fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
    fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn CreateCompatibleBitmap(hdc: HDC, nWidth: i32, nHeight: i32) -> HBITMAP;
    fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    fn StretchDIBits(hdc: HDC, xDest: i32, yDest: i32, wDest: i32, hDest: i32, xSrc: i32, ySrc: i32, wSrc: i32, hSrc: i32, bits: LPCVOID, bmi: *const BITMAPINFO, colorUse: UINT, rop: DWORD) -> i32;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
    fn GetSystemMetrics(nIndex: i32) -> i32;
    fn CreateRoundRectRgn(x1: i32, y1: i32, x2: i32, y2: i32, cx: i32, cy: i32) -> HRGN;
    fn SetWindowRgn(hWnd: HWND, hRgn: HRGN, bRedraw: BOOL) -> i32;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: LPCWSTR) -> *mut std::ffi::c_void;
    fn Sleep(dwMilliseconds: DWORD);
    fn ShellExecuteW(hwnd: HWND, operation: LPCWSTR, file: LPCWSTR, parameters: LPCWSTR, directory: LPCWSTR, nShowCmd: i32) -> isize;
}

fn to_wstring(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// ===== Egui Software Renderer =====

struct EguiRenderer {
    width: usize,
    height: usize,
    buffer: Vec<u8>,
}

impl EguiRenderer {
    fn new() -> Self {
        Self { width: 0, height: 0, buffer: Vec::new() }
    }

    fn resize(&mut self, width: usize, height: usize) {
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.buffer = vec![0u8; width * height * 4];
        }
    }

    fn render(&mut self, full_output: &egui::FullOutput, clear_color: egui::Color32) {
        // Clear buffer
        for pixel in self.buffer.chunks_exact_mut(4) {
            pixel[0] = clear_color.b();
            pixel[1] = clear_color.g();
            pixel[2] = clear_color.r();
            pixel[3] = clear_color.a();
        }
    }

    fn as_bgra(&self) -> &[u8] { &self.buffer }
}

// ===== Search Overlay State =====

struct SearchOverlayState {
    ctx: egui::Context,
    renderer: EguiRenderer,
    query: String,
    selected_index: usize,
    filter: usize,
}

const FILTER_NAMES: &[&str] = &["All", "Folders", "Docs", "Code", "Images", "Archive", "Audio", "Video"];

// ===== Window Procedure =====

unsafe extern "system" fn search_wnd_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !hdc.is_null() {
                let state_ptr = GetWindowLongPtrW(hwnd, 0) as *mut SearchOverlayState;
                if !state_ptr.is_null() {
                    let state = &mut *state_ptr;
                    let width = ps.rcPaint.right - ps.rcPaint.left;
                    let height = ps.rcPaint.bottom - ps.rcPaint.top;
                    if width > 0 && height > 0 {
                        state.renderer.resize(width as usize, height as usize);

                        let input = egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::pos2(0.0, 0.0),
                                egui::vec2(width as f32, height as f32),
                            )),
                            ..Default::default()
                        };

                        state.ctx.begin_frame(input);

                        let mut query_mut = state.query.clone();
                        let mut selected = state.selected_index;
                        let mut filter_mut = state.filter;

                        egui::CentralPanel::default()
                            .frame(egui::Frame::none().fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 240)))
                            .show(&state.ctx, |ui| {
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label("\u{1F50D}");
                                    ui.add_sized(
                                        [ui.available_width() - 100.0, 28.0],
                                        egui::TextEdit::singleline(&mut query_mut)
                                            .hint_text("Search files...")
                                            .font(egui::TextStyle::Body)
                                    );
                                    egui::ComboBox::from_id_source("filter")
                                        .selected_text(FILTER_NAMES[filter_mut])
                                        .show_ui(ui, |ui| {
                                            for (i, name) in FILTER_NAMES.iter().enumerate() {
                                                ui.selectable_value(&mut filter_mut, i, *name);
                                            }
                                        });
                                });

                                ui.add_space(4.0);
                                let results = get_search_results(&query_mut);
                                let _ = CURRENT_RESULTS.lock().map(|mut r| *r = results.clone());

                                egui::ScrollArea::vertical()
                                    .max_height(400.0)
                                    .show(ui, |ui| {
                                        for (i, result) in results.iter().enumerate() {
                                            let is_selected = i == selected;
                                            let bg = if is_selected {
                                                egui::Color32::from_rgba_premultiplied(10, 132, 255, 60)
                                            } else {
                                                egui::Color32::TRANSPARENT
                                            };

                                            let ib = egui::Frame::none()
                                                .fill(bg)
                                                .rounding(4.0);
                                            let response = ib.show(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.set_min_width(ui.available_width());
                                                    ui.label(file_icon(&result.entry));
                                                    ui.vertical(|ui| {
                                                        ui.label(egui::RichText::new(&result.entry.name).size(13.0));
                                                        ui.label(egui::RichText::new(&result.entry.path).size(10.0).color(egui::Color32::GRAY));
                                                    });
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        ui.label(format_size(result.entry.size));
                                                        ui.label(format_date(result.entry.modified));
                                                    });
                                                });
                                            }).response;

                                            if response.clicked() {
                                                open_file(&result.entry.path);
                                                PostQuitMessage(0);
                                            }
                                        }
                                    });
                            });

                        let full_output = state.ctx.end_frame();
                        let clear_color = egui::Color32::from_rgba_premultiplied(30, 30, 30, 240);
                        state.renderer.render(&full_output, clear_color);

                        // Blit to screen
                        let mem_dc = CreateCompatibleDC(hdc);
                        if !mem_dc.is_null() {
                            let bmp = CreateCompatibleBitmap(hdc, width, height);
                            let old = SelectObject(mem_dc, bmp as HGDIOBJ);
                            let bgra = state.renderer.as_bgra();
                            let bmi = BITMAPINFO {
                                bmiHeader: BITMAPINFOHEADER {
                                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as DWORD,
                                    biWidth: width as LONG,
                                    biHeight: -(height as LONG),
                                    biPlanes: 1, biBitCount: 32, biCompression: BI_RGB,
                                    biSizeImage: 0, biXPelsPerMeter: 0, biYPelsPerMeter: 0,
                                    biClrUsed: 0, biClrImportant: 0,
                                },
                                bmiColors: [0; 1],
                            };
                            StretchDIBits(hdc, 0, 0, width, height, 0, 0, width, height,
                                bgra.as_ptr() as LPCVOID, &bmi, DIB_RGB_COLORS, SRCCOPY);
                            SelectObject(mem_dc, old);
                            DeleteObject(bmp as HGDIOBJ);
                            DeleteDC(mem_dc);
                        }

                        state.query = query_mut;
                        state.selected_index = selected;
                        state.filter = filter_mut;
                    }
                }
            }
            EndPaint(hwnd, &ps);
            0
        }
        WM_KEYDOWN => {
            let vk = wparam as u32;
            let state_ptr = GetWindowLongPtrW(hwnd, 0) as *mut SearchOverlayState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                match vk {
                    VK_ESCAPE => { PostQuitMessage(0); }
                    VK_RETURN => {
                        let results = CURRENT_RESULTS.lock().unwrap_or_else(|e| e.into_inner());
                        if !results.is_empty() && state.selected_index < results.len() {
                            let path = results[state.selected_index].entry.path.clone();
                            drop(results);
                            open_file(&path);
                        }
                        PostQuitMessage(0);
                    }
                    VK_UP => { if state.selected_index > 0 { state.selected_index -= 1; } }
                    VK_DOWN => {
                        let results = CURRENT_RESULTS.lock().unwrap_or_else(|e| e.into_inner());
                        if state.selected_index + 1 < results.len() { state.selected_index += 1; }
                    }
                    VK_BACK | VK_DELETE => { state.query.pop(); }
                    _ => {}
                }
            }
            0
        }
        WM_CHAR => {
            let c = wparam as u8 as char;
            let state_ptr = GetWindowLongPtrW(hwnd, 0) as *mut SearchOverlayState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                if c.is_ascii_graphic() || c == ' ' { state.query.push(c); }
            }
            0
        }
        WM_CLOSE => { PostQuitMessage(0); 0 }
        WM_ACTIVATE => { if wparam == 0 { PostQuitMessage(0); } 0 }
        WM_ERASEBKGND => 1,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn get_search_results(query: &str) -> Vec<findex_engine::SearchResult> {
    let searcher = crate::SEARCHER.lock().unwrap_or_else(|e| e.into_inner());
    searcher.as_ref().map(|s| {
        let search_query = findex_engine::SearchQuery {
            query: query.to_string(),
            scope: findex_engine::SearchScope::Global,
            context_path: None,
            max_results: 50,
            offset: 0,
            sort_by: findex_engine::SortBy::Relevance,
        };
        s.search(&search_query)
    }).unwrap_or_default()
}

// ===== Public API =====

pub fn run_search_overlay(hinstance: HINSTANCE) {
    unsafe {
        let class_name = to_wstring("FindexEguiSearchClass");
        RegisterClassExW(&WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0,
            lpfnWndProc: Some(search_wnd_proc),
            cbClsExtra: 0, cbWndExtra: 32,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: std::ptr::null_mut(),
        });

        let screen_w = GetSystemMetrics(0);
        let screen_h = GetSystemMetrics(1);
        let win_w = 600;
        let win_h = 480;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED,
            class_name.as_ptr(),
            to_wstring("Findex Search").as_ptr(),
            WS_POPUP | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            (screen_w - win_w) / 2, (screen_h - win_h) / 3, win_w, win_h,
            std::ptr::null_mut(), std::ptr::null_mut(), hinstance, std::ptr::null_mut(),
        );

        if hwnd.is_null() { return; }

        let hRgn = CreateRoundRectRgn(0, 0, win_w + 1, win_h + 1, 16, 16);
        if !hRgn.is_null() { SetWindowRgn(hwnd, hRgn, 1); }

        let ctx = egui::Context::default();
        let renderer = EguiRenderer::new();

        let mut style = (*ctx.style()).clone();
        style.visuals = egui::Visuals::dark();
        style.visuals.window_fill = egui::Color32::from_rgba_premultiplied(30, 30, 30, 240);
        style.visuals.window_rounding = egui::Rounding::same(8.0);
        style.visuals.panel_fill = egui::Color32::from_rgba_premultiplied(30, 30, 30, 240);
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgba_premultiplied(40, 40, 40, 240);
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgba_premultiplied(50, 50, 50, 240);
        style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(10, 132, 255);
        style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));
        ctx.set_style(style);

        let state_ptr = Box::into_raw(Box::new(SearchOverlayState {
            ctx, renderer,
            query: String::new(), selected_index: 0, filter: 0,
        }));
        SetWindowLongPtrW(hwnd, 0, state_ptr as isize);

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        let mut msg: MSG = std::mem::zeroed();
        let mut running = true;
        while running {
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT { running = false; break; }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if !running { break; }
            InvalidateRect(hwnd, std::ptr::null(), 0);
            UpdateWindow(hwnd);
            Sleep(16);
        }

        let _ = Box::from_raw(GetWindowLongPtrW(hwnd, 0) as *mut SearchOverlayState);
        // Use the extern "system" DestroyWindow
        let _ = DestroyWindow(hwnd);
    }
}

// Local wrapper for DestroyWindow to avoid naming conflicts
unsafe fn DestroyWindow(hwnd: HWND) -> BOOL {
    extern "system" {
        fn DestroyWindow(hWnd: HWND) -> BOOL;
    }
    DestroyWindow(hwnd)
}

// ===== Helper Functions =====

fn open_file(path: &str) {
    unsafe {
        let path_w = to_wstring(path);
        let open = to_wstring("open");
        ShellExecuteW(std::ptr::null_mut(), open.as_ptr(), path_w.as_ptr(),
            std::ptr::null(), std::ptr::null(), 1);
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB { format!("{:.1} GB", bytes as f64 / GB as f64) }
    else if bytes >= MB { format!("{:.1} MB", bytes as f64 / MB as f64) }
    else if bytes >= KB { format!("{:.1} KB", bytes as f64 / KB as f64) }
    else { format!("{} B", bytes) }
}

fn format_date(timestamp: i64) -> String {
    if timestamp <= 0 { return String::new(); }
    let secs = timestamp;
    let days = secs / 86400;
    let years = 1970 + (days / 365) as i32;
    let remaining_days = days % 365;
    let months = (remaining_days / 30) as i32 + 1;
    let day = (remaining_days % 30) as i32 + 1;
    format!("{:04}-{:02}-{:02}", years, months, day)
}

fn file_icon(entry: &findex_engine::FileEntry) -> &'static str {
    if entry.is_dir { return "\u{1F4C1}"; }
    match entry.extension.to_lowercase().as_str() {
        ".txt" | ".md" | ".log" | ".json" | ".xml" | ".yaml" | ".yml" | ".toml" | ".ini" | ".cfg" => "\u{1F4DD}",
        ".rs" | ".py" | ".js" | ".ts" | ".go" | ".java" | ".c" | ".cpp" | ".h" | ".hpp" | ".cs" | ".rb" | ".php" => "\u{1F4BB}",
        ".html" | ".css" | ".scss" | ".less" | ".jsx" | ".tsx" | ".vue" => "\u{1F310}",
        ".jpg" | ".jpeg" | ".png" | ".gif" | ".bmp" | ".svg" | ".webp" | ".ico" => "\u{1F5BC}",
        ".mp3" | ".wav" | ".flac" | ".ogg" | ".aac" | ".wma" | ".m4a" => "\u{1F3B5}",
        ".mp4" | ".avi" | ".mkv" | ".mov" | ".wmv" | ".flv" | ".webm" => "\u{1F3AC}",
        ".pdf" => "\u{1F4D5}",
        ".doc" | ".docx" => "\u{1F4D8}",
        ".xls" | ".xlsx" | ".csv" => "\u{1F4CA}",
        ".ppt" | ".pptx" => "\u{1F4D9}",
        ".zip" | ".rar" | ".7z" | ".tar" | ".gz" | ".bz2" | ".xz" => "\u{1F4E6}",
        ".exe" | ".dll" | ".msi" | ".bin" => "\u{2699}",
        _ => "\u{1F4C4}",
    }
}
