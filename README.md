# Findex - Fast Windows File Search 🔍

> 轻量级 Windows 本地文件搜索工具 · 融合 Everything 的秒级索引速度与 Listary 的场景感知体验

---

## 当前状态 (v0.3.0)

| 维度 | 能力 |
|------|------|
| 索引引擎 | USN Journal API（秒级索引）+ FsWalker 降级方案 |
| 权限模式 | 单进程 AdjustTokenPrivileges |
| 搜索 | Trie 前缀/子串/路径匹配 + 拼音首字母 |
| 搜索浮窗 | egui 原生 Win32 子窗口（Ctrl+Space） |
| 设置窗口 | egui 原生 Win32 窗口（Ctrl+Shift+F） |
| 热键 | Ctrl+Space 搜索 / Ctrl+Shift+F 设置（可配置） |
| 增量更新 | USN Journal 增量读取（基于 last_usn） |
| 存储 | JSON 持久化到 `%APPDATA%/Findex/` |
| **体积** | **单 exe ~2.9 MB (纯 Rust + egui，零 WebView2)** |
| **依赖** | **零外部运行时 — 开箱即用** |

### 构建状态

```
cargo +nightly-x86_64-pc-windows-gnu build --release -p findex (LTO + strip)
  ✅ 成功 (2.9 MB)  ← 推荐
```

当前构建环境:
- Rust: `nightly-x86_64-pc-windows-gnu` + `rust-lld` 链接器
- 无 Node.js / Tauri / WebView2 依赖

---

## 🚀 Quick Start

### GUI
双击 `findex.exe` 启动。首次启动自动建索引，后续启动秒加载。启动后自动常驻系统托盘。

- **Ctrl+Space** — 搜索浮窗（egui 原生）
- **Ctrl+Shift+F** — 设置窗口（egui 原生）
- 托盘右键菜单 — 显示设置 / 退出

### CLI
```powershell
findex index "D:\Projects"   # 索引一个目录
findex search "report"       # 搜索已索引的文件
findex status                # 查看索引状态
findex help                  # 显示帮助
```

### 管理员建议
```powershell
# 以管理员身份运行以获得 USN Journal 秒级索引
右键 → 以管理员身份运行
```

---

## 📦 Downloads

