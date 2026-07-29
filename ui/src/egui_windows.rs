/// Egui-based windows for Findex.
/// Search overlay and settings window using egui UI.

use crate::win32::*;
use crate::SEARCHER;
use crate::egui_win32::EguiRenderer;
#[allow(unused_imports)]
use crate::config::{
    Theme, apply_theme, SearchFilter,
    Config, load_config, get_config, set_config,
    parse_hotkey, format_hotkey,
};

use std::sync::Mutex;

static CURRENT_RESULTS: Mutex<Vec<findex_engine::SearchResult>> = Mutex::new(Vec::new());


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

pub fn run_search_overlay(hinstance: HINSTANCE) {
    unsafe {
        let class_name = to_wstring("FindexEguiSearchClass");
        RegisterClassExW(&WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0, lpfnWndProc: Some(search_wnd_proc),
            cbClsExtra: 0, cbWndExtra: 32,
            hInstance: hinstance, hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(), hIconSm: std::ptr::null_mut(),
        });
        let screen_w = GetSystemMetrics(0);
        let screen_h = GetSystemMetrics(1);
        let win_w = 600; let win_h = 480;
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED,
            class_name.as_ptr(), to_wstring("Findex Search").as_ptr(),
            WS_POPUP | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            (screen_w - win_w) / 2, (screen_h - win_h) / 3, win_w, win_h,
            std::ptr::null_mut(), std::ptr::null_mut(), hinstance, std::ptr::null_mut(),
        );
        if hwnd.is_null() { return; }
        let hRgn = CreateRoundRectRgn(0, 0, win_w + 1, win_h + 1, 16, 16);
        if !hRgn.is_null() { SetWindowRgn(hwnd, hRgn, 1); }
        let ctx = egui::Context::default();
        let renderer = EguiRenderer::new();
        let theme = crate::config::get_effective_theme();
        apply_theme(&ctx, theme);
        let state_ptr = Box::into_raw(Box::new(EguiWindowState {
            ctx, renderer, query: String::new(), selected_index: 0, filter: SearchFilter::All,
        }));
        SetWindowLongPtrW(hwnd, 0, state_ptr as isize);
        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);
        let focused = Box::into_raw(Box::new(true));
        SetWindowLongPtrW(hwnd, 8, focused as isize);
        let mut msg: MSG = std::mem::zeroed();
        let mut running = true;
        while running {
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT { running = false; break; }
                TranslateMessage(&msg); DispatchMessageW(&msg);
            }
            if !running { break; }
            InvalidateRect(hwnd, std::ptr::null(), 0);
            UpdateWindow(hwnd);
            Sleep(16);
        }
        let _ = Box::from_raw(GetWindowLongPtrW(hwnd, 0) as *mut EguiWindowState);
        let _ = Box::from_raw(GetWindowLongPtrW(hwnd, 8) as *mut bool);
        DestroyWindow(hwnd);
    }
}

