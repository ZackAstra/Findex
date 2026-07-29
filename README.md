# Findex - Fast Windows File Search ??

> 轻量级 Windows 本地文件搜索工具 · 融合 Everything 的秒级索引速度与 Listary 的场景感知体验

---

## 当前状态 (v0.2.1)

| 维度 | 能力 |
|------|------|
| 索引引擎 | USN Journal API（秒级索引）+ FsWalker 降级方案 |
| 权限模式 | 单进程 AdjustTokenPrivileges（管理员用户自动启用，标准用户降级 FsWalker） |
| 搜索 | Trie 前缀/子串/路径匹配 + 拼音首字母 |
| UI | egui 搜索浮窗 + 设置窗口（系统托盘常驻） |
| 热键 | Ctrl+Space 搜索 / Ctrl+Shift+F 设置（可配置） |
| 增量更新 | USN Journal 增量读取（基于 last_usn） |
| 存储 | JSON 持久化到 `%APPDATA%/Findex/` |
| 体积 | 单 exe ~5.5MB，零外部依赖 |

---

## ?? Quick Start

### GUI
双击 `findex.exe` 启动。首次启动自动建索引，后续启动秒加载。启动后自动常驻系统托盘。

- **Ctrl+Space** — 搜索浮窗
- **Ctrl+Shift+F** — 设置窗口（索引目录、排除规则、快捷键、主题）
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

## ?? Downloads

