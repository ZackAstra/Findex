/// Configuration module for Findex.
/// Zero external dependencies - hand-rolled JSON serialization.
/// Saved to %APPDATA%/Findex/config.json

use std::fs;
use std::io;
use std::path::Path;
use std::sync::Mutex;

/// Theme mode for the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Light,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    pub fn effective(&self) -> Theme {
        match self {
            Theme::System => {
                if detect_windows_dark_mode() {
                    Theme::Dark
                } else {
                    Theme::Light
                }
            }
            _ => *self,
        }
    }
}

fn detect_windows_dark_mode() -> bool {
    std::process::Command::new("reg")
        .args(&[
            "query",
            r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "/v",
            "AppsUseLightTheme",
        ])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("0x0") {
                    Some(true)
                } else {
                    Some(false)
                }
            } else {
                None
            }
        })
        .unwrap_or(false)
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
    pub fn variants() -> &'static [(SearchFilter, &'static str)] {
        &[
            (SearchFilter::All, "All"),
            (SearchFilter::Folders, "Folders"),
            (SearchFilter::Documents, "Docs"),
            (SearchFilter::Code, "Code"),
            (SearchFilter::Images, "Images"),
            (SearchFilter::Archives, "Archive"),
            (SearchFilter::Audio, "Audio"),
            (SearchFilter::Video, "Video"),
        ]
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

    for mod_part in &parts[..parts.len() - 1] {
        match *mod_part {
            "Ctrl" | "ctrl" => modifiers |= 0x0002,
            "Shift" | "shift" => modifiers |= 0x0004,
            "Alt" | "alt" => modifiers |= 0x0001,
            "Win" | "win" => modifiers |= 0x0008,
            _ => return None,
        }
    }

    let vk = match *key {
        "Space" => 0x20,
        "Enter" => 0x0D,
        "Esc" => 0x1B,
        "Tab" => 0x09,
        "Backspace" => 0x08,
        "Delete" => 0x2E,
        "Insert" => 0x2D,
        "Home" => 0x24,
        "End" => 0x23,
        "PageUp" => 0x21,
        "PageDown" => 0x22,
        "Up" => 0x26,
        "Down" => 0x28,
        "Left" => 0x25,
        "Right" => 0x27,
        "F1" => 0x70, "F2" => 0x71, "F3" => 0x72, "F4" => 0x73,
        "F5" => 0x74, "F6" => 0x75, "F7" => 0x76, "F8" => 0x77,
        "F9" => 0x78, "F10" => 0x79, "F11" => 0x7A, "F12" => 0x7B,
        k if k.len() == 1 => {
            let c = k.chars().next()?;
            let uc = c.to_ascii_uppercase();
            uc as u32
        }
        _ => return None,
    };

    Some((modifiers, vk))
}

/// Format (modifiers, vk) back to a display string.
pub fn format_hotkey(modifiers: u32, vk: u32) -> String {
    let mut parts: Vec<String> = Vec::new();
    if modifiers & 0x0002 != 0 { parts.push("Ctrl".to_string()); }
    if modifiers & 0x0004 != 0 { parts.push("Shift".to_string()); }
    if modifiers & 0x0001 != 0 { parts.push("Alt".to_string()); }
    if modifiers & 0x0008 != 0 { parts.push("Win".to_string()); }
    let key_str: String = match vk {
        0x20 => "Space".to_string(),
        0x0D => "Enter".to_string(),
        0x1B => "Esc".to_string(),
        0x09 => "Tab".to_string(),
        0x08 => "Backspace".to_string(),
        0x2E => "Delete".to_string(),
        0x2D => "Insert".to_string(),
        0x24 => "Home".to_string(),
        0x23 => "End".to_string(),
        0x21 => "PageUp".to_string(),
        0x22 => "PageDown".to_string(),
        0x26 => "Up".to_string(),
        0x28 => "Down".to_string(),
        0x25 => "Left".to_string(),
        0x27 => "Right".to_string(),
        0x70..=0x7B => {
            let n = vk - 0x70 + 1;
            format!("F{}", n)
        }
        c if c >= 0x41 && c <= 0x5A => {
            let ch = char::from_u32(c).unwrap_or('?');
            format!("{}", ch)
        }
        _ => return format!("{}+{:X}", parts.join("+"), vk),
    };
    parts.push(key_str);
    parts.join("+")
}