从 [Releases](https://github.com/ZackAstra/Findex/releases) 下载最新 `findex_{版本号}.exe`。

| 版本 | 亮点 | 体积 |
|------|------|------|
| **v0.3.0** | ✅ **纯原生 egui 方案 — 零外部依赖** | **2.9 MB** |
| v0.2.2 | 设置窗口 UI 重构 (codex-bridge 风格) | 5.5 MB |
| v0.2.1 | USN Journal 增量索引 + 权限管理 | — |
| v0.2.0 | USN Journal 引擎 | — |

---

## ✨ 功能特性

| 特性 | 说明 |
|------|------|
| ⚡ USN Journal 索引 | 1-2 秒扫描全盘，无需全量遍历 |
| 🔄 增量更新 | 基于 last_usn 的增量读取，新增/删除/修改实时感知 |
| 🀄 拼音搜索 | 拼音首字母匹配（`bg` → `报告.docx`） |
| 🎯 模糊搜索 | 前缀/子串/路径片段匹配 + 评分排序 |
| 💾 持久化 | 索引自动保存到 `%APPDATA%/Findex/`，后续秒加载 |
| 🎨 暗色/浅色主题 | 可配置，支持跟随系统 |
| 🔍 文件类型筛选 | 文档/代码/图片/视频/音频/压缩包/文件夹 |
| ⌨️ 全局热键 | Ctrl+Space 搜索，Ctrl+Shift+F 设置（可配置） |
| 🪶 极致轻量 | **单 exe 仅 2.9 MB，零外部依赖** |
| 🖥️ 纯原生渲染 | **egui 软件光栅化 + Win32 GDI，无 GPU / WebView2 依赖** |

---

## 🏗️ Architecture

### 当前架构 (v0.3.0)
```
findex.exe (单进程)
├─ Rust 引擎 (findex-engine)
│   ├─ USN Journal 读取器
│   ├─ Trie 索引
│   ├─ 拼音匹配器
│   └─ FsWalker 降级
├─ 系统托盘 (Win32 FFI)
├─ 全局热键 (Win32 FFI)
├─ 搜索浮窗 (egui + 软件光栅化)
└─ 设置窗口 (egui + 软件光栅化)
```

### 技术栈对比

| 维度 | v0.2.x (纯 egui) | v0.3.0-pre (Tauri v2) | **v0.3.0 (纯 egui)** |
|------|----------------|----------------------|-------------------|
| 运行时依赖 | 无 | **WebView2** | **无** |
| exe 体积 | ~5.5 MB | **~16.8 MB (strip)** | **~2.9 MB** |
| 设置窗口 | egui 原生 | WebView2 HTML/CSS | **egui 原生** |
| 构建复杂度 | 低 | **高 (Tauri/Node.js)** | **低** |
| 开箱即用 | ✅ | **❌ (需 WebView2)** | **✅** |

---

## 📁 Project Structure

```
D:\Findows\
├── engine/src/             # 核心引擎 (Rust 库)
│   ├── ffi.rs              #   kernel32 FFI + USN 数据结构
│   ├── usn_reader.rs       #   USN Journal 读取器
│   ├── searcher.rs         #   搜索路由
│   ├── storage.rs          #   JSON 持久化
│   ├── pinyin.rs           #   拼音匹配
│   ├── walker.rs           #   FsWalker 降级方案
│   └── types.rs            #   公共类型
├── ui/src/                 # egui 应用 (主二进制)
│   ├── main.rs             #   入口 / 热键 / 托盘 / CLI
│   ├── config.rs           #   配置管理 + USN 状态
│   ├── egui_win32.rs       #   egui 软件光栅化渲染器
│   ├── egui_windows.rs     #   egui 搜索浮窗 + 设置窗口
│   └── win32.rs            #   Win32 API FFI 声明
├── scripts/                # 构建工具 (dlltool)
├── PRD.md                  # 产品需求文档
├── IMPLEMENTATION_PLAN.md  # 实施方案
└── DEVELOPMENT_PLAN.md     # 开发计划
```

---

## 🗺️ Roadmap

| 版本 | 目标 | 状态 |
|------|------|------|
| **v0.3.0** | **纯原生 egui 方案 — 移除 Tauri/WebView2 依赖** | **✅ 已完成** |
| v0.3.1 | 应用图标 (windres)、搜索体验优化、开机自启 | 📋 规划中 |
| v0.4.0 | 双进程服务架构（全用户覆盖） | 📋 规划中 |

---

## 📋 Changelog

### v0.3.0 (2026-07-30)
- 🔥 **架构大重构 — 移除 Tauri v2 + WebView2 依赖**
- 🔥 **纯原生 egui 方案** — 搜索浮窗 + 设置窗口均使用 egui 软件光栅化
- 🔥 **极致精简** — exe 从 16.8 MB (Tauri) / 255 MB (debug) **降至 2.9 MB**
- 🔥 **零外部依赖** — 无需 WebView2 / Node.js，开箱即用
- ♻️ **代码简化** — 移除 src-tauri/，合并为单一 ui/ 二进制
- 🚀 **LTO + strip 优化** — release 构建仅 2.9 MB

### v0.2.2 (2026-07-29)
- ✨ **设置窗口 UI 重构** — codex-bridge Apple 风格卡片式布局
- ✨ **Apple 配色** — #0a84ff 强调色，#121212/#f5f5f5 窗口背景
- ✨ **卡片分组** — 主题/快捷键/索引目录/排除规则/搜索选项

### v0.2.1 (2026-07-29)
- ✨ **AdjustTokenPrivileges 权限提升** — 自动启用 SE_BACKUP_NAME 特权
- ✨ **USN Journal 可用性检测** — 启动时检测并提示管理员运行建议
- 🔧 **架构定稿** — 单进程方案，覆盖管理员用户 90%+ 场景

### v0.2.0 (2026-07-28)
- ✨ **USN Journal 增量索引引擎** — 替换 FsWalker 全量遍历
- ✨ **engine/src/ffi.rs** — kernel32 FFI + USN 数据结构
- ✨ **engine/src/usn_reader.rs** — enumerate_volume / read_changes / query_journal_id
- ✨ **增量更新** — 基于 last_usn 的 Added/Deleted/Modified 变更
- ✨ **USN 状态持久化** — usn_state.json 追踪每个卷的 Journal 状态
- 🔧 **自动降级** — USN 不可用时回退到 FsWalker

### v0.1.x (2026-07-28)
- **v0.1.9.1**: 单 exe 合并（CLI+GUI 合并为 findex.exe）
- **v0.1.9**: 首次启动自动建索引，索引路径规范化
- **v1.0.8**: 文件类型筛选，快捷键可配置，排除规则
- **v0.1.7**: 暗色主题，搜索浮窗圆角，文件图标
- **v0.1.6**: egui 设置窗口迁移
- **v0.1.5**: egui 搜索浮窗 + 代码清理
- **v0.1.4**: egui 技术验证 + 渲染引擎
- **v0.1.3**: 系统托盘 + 后台常驻
- **v0.1.2**: 设置窗口 + 配置持久化
- **v0.1.1**: 搜索浮窗连接引擎
- **v0.1.0**: 初始 GUI 版本

### v0.0.1 (2026-07-27)
- 命令行搜索工具 MVP: Trie 索引、拼音匹配、JSON 持久化

---

## 🏗️ Build from Source

```powershell
# 前提：Rust 工具链 (nightly GNU)
rustup toolchain install nightly-x86_64-pc-windows-gnu
$env:Path = "D:\Findows\scripts;$env:Path"

# 构建纯原生 egui 应用 (release)
cargo +nightly-x86_64-pc-windows-gnu build --release -p findex
# → target/release/findex.exe (2.9 MB)
```

---

## 📄 License

MIT
