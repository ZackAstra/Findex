/// Configuration module for Findex.
/// Zero external dependencies - hand-rolled JSON serialization.
/// Saved to %APPDATA%/Findex/config.json

use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    /// Directories to index
    pub index_paths: Vec<String>,
    /// Enable pinyin search
    pub enable_pinyin: bool,
    /// Enable fuzzy matching
    pub enable_fuzzy: bool,
    /// Show hidden files in results
    pub show_hidden: bool,
    /// Max results to display
    pub max_results: usize,
    /// Auto-index on startup
    pub auto_index: bool,
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
        }
    }
}

impl Config {
    /// Get the config file path.
    pub fn path() -> String {
        let appdata = std::env::var("APPDATA").unwrap_or_default();
        if appdata.is_empty() {
            "config.json".to_string()
        } else {
            format!("{}\\Findex\\config.json", appdata)
        }
    }

    /// Load config from the default path. Returns default if not found or invalid.
    pub fn load() -> Self {
        let path = Self::path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Some(config) = Self::from_json(&content) {
                return config;
            }
        }
        Config::default()
    }

    /// Save config to the default path.
    pub fn save(&self) -> io::Result<()> {
        let path = Self::path();
        if let Some(parent) = Path::new(&path).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, self.to_json_pretty())
    }

    /// Serialize to compact JSON.
    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"index_paths":[{}],"enable_pinyin":{},"enable_fuzzy":{},"show_hidden":{},"max_results":{},"auto_index":{}}}"#,
            self.index_paths.iter().map(|p| format!("\"{}\"", json_escape(p))).collect::<Vec<_>>().join(","),
            bool_to_int(self.enable_pinyin),
            bool_to_int(self.enable_fuzzy),
            bool_to_int(self.show_hidden),
            self.max_results,
            bool_to_int(self.auto_index),
        )
    }

    /// Serialize to pretty-printed JSON.
    pub fn to_json_pretty(&self) -> String {
        let paths: Vec<String> = self.index_paths.iter()
            .map(|p| format!("    \"{}\"", json_escape(p)))
            .collect();
        format!(
            "{{\n  \"index_paths\": [\n{}\n  ],\n  \"enable_pinyin\": {},\n  \"enable_fuzzy\": {},\n  \"show_hidden\": {},\n  \"max_results\": {},\n  \"auto_index\": {}\n}}\n",
            paths.join(",\n"),
            bool_to_int(self.enable_pinyin),
            bool_to_int(self.enable_fuzzy),
            bool_to_int(self.show_hidden),
            self.max_results,
            bool_to_int(self.auto_index),
        )
    }

    /// Parse from JSON string.
    pub fn from_json(s: &str) -> Option<Self> {
        let s = s.trim();
        if !s.starts_with('{') || !s.ends_with('}') {
            return None;
        }

        // Helper: extract array values for a key
        fn extract_array(s: &str, key: &str) -> Vec<String> {
            let mut result = Vec::new();
            let pattern = format!("\"{}\"", key);
            let idx = match s.find(&pattern) { Some(i) => i, None => return result };
            let after = &s[idx + pattern.len()..];
            let colon = match after.find(':') { Some(i) => i, None => return result };
            let bracket = match after[colon..].find('[') { Some(i) => i, None => return result };
            let arr_start = colon + bracket + 1;
            let arr_end = match after[arr_start..].find(']') { Some(i) => i, None => return result };
            let content = &after[arr_start..arr_start + arr_end];
            for item in content.split(',') {
                let item = item.trim().trim_matches('"');
                if !item.is_empty() {
                    result.push(item.to_string());
                }
            }
            result
        }

        // Helper: extract a value by key
        fn extract_value(s: &str, key: &str) -> Option<String> {
            let pattern = format!("\"{}\"", key);
            let idx = s.find(&pattern)?;
            let after = &s[idx + pattern.len()..];
            // Find ':'
            let colon = after.find(':')?;
            let val_start = colon + 1;
            let val = after[val_start..].trim();
            // Check if it's a string or number/bool
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