struct EguiWindowState {
    ctx: egui::Context,
    renderer: EguiRenderer,
    query: String,
    selected_index: usize,
    filter: SearchFilter,
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
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::pos2(0.0, 0.0),
                        egui::vec2(width as f32, height as f32),
                    )),
                    ..Default::default()
                };
                let output = state.ctx.run(input, |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none().fill(ctx.style().visuals.window_fill()))
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("\u{1F50D}");
                                ui.heading("Findex Search");
                            });
                            ui.separator();
                            let mut query = state.query.clone();
                            let resp = ui.add_sized(
                                egui::vec2(ui.available_width(), 32.0),
                                egui::TextEdit::singleline(&mut query)
                                    .hint_text("Search file name...")
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Body),
                            );
                            if ctx.input(|i| i.time == 0.0) {
                                resp.request_focus();
                            }
                            if resp.changed() {
                                state.query = query.clone();
                                state.selected_index = 0;
                            }
                            // Filter bar
                            ui.horizontal(|ui| {
                                for (filter_val, label) in SearchFilter::variants() {
                                    let selected = state.filter == *filter_val;
                                    if ui.selectable_label(selected, *label).clicked() {
                                        state.filter = *filter_val;
                                        state.selected_index = 0;
                                    }
                                }
                            });
                            // Perform search
                            let all_results = if !state.query.is_empty() {
                                SEARCHER.lock().unwrap().as_ref().map(|searcher| {
                                    searcher.search(&findex_engine::SearchQuery {
                                        query: state.query.clone(),
                                        scope: findex_engine::SearchScope::Global,
                                        context_path: None,
                                        max_results: 200,
                                        offset: 0,
                                        sort_by: findex_engine::SortBy::Relevance,
                                    })
                                }).unwrap_or_default()
                            } else {
                                Vec::new()
                            };
                            // Apply filter
                            let results: Vec<&findex_engine::SearchResult> = all_results.iter()
                                .filter(|r| state.filter.matches(&r.entry))
                                .collect();
                            *CURRENT_RESULTS.lock().unwrap() = all_results.clone();
                            if !results.is_empty() {
                                ui.label(format!("Found {} results", results.len()));
                            }
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    for (i, result) in results.iter().enumerate() {
                                        let entry = &result.entry;
                                        let icon = file_icon(entry);
                                        let size_str = if entry.is_dir { String::new() } else { format_size(entry.size) };
                                        let date_str = format_date(entry.modified);
                                        let selected = i == state.selected_index;
                                        let label = format!("{} {}", icon, entry.name);
                                        let detail = if !date_str.is_empty() && !size_str.is_empty() {
                                            format!("{}  {}  {}", entry.path, date_str, size_str)
                                        } else if !date_str.is_empty() {
                                            format!("{}  {}", entry.path, date_str)
                                        } else if !size_str.is_empty() {
                                            format!("{}  {}", entry.path, size_str)
                                        } else {
                                            entry.path.clone()
                                        };
                                        let resp = ui.selectable_label(selected, egui::RichText::new(&label).size(14.0));
                                        if resp.clicked() {
                                            open_file(&entry.path);
                                            PostQuitMessage(0);
                                        }
                                        let dim_color = ctx.style().visuals.override_text_color.unwrap_or(
                                            egui::Color32::from_rgb(128, 128, 128)
                                        );
                                        ui.label(egui::RichText::new(&detail).size(11.0).color(
                                            egui::Color32::from_rgba_premultiplied(
                                                dim_color.r(), dim_color.g(), dim_color.b(), 180
                                            )
                                        ));
                                        ui.separator();
                                    }
                                });
                            if results.is_empty() && !state.query.is_empty() {
                                ui.label("No results found");
                            }
                            ctx.input(|i| {
                                for event in &i.events {
                                    match event {
                                        egui::Event::Key { key: egui::Key::ArrowDown, pressed: true, .. } => {
                                            let count = results.len();
                                            if count > 0 { state.selected_index = (state.selected_index + 1) % count; }
                                        }
                                        egui::Event::Key { key: egui::Key::ArrowUp, pressed: true, .. } => {
                                            let count = results.len();
                                            if count > 0 { state.selected_index = (state.selected_index + count - 1) % count; }
                                        }
                                        egui::Event::Key { key: egui::Key::Enter, pressed: true, .. } => {
                                            if !results.is_empty() {
                                                let idx = state.selected_index.min(results.len() - 1);
                                                open_file(&results[idx].entry.path);
                                                PostQuitMessage(0);
                                            }
                                        }
                                        egui::Event::Key { key: egui::Key::Escape, pressed: true, .. } => {
                                            PostQuitMessage(0);
                                        }
                                        _ => {}
                                    }
                                }
                            });
                        });
                });
                state.renderer.render(
                    &state.ctx, output.shapes, &output.textures_delta,
                    output.pixels_per_point, state.ctx.style().visuals.window_fill(),
                );
                let bgra = state.renderer.as_bgra();
                let bmi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as DWORD,
                        biWidth: width as i32, biHeight: -(height as i32),
                        biPlanes: 1, biBitCount: 32, biCompression: BI_RGB,
                        biSizeImage: 0, biXPelsPerMeter: 0, biYPelsPerMeter: 0,
                        biClrUsed: 0, biClrImportant: 0,
                    },
                    bmiColors: [0; 1],
                };
                StretchDIBits(hdc, 0, 0, width as i32, height as i32, 0, 0, width as i32, height as i32,
                    bgra.as_ptr() as LPCVOID, &bmi, DIB_RGB_COLORS, SRCCOPY);
            }
            EndPaint(hwnd, &ps);
            0
        }
        WM_CLOSE => { PostQuitMessage(0); 0 }
        WM_SIZE => { InvalidateRect(hwnd, std::ptr::null(), 0); DefWindowProcW(hwnd, msg, wparam, lparam) }
        WM_ERASEBKGND => 1,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn open_file(path: &str) {
    let wpath = to_wstring(path);
    unsafe {
        ShellExecuteW(std::ptr::null_mut(), to_wstring("open").as_ptr(),
            wpath.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
    }
}

// ============================================================================
// Egui Settings Window
// ============================================================================

pub struct SettingsState {
    pub ctx: egui::Context,
    pub renderer: EguiRenderer,
    pub config: Config,
    pub status_text: String,
    pub selected_path_index: Option<usize>,
    pub selected_exclude_index: Option<usize>,
    pub pending_browse: bool,
    pub pending_index: bool,
    pub pending_save: bool,
    pub pending_cancel: bool,
    pub theme_idx: usize,
    pub recorded_hotkey: Option<String>,
    pub hotkey_search_buffer: String,
    pub hotkey_settings_buffer: String,
    pub exclude_pattern_buffer: String,
}

impl SettingsState {
    pub fn new() -> Self {
        load_config();
        let cfg = get_config();
        let theme_idx = match cfg.theme.as_str() {
            "dark" => 1,
            "system" => 2,
            _ => 0,
        };
        let ctx = egui::Context::default();
        let theme = Theme::from_str(&cfg.theme);
        apply_theme(&ctx, theme);
        SettingsState {
            ctx, renderer: EguiRenderer::new(), config: cfg.clone(),
            status_text: "Ready".to_string(),
            selected_path_index: None, selected_exclude_index: None,
            pending_browse: false, pending_index: false,
            pending_save: false, pending_cancel: false,
            theme_idx, recorded_hotkey: None,
            hotkey_search_buffer: cfg.hotkey_search.clone(),
            hotkey_settings_buffer: cfg.hotkey_settings.clone(),
            exclude_pattern_buffer: String::new(),
        }
    }
}

pub fn run_settings_window(hinstance: HINSTANCE) {
    unsafe {
        let class_name = to_wstring("FindexEguiSettingsClass");
        RegisterClassExW(&WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as UINT,
            style: 0, lpfnWndProc: Some(settings_wnd_proc),
            cbClsExtra: 0, cbWndExtra: 32,
            hInstance: hinstance, hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(), hIconSm: std::ptr::null_mut(),
        });
        let screen_w = GetSystemMetrics(0);
        let screen_h = GetSystemMetrics(1);
        let win_w = 620; let win_h = 580;
        let x = (screen_w - win_w) / 2; let y = (screen_h - win_h) / 3;
        let hwnd = CreateWindowExW(0, class_name.as_ptr(),
            to_wstring("Findex Settings").as_ptr(),
            WS_OVERLAPPEDWINDOW & !WS_MAXIMIZEBOX,
            x, y, win_w, win_h,
            std::ptr::null_mut(), std::ptr::null_mut(), hinstance, std::ptr::null_mut());
        if hwnd.is_null() { return; }
        let state = Box::into_raw(Box::new(SettingsState::new()));
        SetWindowLongPtrW(hwnd, 0, state as isize);
        ShowWindow(hwnd, SW_SHOWNORMAL);
        SetForegroundWindow(hwnd);
        let mut msg: MSG = std::mem::zeroed();
        let mut running = true;
        while running {
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT { running = false; break; }
                TranslateMessage(&msg); DispatchMessageW(&msg);
            }
            if !running { break; }
            let state_ptr = GetWindowLongPtrW(hwnd, 0) as *mut SettingsState;
            if !state_ptr.is_null() {
                let state = &mut *state_ptr;
                if state.pending_browse {
                    state.pending_browse = false;
                    if let Some(path) = browse_folder_dialog(hwnd) {
                        if !state.config.index_paths.contains(&path) {
                            state.config.index_paths.push(path);
                        }
                    }
                }
                if state.pending_index {
                    state.pending_index = false;
                    state.status_text = "Indexing...".to_string();
                    perform_index(state);
                }
                if state.pending_save {
                    state.pending_save = false;
                    state.config.hotkey_search = state.hotkey_search_buffer.clone();
                    state.config.hotkey_settings = state.hotkey_settings_buffer.clone();
                    set_config(state.config.clone());
                    if state.config.save().is_ok() {
                        state.status_text = "Saved. Hotkeys will apply on next Settings open.".to_string();
                    } else {
                        state.status_text = "Save failed".to_string();
                    }
                }
                if state.pending_cancel {
                    state.pending_cancel = false;
                    running = false;
                    PostQuitMessage(0);
                }
            }
            InvalidateRect(hwnd, std::ptr::null(), 0);
            UpdateWindow(hwnd);
            Sleep(16);
        }
        let _ = Box::from_raw(GetWindowLongPtrW(hwnd, 0) as *mut SettingsState);
        DestroyWindow(hwnd);
    }
}

