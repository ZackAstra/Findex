# 实施方案：Findex - Windows 本地文件搜索工具

## 项目结构

```
D:\Findows\
├── PRD.md                     # 本 PRD 文档
├── docs/                      # 设计文档
│   ├── architecture.md        # 架构设计详述
│   ├── usn-journal-research.md # USN Journal API 调研
│   └── ui-prototype.md        # UI 交互原型
├── engine/                    # Rust 核心引擎
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # 引擎入口 / IPC 服务
│       ├── usn_reader.rs      # USN Journal 读取器
│       ├── index_engine.rs    # 索引引擎 (Trie + 倒排)
│       ├── searcher.rs        # 搜索路由器
│       ├── pinyin.rs          # 拼音匹配
│       ├── monitor.rs         # 文件变更监控
│       ├── storage.rs         # SQLite 持久化
│       ├── ipc.rs             # Named Pipes IPC
│       └── types.rs           # 公共类型定义
├── ui/                        # WPF UI 浮层
│   ├── Findex.UI.sln
│   └── Findex.UI/
│       ├── MainWindow.xaml
│       ├── SearchOverlay.xaml
│       └── ...
├── hooks/                     # Shell 钩子 DLL
│   └── src/
│       ├── keyboard_hook.rs   # 全局键盘钩子
│       └── shell_ext.rs       # Shell 扩展
└── cli/                       # 命令行工具
    └── src/
        └── main.rs
```

## 关键 API 接口

### IPC 协议 (Named Pipes, JSON-RPC)

```jsonc
// 搜索请求
{
  "method": "search",
  "params": {
    "query": "report",
    "scope": "current_folder",    // "global" | "current_folder" | "path"
    "context_path": "D:\\project",
    "max_results": 100,
    "offset": 0,
    "sort_by": "relevance"        // "relevance" | "name" | "date" | "size"
  }
}

// 搜索响应
{
  "result": {
    "total": 42,
    "results": [
      {
        "name": "report.docx",
        "path": "D:\\project\\docs\\report.docx",
        "size": 1024000,
        "modified": "2026-07-27T10:00:00Z",
        "created": "2026-07-01T08:00:00Z",
        "is_dir": false,
        "extension": ".docx",
        "score": 95,
        "highlights": {
          "name": [[0, 6]],
          "path": [[14, 6]]
        }
      }
    ]
  }
}

// 索引状态
{
  "method": "get_status",
  "params": {}
}
// 响应
{
  "result": {
    "total_files": 1234567,
    "indexed_files": 1234567,
    "last_index_time": "2026-07-27T10:00:00Z",
    "memory_usage_mb": 25.3,
    "volumes": ["C:", "D:"]
  }
}
```

## 第一轮 (P0) 技术验证任务

1. 创建 Rust 项目，集成 `windows` crate
2. 实现 USN Journal 读取器原型（读取 C: 卷 USN Journal，输出文件变更记录）
3. 测量：读取 100 万条 USN 记录耗时、内存占用
4. 实现 Trie 索引原型，测量 100 万文件名的内存占用
5. 实现 Named Pipes IPC 原型，测量延迟
6. 写 WPF 最小窗口原型，测量热键呼出延迟
7. 撰写技术验证报告
