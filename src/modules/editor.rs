use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll: usize,
    pub file_path: Option<PathBuf>,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll: 0,
            file_path: None,
        }
    }

    fn clamp_cursor(&mut self) {
        let line_len = self.lines[self.cursor_line].chars().count();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_cursor();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.clamp_cursor();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].chars().count();
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_line].chars().count();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let byte_idx = char_idx_to_byte_idx(&self.lines[self.cursor_line], self.cursor_col);
        self.lines[self.cursor_line].insert(byte_idx, c);
        self.cursor_col += 1;
    }

    pub fn insert_newline(&mut self) {
        let byte_idx = char_idx_to_byte_idx(&self.lines[self.cursor_line], self.cursor_col);
        let remainder = self.lines[self.cursor_line].split_off(byte_idx);
        self.lines.insert(self.cursor_line + 1, remainder);
        self.cursor_line += 1;
        self.cursor_col = 0;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_col > 0 {
            let byte_idx = char_idx_to_byte_idx(&self.lines[self.cursor_line], self.cursor_col - 1);
            self.lines[self.cursor_line].remove(byte_idx);
            self.cursor_col = self.cursor_col.saturating_sub(1);
        } else if self.cursor_line > 0 {
            let prev_len = self.lines[self.cursor_line - 1].chars().count();
            let rest = self.lines.remove(self.cursor_line);
            self.lines[self.cursor_line - 1].push_str(&rest);
            self.cursor_line -= 1;
            self.cursor_col = prev_len;
        }
    }

    pub fn adjust_scroll(&mut self, visible_lines: usize) {
        if visible_lines == 0 {
            return;
        }
        if self.cursor_line < self.scroll {
            self.scroll = self.cursor_line;
        } else if self.cursor_line >= self.scroll + visible_lines {
            self.scroll = self.cursor_line - visible_lines + 1;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, save_mod: KeyModifiers) {
        let is_save = matches!(key.code, KeyCode::Char(c) if c == 's' || c == 'S')
            && key.modifiers.contains(save_mod);
        if is_save || key.code == KeyCode::Char('\x13') {
            let _ = save_file(self);
            return;
        }
        match key.code {
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Char(c) => self.insert_char(c),
            _ => {}
        }
    }
}

pub fn save_file(editor: &EditorState) -> std::io::Result<()> {
    if let Some(path) = &editor.file_path {
        let content = editor.lines.join("\n");
        fs::write(path, content)?;
    }
    Ok(())
}

pub fn open_file_in_editor(path: &std::path::Path, editor: &mut EditorState) {
    if let Ok(content) = fs::read_to_string(path) {
        let lines: Vec<String> = content
            .lines()
            .map(|l| l.trim_end_matches('\r').to_string())
            .collect();
        let mut lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        lines.push(String::new());
        editor.lines = lines;
        editor.cursor_line = 0;
        editor.cursor_col = 0;
        editor.scroll = 0;
        editor.file_path = Some(path.to_path_buf());
    }
}

fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
