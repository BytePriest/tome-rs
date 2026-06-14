# AGENTS.md — tome

A Rust TUI file explorer + text editor built with Ratatui and crossterm.

## Quick reference

```bash
cargo check              # fast type-check
cargo clippy -- -D warnings
cargo fmt --check
cargo test               # no tests exist yet
```

Package: `tome`, Rust edition 2024 (`let` chains, `.. &&` patterns in use).

## Architecture

```
src/main.rs   — entry point, event loop, ratatui init/restore
src/app.rs    — App, ExplorerState, EditorState structs + key handling
src/ui.rs     — render functions (3-column layout: 30% | 1-char | fill)
```

No modules are `pub` beyond the crate root — everything is `mod`-internal.

## Keybindings

| Key | Action |
|-----|--------|
| `Esc`, `q`, `Ctrl+C` | Quit |
| `Tab` | Toggle focus (Explorer ↔ Editor) |
| `↑`/`↓`/`Enter` | Explorer: navigate / open file or dir |
| `↑`/`↓`/`←`/`→` | Editor: move cursor |
| `Enter` / `Backspace` | Editor: newline / delete |
| any other char | Editor: insert |

## Notable

- No tests, no CI config, no README.
- `Focus::Explorer` is the default and has a `..`-parent entry at index 0.
- `EditorState::adjust_scroll(visible_lines)` is called from `ui::render` each frame.
