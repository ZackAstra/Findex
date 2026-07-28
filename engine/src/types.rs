use std::fmt;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub id: i64,
    pub name: String,
    pub path: String,
    pub parent_path: String,
    pub size: u64,
    pub created: i64,
    pub modified: i64,
    pub accessed: i64,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub is_readonly: bool,
    pub extension: String,
    pub volume: String,
}

impl FileEntry {
    pub fn to_json(&self) -> String {
        format!(
            r#"{{"id":{},"name":"{}","path":"{}","parent_path":"{}","size":{},"created":{},"modified":{},"accessed":{},"is_dir":{},"is_hidden":{},"is_readonly":{},"extension":"{}","volume":"{}"}}"#,
            self.id,
            json_escape(&self.name),
            json_escape(&self.path),
            json_escape(&self.parent_path),
            self.size,
            self.created,
            self.modified,
            self.accessed,
            self.is_dir as i32,
            self.is_hidden as i32,
            self.is_readonly as i32,
            json_escape(&self.extension),
            json_escape(&self.volume),
        )
    }

    pub fn from_json(s: &str) -> Option<Self> {
        let v: Vec<&str> = s.split(',').collect();
        if v.len() < 13 { return None; }

        fn get_val<'a>(parts: &[&'a str], idx: usize) -> Option<&'a str> {
            let s = parts.get(idx)?;
            let colon = s.find(':')?;
            let val = &s[colon+1..];
            let val = val.trim().trim_matches('"').trim_matches('}');
            if val.is_empty() && !s.contains('"') { return None; }
            Some(val)
        }
        fn get_i64(parts: &[&str], idx: usize) -> Option<i64> {
            get_val(parts, idx)?.parse().ok()
        }
        fn get_u64(parts: &[&str], idx: usize) -> Option<u64> {
            get_val(parts, idx)?.parse().ok()
        }
        fn get_str(parts: &[&str], idx: usize) -> Option<String> {
            Some(get_val(parts, idx)?.to_string())
        }

        Some(FileEntry {
            id: get_i64(&v, 0)?,
            name: get_str(&v, 1)?,
            path: get_str(&v, 2)?,
            parent_path: get_str(&v, 3)?,
            size: get_u64(&v, 4)?,
            created: get_i64(&v, 5)?,
            modified: get_i64(&v, 6)?,
            accessed: get_i64(&v, 7)?,
            is_dir: get_val(&v, 8)? == "1",
            is_hidden: get_val(&v, 9)? == "1",
            is_readonly: get_val(&v, 10)? == "1",
            extension: get_str(&v, 11)?,
            volume: get_str(&v, 12)?,
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

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: FileEntry,
    pub score: i32,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchType {
    ExactPrefix,
    Substring,
    Pinyin,
    Fuzzy,
    PathSegment,
}

impl fmt::Display for MatchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatchType::ExactPrefix => write!(f, "PREFIX"),
            MatchType::Substring => write!(f, "SUBSTR"),
            MatchType::Pinyin => write!(f, "PINYIN"),
            MatchType::Fuzzy => write!(f, "FUZZY"),
            MatchType::PathSegment => write!(f, "PATH"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub scope: SearchScope,
    pub context_path: Option<String>,
    pub max_results: usize,
    pub offset: usize,
    pub sort_by: SortBy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchScope {
    Global,
    CurrentFolder,
    Path(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortBy {
    Relevance,
    Name,
    Date,
    Size,
}

#[derive(Debug, Clone)]
pub struct IndexStatus {
    pub total_files: u64,
    pub total_folders: u64,
    pub indexed_volumes: Vec<String>,
    pub memory_usage_bytes: u64,
    pub last_index_time: i64,
    pub is_indexing: bool,
}