#[derive(Debug, Clone)]
pub struct Config {
    pub index_paths: Vec<String>,
    pub enable_pinyin: bool,
    pub enable_fuzzy: bool,
    pub show_hidden: bool,
    pub max_results: usize,
    pub auto_index: bool,
    pub theme: String,
    pub exclude_patterns: Vec<String>,
    pub hotkey_search: String,
    pub hotkey_settings: String,
}

static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

pub fn load_config() {
    let cfg = Config::load();
    *CONFIG.lock().unwrap() = Some(cfg);
}

pub fn get_config() -> Config {
    CONFIG.lock().unwrap().clone().unwrap_or_default()
}

pub fn set_config(cfg: Config) {
    *CONFIG.lock().unwrap() = Some(cfg);
}

pub fn get_effective_theme() -> Theme {
    let cfg = get_config();
    Theme::from_str(&cfg.theme).effective()
}

pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let effective = theme.effective();
    let mut style = (*ctx.style()).clone();
    match effective {
        Theme::Dark => {
            style.visuals = egui::Visuals::dark();
            style.visuals.window_fill = egui::Color32::from_rgb(30, 30, 30);
            style.visuals.panel_fill = egui::Color32::from_rgb(25, 25, 25);
            style.visuals.faint_bg_color = egui::Color32::from_rgb(35, 35, 35);
            style.visuals.extreme_bg_color = egui::Color32::from_rgb(20, 20, 20);
            style.visuals.code_bg_color = egui::Color32::from_rgb(40, 40, 40);
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(50, 100, 180);
            style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(80, 140, 220));
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(35, 35, 35);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(40, 40, 40);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(55, 55, 55);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 60, 60);
            style.visuals.window_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(60, 60, 60));
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(220, 220, 220));
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        }
        Theme::Light => {
            style.visuals = egui::Visuals::light();
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(30, 30, 30));
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        }
        _ => {}
    }
    ctx.set_style(style);
}

impl Default for Config {
    fn default() -> Self {
        Config {
            index_paths: vec![],
            enable_pinyin: true,
            enable_fuzzy: false,
            show_hidden: false,
            max_results: 100,
            auto_index: true,
            theme: "system".to_string(),
            exclude_patterns: vec![".git".to_string(), "node_modules".to_string(), "target".to_string()],
            hotkey_search: "Ctrl+Space".to_string(),
            hotkey_settings: "Ctrl+Shift+F".to_string(),
        }
    }
}

