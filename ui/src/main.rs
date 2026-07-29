#![allow(non_snake_case)]
#![allow(unused_unsafe)]
#![allow(dead_code)]

/// Findex - Fast Windows file search tool
/// Single binary: no args → GUI/tray mode, CLI commands → CLI mode

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
    let args: Vec<String> = std::env::args().collect();

    // Detect CLI mode: if args contain a known command, run CLI
    if args.len() > 1 {
        let cmd = args[1].as_str();
        if matches!(cmd, "search" | "index" | "status" | "help" | "--help" | "-h") {
            cli_mode(&args);
            return;
        }
    }

    // GUI / tray mode
    unsafe {
        let hinstance = GetModuleHandleW(std::ptr::null());
        if hinstance.is_null() {
            eprintln!("Failed to get module handle");
            return;
        }

        // Load or build the index
        config::load_config();
        load_or_build_index();

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

// ===== CLI Mode =====

fn cli_mode(args: &[String]) {
    let prog = "findex";
    match args[1].as_str() {
        "search" => cmd_search(&args[2..]),
        "index" => cmd_index(&args[2..]),
        "status" => cmd_status(&args[2..]),
        "help" | "--help" | "-h" => print_usage(prog),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage(prog);
        }
    }
}

fn print_usage(prog: &str) {
    eprintln!("Findex v{} - Fast Windows file search", env!("CARGO_PKG_VERSION"));
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  {prog}                         Start GUI (system tray)");
    eprintln!("  {prog} index <directory>        Index a directory");
    eprintln!("  {prog} search <query>           Search indexed files");
    eprintln!("  {prog} status                   Show index status");
    eprintln!("  {prog} help                     Show this help");
    eprintln!();
    eprintln!("SEARCH OPTIONS:");
    eprintln!("  --max <n>       Maximum results (default: 50)");
    eprintln!("  --context <p>   Context path for scoring");
    eprintln!("  --json          Output as JSON");
    eprintln!();
    eprintln!("INDEX OPTIONS:");
    eprintln!("  --depth <n>     Max recursion depth (0 = unlimited)");
}

fn parse_flag(args: &[String], flag: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(a) = iter.next() {
        if a == flag {
            return iter.next().cloned();
        }
    }
    None
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn cmd_search(args: &[String]) {
    if args.is_empty() || args[0].starts_with("--") {
        eprintln!("Error: missing search query");
        eprintln!("Usage: findex search <query> [options]");
        return;
    }

    let query = args[0].clone();
    let rest = &args[1..];
    let max_results: usize = parse_flag(rest, "--max").and_then(|s| s.parse().ok()).unwrap_or(50);
    let context = parse_flag(rest, "--context");
    let json_output = has_flag(rest, "--json");

    // Use the AppData index path (same as GUI mode)
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let db_path = if appdata.is_empty() {
        "findex.db".to_string()
    } else {
        format!("{}\\Findex\\index.db", appdata)
    };

    let storage = match findex_engine::Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            return;
        }
    };

    let entries = match storage.load_entries() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error loading entries: {}", e);
            return;
        }
    };

    if entries.is_empty() {
        eprintln!("No entries in database. Run 'findex index' first.");
        return;
    }

    let mut index = findex_engine::TrieIndex::new();
    index.load_entries(entries);
    eprintln!("Searching {} indexed entries...", index.len());

    let searcher = findex_engine::Searcher::new(index);
    let search_query = findex_engine::SearchQuery {
        query: query.clone(),
        scope: findex_engine::SearchScope::Global,
        context_path: context,
        max_results,
        offset: 0,
        sort_by: findex_engine::SortBy::Relevance,
    };

    let results = searcher.search(&search_query);

    if json_output {
        println!("{}", results_to_json(&results));
    } else {
        if results.is_empty() {
            println!("No results found for '{}'", query);
            return;
        }
        println!("Found {} results for '{}':", results.len(), query);
        println!("{:-^80}", "");
        for (i, result) in results.iter().enumerate().take(50) {
            let entry = &result.entry;
            let size_str = if entry.is_dir {
                String::new()
            } else {
                format_size(entry.size)
            };
            let match_str = format!("[{} {}]", result.match_type, result.score);
            println!(
                "{:>4}. {:<12} {:>8}  {}",
                i + 1, match_str, size_str, entry.path
            );
        }
        if results.len() > 50 {
            println!("... and {} more results", results.len() - 50);
        }
    }
}

