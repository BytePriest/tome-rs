# tome-rs

A terminal file explorer and text editor built with Rust and Ratatui.

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
| `↑` / `↓` / `←` / `→` | Move cursor |
| `Enter` | Newline |
| `Backspace` | Delete character |
| Type any character | Insert text |
| `Ctrl+S` | Save current file |

### Search (`Ctrl+F`)
| Key | Action |
|-----|--------|
| `Ctrl+F` | Open search bar (bottom of editor) |
| Type | Enter search keyword |
| `Enter` | Perform search, jump to first match after cursor |
| `Alt+N` | Next match |
| `Alt+P` | Previous match / backward search from cursor |
| `Esc` | Close search bar |
