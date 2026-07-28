# Findex - Fast Windows File Search 🔍

Findex is a lightweight Windows file search tool that combines the indexing speed of **Everything** with the context-aware experience of **Listary**.

## Features

- **Fast Indexing**: Filesystem walker for building a searchable file index
- **Instant Search**: Prefix, substring, and path segment matching
- **Pinyin Search**: Chinese pinyin initial matching (e.g., "bg" → "报告.docx")
- **Persistent Storage**: JSON-based index that persists between sessions
- **Lightweight**: Single ~1.3MB executable, zero external dependencies

## Quick Start

```bash
# Index a directory
findex index "C:\Users\YourName" --depth 3

# Search for files
findex search "report"
findex search ".pdf" --max 100
findex search "src/main" --json

# Check index status
findex status
```

## Build from Source

```bash
# Requires Rust with GNU toolchain
cargo +stable-x86_64-pc-windows-gnu build --release
```

## Project Structure

```
findex/
├── engine/       # Core search engine (Rust library)
│   └── src/
│       ├── index_engine.rs  # Trie-based file index
│       ├── pinyin.rs        # Chinese pinyin matching
│       ├── searcher.rs      # Search router & scoring
│       ├── storage.rs       # JSON persistence
│       ├── types.rs         # Core data types
│       └── walker.rs        # Filesystem scanner
├── cli/          # CLI tool (Rust binary)
│   └── src/
│       └── main.rs
└── docs/         # Documentation
```

## License

MIT
