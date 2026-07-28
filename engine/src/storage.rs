use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::types::FileEntry;

pub struct Storage {
    path: String,
}

impl Storage {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_string_lossy().to_string();
        let storage = Storage { path };
        if !Path::new(&storage.path).exists() {
            fs::write(&storage.path, "[]")?;
        }
        Ok(storage)
    }

    pub fn save_entries(&self, entries: &[FileEntry]) -> io::Result<()> {
        let mut file = fs::File::create(&self.path)?;
        writeln!(file, "[")?;
        for (i, entry) in entries.iter().enumerate() {
            let comma = if i < entries.len() - 1 { "," } else { "" };
            writeln!(file, "  {}{}", entry.to_json(), comma)?;
        }
        writeln!(file, "]")?;
        Ok(())
    }

    pub fn load_entries(&self) -> io::Result<Vec<FileEntry>> {
        let content = fs::read_to_string(&self.path)?;
        let content = content.trim();
        if content == "[]" || content.is_empty() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let mut depth = 0;
        let mut start = 0;

        for (i, ch) in content.char_indices() {
            match ch {
                '[' => depth += 1,
                ']' => depth -= 1,
                '{' if depth == 1 => start = i,
                '}' if depth == 1 => {
                    let json = &content[start..=i];
                    if let Some(entry) = FileEntry::from_json(json) {
                        entries.push(entry);
                    }
                }
                _ => {}
            }
        }

        Ok(entries)
    }

    pub fn entry_count(&self) -> io::Result<u64> {
        let entries = self.load_entries()?;
        Ok(entries.len() as u64)
    }
}