fn cmd_index(args: &[String]) {
    let path = if args.is_empty() || args[0].starts_with("--") {
        ".".to_string()
    } else {
        args[0].clone()
    };

    let rest = if args.is_empty() || args[0].starts_with("--") { args } else { &args[1..] };
    let depth: usize = parse_flag(rest, "--depth").and_then(|s| s.parse().ok()).unwrap_or(0);

    // Use AppData path
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let dir = if appdata.is_empty() { String::new() } else { format!("{}\\Findex", appdata) };
    let _ = std::fs::create_dir_all(&dir);
    let db_path = if appdata.is_empty() { "findex.db".to_string() } else { format!("{}\\index.db", dir) };

    let root = std::path::PathBuf::from(&path);
    if !root.exists() {
        eprintln!("Error: path does not exist: {}", path);
        return;
    }

    eprintln!("Indexing {} (depth: {})...", root.display(), if depth == 0 { "unlimited".to_string() } else { depth.to_string() });

    let start = std::time::Instant::now();
    let entries = match findex_engine::FsWalker::walk(&root, depth) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error walking directory: {}", e);
            return;
        }
    };
    let elapsed = start.elapsed();

    if entries.is_empty() {
        eprintln!("No files found (or all were skipped).");
        return;
    }

    eprintln!("Found {} files/directories in {:?}", entries.len(), elapsed);

    let mut index = findex_engine::TrieIndex::new();
    for entry in &entries {
        index.insert(entry.clone());
    }
    eprintln!("Index built with {} entries", index.len());

    let storage = match findex_engine::Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            return;
        }
    };

    if let Err(e) = storage.save_entries(&entries) {
        eprintln!("Error saving index: {}", e);
        return;
    }

    eprintln!("Index saved to {}", db_path);

    let file_count = entries.iter().filter(|e| !e.is_dir).count();
    let dir_count = entries.iter().filter(|e| e.is_dir).count();
    eprintln!("Stats: {} files, {} directories", file_count, dir_count);
}

fn cmd_status(args: &[String]) {
    let appdata = std::env::var("APPDATA").unwrap_or_default();
    let db_path = if appdata.is_empty() {
        parse_flag(args, "--db").unwrap_or_else(|| "findex.db".to_string())
    } else {
        format!("{}\\Findex\\index.db", appdata)
    };

    let storage = match findex_engine::Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error opening database: {}", e);
            return;
        }
    };

    let entries = match storage.load_entries() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error loading entries: {}", e);
            return;
        }
    };

    let total = entries.len();
    let files = entries.iter().filter(|e| !e.is_dir).count();
    let dirs = entries.iter().filter(|e| e.is_dir).count();
    let mem_estimate: usize = entries.iter().map(|e| e.name.len() + e.path.len() + e.parent_path.len() + 64).sum();

    println!("Findex Index Status");
    println!("{:-^50}", "");
    println!("Database:     {}", db_path);
    println!("Total:        {} entries", total);
    println!("  Files:      {}", files);
    println!("  Directories: {}", dirs);
    println!("Memory est.:  {} KB", mem_estimate / 1024);
}

fn results_to_json(results: &[findex_engine::SearchResult]) -> String {
    let mut json = String::from("[");
    for (i, r) in results.iter().enumerate() {
        if i > 0 { json.push(','); }
        json.push_str(&format!(
            r#"{{"score":{},"match":"{}","entry":{}}}"#,
            r.score, r.match_type, r.entry.to_json()
        ));
    }
    json.push(']');
    json
}

/// Get the canonical index path in %APPDATA%\Findex\index.db.

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

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

fn get_index_path() -> Option<String> {
    let appdata = std::env::var("APPDATA").ok()?;
    let dir = format!("{}\\Findex", appdata);
    let _ = std::fs::create_dir_all(&dir);
    Some(format!("{}\\index.db", dir))
}

/// Load existing index and apply incremental USN Journal updates.
/// On first launch, uses USN Journal for fast full enumeration.