从 [Releases](https://github.com/ZackAstra/Findex/releases) 下载最新 `findex.exe`。

| 版本 | 亮点 |
|------|------|
| **v0.2.1** | ? USN Journal 增量索引 + 权限管理 |
| v0.2.0 | USN Journal 引擎 |
| v0.1.9.1 | 单 exe 合并 |
| v0.1.0 | 初始 GUI 版本 |

---

## ? 功能特性

| 特性 | 说明 |
|------|------|
| ? USN Journal 索引 | 1-2 秒扫描全盘，无需全量遍历 |
| ?? 增量更新 | 基于 last_usn 的增量读取，新增/删除/修改实时感知 |
| ?? 拼音搜索 | 拼音首字母匹配（`bg` → `报告.docx`） |
| ?? 模糊搜索 | 前缀/子串/路径片段匹配 + 评分排序 |
| ?? 持久化 | 索引自动保存到 `%APPDATA%/Findex/`，后续秒加载 |
| ?? 暗色主题 | 自定义 egui 暗色主题，跟随系统 |
| ?? 文件类型筛选 | 文档/代码/图片/视频/音频/压缩包/文件夹 |
| ?? 全局热键 | Ctrl+Space 搜索，Ctrl+Shift+F 设置（可配置） |
| ?? 轻量 | 单 exe ~5.5MB，零外部依赖，零安装 |

---

## ??? Architecture

### 进程模型
```
findex.exe (单进程)
  ├─ 系统托盘（保活）
  ├─ USN Journal 读取器
  │   ├─ AdjustTokenPrivileges → 启用 SE_BACKUP_NAME
  │   ├─ CreateFileW(\\.\C:) → USN Journal 枚举
  │   └─ 失败 → 降级 FsWalker
  ├─ Ctrl+Space → egui 搜索浮窗
  ├─ Ctrl+Shift+F → egui 设置窗口
  └─ 托盘右键 → 显示设置 / 退出
```

### 与 Everything/Listary 对比

| 维度 | Everything | Listary | Findex |
|------|-----------|---------|--------|
| 进程数 | 2 (服务+UI) | 2 (后台+UI) | **1 (单进程)** |
| 权限来源 | SYSTEM 账户 | 安装时提权 | 当前令牌 |
| 标准用户支持 | ? | ? | ? (降级 FsWalker) |
| 安装复杂度 | 高 (服务注册) | 中 | **零 (绿色单 exe)** |
| 开发复杂度 | 高 (IPC) | 中 | **低** |
| 覆盖范围 | 100% | 100% | **90%+ (管理员用户)** |

### 架构演进路线
```
v0.2.x ── egui 单进程优化 (当前)
  ├─ v0.2.2: 设置窗口 UI 重构 (codex-bridge 风格)
  ├─ v0.2.3: 开机自启 + 搜索体验优化
  └─ v0.2.4: 搜索性能优化

v0.3.x ── 多线程消息循环
  └─ 搜索浮窗 + 设置窗口可同时打开

v0.4.x ── Tauri 迁移 (需网络环境)
  └─ 设置窗口 WebView2，搜索浮窗保持 egui 原生

v0.5.x ── 双进程服务架构
  └─ SYSTEM 账户服务 + 普通用户 UI 进程 (全用户覆盖)
```

---

## ?? Project Structure

```
D:\Findows\
├── engine/src/             # 核心引擎 (Rust 库)
│   ├── ffi.rs              #   kernel32 FFI 声明 + USN 数据结构
│   ├── usn_reader.rs       #   USN Journal 读取器
│   ├── index_engine.rs     #   Trie 索引
│   ├── searcher.rs         #   搜索路由器
│   ├── storage.rs          #   JSON 持久化
│   ├── pinyin.rs           #   拼音匹配
│   ├── walker.rs           #   FsWalker 降级方案
│   └── types.rs            #   公共类型
├── ui/src/                 # 单一二进制 (Rust, Win32 + egui)
│   ├── main.rs             #   入口 / 热键 / 托盘 / CLI
│   ├── config.rs           #   配置管理 + USN 状态
│   ├── egui_win32.rs       #   egui 软件光栅化渲染器
│   ├── egui_windows.rs     #   egui 搜索浮窗 + 设置窗口
│   └── win32.rs            #   Win32 API FFI 声明
├── scripts/                # 构建脚本
├── PRD.md                  # 产品需求文档
├── IMPLEMENTATION_PLAN.md  # 实施方案
└── DEVELOPMENT_PLAN.md     # 开发计划 (本地跟踪, 不提交)
```

---

## ??? Roadmap

| 版本 | 目标 | 状态 |
|------|------|------|
| v0.2.2 | 设置窗口 UI 重构（codex-bridge 风格） | ?? 进行中 |
| v0.2.3 | 开机自启 + 搜索体验优化 | ?? 规划中 |
| v0.2.4 | 搜索性能优化（Trie 压缩、缓存、虚拟列表） | ?? 规划中 |
| v0.3.0 | 多线程消息循环（搜索+设置可同时打开） | ?? 规划中 |
| v0.4.0 | Tauri 迁移（设置窗口 WebView2，需网络环境） | ?? 规划中 |
| v0.5.0 | 双进程服务架构（全用户覆盖） | ?? 规划中 |

---

## ?? Changelog

### v0.2.1 (2026-07-29)
- ? **AdjustTokenPrivileges 权限提升** — 自动启用 SE_BACKUP_NAME 特权
- ? **USN Journal 可用性检测** — 启动时检测并提示管理员运行建议
- ?? **架构定稿** — 单进程方案，覆盖管理员用户 90%+ 场景
- ?? **架构文档** — Everything/Listary 三方对比分析

### v0.2.0 (2026-07-28)
- ? **USN Journal 增量索引引擎** — 替换 FsWalker 全量遍历
- ? **engine/src/ffi.rs** — kernel32 FFI + USN 数据结构
- ? **engine/src/usn_reader.rs** — enumerate_volume / read_changes / query_journal_id
- ? **增量更新** — 基于 last_usn 的 Added/Deleted/Modified 变更
- ? **USN 状态持久化** — usn_state.json 追踪每个卷的 Journal 状态
- ?? **自动降级** — USN 不可用时回退到 FsWalker

### v0.1.x (2026-07-28)
- **v0.1.9.1**: 单 exe 合并（CLI+GUI 合并为 findex.exe）
- **v0.1.9**: 首次启动自动建索引，索引路径规范化
- **v0.1.8**: 文件类型筛选，快捷键可配置，排除规则
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

## ??? Build from Source

```bash
# 需要 Rust + GNU 工具链
$env:Path = "D:\Findows\scripts;$env:Path"
cargo +stable-x86_64-pc-windows-gnu build --release -p findex
```

---

## ?? License

MIT
