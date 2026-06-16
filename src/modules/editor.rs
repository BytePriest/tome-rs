use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::Path;

use crate::modules::file::File;

#[derive(Debug)]
pub struct EditorState {
    pub file: File,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            file: File::new(),
            cursor_line: 0,
            cursor_col: 0,
            scroll: 0,
        }
    }

    pub fn load_file(&mut self, path: &Path) {
        tracing::debug!("load file: {}", path.display());
        self.file = File::open(path);
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll = 0;
    }

    fn clamp_cursor(&mut self) {
        let line_len = self.file.lines[self.cursor_line].chars().count();
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
        if self.cursor_line + 1 < self.file.lines.len() {
            self.cursor_line += 1;
            self.clamp_cursor();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.file.lines[self.cursor_line].chars().count();
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.file.lines[self.cursor_line].chars().count();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.file.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let byte_idx = char_idx_to_byte_idx(&self.file.lines[self.cursor_line], self.cursor_col);
        self.file.lines[self.cursor_line].insert(byte_idx, c);
        self.cursor_col += 1;
        self.file.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let byte_idx = char_idx_to_byte_idx(&self.file.lines[self.cursor_line], self.cursor_col);
        let remainder = self.file.lines[self.cursor_line].split_off(byte_idx);
        self.file.lines.insert(self.cursor_line + 1, remainder);
        self.cursor_line += 1;
        self.cursor_col = 0;
        self.file.dirty = true;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_col > 0 {
            let byte_idx =
                char_idx_to_byte_idx(&self.file.lines[self.cursor_line], self.cursor_col - 1);
            self.file.lines[self.cursor_line].remove(byte_idx);
            self.cursor_col = self.cursor_col.saturating_sub(1);
            self.file.dirty = true;
        } else if self.cursor_line > 0 {
            let prev_len = self.file.lines[self.cursor_line - 1].chars().count();
            let rest = self.file.lines.remove(self.cursor_line);
            self.file.lines[self.cursor_line - 1].push_str(&rest);
            self.cursor_line -= 1;
            self.cursor_col = prev_len;
            self.file.dirty = true;
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
            tracing::debug!(
                "save triggered: code={:?} mod={:?} path={:?}",
                key.code,
                key.modifiers,
                self.file.path.as_ref().map(|p| p.display().to_string())
            );
            match self.file.save() {
                Ok(()) => tracing::debug!("save succeeded"),
                Err(e) => tracing::warn!("save failed: {e}"),
            }
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

fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