unsafe fn browse_folder_dialog(hwnd: HWND) -> Option<String> {
    let mut display_name = [0u16; 260];
    let title = to_wstring("Select folder to index");
    let mut bi = BROWSEINFOW {
        hwndOwner: hwnd,
        pidlRoot: std::ptr::null_mut(),
        pszDisplayName: display_name.as_mut_ptr(),
        lpszTitle: title.as_ptr(),
        ulFlags: BIF_USENEWUI | BIF_RETURNONLYFSDIRS,
        lpfn: std::ptr::null_mut(),
        lParam: 0, iImage: 0,
    };
    let pidl = SHBrowseForFolderW(&mut bi);
    if pidl.is_null() { return None; }
    let mut path_buf = vec![0u16; 260];
    let success = SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr());
    CoTaskMemFree(pidl);
    if success == 0 { return None; }
    let path = from_wstring(path_buf.as_ptr());
    if path.is_empty() { None } else { Some(path) }
}

unsafe fn perform_index(state: &mut SettingsState) {
    let paths: Vec<String> = state.config.index_paths.clone();
    let excludes: Vec<String> = state.config.exclude_patterns.clone();
    let mut all_entries = Vec::new();
    for path_str in &paths {
        let path = std::path::Path::new(path_str);
        if path.exists() {
            if let Ok(entries) = findex_engine::FsWalker::walk_with_excludes(path, 0, &excludes) {
                all_entries.extend(entries);
            }
        }
    }
    if !all_entries.is_empty() {
        let mut seen = std::collections::HashSet::new();
        all_entries.retain(|e| seen.insert(e.path.clone()));
        let mut index = findex_engine::TrieIndex::new();
        index.load_entries(all_entries);
        let searcher = findex_engine::Searcher::new(index);
        *SEARCHER.lock().unwrap() = Some(searcher);
        let s = SEARCHER.lock().unwrap();
        if let Some(ref searcher) = *s {
            let st = searcher.status();
            state.status_text = format!("Indexed: {} files, {} folders", st.total_files, st.total_folders);
        } else {
            state.status_text = "Index complete".to_string();
        }
    } else {
        state.status_text = "No files found".to_string();
    }
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_PAINT => {
            let state_ptr = GetWindowLongPtrW(hwnd, 0) as *mut SettingsState;
            if state_ptr.is_null() { return DefWindowProcW(hwnd, msg, wparam, lparam); }
            let state = &mut *state_ptr;
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = BeginPaint(hwnd, &mut ps);
            let mut rect: RECT = std::mem::zeroed();
            GetClientRect(hwnd, &mut rect);
            let width = (rect.right - rect.left) as usize;
            let height = (rect.bottom - rect.top) as usize;
            if width > 0 && height > 0 {
                state.renderer.resize(width, height);
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::pos2(0.0, 0.0), egui::vec2(width as f32, height as f32))),
                    ..Default::default()
                };
                let new_theme = match state.theme_idx { 1 => "dark", 2 => "system", _ => "light" };
                if state.config.theme != new_theme {
                    state.config.theme = new_theme.to_string();
                    apply_theme(&state.ctx, Theme::from_str(new_theme));
                }
                let is_dark = state.ctx.style().visuals.dark_mode;
                let card_bg = if is_dark { egui::Color32::from_rgb(30, 30, 30) } else { egui::Color32::from_rgb(255, 255, 255) };
                let card_stroke = if is_dark { egui::Color32::from_rgb(51, 51, 51) } else { egui::Color32::from_rgb(224, 224, 224) };
                let accent = egui::Color32::from_rgb(10, 132, 255);
                let text_secondary = if is_dark { egui::Color32::from_rgb(160, 160, 160) } else { egui::Color32::from_rgb(102, 102, 102) };
                let card_frame = egui::Frame {
                    fill: card_bg,
                    stroke: egui::Stroke::new(1.0_f32, card_stroke),
                    rounding: egui::Rounding::same(8.0),
                    inner_margin: egui::Margin::symmetric(12.0, 10.0),
                    ..Default::default()
                };
                let output = state.ctx.run(input, |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none().fill(ctx.style().visuals.window_fill()))
                        .show(ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                // Title
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("\u{2699}").size(18.0).color(accent));
                                    let title_color = if is_dark { egui::Color32::from_rgb(255, 255, 255) } else { egui::Color32::from_rgb(29, 29, 31) };
                                    ui.heading(egui::RichText::new("Findex Settings").color(title_color));
                                });
                                ui.add_space(8.0);
                                // Theme Card
                                card_frame.show(ui, |ui| {
                                    ui.label(egui::RichText::new("\u{1F3A8} Theme").size(14.0).color(accent).strong());
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        ui.selectable_value(&mut state.theme_idx, 0, "Light");
                                        ui.selectable_value(&mut state.theme_idx, 1, "Dark");
                                        ui.selectable_value(&mut state.theme_idx, 2, "System");
                                    });
                                });
                                ui.add_space(6.0);
                                // Hotkeys Card
                                card_frame.show(ui, |ui| {
                                    ui.label(egui::RichText::new("\u{2328} Hotkeys").size(14.0).color(accent).strong());
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("Search:").color(text_secondary));
                                        if ui.button(&state.hotkey_search_buffer).clicked() {
                                            state.recorded_hotkey = Some("search".to_string());
                                            state.hotkey_search_buffer = "Recording...".to_string();
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("Settings:").color(text_secondary));
                                        if ui.button(&state.hotkey_settings_buffer).clicked() {
                                            state.recorded_hotkey = Some("settings".to_string());
                                            state.hotkey_settings_buffer = "Recording...".to_string();
                                        }
                                    });
                                    if let Some(ref target) = state.recorded_hotkey.clone() {
                                        ctx.input(|i| {
                                            for event in &i.events {
                                                if let egui::Event::Key { key, pressed: true, modifiers, .. } = event {
                                                    let has_mod = modifiers.ctrl || modifiers.shift || modifiers.alt || modifiers.mac_cmd;
                                                    if has_mod {
                                                        let mut mods = 0u32;
                                                        if modifiers.ctrl { mods |= 0x0002; }
                                                        if modifiers.shift { mods |= 0x0004; }
                                                        if modifiers.alt { mods |= 0x0001; }
                                                        if modifiers.mac_cmd { mods |= 0x0008; }
                                                        let vk = match key {
                                                            egui::Key::Space => 0x20, egui::Key::Enter => 0x0D,
                                                            egui::Key::Escape => 0x1B, egui::Key::Tab => 0x09,
                                                            egui::Key::F1 => 0x70, egui::Key::F2 => 0x71,
                                                            egui::Key::F3 => 0x72, egui::Key::F4 => 0x73,
                                                            egui::Key::F5 => 0x74, egui::Key::F6 => 0x75,
                                                            egui::Key::F7 => 0x76, egui::Key::F8 => 0x77,
                                                            egui::Key::F9 => 0x78, egui::Key::F10 => 0x79,
                                                            egui::Key::F11 => 0x7A, egui::Key::F12 => 0x7B,
                                                            egui::Key::ArrowUp => 0x26, egui::Key::ArrowDown => 0x28,
                                                            egui::Key::ArrowLeft => 0x25, egui::Key::ArrowRight => 0x27,
                                                            _ => continue,
                                                        };
                                                        let hotkey_str = format_hotkey(mods, vk);
                                                        if target == "search" {
                                                            state.hotkey_search_buffer = hotkey_str;
                                                        } else {
                                                            state.hotkey_settings_buffer = hotkey_str;
                                                        }
                                                        state.recorded_hotkey = None;
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                                ui.add_space(6.0);
                                // Index Paths Card
                                card_frame.show(ui, |ui| {
                                    ui.label(egui::RichText::new("\u{1F4C1} Index Paths").size(14.0).color(accent).strong());
                                    ui.add_space(6.0);
                                    let mut remove_idx = None;
                                    for (i, path) in state.config.index_paths.clone().iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            let sel = state.selected_path_index == Some(i);
                                            if ui.selectable_label(sel, &**path).clicked() { state.selected_path_index = Some(i); }
                                            if ui.button(egui::RichText::new("X").color(egui::Color32::from_rgb(255, 69, 58))).clicked() { remove_idx = Some(i); }
                                        });
                                    }
                                    if let Some(idx) = remove_idx {
                                        if idx < state.config.index_paths.len() { state.config.index_paths.remove(idx); state.selected_path_index = None; }
                                    }
                                    ui.horizontal(|ui| {
                                        if ui.button("+ Browse").clicked() { state.pending_browse = true; }
                                        ui.label(egui::RichText::new("(click + to add folder)").size(11.0).color(text_secondary));
                                    });
                                });
                                ui.add_space(6.0);
                                // Exclude Patterns Card
                                card_frame.show(ui, |ui| {
                                    ui.label(egui::RichText::new("\u{1F6AB} Exclude Patterns").size(14.0).color(accent).strong());
                                    ui.add_space(6.0);
                                    let mut remove_exclude = None;
                                    for (i, pattern) in state.config.exclude_patterns.clone().iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            let sel = state.selected_exclude_index == Some(i);
                                            if ui.selectable_label(sel, &**pattern).clicked() { state.selected_exclude_index = Some(i); }
                                            if ui.button(egui::RichText::new("X").color(egui::Color32::from_rgb(255, 69, 58))).clicked() { remove_exclude = Some(i); }
                                        });
                                    }
                                    if let Some(idx) = remove_exclude {
                                        if idx < state.config.exclude_patterns.len() { state.config.exclude_patterns.remove(idx); state.selected_exclude_index = None; }
                                    }
                                    ui.horizontal(|ui| {
                                        let _resp = ui.add_sized(egui::vec2(200.0, 20.0),
                                            egui::TextEdit::singleline(&mut state.exclude_pattern_buffer)
                                                .hint_text("e.g. node_modules"));
                                        if ui.button("Add").clicked() && !state.exclude_pattern_buffer.is_empty() {
                                            let p = state.exclude_pattern_buffer.trim().to_string();
                                            if !p.is_empty() && !state.config.exclude_patterns.contains(&p) {
                                                state.config.exclude_patterns.push(p);
                                            }
                                            state.exclude_pattern_buffer.clear();
                                        }
                                    });
                                });
                                ui.add_space(6.0);
                                // Search Options Card
                                card_frame.show(ui, |ui| {
                                    ui.label(egui::RichText::new("\u{1F50D} Search Options").size(14.0).color(accent).strong());
                                    ui.add_space(6.0);
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut state.config.enable_pinyin, "Enable Pinyin");
                                        ui.checkbox(&mut state.config.enable_fuzzy, "Enable Fuzzy");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut state.config.show_hidden, "Show Hidden");
                                        ui.checkbox(&mut state.config.auto_index, "Auto-Index");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(egui::RichText::new("Max Results:").color(text_secondary));
                                        ui.add(egui::Slider::new(&mut state.config.max_results, 10..=500).text("items"));
                                    });
                                });
                                ui.add_space(8.0);
                                // Action Buttons
                                ui.horizontal(|ui| {
                                    if ui.add(egui::Button::new(egui::RichText::new("Index Now").color(accent)).fill(card_bg).rounding(8.0)).clicked() {
                                        state.pending_index = true;
                                    }
                                    if ui.add(egui::Button::new(egui::RichText::new("Save").color(egui::Color32::from_rgb(48, 209, 88))).fill(card_bg).rounding(8.0)).clicked() {
                                        state.pending_save = true;
                                    }
                                    if ui.add(egui::Button::new(egui::RichText::new("Cancel").color(egui::Color32::from_rgb(255, 69, 58))).fill(card_bg).rounding(8.0)).clicked() {
                                        state.pending_cancel = true;
                                    }
                                });
                                ui.add_space(6.0);
                                // Status Bar
                                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                                    ui.label(egui::RichText::new(&state.status_text).size(11.0).color(text_secondary));
                                });
                            });
                        });
                });
                state.renderer.render(&state.ctx, output.shapes, &output.textures_delta,
                    output.pixels_per_point, state.ctx.style().visuals.window_fill());
                let bgra = state.renderer.as_bgra();
                let bmi = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as DWORD,
                        biWidth: width as i32, biHeight: -(height as i32),
                        biPlanes: 1, biBitCount: 32, biCompression: BI_RGB,
                        biSizeImage: 0, biXPelsPerMeter: 0, biYPelsPerMeter: 0,
                        biClrUsed: 0, biClrImportant: 0,
                    },
                    bmiColors: [0; 1],
                };
                StretchDIBits(hdc, 0, 0, width as i32, height as i32, 0, 0, width as i32, height as i32,
                    bgra.as_ptr() as LPCVOID, &bmi, DIB_RGB_COLORS, SRCCOPY);
            }
            EndPaint(hwnd, &ps);
            0
        }
        WM_CLOSE => { PostQuitMessage(0); 0 }
        WM_SIZE => { InvalidateRect(hwnd, std::ptr::null(), 0); DefWindowProcW(hwnd, msg, wparam, lparam) }
        WM_ERASEBKGND => 1,
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}




