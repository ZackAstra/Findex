/// Filesystem walker for initial indexing.
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::FileEntry;

pub struct FsWalker;

impl FsWalker {
    pub fn walk(root: &Path, max_depth: usize) -> std::io::Result<Vec<FileEntry>> {
        Self::walk_with_excludes(root, max_depth, &[])
    }

    pub fn walk_with_excludes(root: &Path, max_depth: usize, exclude_patterns: &[String]) -> std::io::Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        let root_str = root.to_string_lossy().to_string();
        let volume = root_str
            .split(':')
            .next()
            .unwrap_or("C")
            .to_string();

        Self::walk_recursive(root, &root_str, max_depth, 0, &volume, exclude_patterns, &mut entries)?;

        for (i, entry) in entries.iter_mut().enumerate() {
            entry.id = (i + 1) as i64;
        }

        Ok(entries)
    }

    fn walk_recursive(
        dir: &Path,
        root_str: &str,
        max_depth: usize,
        current_depth: usize,
        volume: &str,
        exclude_patterns: &[String],
        entries: &mut Vec<FileEntry>,
    ) -> std::io::Result<()> {
        if max_depth > 0 && current_depth > max_depth {
            return Ok(());
        }

        let dir_str = dir.to_string_lossy().to_string();
        let lower = dir_str.to_lowercase();

        // Built-in system excludes
        if lower.contains("\\windows\\system32")
            || lower.contains("\\windows\\system")
            || lower.contains("\\windows\\winsxs")
            || lower.contains("\\.bin")
            || lower.contains("\\system volume information")
        {
            return Ok(());
        }

        // User-defined exclude patterns
        for pattern in exclude_patterns {
            if !pattern.is_empty() && lower.contains(&pattern.to_lowercase()) {
                return Ok(());
            }
        }

        let read_dir = match std::fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return Ok(()),
        };

        for entry in read_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();
            let path_str = path.to_string_lossy().to_string();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let file_type = entry.file_type().ok();
            let is_dir = file_type.map(|t| t.is_dir()).unwrap_or(false);

            let attrs = metadata.file_attributes();
            if attrs & 0x400 != 0 {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();

            if name.starts_with('.') {
                continue;
            }

            // Check user exclude patterns against the current path
            let mut excluded = false;
            for pattern in exclude_patterns {
                if !pattern.is_empty() && name.to_lowercase().contains(&pattern.to_lowercase()) {
                    excluded = true;
                    break;
                }
            }
            if excluded {
                continue;
            }

            let is_hidden = attrs & 0x2 != 0;
            let is_readonly = attrs & 0x1 != 0;

            let extension = if is_dir {
                String::new()
            } else {
                Path::new(&name)
                    .extension()
                    .map(|e| format!(".{}", e.to_string_lossy()))
                    .unwrap_or_default()
            };

            let size = metadata.len();
            let created = filetime_to_unix(metadata.created().ok());
            let modified = filetime_to_unix(metadata.modified().ok());
            let accessed = filetime_to_unix(metadata.accessed().ok());

            entries.push(FileEntry {
                id: 0,
                name,
                path: path_str,
                parent_path: dir_str.clone(),
                size,
                created,
                modified,
                accessed,
                is_dir,
                is_hidden,
                is_readonly,
                extension,
                volume: volume.to_string(),
            });

            if is_dir {
                Self::walk_recursive(
                    &path, root_str, max_depth, current_depth + 1,
                    volume, exclude_patterns, entries,
                )?;
            }
        }

        Ok(())
    }
}

fn filetime_to_unix(ft: Option<SystemTime>) -> i64 {
    ft.and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}