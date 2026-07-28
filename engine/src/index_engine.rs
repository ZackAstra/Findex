use std::collections::HashMap;
use crate::types::FileEntry;

#[derive(Debug)]
pub struct TrieNode {
    pub children: HashMap<char, Box<TrieNode>>,
    pub file_ids: Vec<i64>,
    pub is_end: bool,
}

impl TrieNode {
    pub fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            file_ids: Vec::new(),
            is_end: false,
        }
    }
}

#[derive(Debug)]
pub struct TrieIndex {
    root: TrieNode,
    entries: HashMap<i64, FileEntry>,
    next_id: i64,
}

impl TrieIndex {
    pub fn new() -> Self {
        TrieIndex {
            root: TrieNode::new(),
            entries: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, mut entry: FileEntry) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        entry.id = id;

        let lower_name = entry.name.to_lowercase();
        self._insert_str(&lower_name, id);

        if let Some(dot) = lower_name.rfind('.') {
            let stem = &lower_name[..dot];
            if !stem.is_empty() {
                self._insert_str(stem, id);
            }
        }

        self.entries.insert(id, entry);
        id
    }

    fn _insert_str(&mut self, s: &str, id: i64) {
        let mut node = &mut self.root;
        for ch in s.chars() {
            node = node.children.entry(ch).or_insert_with(|| Box::new(TrieNode::new()));
        }
        node.is_end = true;
        node.file_ids.push(id);
    }

    pub fn remove(&mut self, id: i64) {
        if let Some(entry) = self.entries.remove(&id) {
            let lower_name = entry.name.to_lowercase();
            remove_from_trie(&mut self.root, &lower_name, id, 0);
        }
    }

    pub fn search_prefix(&self, query: &str, max_results: usize) -> Vec<FileEntry> {
        let lower_query = query.to_lowercase();
        let mut node = &self.root;
        for ch in lower_query.chars() {
            match node.children.get(&ch) {
                Some(child) => node = child,
                None => return Vec::new(),
            }
        }
        let mut ids = Vec::new();
        collect_ids(node, &mut ids);
        ids.sort_unstable();
        ids.dedup();
        ids.truncate(max_results);
        ids.into_iter().filter_map(|id| self.entries.get(&id).cloned()).collect()
    }

    pub fn search_substring(&self, query: &str, max_results: usize) -> Vec<FileEntry> {
        let lower_query = query.to_lowercase();
        let mut results: Vec<(i64, FileEntry)> = self
            .entries
            .values()
            .filter(|e| e.name.to_lowercase().contains(&lower_query))
            .map(|e| (e.id, e.clone()))
            .collect();
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results.truncate(max_results);
        results.into_iter().map(|(_, e)| e).collect()
    }

    pub fn search_path_segment(&self, query: &str, max_results: usize) -> Vec<FileEntry> {
        let lower_query = query.to_lowercase();
        let mut results: Vec<(i64, FileEntry)> = self
            .entries
            .values()
            .filter(|e| e.path.to_lowercase().contains(&lower_query))
            .map(|e| (e.id, e.clone()))
            .collect();
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results.truncate(max_results);
        results.into_iter().map(|(_, e)| e).collect()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn all_entries(&self) -> Vec<FileEntry> {
        let mut entries: Vec<FileEntry> = self.entries.values().cloned().collect();
        entries.sort_by_key(|e| e.id);
        entries
    }

    pub fn load_entries(&mut self, entries: Vec<FileEntry>) {
        self.entries.clear();
        self.root = TrieNode::new();
        for entry in entries {
            let id = entry.id;
            self.entries.insert(id, entry.clone());
            let lower_name = entry.name.to_lowercase();
            self._insert_str(&lower_name, id);
            if let Some(dot) = lower_name.rfind('.') {
                let stem = &lower_name[..dot];
                if !stem.is_empty() {
                    self._insert_str(stem, id);
                }
            }
            if id >= self.next_id {
                self.next_id = id + 1;
            }
        }
    }
}

fn remove_from_trie(node: &mut TrieNode, s: &str, id: i64, depth: usize) {
    if depth == s.len() {
        node.file_ids.retain(|&x| x != id);
        return;
    }
    if let Some(ch) = s.chars().nth(depth) {
        if let Some(child) = node.children.get_mut(&ch) {
            remove_from_trie(child, s, id, depth + 1);
        }
    }
}

fn collect_ids(node: &TrieNode, ids: &mut Vec<i64>) {
    ids.extend_from_slice(&node.file_ids);
    for child in node.children.values() {
        collect_ids(child, ids);
    }
}

