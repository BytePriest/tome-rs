# tome-rs

基于 Rust、[Ratatui](https://ratatui.rs) 和 [Ropey](https://github.com/cessen/ropey) 的终端文件浏览器与文本编辑器。

## 功能特性

- **文件树浏览** — 左侧面板以 VSCode 风格目录树浏览和打开文件
- **文本编辑** — 插入、删除、换行、保存；基于 grapheme cluster 的光标移动
- **软折行** — 超出终端宽度的长行自动折行；↑/↓ 在视觉行间导航
- **视觉列保持** — ↑/↓ 保持光标在折行段与行间的横向位置
- **CJK 支持** — 中日韩全角字符以 2 列宽度正确渲染
- **增量搜索** — `Ctrl+F` 打开底部搜索栏，匹配高亮显示
- **搜索选项** — `Alt+C` 大小写敏感、`Alt+W` 全词匹配、`Alt+R` 替换模式
- **匹配导航** — `Alt+N` / `Alt+P` 前后循环跳转，带边界提示
- **错误处理** — 通过 `anyhow` 处理，保存失败仅记录不崩溃
- **诊断日志** — `tracing` 输出到 `debug.log`（默认 debug 级别）

## 快速开始

```bash
# 在当前目录运行（从 cwd 打开文件树）
cargo run

# 终端需支持 stty -ixon，防止 Ctrl+S 被系统拦截
```

`stty -ixon` 在启动时自动设置，退出时恢复。

## 快捷键

### 全局
| 按键 | 功能 |
|------|------|
| `Ctrl+Q` / `Ctrl+C` | 退出程序 |
| `Tab` | 切换焦点（文件树 ↔ 编辑区） |

### 文件树（左侧面板）
| 按键 | 功能 |
|------|------|
| `↑` / `↓` | 上下移动选择 |
| `Enter` | 打开文件（加载到编辑器）/ 展开或收起目录 |

### 编辑器（右侧面板）
| 按键 | 功能 |
|------|------|
| `↑` / `↓` / `←` / `→` | 移动光标（↑/↓ 保持视觉列） |
| `Enter` | 换行 |
| `Backspace` | 向后删除字符（按 grapheme cluster） |
| 输入任意字符 | 插入文本 |
| `Ctrl+S` | 保存当前文件 |

### 搜索（`Ctrl+F`）
| 按键 | 功能 |
|------|------|
| `Ctrl+F` | 打开 / 关闭搜索栏（编辑器底部） |
| 输入文字 | 键入搜索关键词 |
| `Enter` | 执行搜索，跳转到光标后的第一个匹配 |
| `Alt+N` | 下一个匹配 |
| `Alt+P` | 上一个匹配（若无可从光标处向上搜索） |
| `Alt+C` | 切换大小写敏感匹配 |
| `Alt+W` | 切换全词匹配 |
| `Alt+R` | 切换替换模式 |
| `Esc` | 关闭搜索栏 |

当搜索到达文件首尾时，搜索栏中显示 `已到开头` / `已到结尾` 提示。

## 架构

```
src/
├── main.rs          入口、事件循环、tty 设置
├── app.rs           App 结构体、Focus 枚举、模块组合
├── core/            业务逻辑层（无渲染依赖）
│   ├── cursor.rs    光标位置、移动、视觉行/列计算
│   ├── document.rs  Document（持有 File + 段计算 + 查询方法）
│   ├── editor.rs    EditorState（组合 Document + Cursor），编辑操作
│   ├── explorer.rs  TreeNode、VisibleEntry、ExplorerState
│   ├── file.rs      File 结构体（path、Rope、dirty、open/save）
│   ├── keyboard.rs  键盘事件分发
│   ├── mod.rs
│   └── search.rs    SearchState（含开关标志）
├── infra/           算法与工具函数
│   ├── engine.rs    find_all() 字节索引搜索、Match 结构体
│   └── mod.rs
└── ui/              Ratatui 渲染层（纯视图）
    ├── divider.rs   面板间竖线分隔符
    ├── editor.rs    软折行渲染、光标定位
    ├── explorer.rs  文件树渲染（带缩进标志）
    ├── layout.rs    三列布局编排器
    ├── mod.rs
    └── search.rs    底部搜索栏渲染（含开关按钮）
```

## 依赖

- **ratatui** / **crossterm** — TUI 框架与终端后端
- **ropey** — 绳数据结构，O(log n) 文本操作
- **unicode-segmentation** — Grapheme cluster 边界检测
- **unicode-width** — 显示宽度（CJK=2，ASCII=1）
- **anyhow** — 错误处理
- **tracing** / **tracing-subscriber** — 结构化日志输出到 `debug.log`

## 开发

```bash
cargo check              # 快速类型检查
cargo clippy -- -D warnings
cargo fmt --check
cargo test               # 49 个测试
cargo run
```

## 许可证

MIT
