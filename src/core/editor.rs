use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::Path;

use crate::core::cursor::Cursor;
use crate::core::document::Document;

#[derive(Debug)]
pub struct EditorState {
    pub document: Document,
    pub cursor: Cursor,
    pub scroll: usize,
    pub visible_lines: usize,
    pub available_cols: usize,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            document: Document::new(),
            cursor: Cursor::new(),
            scroll: 0,
            visible_lines: 0,
            available_cols: 80,
        }
    }

    pub fn load_file(&mut self, path: &Path) {
        tracing::debug!("load file: {}", path.display());
        self.document.load_file(path);
        self.cursor.line = 0;
        self.cursor.byte = 0;
        self.scroll = 0;
    }

    // --- Edit operations (coordinate document + cursor) ---

    pub fn insert_char(&mut self, c: char) {
        let byte = self.cursor.byte;
        let char_idx = self.document.file.rope.byte_to_char(byte);
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        tracing::debug!(
            "insert_char: c='{c}' cursor_byte={} char_idx={} cursor_line={}",
            byte,
            char_idx,
            self.cursor.line,
        );
        self.document.file.rope.insert(char_idx, s);
        self.cursor.byte += c.len_utf8();
        self.document.file.dirty = true;
    }

    pub fn insert_newline(&mut self) {
        let byte = self.cursor.byte;
        let char_idx = self.document.file.rope.byte_to_char(byte);
        let old_line_count = self.document.file.rope.len_lines();
        tracing::debug!(
            "insert_newline: cursor_byte={} char_idx={} cursor_line={} old_lines={}",
            byte,
            char_idx,
            self.cursor.line,
            old_line_count,
        );
        self.document.file.rope.insert(char_idx, "\n");
        self.cursor.line += 1;
        self.cursor.byte += 1;
        tracing::debug!(
            "insert_newline: after -> cursor_byte={} cursor_line={} new_lines={}",
            self.cursor.byte,
            self.cursor.line,
            self.document.file.rope.len_lines(),
        );
        self.document.file.dirty = true;
    }

    pub fn delete_char(&mut self) {
        if self.cursor.byte == 0 {
            tracing::debug!("delete_char: at start of rope, no-op");
            return;
        }
        let byte = self.cursor.byte;
        let line = self.cursor.line;
        tracing::debug!("delete_char: cursor_byte={} cursor_line={}", byte, line);
        let line_start = self.document.line_to_byte(line);
        if byte > line_start {
            let line_slice = self.document.line(line);
            let byte_in_line = byte - line_start;
            let len = if let Some(line_str) = line_slice.as_str() {
                let mut gc =
                    unicode_segmentation::GraphemeCursor::new(byte_in_line, line_str.len(), true);
                match gc.prev_boundary(line_str, 0) {
                    Ok(Some(prev)) => prev,
                    _ => byte_in_line.saturating_sub(1),
                }
            } else {
                byte_in_line.saturating_sub(1)
            };
            let start_char = self.document.file.rope.byte_to_char(line_start + len);
            let end_char = self.document.file.rope.byte_to_char(byte);
            self.document.file.rope.remove(start_char..end_char);
            self.cursor.byte = line_start + len;
            self.document.file.dirty = true;
        } else {
            let start_char = self.document.file.rope.byte_to_char(byte - 1);
            let end_char = self.document.file.rope.byte_to_char(byte);
            self.document.file.rope.remove(start_char..end_char);
            self.cursor.line -= 1;
            self.cursor.byte -= 1;
            self.document.file.dirty = true;
        }
    }

    pub fn adjust_scroll(&mut self, visible_lines: usize) {
        if visible_lines == 0 {
            return;
        }
        let vrow = self.cursor.visual_row(&self.document, self.available_cols);
        if vrow < self.scroll {
            self.scroll = vrow;
        } else if vrow >= self.scroll + visible_lines {
            self.scroll = vrow - visible_lines + 1;
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
                self.document
                    .file
                    .path
                    .as_ref()
                    .map(|p| p.display().to_string())
            );
            match self.document.file.save() {
                Ok(()) => {
                    self.document.file.dirty = false;
                    tracing::debug!("save succeeded");
                }
                Err(e) => tracing::warn!("save failed: {e}"),
            }
            return;
        }
        match key.code {
            KeyCode::Up => self.cursor.move_up(&self.document, self.available_cols),
            KeyCode::Down => self.cursor.move_down(&self.document, self.available_cols),
            KeyCode::Left => self.cursor.move_left(&self.document, self.available_cols),
            KeyCode::Right => self.cursor.move_right(&self.document, self.available_cols),
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Char(c) => self.insert_char(c),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Tests: uses Document, Cursor, and EditorState directly
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use ropey::Rope;

    use crate::core::cursor::Cursor;
    use crate::core::document::Document;
    use crate::core::file::File;

    use super::EditorState;

    // ---- helpers ---------------------------------------------------------

    fn make_editor(text: &str, cols: usize) -> EditorState {
        EditorState {
            document: Document {
                file: File {
                    path: None,
                    rope: Rope::from_str(text),
                    dirty: false,
                },
            },
            cursor: Cursor { line: 0, byte: 0 },
            scroll: 0,
            visible_lines: 20,
            available_cols: cols,
        }
    }

    fn rope_str(editor: &EditorState) -> String {
        editor.document.file.rope.to_string()
    }

    // ---- segments --------------------------------------------------------

    fn segs(editor: &EditorState, line: usize) -> Vec<(usize, usize)> {
        editor.document.segments_for_line(line, editor.available_cols)
    }

    #[test]
    fn segments_short_line() {
        let e = make_editor("hello", 80);
        assert_eq!(segs(&e, 0), vec![(0, 5)]);
    }

    #[test]
    fn segments_exact_width() {
        let e = make_editor("ABCDE", 5);
        assert_eq!(segs(&e, 0), vec![(0, 5)]);
    }

    #[test]
    fn segments_wrap() {
        let e = make_editor("ABCDEFGHIJ", 5);
        assert_eq!(segs(&e, 0), vec![(0, 5), (5, 10)]);
    }

    #[test]
    fn segments_trailing_newline() {
        let e = make_editor("ABCDEFGH\n", 5);
        // The segment end includes the \n
        assert_eq!(segs(&e, 0), vec![(0, 5), (5, 9)]);
    }

    #[test]
    fn segments_empty_line() {
        let e = make_editor("", 80);
        assert_eq!(segs(&e, 0), vec![(0, 0)]);
    }

    #[test]
    fn segments_cjk_each_takes_two() {
        let e = make_editor("你好世界", 4);
        // 你(2col) + 好(2col) = 4 exactly, then 世 triggers split at byte 6
        assert_eq!(segs(&e, 0), vec![(0, 6), (6, 12)]);
    }

    #[test]
    fn segments_zero_max_cols() {
        let e = make_editor("hello", 0);
        assert_eq!(segs(&e, 0), vec![(0, 5)]);
    }

    #[test]
    fn segments_multi_line() {
        let e = make_editor("abc\ndefghij", 4);
        assert_eq!(segs(&e, 0), vec![(0, 4)]); // "abc\n"
        // line 1: "defghij" (7 bytes), width=7 > 4, wrap at byte 4 → (0,4),(4,7)
        assert_eq!(segs(&e, 1), vec![(0, 4), (4, 7)]);
    }

    // ---- Cursor::visual_row ----------------------------------------------

    fn vrow(e: &EditorState) -> usize {
        e.cursor.visual_row(&e.document, e.available_cols)
    }

    #[test]
    fn visual_row_first_line_first_seg() {
        let e = make_editor("hello", 80);
        assert_eq!(vrow(&e), 0);
    }

    #[test]
    fn visual_row_second_segment() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 5;
        assert_eq!(vrow(&e), 1);
    }

    #[test]
    fn visual_row_eol_last_segment() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 10; // past last char = end of line
        assert_eq!(vrow(&e), 1); // last segment = (5,10)
    }

    #[test]
    fn visual_row_middle_of_first_seg() {
        let mut e = make_editor("hello world", 80);
        e.cursor.byte = 3;
        assert_eq!(vrow(&e), 0);
    }

    #[test]
    fn visual_row_exact_seg_boundary() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 5; // first byte of second segment
        assert_eq!(vrow(&e), 1);
    }

    // ---- Cursor::visual_col ---------------------------------------------

    fn vcol(e: &EditorState) -> usize {
        e.cursor.visual_col(&e.document, e.available_cols)
    }

    #[test]
    fn visual_col_first_segment() {
        let e = make_editor("ABCDEFGHIJ", 80);
        assert_eq!(vcol(&e), 0); // byte 0
    }

    #[test]
    fn visual_col_first_segment_mid() {
        let mut e = make_editor("ABCDEFGHIJ", 80);
        e.cursor.byte = 3;
        assert_eq!(vcol(&e), 3);
    }

    #[test]
    fn visual_col_second_segment_start() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 5;
        assert_eq!(vcol(&e), 0); // first byte of "FGHIJ"
    }

    #[test]
    fn visual_col_second_segment_mid() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 8; // byte 3 of "FGHIJ" = 'I'
        assert_eq!(vcol(&e), 3);
    }

    #[test]
    fn visual_col_second_segment_end() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 10; // past end = visual col = segment display width
        assert_eq!(vcol(&e), 5);
    }

    #[test]
    fn visual_col_cjk_wrapped() {
        let mut e = make_editor("你好世界", 4);
        // segments: (0,6), (6,12), each CJK char = 2 col
        e.cursor.byte = 9; // byte 3 within seg 1 = "世" start
        // seg 1 display = "世" (wait, display = s[6..12].strip_suffix('\n') = "世"... 
        // no, s[6..12] = "世界"? No, 8 bytes total? 你=3, 好=3, 世=3, 界=3 = 12 bytes.
        // s[6..12] without \n = "世界"? No, bytes 6-8 = "世", 9-11 = "界".
        // Actually 你=0-2, 好=3-5, 世=6-8, 界=9-11
        // s[6..12] = "世界" (6 bytes, 2 chars)
        // But segments_for_line(0, 4) gives (0,6),(6,12) due to CJK wrapping at col 4
        // So seg 1 = s[6..12] = "世界", display = "世界" (no \n)
        // byte_in_seg = 9 - 6 = 3. display[..3] = "世".width() = 2
        assert_eq!(vcol(&e), 2);
    }

    // ---- Cursor::move_up / move_down ------------------------------------

    #[test]
    fn move_up_same_segment() {
        let mut e = make_editor("hello\nworld", 80);
        e.cursor.line = 1;
        e.cursor.byte = 8; // line 1, col 2 (the 'r' in "world")
        e.cursor.move_up(&e.document, e.available_cols);
        assert_eq!(e.cursor.line, 0);
        // preserves visual column 2 within last segment of prev line
        assert_eq!(e.cursor.byte, 2);
    }

    #[test]
    fn move_down_same_segment() {
        let mut e = make_editor("hello\nworld", 80);
        e.cursor.byte = 2; // "hello"@2, visual col = 2
        e.cursor.move_down(&e.document, e.available_cols);
        assert_eq!(e.cursor.line, 1);
        // preserves visual column 2 within first segment of next line
        assert_eq!(e.cursor.byte, 8);
    }

    #[test]
    fn move_up_to_prev_segment_from_last_seg_of_wrapped_line() {
        let mut e = make_editor("bcdef", 3);
        // "bcdef" width=5 > 3, wraps: segments (0,3),(3,5)
        e.cursor.byte = 4; // second segment, col 1 within segment
        e.cursor.move_up(&e.document, e.available_cols);
        // Preserves visual col 1 in previous segment (byte 1)
        assert_eq!(e.cursor.byte, 1);
        assert_eq!(e.cursor.line, 0);
    }

    #[test]
    fn move_up_soft_wrap() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        // segments: (0,5), (5,10)
        e.cursor.byte = 8; // second segment, visual col = 3 within segment
        e.cursor.move_up(&e.document, e.available_cols);
        // Preserves visual col 3 in previous segment (byte 3)
        assert_eq!(e.cursor.byte, 3);
    }

    #[test]
    fn move_down_soft_wrap() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 3; // first segment, visual col = 3
        e.cursor.move_down(&e.document, e.available_cols);
        // Preserves visual col 3 in next segment (byte 8)
        assert_eq!(e.cursor.byte, 8);
    }

    #[test]
    fn move_up_at_top_stays() {
        let mut e = make_editor("hello", 80);
        e.cursor.move_up(&e.document, e.available_cols);
        assert_eq!(e.cursor.line, 0);
        assert_eq!(e.cursor.byte, 0);
    }

    #[test]
    fn move_down_at_bottom_stays() {
        let mut e = make_editor("hello\nworld", 80);
        e.cursor.line = 1;
        e.cursor.byte = 6;
        e.cursor.move_down(&e.document, e.available_cols);
        assert_eq!(e.cursor.line, 1);
        assert_eq!(e.cursor.byte, 6);
    }

    // ---- Cursor::move_left / move_right ----------------------------------

    #[test]
    fn move_left_basic() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 3;
        e.cursor.move_left(&e.document, e.available_cols);
        assert_eq!(e.cursor.byte, 2);
        assert_eq!(e.cursor.line, 0);
    }

    #[test]
    fn move_right_basic() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 2;
        e.cursor.move_right(&e.document, e.available_cols);
        assert_eq!(e.cursor.byte, 3);
        assert_eq!(e.cursor.line, 0);
    }

    #[test]
    fn move_left_at_line_start_goes_up() {
        let mut e = make_editor("hello\nworld", 80);
        e.cursor.line = 1;
        e.cursor.byte = e.document.line_to_byte(1); // start of line 1
        e.cursor.move_left(&e.document, e.available_cols);
        // Goes to end of previous line (before \n)
        assert_eq!(e.cursor.line, 0);
        assert_eq!(e.cursor.byte, 5); // end of "hello" (before \n)
    }

    #[test]
    fn move_right_at_line_end_goes_down() {
        let mut e = make_editor("hello\nworld", 80);
        e.cursor.byte = 5; // "hello" end (before \n)
        e.cursor.move_right(&e.document, e.available_cols);
        assert_eq!(e.cursor.line, 1);
        assert_eq!(e.cursor.byte, e.document.line_to_byte(1)); // start of line 1
    }

    #[test]
    fn move_left_soft_wrap() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 5; // start of second segment
        e.cursor.move_left(&e.document, e.available_cols);
        // Should go to end of first segment = byte 4 (the 'E')
        assert_eq!(e.cursor.byte, 4);
    }

    #[test]
    fn move_right_soft_wrap() {
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 4; // end of first segment
        e.cursor.move_right(&e.document, e.available_cols);
        // Should go to start of second segment = byte 5
        assert_eq!(e.cursor.byte, 5);
    }

    #[test]
    fn move_left_at_rope_start_stays() {
        let mut e = make_editor("hello", 80);
        e.cursor.move_left(&e.document, e.available_cols);
        assert_eq!(e.cursor.byte, 0);
    }

    #[test]
    fn move_right_at_rope_end_past_last_char() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 5; // after last char 'o'
        e.cursor.move_right(&e.document, e.available_cols);
        assert_eq!(e.cursor.byte, 5); // stays
    }

    #[test]
    fn move_right_on_last_line_last_char_eol() {
        let mut e = make_editor("hello", 80);
        // past last char — should stay
        e.cursor.byte = 5;
        e.cursor.move_right(&e.document, e.available_cols);
        assert_eq!(e.cursor.byte, 5);
    }

    // ---- EditorState::insert_char ----------------------------------------

    #[test]
    fn insert_char_basic() {
        let mut e = make_editor("hllo", 80);
        e.cursor.byte = 1;
        e.insert_char('e');
        assert!(e.document.file.dirty);
        assert_eq!(rope_str(&e), "hello");
        assert_eq!(e.cursor.byte, 2);
    }

    #[test]
    fn insert_char_at_end() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 5;
        e.insert_char('!');
        assert_eq!(rope_str(&e), "hello!");
        assert_eq!(e.cursor.byte, 6);
    }

    #[test]
    fn insert_char_cjk() {
        let mut e = make_editor("你好世界", 80);
        // cursor at byte 6 (after "你好")
        e.cursor.byte = 6;
        e.insert_char('啊');
        assert_eq!(rope_str(&e), "你好啊世界");
        assert_eq!(e.cursor.byte, 9); // 6 + 3 bytes for '啊'
    }

    // ---- EditorState::insert_newline -------------------------------------

    #[test]
    fn insert_newline_middle() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 3;
        e.insert_newline();
        assert_eq!(rope_str(&e), "hel\nlo");
        assert_eq!(e.cursor.byte, 4);
        assert_eq!(e.cursor.line, 1);
    }

    #[test]
    fn insert_newline_at_end() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 5;
        e.insert_newline();
        assert_eq!(rope_str(&e), "hello\n");
        assert_eq!(e.cursor.byte, 6);
        assert_eq!(e.cursor.line, 1);
    }

    // ---- EditorState::delete_char ----------------------------------------

    #[test]
    fn delete_char_basic() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 3;
        e.delete_char();
        assert!(e.document.file.dirty);
        assert_eq!(rope_str(&e), "helo");
        assert_eq!(e.cursor.byte, 2);
    }

    #[test]
    fn delete_char_at_start_noop() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 0;
        e.delete_char();
        assert_eq!(rope_str(&e), "hello");
    }

    #[test]
    fn delete_char_merge_lines() {
        let mut e = make_editor("hel\nlo", 80);
        e.cursor.line = 1;
        e.cursor.byte = 4;
        e.delete_char(); // deletes the \n at byte 3
        assert_eq!(rope_str(&e), "hello");
        assert_eq!(e.cursor.line, 0);
        assert_eq!(e.cursor.byte, 3);
    }

    #[test]
    fn delete_char_cjk() {
        let mut e = make_editor("你好世界", 80);
        // 你=0-2, 好=3-5, 世=6-8, 界=9-11
        e.cursor.byte = 9; // after "你好世"
        e.delete_char();
        // prev_boundary from 9 → 6 → removes bytes 6..9 = "世"
        assert_eq!(rope_str(&e), "你好界");
    }

    // ---- EditorState::adjust_scroll --------------------------------------

    #[test]
    fn scroll_follows_cursor() {
        let mut e = make_editor("a\nb\nc\nd\ne\nf\ng\nh\ni\nj\nk\nl\nm\nn\no\np", 80);
        e.visible_lines = 5;
        e.cursor.line = 7;
        e.cursor.byte = e.document.line_to_byte(7);
        e.adjust_scroll(5);
        assert!(e.scroll <= e.cursor.line);
        assert!(e.scroll + 5 > e.cursor.line);
    }

    #[test]
    fn scroll_does_not_go_negative() {
        let mut e = make_editor("abc\ndef", 80);
        e.cursor.line = 0;
        e.cursor.byte = 0;
        e.adjust_scroll(5);
        assert_eq!(e.scroll, 0);
    }

    // ---- Cursor visual_row + segments: boundary edge cases ---------------

    #[test]
    fn visual_row_cursor_at_eol_last_segment() {
        // This is the bug scenario: cursor at end of line should map
        // to the last segment's visual row, NOT segment 0.
        let mut e = make_editor("ABCDEFGHIJ", 5);
        e.cursor.byte = 10; // past all content
        assert_eq!(vrow(&e), 1); // last segment index = 1
    }

    #[test]
    fn visual_row_empty_line() {
        let e = make_editor("", 80);
        assert_eq!(vrow(&e), 0);
    }

    #[test]
    fn insert_delete_roundtrip() {
        let mut e = make_editor("hello", 80);
        e.cursor.byte = 5;
        e.insert_char('!');
        // cursor is now at byte 6 (past the '!')
        e.delete_char();
        assert_eq!(rope_str(&e), "hello");
    }
}

