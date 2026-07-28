# Findex - Fast Windows File Search 🔍

Findex is a lightweight Windows file search tool that combines the indexing speed of **Everything** with the context-aware experience of **Listary**.

## 🚀 Quick Start

### CLI (命令行搜索)
```powershell
# 索引一个目录
findex index "D:\Projects"

# 搜索文件
findex search "report"
findex search ".pdf" --max 100
findex search "src/main" --json

# 查看索引状态
findex status
```

### GUI (图形界面)
双击 `findex-ui.exe` 启动设置窗口。支持：
- 快捷键 `Ctrl+Space` 呼出搜索浮窗
- 快捷键 `Ctrl+Shift+F` 打开设置
- 拼音搜索（如 `bg` → `报告.docx`）
- 文件索引管理

## 📦 Downloads

| 版本 | 文件 | 说明 |
|------|------|------|
| v0.1.0 | `findex.exe` | 命令行搜索工具 (~1.3MB) |
| v0.1.0 | `findex-ui.exe` | 图形界面工具 (~1.2MB) |

从 [Releases](https://github.com/ZackAstra/Findex/releases) 页面下载最新版本。

## ✨ Features

- **🔍 Fast Indexing**: Filesystem walker for building a searchable file index
- **⚡ Instant Search**: Prefix, substring, and path segment matching
- **🀄 Pinyin Search**: Chinese pinyin initial matching (e.g., `bg` → `报告.docx`)
- **💾 Persistent Storage**: JSON-based index that persists between sessions
- **🪶 Lightweight**: Single ~1.3MB executable, zero external dependencies
- **🖥️ Native UI**: Windows native GUI with settings page and search overlay
- **⌨️ Global Hotkeys**: Ctrl+Space for search, Ctrl+Shift+F for settings

## 📖 Usage

### CLI
```powershell
findex index <directory>   # 索引一个目录
findex search <query>      # 搜索已索引的文件
findex status              # 查看索引状态
findex help                # 显示帮助

# 选项
--max <n>       最大结果数 (默认: 50)
--context <p>   上下文路径 (影响排序)
--json          输出为 JSON 格式
--db <path>     指定数据库路径 (默认: findex.db)
--depth <n>     索引递归深度 (0=无限)
```

### GUI
- 启动即显示设置窗口
- 配置索引目录、快捷键、搜索选项
- 搜索浮窗支持实时搜索

## 🏗️ Build from Source

```bash
# 需要 Rust + GNU 工具链
cargo +stable-x86_64-pc-windows-gnu build --release -p findex-cli
cargo +stable-x86_64-pc-windows-gnu build --release -p findex-ui
```

## 📁 Project Structure

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
│   └── src/main.rs
├── ui/           # GUI tool (Rust binary, Win32 native)
│   └── src/
│       ├── win32.rs         # Win32 API FFI declarations
│       ├── settings.rs      # Settings window
│       ├── search_overlay.rs # Search floating overlay
│       └── main.rs          # Entry point & hotkey handling
└── docs/         # Documentation
```

## 📋 Changelog

### v0.1.0 (2026-07-27)
- ✨ **新增: 图形界面 (findex-ui)**
  - 设置窗口：索引目录管理、快捷键配置、搜索选项
  - 搜索浮窗：全局热键呼出、实时搜索、结果列表
  - 原生 Win32 API 实现，零外部依赖
- ✨ **新增: 全局热键**
  - `Ctrl+Space` 呼出搜索浮窗
  - `Ctrl+Shift+F` 打开设置
- 🔧 **优化: 搜索结果去重**
- 🔧 **优化: 导出为独立 exe 文件**

### v0.0.1 (2026-07-27)
- ✨ 初始 MVP 发布
- 命令行搜索工具 (findex.exe)
- Trie 索引、前缀/子串/路径搜索
- 拼音首字母匹配
- JSON 持久化存储

## 🤝 Contributing

Issues and PRs are welcome! See [docs/](docs/) for development documentation.

## 📄 License

MIT