/// Get the current executable path.
fn get_exe_path() -> Vec<u16> {
    unsafe {
        let mut buf = vec![0u16; 260];
        GetModuleFileNameW(std::ptr::null_mut(), buf.as_mut_ptr(), 260);
        let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
        buf.truncate(len + 1);
        buf
    }
}
fn load_or_build_index() {
    // 1. Try loading existing index from AppData
    let mut need_full_index = true;
    if let Some(index_path) = get_index_path() {
        if let Ok(storage) = findex_engine::Storage::open(&index_path) {
            if let Ok(entries) = storage.load_entries() {
                if !entries.is_empty() {
                    let mut index = findex_engine::TrieIndex::new();
                    index.load_entries(entries);
                    let searcher = findex_engine::Searcher::new(index);
                    *SEARCHER.lock().unwrap() = Some(searcher);
                    eprintln!("Loaded index from {}: {} entries", index_path, storage.entry_count().unwrap_or(0));
                    need_full_index = false;

                    // 2. Try incremental USN Journal update
                    let changes = apply_usn_incremental_updates();
                    if changes > 0 {
                        eprintln!("Applied {} incremental changes from USN Journal", changes);
                        // Save the updated index
                        if let Some(ref searcher) = *SEARCHER.lock().unwrap() {
                            if let Ok(storage) = findex_engine::Storage::open(&index_path) {
                                let entries = searcher.index().all_entries();
                                let _ = storage.save_entries(&entries);
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. No index found — build full index via USN Journal
    if need_full_index {
        eprintln!("No existing index found. Building index via USN Journal...");
        build_index_via_usn();
    }
}

/// Apply incremental updates from USN Journal.
/// Returns the number of changes applied.
fn apply_usn_incremental_updates() -> usize {
    let mut journal_state = config::UsnJournalState::load();
    let mut total_changes = 0;

    for letter in 'C'..='Z' {
        let vol = format!("{}:\\", letter);
        if !std::path::Path::new(&vol).exists() {
            continue;
        }

        // Check if we have a saved state for this volume
        let vol_state = journal_state.get_volume(&letter.to_string());
        if vol_state.is_none() {
            continue;
        }
        let vol_state = vol_state.unwrap();

        // Query current journal ID to check validity
        if let Ok((current_usn, current_journal_id, _)) = findex_engine::UsnReader::query_journal_id(letter) {
            // Check if journal has been reset
            if !journal_state.is_journal_valid(&letter.to_string(), current_journal_id) {
                eprintln!("USN Journal reset for {}: (re)building full index", vol);
                // Full rebuild needed for this volume
                continue;
            }

            // Read changes since last_usn
            if current_usn > vol_state.last_usn {
                eprintln!("Reading USN Journal changes for {} from USN {} to {}...", vol, vol_state.last_usn, current_usn);
                if let Ok(changes) = findex_engine::UsnReader::read_changes(letter, vol_state.last_usn) {
                    let mut searcher_guard = SEARCHER.lock().unwrap();
                    if let Some(ref mut searcher) = *searcher_guard {
                        let mut changed = 0;
                        for change in &changes {
                            match change {
                                findex_engine::usn_reader::FileChange::Added(entry) |
                                findex_engine::usn_reader::FileChange::Modified(entry) => {
                                    searcher.index_mut().insert(entry.clone());
                                    changed += 1;
                                }
                                findex_engine::usn_reader::FileChange::Deleted(path) => {
                                    // Find and remove by path
                                    let to_remove: Vec<i64> = searcher.index().all_entries()
                                        .iter()
                                        .filter(|e| e.path == *path)
                                        .map(|e| e.id)
                                        .collect();
                                    for id in to_remove {
                                        searcher.index_mut().remove(id);
                                        changed += 1;
                                    }
                                }
                                findex_engine::usn_reader::FileChange::Renamed(old_path, _new_path) => {
                                    // Find old entry and update path
                                    let to_remove: Vec<i64> = searcher.index().all_entries()
                                        .iter()
                                        .filter(|e| e.path == *old_path)
                                        .map(|e| e.id)
                                        .collect();
                                    for id in to_remove {
                                        searcher.index_mut().remove(id);
                                        changed += 1;
                                    }
                                    // The new name will be picked up by the next Added/Modified event
                                    // Or we could add it here if we had the full entry
                                }
                            }
                        }
                        total_changes += changed;
                        eprintln!("Applied {} changes for {}", changed, vol);
                    }
                    // Update journal state
                    journal_state.update_volume(&letter.to_string(), current_usn, current_journal_id);
                }
            }
        }
    }

    journal_state.save();
    total_changes
}

/// Build full index using USN Journal (fast, ~1-2 seconds per volume).
/// Falls back to FsWalker if USN Journal is unavailable.
fn build_index_via_usn() {
    // Check if we can use USN Journal; if not, offer to relaunch as admin
    let mut any_usn_available = false;
    for letter in 'C'..='Z' {
        let vol = format!("{}:\\", letter);
        if std::path::Path::new(&vol).exists() {
            if findex_engine::UsnReader::is_usn_available(letter) {
                any_usn_available = true;
                break;
            }
        }
    }

    if !any_usn_available {
        eprintln!("  Tip: Run as administrator for USN Journal fast indexing (1-2 seconds per volume).");
        eprintln!("  Using standard scanning (FsWalker) as fallback.");
    }

    let mut journal_state = config::UsnJournalState::load();
    let mut all_entries = Vec::new();
    let mut built_via_usn = Vec::new();
    let mut built_via_walker = Vec::new();

    for letter in 'C'..='Z' {
        let vol = format!("{}:\\", letter);
        if !std::path::Path::new(&vol).exists() {
            continue;
        }

        // Try USN Journal first
        if findex_engine::UsnReader::is_usn_available(letter) {
            eprintln!("Indexing {} via USN Journal...", vol);
            match findex_engine::UsnReader::enumerate_volume(letter) {
                Ok(entries) => {
                    eprintln!("  USN Journal returned {} entries for {}", entries.len(), vol);
                    all_entries.extend(entries);
                    built_via_usn.push(letter.to_string());

                    // Save journal state for incremental updates
                    if let Ok((next_usn, journal_id, _)) = findex_engine::UsnReader::query_journal_id(letter) {
                        journal_state.update_volume(&letter.to_string(), next_usn, journal_id);
                    }
                    continue;
                }
                Err(e) => {
                    eprintln!("  USN Journal failed for {}: {}. Falling back to FsWalker.", vol, e);
                }
            }
        } else {
            eprintln!("  USN Journal not available for {}. Needs admin rights. Using FsWalker.", vol);
        }

        // Fallback: FsWalker
        let path = std::path::Path::new(&vol);
        let excludes: Vec<String> = config::get_config().exclude_patterns.clone();
        if let Ok(entries) = findex_engine::FsWalker::walk_with_excludes(path, 0, &excludes) {
            eprintln!("  FsWalker returned {} entries for {}", entries.len(), vol);
            all_entries.extend(entries);
            built_via_walker.push(letter.to_string());
        }
    }

    // Deduplicate by path
    let mut seen = std::collections::HashSet::new();
    all_entries.retain(|e| seen.insert(e.path.clone()));

    if !all_entries.is_empty() {
        let mut index = findex_engine::TrieIndex::new();
        index.load_entries(all_entries);
        let searcher = findex_engine::Searcher::new(index);

        // Save to AppData
        if let Some(index_path) = get_index_path() {
            if let Ok(storage) = findex_engine::Storage::open(&index_path) {
                let entries = searcher.index().all_entries();
                if let Err(e) = storage.save_entries(&entries) {
                    eprintln!("Failed to save index: {}", e);
                } else {
                    eprintln!("Saved index to {}: {} entries", index_path, entries.len());
                }
            }
        }

        // Save USN journal state for incremental updates
        journal_state.save();

        let status = searcher.status();
        *SEARCHER.lock().unwrap() = Some(searcher);

        let sources: Vec<String> = built_via_usn.iter().map(|v| format!("{} (USN)", v))
            .chain(built_via_walker.iter().map(|v| format!("{} (FsWalker)", v)))
            .collect();
        eprintln!("Index built: {} files, {} folders on {}", status.total_files, status.total_folders, sources.join(", "));
    } else {
        eprintln!("No files found during indexing");
    }
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










