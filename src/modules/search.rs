use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::modules::editor::EditorState;
use crate::search::engine::{Match, find_all};

#[derive(Debug)]
pub struct SearchState {
    pub visible: bool,
    pub input: String,
    pub cursor: usize,
    pub matches: Vec<Match>,
    pub current: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            visible: false,
            input: String::new(),
            cursor: 0,
            matches: Vec::new(),
            current: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.input.clear();
            self.cursor = 0;
            self.matches.clear();
            self.current = 0;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, editor: &mut EditorState) {
        match key.code {
            KeyCode::Esc => {
                self.visible = false;
            }
            KeyCode::Enter | KeyCode::Char('\n') => {
                tracing::debug!(
                    "search Enter: input={:?} matches={}",
                    self.input,
                    self.matches.len()
                );
                if self.matches.is_empty() {
                    self.matches = find_all(&editor.file.lines, &self.input);
                }
                if !self.matches.is_empty() {
                    self.current = self
                        .matches
                        .iter()
                        .position(|m| {
                            m.line > editor.cursor_line
                                || (m.line == editor.cursor_line && m.col >= editor.cursor_col)
                        })
                        .unwrap_or(0);
                    self.go_to_match(self.current, editor);
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                tracing::debug!(
                    "search Alt+N: current={} total={}",
                    self.current,
                    self.matches.len()
                );
                if !self.matches.is_empty() {
                    self.current = (self.current + 1).min(self.matches.len() - 1);
                    self.go_to_match(self.current, editor);
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                tracing::debug!(
                    "search Alt+P: current={} total={}",
                    self.current,
                    self.matches.len()
                );
                self.prev_or_backward_search(editor);
            }
            KeyCode::Char(c) => {
                let byte_idx = char_idx_to_byte_idx(&self.input, self.cursor);
                self.input.insert(byte_idx, c);
                self.cursor += 1;
                self.matches.clear();
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let byte_idx = char_idx_to_byte_idx(&self.input, self.cursor - 1);
                    self.input.remove(byte_idx);
                    self.cursor -= 1;
                    self.matches.clear();
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.input.chars().count() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.input.chars().count(),
            _ => {}
        }
    }

    fn go_to_match(&self, idx: usize, editor: &mut EditorState) {
        if let Some(m) = self.matches.get(idx)
            && m.line < editor.file.lines.len()
        {
            editor.cursor_line = m.line;
            editor.cursor_col = m.col;
            let visible = editor.visible_lines;
            if visible > 0 {
                let half = visible / 2;
                editor.scroll = m.line.saturating_sub(half);
            }
        }
    }

    fn prev_or_backward_search(&mut self, editor: &mut EditorState) {
        if self.matches.is_empty() {
            self.matches = find_all(&editor.file.lines, &self.input);
            if !self.matches.is_empty() {
                self.current = self
                    .matches
                    .iter()
                    .rposition(|m| {
                        m.line < editor.cursor_line
                            || (m.line == editor.cursor_line && m.col <= editor.cursor_col)
                    })
                    .unwrap_or(self.matches.len() - 1);
            }
        } else if !self.matches.is_empty() {
            self.current = self.current.saturating_sub(1);
        }
        if !self.matches.is_empty() {
            self.go_to_match(self.current, editor);
        }
    }

    pub fn current_match(&self) -> Option<&Match> {
        if self.matches.is_empty() {
            None
        } else {
            self.matches.get(self.current)
        }
    }

    pub fn match_count(&self) -> (usize, usize) {
        if self.matches.is_empty() {
            (0, 0)
        } else {
            (self.current + 1, self.matches.len())
        }
    }
}

fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
