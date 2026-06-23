# tome-rs

A terminal file explorer and text editor built with Rust, [Ratatui](https://ratatui.rs), and [Ropey](https://github.com/cessen/ropey).

## Features

- **File tree explorer** — browse and open files from a VSCode-style directory tree (left panel)
- **Text editor** — insert, delete, newline, save; grapheme-cluster-aware cursor movement
- **Soft line wrapping** — long lines wrap at terminal width; up/down navigate between visual segments
- **Visual column preservation** — ↑/↓ maintain cursor x-position across wrapped segments and lines
- **CJK support** — fullwidth characters (Chinese, Japanese, Korean) render at 2-column width
- **Incremental search** — `Ctrl+F` opens a bottom search bar with match highlighting
- **Search toggles** — `Alt+C` case sensitivity, `Alt+W` whole word, `Alt+R` replace mode
- **Match navigation** — `Alt+N` / `Alt+P` cycle forward/backward with boundary hints
- **Error handling** — via `anyhow`; save failures logged but non-fatal
- **Diagnostic logging** — `tracing` to `debug.log` (debug level by default)

## Quick start

```bash
# Run in the current directory (opens file tree from cwd)
cargo run

# Terminal must support `stty -ixon` to prevent Ctrl+S being captured
```

`stty -ixon` is applied automatically on startup (restored on exit).

## Keybindings

### Global
| Key | Action |
|-----|--------|
| `Ctrl+Q` / `Ctrl+C` | Quit |
| `Tab` | Toggle focus (Explorer ↔ Editor) |

### Explorer (left panel)
| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate entries |
| `Enter` | Open file (loads into editor) / toggle directory expand |

### Editor (right panel)
| Key | Action |
|-----|--------|
| `↑` / `↓` / `←` / `→` | Move cursor (↑/↓ preserve visual column) |
| `Enter` | Newline |
| `Backspace` | Delete character backward (grapheme cluster) |
| Type any character | Insert text |
| `Ctrl+S` | Save current file |

### Search (`Ctrl+F`)
| Key | Action |
|-----|--------|
| `Ctrl+F` | Open / close search bar (bottom of editor) |
| Type | Enter search keyword |
| `Enter` | Perform search, jump to first match after cursor |
| `Alt+N` | Next match |
| `Alt+P` | Previous match (or backward search from cursor if no results) |
| `Alt+C` | Toggle case-sensitive matching |
| `Alt+W` | Toggle whole-word matching |
| `Alt+R` | Toggle replace mode |
| `Esc` | Close search bar |

When search reaches the start or end of the file, a hint (`已到开头` / `已到结尾`) is shown in the search bar.

## Architecture

```
src/
├── main.rs          Entry point, event loop, tty setup
├── app.rs           App struct, Focus enum, module wiring
├── core/            Business logic (no rendering dependencies)
│   ├── cursor.rs    Cursor position, movement, visual row/col
│   ├── document.rs  Document (owns File + segments + queries)
│   ├── editor.rs    EditorState (composes Document + Cursor), edit ops
│   ├── explorer.rs  TreeNode, VisibleEntry, ExplorerState
│   ├── file.rs      File struct (path, Rope, dirty, open/save)
│   ├── keyboard.rs  Key event dispatch
│   ├── mod.rs
│   └── search.rs    SearchState with toggle flags
├── infra/           Algorithms and utilities
│   ├── engine.rs    find_all() byte-index search, Match struct
│   └── mod.rs
└── ui/              Ratatui rendering (pure view layer)
    ├── divider.rs   Vertical divider between panels
    ├── editor.rs    Soft-wrapping render, cursor positioning
    ├── explorer.rs  File tree render with indent markers
    ├── layout.rs    3-column layout orchestrator
    ├── mod.rs
    └── search.rs    Bottom-bar search render with toggle buttons
```

## Dependencies

- **ratatui** / **crossterm** — TUI framework and terminal backend
- **ropey** — Rope data structure for O(log n) text operations
- **unicode-segmentation** — Grapheme cluster boundary detection
- **unicode-width** — Display width (handles CJK = 2, ASCII = 1)
- **anyhow** — Error handling
- **tracing** / **tracing-subscriber** — Structured logging to `debug.log`

## Development

```bash
cargo check              # Fast type-check
cargo clippy -- -D warnings
cargo fmt --check
cargo test               # 49 tests
cargo run
```

## License

MIT
