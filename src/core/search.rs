use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::core::editor::EditorState;
use crate::infra::{Match, find_all};

#[derive(Debug)]
pub struct SearchState {
    pub visible: bool,
    pub input: String,
    pub cursor: usize,
    pub matches: Vec<Match>,
    pub current: usize,
    pub hint: Option<String>,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub replace_mode: bool,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            visible: false,
            input: String::new(),
            cursor: 0,
            matches: Vec::new(),
            current: 0,
            hint: None,
            case_sensitive: false,
            whole_word: false,
            replace_mode: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.input.clear();
            self.cursor = 0;
            self.matches.clear();
            self.current = 0;
            self.hint = None;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, editor: &mut EditorState) {
        match key.code {
            KeyCode::Esc => {
                self.visible = false;
            }
            KeyCode::Enter | KeyCode::Char('\n') => {
                self.hint = None;
                tracing::debug!(
                    "search Enter: input={:?} matches={}",
                    self.input,
                    self.matches.len()
                );
                if self.matches.is_empty() {
                    self.matches = find_all(&editor.document.file.rope, &self.input);
                }
                if !self.matches.is_empty() {
                    let cursor_byte_in_line = editor.cursor.byte
                        - editor.document.file.rope.line_to_byte(editor.cursor.line);
                    self.current = self
                        .matches
                        .iter()
                        .position(|m| {
                            m.line > editor.cursor.line
                                || (m.line == editor.cursor.line && m.col >= cursor_byte_in_line)
                        })
                        .unwrap_or(0);
                    self.go_to_match(self.current, editor);
                }
            }
            KeyCode::Char('n') | KeyCode::Char('N')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.hint = None;
                tracing::debug!(
                    "search Alt+N: current={} total={}",
                    self.current,
                    self.matches.len()
                );
                if !self.matches.is_empty() {
                    if self.current + 1 < self.matches.len() {
                        self.current += 1;
                        self.go_to_match(self.current, editor);
                    } else {
                        self.hint = Some("已到结尾".to_string());
                    }
                }
            }
            KeyCode::Char('p') | KeyCode::Char('P')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.hint = None;
                tracing::debug!(
                    "search Alt+P: current={} total={}",
                    self.current,
                    self.matches.len()
                );
                if self.matches.is_empty() {
                    self.matches = find_all(&editor.document.file.rope, &self.input);
                    if !self.matches.is_empty() {
                        let cursor_byte_in_line = editor.cursor.byte
                            - editor.document.file.rope.line_to_byte(editor.cursor.line);
                        self.current = self
                            .matches
                            .iter()
                            .rposition(|m| {
                                m.line < editor.cursor.line
                                    || (m.line == editor.cursor.line
                                        && m.col <= cursor_byte_in_line)
                            })
                            .unwrap_or(self.matches.len() - 1);
                        self.go_to_match(self.current, editor);
                    }
                } else if !self.matches.is_empty() {
                    if self.current > 0 {
                        self.current -= 1;
                        self.go_to_match(self.current, editor);
                    } else {
                        self.hint = Some("已到开头".to_string());
                    }
                }
            }
            KeyCode::Char('c') | KeyCode::Char('C')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.case_sensitive = !self.case_sensitive;
                self.matches.clear();
            }
            KeyCode::Char('w') | KeyCode::Char('W')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.whole_word = !self.whole_word;
                self.matches.clear();
            }
            KeyCode::Char('r') | KeyCode::Char('R')
                if key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.replace_mode = !self.replace_mode;
            }
            KeyCode::Char(c) => {
                self.hint = None;
                let byte_idx = self
                    .input
                    .char_indices()
                    .nth(self.cursor)
                    .map(|(i, _)| i)
                    .unwrap_or(self.input.len());
                self.input.insert(byte_idx, c);
                self.cursor += 1;
                self.matches.clear();
            }
            KeyCode::Backspace => {
                self.hint = None;
                if self.cursor > 0 {
                    let byte_idx = self
                        .input
                        .char_indices()
                        .nth(self.cursor - 1)
                        .map(|(i, _)| i)
                        .unwrap_or(self.input.len());
                    self.input.remove(byte_idx);
                    self.cursor -= 1;
                    self.matches.clear();
                }
            }
            KeyCode::Left => {
                self.hint = None;
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                self.hint = None;
                if self.cursor < self.input.chars().count() {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => {
                self.hint = None;
                self.cursor = 0;
            }
            KeyCode::End => {
                self.hint = None;
                self.cursor = self.input.chars().count();
            }
            _ => {}
        }
    }

    fn go_to_match(&self, idx: usize, editor: &mut EditorState) {
        if let Some(m) = self.matches.get(idx)
            && m.line < editor.document.file.rope.len_lines()
        {
            editor.cursor.line = m.line;
            editor.cursor.byte =
                editor.document.file.rope.line_to_byte(m.line) + m.col;
            let visible = editor.visible_lines;
            if visible > 0 {
                let half = visible / 2;
                let vrow = editor.cursor.visual_row(&editor.document, editor.available_cols);
                editor.scroll = vrow.saturating_sub(half);
            }
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