impl Config {
    pub fn path() -> String {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        if appdata.is_empty() {
            "config.json".to_string()
        } else {
            format!("{}\\Findex\\config.json", appdata)
        }
    }

    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(config) = Self::from_json(&content) {
                return config;
            }
        }
        Config::default()
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        if let Some(parent) = Path::new(&path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, self.to_json_pretty())
    }

    pub fn to_json(&self) -> String {
        let paths = self.index_paths.iter().map(|p| format!("\"{}\"", json_escape(p))).collect::<Vec<_>>().join(",");
        let excludes = self.exclude_patterns.iter().map(|p| format!("\"{}\"", json_escape(p))).collect::<Vec<_>>().join(",");
        format!(
            r#"{{"index_paths":[{}],"enable_pinyin":{},"enable_fuzzy":{},"show_hidden":{},"max_results":{},"auto_index":{},"theme":"{}","exclude_patterns":[{}],"hotkey_search":"{}","hotkey_settings":"{}"}}"#,
            paths, bool_to_int(self.enable_pinyin), bool_to_int(self.enable_fuzzy),
            bool_to_int(self.show_hidden), self.max_results, bool_to_int(self.auto_index),
            json_escape(&self.theme), excludes, json_escape(&self.hotkey_search), json_escape(&self.hotkey_settings),
        )
    }

    pub fn to_json_pretty(&self) -> String {
        let paths = self.index_paths.iter().map(|p| format!("    \"{}\"", json_escape(p))).collect::<Vec<_>>().join(",\n");
        let excludes = self.exclude_patterns.iter().map(|p| format!("    \"{}\"", json_escape(p))).collect::<Vec<_>>().join(",\n");
        format!(
            "{{\n  \"index_paths\": [\n{}\n  ],\n  \"enable_pinyin\": {},\n  \"enable_fuzzy\": {},\n  \"show_hidden\": {},\n  \"max_results\": {},\n  \"auto_index\": {},\n  \"theme\": \"{}\",\n  \"exclude_patterns\": [\n{}\n  ],\n  \"hotkey_search\": \"{}\",\n  \"hotkey_settings\": \"{}\"\n}}\n",
            paths, bool_to_int(self.enable_pinyin), bool_to_int(self.enable_fuzzy),
            bool_to_int(self.show_hidden), self.max_results, bool_to_int(self.auto_index),
            json_escape(&self.theme), excludes, json_escape(&self.hotkey_search), json_escape(&self.hotkey_settings),
        )
    }

    pub fn from_json(s: &str) -> Option<Self> {
        let s = s.trim();
        if !s.starts_with('{') || !s.ends_with('}') { return None; }

        fn extract_array(s: &str, key: &str) -> Vec<String> {
            let mut result = Vec::new();
            let pattern = format!("\"{}\"", key);
            if let Some(idx) = s.find(&pattern) {
                let after = &s[idx + pattern.len()..];
                if let Some(colon) = after.find(':') {
                    let after_colon = &after[colon..];
                    if let Some(bracket) = after_colon.find('[') {
                        let arr_start = bracket + 1;
                        if let Some(arr_end) = after_colon[arr_start..].find(']') {
                            let content = &after_colon[arr_start..arr_start + arr_end];
                            for item in content.split(',') {
                                let item = item.trim().trim_matches('"');
                                if !item.is_empty() {
                                    result.push(item.to_string());
                                }
                            }
                        }
                    }
                }
            }
            result
        }

        fn extract_value(s: &str, key: &str) -> Option<String> {
            let pattern = format!("\"{}\"", key);
            let idx = s.find(&pattern)?;
            let after = &s[idx + pattern.len()..];
            let colon = after.find(':')?;
            let val_start = colon + 1;
            let val = after[val_start..].trim();
            if val.starts_with('"') {
                let end = val[1..].find('"')?;
                Some(val[1..=end].to_string())
            } else {
                let end = val.find(|c: char| c == ',' || c == '}' || c == '\n' || c == '\r').unwrap_or(val.len());
                Some(val[..end].trim().to_string())
            }
        }

        fn parse_bool(s: &str, key: &str) -> bool {
            extract_value(s, key).map(|v| v == "1" || v == "true").unwrap_or(false)
        }

        fn parse_usize(s: &str, key: &str) -> usize {
            extract_value(s, key).and_then(|v| v.parse().ok()).unwrap_or(100)
        }

        Some(Config {
            index_paths: extract_array(s, "index_paths"),
            enable_pinyin: parse_bool(s, "enable_pinyin"),
            enable_fuzzy: parse_bool(s, "enable_fuzzy"),
            show_hidden: parse_bool(s, "show_hidden"),
            max_results: parse_usize(s, "max_results"),
            auto_index: parse_bool(s, "auto_index"),
            theme: extract_value(s, "theme").unwrap_or_else(|| "system".to_string()),
            exclude_patterns: extract_array(s, "exclude_patterns"),
            hotkey_search: extract_value(s, "hotkey_search").unwrap_or_else(|| "Ctrl+Space".to_string()),
            hotkey_settings: extract_value(s, "hotkey_settings").unwrap_or_else(|| "Ctrl+Shift+F".to_string()),
        })
    }
}

fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn bool_to_int(b: bool) -> i32 {
    if b { 1 } else { 0 }
}

// ===== USN Journal State Tracking =====

/// USN Journal state for a single volume.
#[derive(Debug, Clone)]
pub struct VolumeJournalState {
    pub volume_letter: String,
    pub last_usn: i64,
    pub usn_journal_id: u64,
}

/// USN Journal state for all volumes (persisted to usn_state.json).
#[derive(Debug, Clone)]
pub struct UsnJournalState {
    pub volumes: Vec<VolumeJournalState>,
}

impl UsnJournalState {
    pub fn new() -> Self {
        UsnJournalState { volumes: Vec::new() }
    }

    /// Get the USN journal state path in AppData.
    fn get_state_path() -> Option<String> {
        let appdata = std::env::var("APPDATA").ok()?;
        let dir = format!("{}\\Findex", appdata);
        let _ = std::fs::create_dir_all(&dir);
        Some(format!("{}\\usn_state.json", dir))
    }

    /// Load USN journal state from disk.
    pub fn load() -> Self {
        if let Some(path) = Self::get_state_path() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let content = content.trim();
                if content.starts_with('{') {
                    return Self::from_json(content);
                }
            }
        }
        Self::new()
    }

    /// Save USN journal state to disk.
    pub fn save(&self) {
        if let Some(path) = Self::get_state_path() {
            let json = self.to_json();
            let _ = std::fs::write(&path, json);
        }
    }

    /// Update a volume's journal state.
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

    /// Get the state for a specific volume.
    pub fn get_volume(&self, volume_letter: &str) -> Option<&VolumeJournalState> {
        self.volumes.iter().find(|v| v.volume_letter == volume_letter)
    }

    /// Check if a volume's journal ID has changed (journal was reset).
    pub fn is_journal_valid(&self, volume_letter: &str, current_journal_id: u64) -> bool {
        self.volumes.iter()
            .find(|v| v.volume_letter == volume_letter)
            .map(|v| v.usn_journal_id == current_journal_id)
            .unwrap_or(false)
    }

    fn to_json(&self) -> String {
        let vols: Vec<String> = self.volumes.iter().map(|v| {
            format!(
                r#"{{"volume_letter":"{}","last_usn":{},"usn_journal_id":{}}}"#,
                json_escape(&v.volume_letter), v.last_usn, v.usn_journal_id
            )
        }).collect();
        let vols_str = vols.join(",\n    ");
        format!("{{\n  \"volumes\": [\n    {}\n  ]\n}}\n", vols_str)
    }

    fn from_json(s: &str) -> Self {
        let mut volumes = Vec::new();
        if let Some(vol_start) = s.find('[') {
            if let Some(vol_end) = s[vol_start..].find(']') {
                let content = &s[vol_start + 1..vol_start + vol_end];
                for item in content.split('{').skip(1) {
                    let item = format!("{{{}}}", item.trim_end_matches(',').trim_end_matches('}').trim());
                    let letter = extract_json_value(&item, "volume_letter").unwrap_or_default();
                    let last_usn = extract_json_value(&item, "last_usn")
                        .and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
                    let journal_id = extract_json_value(&item, "usn_journal_id")
                        .and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                    if !letter.is_empty() {
                        volumes.push(VolumeJournalState {
                            volume_letter: letter,
                            last_usn,
                            usn_journal_id: journal_id,
                        });
                    }
                }
            }
        }
        UsnJournalState { volumes }
    }
}

fn extract_json_value(s: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx = s.find(&pattern)?;
    let after = &s[idx + pattern.len()..];
    let colon = after.find(':')?;
    let val_start = colon + 1;
    let val = after[val_start..].trim();
    if val.starts_with('"') {
        let end = val[1..].find('"')?;
        Some(val[1..=end].to_string())
    } else {
        let end = val.find(|c: char| c == ',' || c == '}' || c == '\n' || c == '\r').unwrap_or(val.len());
        Some(val[..end].trim().to_string())
    }
}
