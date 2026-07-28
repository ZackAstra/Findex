/// Findex CLI - command-line file search tool.
use std::path::PathBuf;
// use std::time::{SystemTime, UNIX_EPOCH};

use findex_engine::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let prog = args.first().map(|s| s.as_str()).unwrap_or("findex");

    if args.len() < 2 {
        print_usage(prog);
        return;
    }

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
    eprintln!("  {prog} index <directory>       Index a directory");
    eprintln!("  {prog} search <query>           Search indexed files");
    eprintln!("  {prog} status                   Show index status");
    eprintln!("  {prog} help                     Show this help");
    eprintln!();
    eprintln!("SEARCH OPTIONS:");
    eprintln!("  --max <n>       Maximum results (default: 50)");
    eprintln!("  --context <p>   Context path for scoring");
    eprintln!("  --json          Output as JSON");
    eprintln!("  --db <path>     Database path (default: findex.db)");
    eprintln!();
    eprintln!("INDEX OPTIONS:");
    eprintln!("  --db <path>     Database path (default: findex.db)");
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
    let db_path = parse_flag(rest, "--db").unwrap_or_else(|| "findex.db".to_string());

    let storage = match Storage::open(&db_path) {
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
        eprintln!("No entries in database. Use 'findex index' first.");
        return;
    }

    let mut index = TrieIndex::new();
    index.load_entries(entries);
    eprintln!("Searching {} indexed entries...", index.len());

    let searcher = Searcher::new(index);
    let search_query = SearchQuery {
        query: query.clone(),
        scope: SearchScope::Global,
        context_path: context,
        max_results,
        offset: 0,
        sort_by: SortBy::Relevance,
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
            let _type_str = if entry.is_dir { "DIR" } else { "    " };
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
    let db_path = parse_flag(rest, "--db").unwrap_or_else(|| "findex.db".to_string());
    let depth: usize = parse_flag(rest, "--depth").and_then(|s| s.parse().ok()).unwrap_or(0);

    let root = PathBuf::from(&path);
    if !root.exists() {
        eprintln!("Error: path does not exist: {}", path);
        return;
    }

    eprintln!("Indexing {} (depth: {})...", root.display(), if depth == 0 { "unlimited".to_string() } else { depth.to_string() });

    let start = std::time::Instant::now();
    let entries = match FsWalker::walk(&root, depth) {
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

    let mut index = TrieIndex::new();
    for entry in &entries {
        index.insert(entry.clone());
    }
    eprintln!("Index built with {} entries", index.len());

    let storage = match Storage::open(&db_path) {
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
    let db_path = parse_flag(args, "--db").unwrap_or_else(|| "findex.db".to_string());

    let storage = match Storage::open(&db_path) {
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

    // Estimate memory size
    let mem_estimate: usize = entries.iter().map(|e| e.name.len() + e.path.len() + e.parent_path.len() + 64).sum();

    println!("Findex Index Status");
    println!("{:-^50}", "");
    println!("Database:     {}", db_path);
    println!("Total:        {} entries", total);
    println!("  Files:      {}", files);
    println!("  Directories: {}", dirs);
    println!("Memory est.:  {} KB", mem_estimate / 1024);
}

fn results_to_json(results: &[SearchResult]) -> String {
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

