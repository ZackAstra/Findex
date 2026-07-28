# Findex - Fast Windows File Search 🔍

Findex is a lightweight Windows file search tool that combines the indexing speed of **Everything** with the context-aware experience of **Listary**.

## 🚀 Quick Start

### GUI (图形界面)
双击 `findex.exe` 启动。**首次启动自动索引所有可用磁盘**，后续启动秒加载。

启动后自动常驻系统托盘，无窗口显示。

- **快捷键 `Ctrl+Space`** — 呼出搜索浮窗（egui 现代化 UI）
- **快捷键 `Ctrl+Shift+F`** — 打开设置窗口
- 拼音搜索（如 `bg` → `报告.docx`）
- 文件索引管理
- 系统托盘右键菜单（显示设置 / 退出）

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

## 📦 Downloads

从 [Releases](https://github.com/ZackAstra/Findex/releases) 页面下载最新版本。

| 版本 | 说明 |
|------|------|
| v0.1.9.1 | 最新版 — 单 exe 合并 + 自动建索引 + 持久化 |
| v0.1.8 | 文件类型筛选 + 快捷键配置 + 排除规则 |
| v0.1.4 | egui 技术验证 + 渲染引擎 |
| v0.1.0 | 初始 GUI 版本 |

## ✨ Features

- **🔍 Fast Indexing**: Filesystem walker for building a searchable file index
- **⚡ First-Launch Auto-Index**: Automatically indexes all available drives on first run
- **⚡ Instant Search**: Prefix, substring, and path segment matching
- **🀄 Pinyin Search**: Chinese pinyin initial matching (e.g., `bg` → `报告.docx`)
- **💾 Persistent Storage**: JSON-based index persists in `%APPDATA%/Findex/`
- **🪶 Lightweight**: Single ~5.5MB executable, zero external dependencies
- **🖥️ egui 现代化 UI**: 软件光栅化渲染，零 GPU 依赖
- **⌨️ Global Hotkeys**: Ctrl+Space for search, Ctrl+Shift+F for settings
- **🔄 系统托盘常驻**: 启动即后台运行，不占窗口

## 📖 Usage

### GUI
- 启动即系统托盘常驻，不显示窗口
- 首次自动建索引，后续启动秒加载
- `Ctrl+Space` 呼出 egui 搜索浮窗（搜索框 + 实时结果列表 + 文件类型筛选）
- `Ctrl+Shift+F` 打开设置窗口（配置索引目录、搜索选项、快捷键、排除规则、主题）
- 托盘右键菜单：显示设置 / 退出

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
--db <path>     指定数据库路径 (默认: %APPDATA%/Findex/index.db)
--depth <n>     索引递归深度 (0=无限)
```

## 🏗️ Build from Source

```bash
# 需要 Rust + GNU 工具链
# 确保 dlltool 在 PATH 中（或使用 scripts/dlltool.rs 绕行方案）
$env:Path = "D:\Findows\scripts;$env:Path"

cargo +stable-x86_64-pc-windows-gnu build --release -p findex
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
│   (已合并到 ui/)
├── ui/           # GUI tool (Rust binary, Win32 + egui)
│   └── src/
│       ├── win32.rs         # Win32 API FFI declarations
│       ├── egui_win32.rs    # egui 软件光栅化渲染器
│       ├── egui_windows.rs  # egui 搜索浮窗 + 设置窗口
│       ├── config.rs        # 配置管理（JSON 持久化）
│       └── main.rs          # 入口 & 热键 & 系统托盘
├── scripts/      # 构建工具脚本
├── DEVELOPMENT_PLAN.md  # 开发计划（不提交到远程）
├── PRD.md               # 产品需求文档
└── IMPLEMENTATION_PLAN.md # 实施方案
```

## 📋 Changelog

### v0.1.9.1 (2026-07-28)
- 🔧 **单 exe 合并** — CLI + GUI 合并为单个 \index.exe\，\index search/index/status/help\ 命令行模式
- ✨ **索引持久化确认** — 索引保存到 \%APPDATA%/Findex/index.db\，后续启动秒加载
- 🔧 **路径无关** — exe 可在任意位置运行，索引路径统一
- 🗑️ **删除独立 findex.exe / findex.exe** — 统一为单一 findex.exe
- 📖 **更新使用说明**
### v0.1.9 (2026-07-28)
- ✨ **首次启动自动建索引** — 检测所有可用盘符，自动构建 Trie 索引
- ✨ **索引路径规范化** — 统一保存在 `%APPDATA%/Findex/index.db`，exe 可任意位置运行
- ✨ **驱动自动检测** — 枚举系统 C:~Z: 固定驱动器
- 🔧 **删除"未找到索引"错误** — 不再显示 `No index found` 提示
- ✅ **编译零警告**

### v0.1.8 (2026-07-28)
- ✨ **文件类型筛选** — 搜索浮窗过滤栏（All/Folders/Docs/Code/Images/Archives/Audio/Video）
- ✨ **快捷键可配置** — 设置窗口热键录制器，可自定义 Ctrl+Space/Ctrl+Shift+F
- ✨ **索引排除规则** — 配置排除模式（如 .git/node_modules/target），索引时自动跳过
- 🔧 **热键重构** — 配置变更后自动重注册，即时生效
- 🔧 **引擎升级** — FsWalker::walk_with_excludes() 支持排除模式
- ✅ **构建零警告**

### v0.1.7 (2026-07-28)
- ✨ **暗色主题** — 自定义 egui 暗色主题配色，兼容 Windows 深色模式跟随
- ✨ **跟随系统主题** — 检测 Windows 注册表 AppsUseLightTheme，自动切换
- ✨ **搜索浮窗圆角** — SetWindowRgn + CreateRoundRectRgn 实现圆角窗口
- ✨ **文件图标美化** — 基于扩展名显示不同文件类型图标
- ✨ **文件大小显示** — KB/MB/GB 格式化显示
- ✨ **修改日期显示** — 搜索结果显示文件修改日期
- ✨ **设置窗口主题选择器** — 浅色/深色/跟随系统，实时切换
- ✅ **构建零警告**

### v0.1.6 (2026-07-28)
- ✨ **egui 设置窗口** — 索引目录管理、搜索选项配置、立即索引、保存/取消
- ✨ **文件夹浏览** — SHBrowseForFolderW 集成到 egui UI
- 🔧 **配置管理迁移** — load_config / get_config / set_config 移到 config.rs
- 🗑️ **删除旧 settings.rs** — 所有 UI 统一为 egui
- ✅ **构建零警告**

### v0.1.5 (2026-07-28)
- 🔧 **修复: 字体纹理 API** — 锁定 Mutex 后再调用 size() 和 image()
- 🧹 **清理: 移除未使用代码**
- 🗑️ **清理: 删除旧 search_overlay.rs**
- ✅ **构建零警告**

### v0.1.4 (2026-07-28)
- ✨ **egui 技术验证** — egui + epaint 成功编译 (GNU 工具链)
- ✨ **软件光栅化渲染器** — 三角形渲染 + 阿尔法混合 + 字体纹理采样
- ✨ **StretchDIBits 像素输出** — 像素缓冲区输出到 Win32 窗口
- 📦 **二进制体积** ~5.5MB（含字体数据）

### v0.1.3 (2026-07-28)
- ✨ **系统托盘图标** — NOTIFYICONDATAW + Shell_NotifyIconW
- ✨ **托盘右键菜单** — 显示设置 / 退出
- ✨ **启动无窗口** — 仅托盘图标，按需显示窗口

### v0.1.2 (2026-07-28)
- ✨ **配置持久化** — Config 结构体 + JSON 序列化/反序列化
- ✨ **设置窗口绑定** — 浏览、添加、删除、保存、取消、立即索引
- ✨ **文件夹选择** — SHBrowseForFolderW 对话框

### v0.1.1 (2026-07-28)
- ✨ **搜索浮窗连接引擎** — Ctrl+Space 实时搜索已索引文件
- ✨ **键盘导航** — ↑↓ 选择结果，Enter 打开文件，Esc 关闭浮窗
- ✨ **自动加载索引** — 启动时从 `%APPDATA%/Findex/index.db` 加载

### v0.1.0 (2026-07-27)
- ✨ **新增: 图形界面 (findex-ui)**
- ✨ **全局热键**: Ctrl+Space / Ctrl+Shift+F
- 🔧 **搜索结果去重优化**

### v0.0.1 (2026-07-27)
- ✨ 初始 MVP 发布
- 命令行搜索工具 (findex.exe)
- Trie 索引、前缀/子串/路径搜索
- 拼音首字母匹配
- JSON 持久化存储

## 🤝 Contributing

Issues and PRs are welcome! See [DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md) for development documentation.

## 📄 License

MIT



