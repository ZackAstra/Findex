/// Findex Tauri v2 App
/// Architecture: Tauri WebView2 settings + egui search overlay + Win32 hotkeys/tray

use std::sync::Mutex;
use std::path::Path;
use tauri::Emitter;

mod config;
mod win32;
mod egui_overlay;

use findex_engine::Searcher;
use findex_engine::TrieIndex;
use findex_engine::Storage;
use findex_engine::SearchQuery;
use findex_engine::SearchScope;
use findex_engine::SortBy;
use findex_engine::UsnReader;
use findex_engine::FsWalker;

/// Global searcher instance, shared between Tauri commands and egui overlay.
pub static SEARCHER: Mutex<Option<Searcher>> = Mutex::new(None);
/// Global config instance.
pub static CONFIG: Mutex<Option<config::AppConfig>> = Mutex::new(None);

const INDEX_PATH: &str = "index.json";

fn get_index_path() -> Option<String> {
    let appdata = std::env::var("APPDATA").ok()?;
    let dir = format!("{}\\Findex", appdata);
    let _ = std::fs::create_dir_all(&dir);
    Some(format!("{}\\{}", dir, INDEX_PATH))
}

// ===== Tauri Commands =====

#[tauri::command]
fn search(query: String, max_results: Option<usize>, filter: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    let searcher = SEARCHER.lock().map_err(|e| e.to_string())?;
    let searcher = searcher.as_ref().ok_or("Index not loaded")?;

    let max = max_results.unwrap_or(50);
    let search_query = SearchQuery {
        query: query.clone(),
        scope: SearchScope::Global,
        context_path: None,
        max_results: max,
        offset: 0,
        sort_by: SortBy::Relevance,
    };

    let results = searcher.search(&search_query);

    let filtered: Vec<serde_json::Value> = results.iter()
        .filter(|r| {
            if let Some(ref f) = filter {
                config::SearchFilter::from_str(f).matches(&r.entry)
            } else {
                true
            }
        })
        .map(|r| {
            serde_json::json!({
                "id": r.entry.id,
                "name": r.entry.name,
                "path": r.entry.path,
                "parent_path": r.entry.parent_path,
                "size": r.entry.size,
                "created": r.entry.created,
                "modified": r.entry.modified,
                "is_dir": r.entry.is_dir,
                "is_hidden": r.entry.is_hidden,
                "extension": r.entry.extension,
                "volume": r.entry.volume,
                "score": r.score,
                "match_type": r.match_type.to_string(),
            })
        })
        .collect();

    Ok(filtered)
}

#[tauri::command]
fn index_status() -> Result<serde_json::Value, String> {
    let searcher = SEARCHER.lock().map_err(|e| e.to_string())?;
    let searcher = searcher.as_ref().ok_or("Index not loaded")?;
    let status = searcher.status();

    Ok(serde_json::json!({
        "total_files": status.total_files,
        "total_folders": status.total_folders,
        "indexed_volumes": status.indexed_volumes,
        "memory_usage_bytes": status.memory_usage_bytes,
        "last_index_time": status.last_index_time,
        "is_indexing": status.is_indexing,
    }))
}

#[tauri::command]
fn index_now(app_handle: tauri::AppHandle) -> Result<String, String> {
    let handle = app_handle.clone();
    std::thread::spawn(move || {
        match build_index() {
            Ok(_) => {
                let _ = handle.emit("index-complete", serde_json::json!({}));
            }
            Err(e) => {
                eprintln!("Index error: {}", e);
            }
        }
    });
    Ok("Indexing started".to_string())
}

#[tauri::command]
fn config_read() -> Result<config::AppConfig, String> {
    let cfg = CONFIG.lock().map_err(|e| e.to_string())?;
    Ok(cfg.clone().unwrap_or_default())
}

#[tauri::command]
fn config_write(new_config: config::AppConfig) -> Result<(), String> {
    config::save_config(&new_config);
    if let Ok(mut cfg) = CONFIG.lock() {
        *cfg = Some(new_config.clone());
    }
    Ok(())
}

// ===== App Entry Point =====

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load config
    config::load_config();

    // Load cached index from disk (fast startup)
    if let Err(e) = load_index_from_disk() {
        eprintln!("Failed to load cached index: {}", e);
    }

    // Build fresh index in background
    std::thread::spawn(|| {
        if let Err(e) = build_index() {
            eprintln!("Index build error: {}", e);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                win32::run_hotkey_loop(app_handle);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            search,
            index_status,
            index_now,
            config_read,
            config_write,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ===== Index Building =====

fn build_index() -> Result<(), String> {
    let config_data = {
        let guard = CONFIG.lock().map_err(|e| e.to_string())?;
        guard.clone().unwrap_or_default()
    };

    let volumes = if config_data.index_dirs.is_empty() {
        vec!["C:\\".to_string()]
    } else {
        config_data.index_dirs.clone()
    };

    let exclude_patterns = config_data.exclude_patterns.clone();
    drop(config_data);

    let mut trie = TrieIndex::new();
    let journal_state = config::UsnJournalState::load();

    for volume in &volumes {
        let vol_letter = volume.trim_end_matches('\\').trim_end_matches(':');
        let vol_char = vol_letter.chars().next().unwrap_or('C');
        let vol_str = vol_letter.to_uppercase();

        // Try USN Journal first
        match UsnReader::enumerate_volume(vol_char) {
            Ok(entries) => {
                for entry in &entries {
                    let _id = trie.insert(entry.clone());
                }
                eprintln!("USN indexed {}: {} entries", vol_str, entries.len());
            }
            Err(_) => {
                // Fallback to FsWalker
                let root_path = Path::new(volume);
                match FsWalker::walk_with_excludes(root_path, 0, &exclude_patterns) {
                    Ok(entries) => {
                        for entry in &entries {
                            let _id = trie.insert(entry.clone());
                        }
                        eprintln!("FsWalker indexed {}: {} entries", vol_str, entries.len());
                    }
                    Err(e) => {
                        eprintln!("FsWalker failed for {}: {}", volume, e);
                    }
                }
            }
        }
    }

    // Save journal state
    journal_state.save();

    // Save index to disk
    if let Some(index_path) = get_index_path() {
        if let Ok(storage) = Storage::open(&index_path) {
            let entries = trie.all_entries();
            if let Err(e) = storage.save_entries(&entries) {
                eprintln!("Failed to save index: {}", e);
            }
        }
    }

    // Create searcher and update global
    let searcher = Searcher::new(trie);
    if let Ok(mut s) = SEARCHER.lock() {
        *s = Some(searcher);
    }

    Ok(())
}

// ===== Load Index from Disk =====

pub fn load_index_from_disk() -> Result<(), String> {
    if let Some(index_path) = get_index_path() {
        if Path::new(&index_path).exists() {
            let storage = Storage::open(&index_path).map_err(|e| e.to_string())?;
            let entries = storage.load_entries().map_err(|e| e.to_string())?;
            let mut trie = TrieIndex::new();
            trie.load_entries(entries);
            let searcher = Searcher::new(trie);
            if let Ok(mut s) = SEARCHER.lock() {
                *s = Some(searcher);
            }
            eprintln!("Loaded {} entries from disk cache", index_path);
        }
    }
    Ok(())
}
