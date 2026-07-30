/// Tauri-compatible config module using serde for IPC.
/// Manages config.json and usn_state.json in %APPDATA%/Findex/

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub search_hotkey: String,
    pub settings_hotkey: String,
    pub index_dirs: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_results: usize,
    pub enable_pinyin: bool,
    pub enable_fuzzy: bool,
    pub show_hidden: bool,
    pub auto_index: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            search_hotkey: "Ctrl+Space".to_string(),
            settings_hotkey: "Ctrl+Shift+F".to_string(),
            index_dirs: vec!["C:\\".to_string()],
            exclude_patterns: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "__pycache__".to_string(),
                ".cache".to_string(),
                "venv".to_string(),
                ".venv".to_string(),
            ],
            max_results: 50,
            enable_pinyin: true,
            enable_fuzzy: true,
            show_hidden: false,
            auto_index: true,
        }
    }
}

/// Search filter for file types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchFilter {
    All,
    Folders,
    Documents,
    Code,
    Images,
    Archives,
    Audio,
    Video,
}

impl SearchFilter {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "folders" => SearchFilter::Folders,
            "documents" | "docs" => SearchFilter::Documents,
            "code" => SearchFilter::Code,
            "images" => SearchFilter::Images,
            "archives" | "archive" => SearchFilter::Archives,
            "audio" => SearchFilter::Audio,
            "video" => SearchFilter::Video,
            _ => SearchFilter::All,
        }
    }

    pub fn matches(&self, entry: &findex_engine::FileEntry) -> bool {
        match self {
            SearchFilter::All => true,
            SearchFilter::Folders => entry.is_dir,
            SearchFilter::Documents => {
                if entry.is_dir { return false; }
                matches!(entry.extension.to_lowercase().as_str(),
                    ".txt" | ".md" | ".log" | ".json" | ".xml" | ".yaml" | ".yml"
                    | ".toml" | ".ini" | ".cfg" | ".pdf" | ".doc" | ".docx"
                    | ".xls" | ".xlsx" | ".csv" | ".ppt" | ".pptx" | ".rtf")
            }
            SearchFilter::Code => {
                if entry.is_dir { return false; }
                matches!(entry.extension.to_lowercase().as_str(),
                    ".rs" | ".py" | ".js" | ".ts" | ".go" | ".java" | ".c" | ".cpp"
                    | ".h" | ".hpp" | ".cs" | ".rb" | ".php" | ".swift" | ".kt"
                    | ".scala" | ".html" | ".css" | ".scss" | ".less" | ".vue"
                    | ".jsx" | ".tsx" | ".sql" | ".sh" | ".bat" | ".ps1")
            }
            SearchFilter::Images => {
                if entry.is_dir { return false; }
                matches!(entry.extension.to_lowercase().as_str(),
                    ".jpg" | ".jpeg" | ".png" | ".gif" | ".bmp" | ".svg"
                    | ".webp" | ".ico" | ".tiff" | ".tif" | ".raw")
            }
            SearchFilter::Archives => {
                if entry.is_dir { return false; }
                matches!(entry.extension.to_lowercase().as_str(),
                    ".zip" | ".rar" | ".7z" | ".tar" | ".gz" | ".bz2" | ".xz"
                    | ".zst" | ".iso" | ".dmg")
            }
            SearchFilter::Audio => {
                if entry.is_dir { return false; }
                matches!(entry.extension.to_lowercase().as_str(),
                    ".mp3" | ".wav" | ".flac" | ".ogg" | ".aac" | ".wma" | ".m4a")
            }
            SearchFilter::Video => {
                if entry.is_dir { return false; }
                matches!(entry.extension.to_lowercase().as_str(),
                    ".mp4" | ".avi" | ".mkv" | ".mov" | ".wmv" | ".flv" | ".webm")
            }
        }
    }
}

/// Parse a hotkey string like "Ctrl+Space" into (modifiers, vk).
pub fn parse_hotkey(s: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = s.split('+').collect();
    if parts.is_empty() { return None; }

    let key = parts.last()?;
    let mut modifiers = 0u32;

    for part in &parts[..parts.len() - 1] {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= 0x0002, // MOD_CONTROL
            "alt" => modifiers |= 0x0001,               // MOD_ALT
            "shift" => modifiers |= 0x0004,              // MOD_SHIFT
            "win" | "super" | "windows" => modifiers |= 0x0008, // MOD_WIN
            _ => {}
        }
    }

    let vk = match key.to_lowercase().as_str() {
        "space" => 0x20,
        "enter" | "return" => 0x0D,
        "tab" => 0x09,
        "esc" | "escape" => 0x1B,
        "backspace" | "back" => 0x08,
        "delete" | "del" => 0x2E,
        "insert" | "ins" => 0x2D,
        "home" => 0x24,
        "end" => 0x23,
        "pageup" => 0x21,
        "pagedown" => 0x22,
        "up" => 0x26,
        "down" => 0x28,
        "left" => 0x25,
        "right" => 0x27,
        "f1" => 0x70,
        "f2" => 0x71,
        "f3" => 0x72,
        "f4" => 0x73,
        "f5" => 0x74,
        "f6" => 0x75,
        "f7" => 0x76,
        "f8" => 0x77,
        "f9" => 0x78,
        "f10" => 0x79,
        "f11" => 0x7A,
        "f12" => 0x7B,
        k if k.len() == 1 => {
            let c = k.chars().next()?;
            c.to_ascii_uppercase() as u32
        }
        _ => return None,
    };

    Some((modifiers, vk))
}

