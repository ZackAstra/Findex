/// Search router: coordinates multi-source search and result ranking.
use crate::index_engine::TrieIndex;
use crate::pinyin;
use crate::types::*;

/// Search engine that coordinates different search strategies.
pub struct Searcher {
    index: TrieIndex,
}

impl Searcher {
    pub fn new(index: TrieIndex) -> Self {
        Searcher { index }
    }

    /// Get reference to the underlying index.
    pub fn index(&self) -> &TrieIndex {
        &self.index
    }

    /// Get mutable reference to the underlying index.
    pub fn index_mut(&mut self) -> &mut TrieIndex {
        &mut self.index
    }

    /// Execute a search query.
    pub fn search(&self, query: &SearchQuery) -> Vec<SearchResult> {
        let q = query.query.trim();
        if q.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        // 1. Prefix match (highest priority)
        for entry in self.index.search_prefix(q, query.max_results) {
            let score = if is_in_context(&entry, &query.context_path) {
                130
            } else {
                100
            };
            results.push(SearchResult {
                entry,
                score,
                match_type: MatchType::ExactPrefix,
            });
        }

        // 2. Path segment match
        if q.contains(&['/', '\\'][..]) || q.contains('.') {
            for entry in self.index.search_path_segment(q, query.max_results) {
                if !results.iter().any(|r| r.entry.id == entry.id) {
                    let score = if is_in_context(&entry, &query.context_path) {
                        80
                    } else {
                        60
                    };
                    results.push(SearchResult {
                        entry,
                        score,
                        match_type: MatchType::PathSegment,
                    });
                }
            }
        }

        // 3. Substring match
        if results.len() < query.max_results {
            let remaining = query.max_results - results.len();
            for entry in self.index.search_substring(q, remaining) {
                if !results.iter().any(|r| r.entry.id == entry.id) {
                    let score = if is_in_context(&entry, &query.context_path) {
                        55
                    } else {
                        40
                    };
                    results.push(SearchResult {
                        entry,
                        score,
                        match_type: MatchType::Substring,
                    });
                }
            }
        }

        // 4. Pinyin match
        if pinyin::has_chinese(q) || q.len() <= 3 {
            let remaining = query.max_results.saturating_sub(results.len());
            let mut pinyin_results = Vec::new();
            for entry in self.index.all_entries() {
                if pinyin::match_pinyin(&entry.name, q) {
                    if !results.iter().any(|r| r.entry.id == entry.id) {
                        let score = if is_in_context(&entry, &query.context_path) {
                            35
                        } else {
                            25
                        };
                        pinyin_results.push(SearchResult {
                            entry,
                            score,
                            match_type: MatchType::Pinyin,
                        });
                    }
                }
                if pinyin_results.len() >= remaining {
                    break;
                }
            }
            results.extend(pinyin_results);
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        // Apply offset and limit
        let start = query.offset.min(results.len());
        let end = (start + query.max_results).min(results.len());
        results[start..end].to_vec()
    }

    /// Get index status.
    pub fn status(&self) -> IndexStatus {
        let entries = self.index.all_entries();
        let total_files = entries.iter().filter(|e| !e.is_dir).count() as u64;
        let total_folders = entries.iter().filter(|e| e.is_dir).count() as u64;

        let volumes: Vec<String> = {
            let mut vols: Vec<String> = entries
                .iter()
                .map(|e| e.volume.clone())
                .filter(|v| !v.is_empty())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            vols.sort();
            vols
        };

        IndexStatus {
            total_files,
            total_folders,
            indexed_volumes: volumes,
            memory_usage_bytes: 0, // Would need memory tracking
            last_index_time: 0,
            is_indexing: false,
        }
    }
}

/// Check if a file entry is within the user's current context folder.
fn is_in_context(entry: &FileEntry, context_path: &Option<String>) -> bool {
    match context_path {
        Some(ctx) => {
            let ctx_lower = ctx.to_lowercase();
            let entry_path_lower = entry.path.to_lowercase();
            entry_path_lower.starts_with(&ctx_lower)
        }
        None => false,
    }
}
