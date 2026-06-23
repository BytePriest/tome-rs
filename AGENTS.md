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
src/
  main.rs          — entry, event loop, ratatui init/restore, stty -ixon
  app.rs           — App, Focus enum (Explorer/Editor/Search), module wiring
  ui.rs            — 3-column layout (explorer | divider | editor) + search bar
  modules/
    file.rs        — File struct: path, rope (Rope), dirty, open()/save()
    editor.rs      — EditorState: cursor_byte (abs byte), cursor_line, scroll
    explorer.rs    — TreeNode, VisibleEntry, ExplorerState (file tree)
    keyboard.rs    — handle_key_event: dispatches Ctrl+Q/Ctrl+F/Tab/Enter etc.
    search.rs      — SearchState: input, matches, current, hint, go_to_match()
  algo/
    engine.rs      — Match struct, find_all(rope, pattern) byte-index search
  widget/
    editor.rs      — render_editor: line numbers, search highlight, cursor pos
    explorer.rs    — render_explorer: file tree with ▶/▼/indent
    search.rs      — render_search: bottom bar with hint, match count, cursor
```

## Storage & Cursor Model

- **Rope** (`ropey`): All file content stored in a single `Rope` (not `Vec<String>`).
  - `rope.insert(byte_offset, &str)`, `rope.remove(range)` for O(log n) edits.
  - `rope.lines()` yields `RopeSlice` including trailing `\n` (except last line).
  - `rope.line_to_byte(n)` / `rope.len_lines()` for line↔byte conversion.
- **Cursor**: absolute byte offset (`cursor_byte`) into the rope. `cursor_line` is a cached line index, updated on vertical moves.
- **Grapheme movement**: `unicode_segmentation::GraphemeCursor` for left/right/delete-by-cluster, operating on the line slice.
- **Column position**: `line_str[..byte_in_line].width()` via `unicode-width` (CJK=2, ASCII=1).

## Keybindings

| Key | Action |
|-----|--------|
| `Ctrl+Q`, `Ctrl+C` | Quit |
| `Tab` | Toggle focus (Explorer ↔ Editor) |
| `Ctrl+S` | Save file (also matches raw `\x13`) |
| `Ctrl+F` | Open search bar |
| `Esc` | Close search bar (does NOT quit) |
| `↑`/`↓`/`Enter` | Explorer: navigate / open file or dir |
| `↑`/`↓`/`←`/`→` | Editor: move cursor |
| `Enter` / `Backspace` | Editor: newline / delete |
| `Alt+N` / `Alt+P` | Search: next / previous match |
| any other char | Editor: insert (or search: type) |

## Notable

- Tests exist in `src/core/editor.rs` (`cargo test`).
- No CI config.
- `Focus::Explorer` is the default.
- `EditorState::adjust_scroll(visible_lines)` is called from `ui::render` each frame.
- `stty -ixon` required to prevent Ctrl+S being intercepted as XOFF.
- Search bar is a bottom status line (not popup), shows match count, cursor, hint strings.
- All logging via `tracing` to `debug.log` (not stderr), default level `debug`.
- Error handling via `anyhow` (not `color-eyre`).
- `event::read()` filtering: only `KeyEventKind::Press` passed through.