/// Format hotkey back to string.
pub fn format_hotkey(modifiers: u32, vk: u32) -> String {
    let mut parts: Vec<String> = Vec::new();
    if modifiers & 0x0002 != 0 { parts.push("Ctrl".to_string()); }
    if modifiers & 0x0001 != 0 { parts.push("Alt".to_string()); }
    if modifiers & 0x0004 != 0 { parts.push("Shift".to_string()); }
    if modifiers & 0x0008 != 0 { parts.push("Win".to_string()); }

    let key = match vk {
        0x20 => "Space".to_string(),
        0x0D => "Enter".to_string(),
        0x09 => "Tab".to_string(),
        0x1B => "Esc".to_string(),
        0x08 => "Backspace".to_string(),
        0x2E => "Delete".to_string(),
        0x2D => "Insert".to_string(),
        0x24 => "Home".to_string(),
        0x23 => "End".to_string(),
        0x21 => "PageUp".to_string(),
        0x22 => "PageDown".to_string(),
        0x70..=0x7B => {
            let n = vk - 0x70 + 1;
            parts.push(format!("F{}", n));
            return parts.join("+");
        }
        _ => {
            if let Some(c) = char::from_u32(vk) {
                c.to_string()
            } else {
                "?".to_string()
            }
        }
    };
    parts.push(key);
    parts.join("+")
}

// ===== Config Persistence =====

fn get_config_path() -> Option<String> {
    let appdata = std::env::var("APPDATA").ok()?;
    let dir = format!("{}\\Findex", appdata);
    let _ = fs::create_dir_all(&dir);
    Some(format!("{}\\config.json", dir))
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if let Some(p) = path {
        if let Ok(content) = fs::read_to_string(&p) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                if let Ok(mut guard) = crate::CONFIG.lock() {
                    *guard = Some(cfg.clone());
                }
                return cfg;
            }
        }
    }
    let cfg = AppConfig::default();
    save_config(&cfg);
    if let Ok(mut guard) = crate::CONFIG.lock() {
        *guard = Some(cfg.clone());
    }
    cfg
}

pub fn save_config(config: &AppConfig) {
    if let Some(path) = get_config_path() {
        if let Ok(json) = serde_json::to_string_pretty(config) {
            let _ = fs::write(&path, json);
        }
    }
}

// ===== USN Journal State =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeJournalState {
    pub volume_letter: String,
    pub last_usn: i64,
    pub usn_journal_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsnJournalState {
    pub volumes: Vec<VolumeJournalState>,
}

impl UsnJournalState {
    pub fn new() -> Self {
        UsnJournalState { volumes: Vec::new() }
    }

    fn get_state_path() -> Option<String> {
        let appdata = std::env::var("APPDATA").ok()?;
        let dir = format!("{}\\Findex", appdata);
        let _ = fs::create_dir_all(&dir);
        Some(format!("{}\\usn_state.json", dir))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::get_state_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<UsnJournalState>(&content) {
                    return state;
                }
            }
        }
        Self::new()
    }

    pub fn save(&self) {
        if let Some(path) = Self::get_state_path() {
            if let Ok(json) = serde_json::to_string_pretty(self) {
                let _ = fs::write(&path, json);
            }
        }
    }

    pub fn update_volume(&mut self, volume_letter: &str, last_usn: i64, usn_journal_id: u64) {
        for v in &mut self.volumes {
            if v.volume_letter == volume_letter {
                v.last_usn = last_usn;
                v.usn_journal_id = usn_journal_id;
                return;
            }
        }
        self.volumes.push(VolumeJournalState {
            volume_letter: volume_letter.to_string(),
            last_usn,
            usn_journal_id,
        });
    }

    pub fn get_volume(&self, volume_letter: &str) -> Option<&VolumeJournalState> {
        self.volumes.iter().find(|v| v.volume_letter == volume_letter)
    }

    pub fn is_journal_valid(&self, volume_letter: &str, current_journal_id: u64) -> bool {
        self.volumes.iter()
            .find(|v| v.volume_letter == volume_letter)
            .map(|v| v.usn_journal_id == current_journal_id)
            .unwrap_or(false)
    }
}
