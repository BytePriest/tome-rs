use unicode_segmentation::GraphemeCursor;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::core::document::Document;

#[derive(Debug)]
pub struct Cursor {
    pub line: usize,
    pub byte: usize,
}

/// Returns the display text for a segment (stripped of trailing `\n`).
fn segment_display(doc: &Document, line: usize, seg_start: usize, seg_end: usize) -> &str {
    let s = doc.line(line).as_str().unwrap_or("");
    let raw = &s[seg_start..seg_end];
    raw.strip_suffix('\n').unwrap_or(raw)
}

/// Returns the byte offset within `display` where the display column
/// first reaches `target_col`.  If the text is shorter, returns `display.len()`.
fn byte_at_col(display: &str, target_col: usize) -> usize {
    let mut col = 0;
    for (i, c) in display.char_indices() {
        if col >= target_col {
            return i;
        }
        col += c.width().unwrap_or(1);
    }
    display.len()
}

impl Cursor {
    pub fn new() -> Self {
        Self { line: 0, byte: 0 }
    }

    pub fn byte_in_line(&self, doc: &Document) -> usize {
        self.byte - doc.line_to_byte(self.line)
    }

    /// Visual column of the cursor within its current segment (0-indexed display column).
    /// Returns the column where `set_cursor_position` should place the cursor.
    pub fn visual_col(&self, doc: &Document, max_cols: usize) -> usize {
        let segments = doc.segments_for_line(self.line, max_cols);
        let byte_in_line = self.byte_in_line(doc);
        let seg_idx = segments
            .iter()
            .rposition(|&(s, e)| byte_in_line >= s && byte_in_line <= e)
            .unwrap_or(0);
        let (seg_start, _seg_end) = segments[seg_idx];
        let display = segment_display(doc, self.line, seg_start, _seg_end);
        let byte_in_seg = byte_in_line.saturating_sub(seg_start);
        // display is already a segment-local slice (0-based).
        let len = byte_in_seg.min(display.len());
        display[..len].width()
    }

    pub fn visual_row(&self, doc: &Document, max_cols: usize) -> usize {
        let mut vrow = 0;
        for l in 0..self.line {
            vrow += doc.line_visual_count(l, max_cols);
        }
        let segments = doc.segments_for_line(self.line, max_cols);
        let byte_in_line = self.byte - doc.line_to_byte(self.line);
        let seg_idx = segments
            .iter()
            .rposition(|&(s, e)| byte_in_line >= s && byte_in_line <= e);
        vrow + seg_idx.unwrap_or(0)
    }

    pub fn move_up(&mut self, doc: &Document, max_cols: usize) {
        let segments = doc.segments_for_line(self.line, max_cols);
        let line_start = doc.line_to_byte(self.line);
        let byte_in_line = self.byte - line_start;

        // Find current segment using `<=` so EOL positions are included.
        let seg_idx = segments
            .iter()
            .rposition(|&(s, e)| byte_in_line >= s && byte_in_line <= e);

        let cur_visual_col = seg_idx.map_or(0, |idx| {
            let (s, e) = segments[idx];
            let display = segment_display(doc, self.line, s, e);
            let byte_in_seg = (byte_in_line - s).min(display.len());
            display[..byte_in_seg].width()
        });

        tracing::debug!(
            "move_up: line={} byte_in_line={} segments={:?} seg_idx={:?} visual_col={}",
            self.line,
            byte_in_line,
            segments,
            seg_idx,
            cur_visual_col,
        );

        // Same line, previous segment – keep the visual column.
        if let Some(idx) = seg_idx
            && idx > 0
        {
            let (prev_s, prev_e) = segments[idx - 1];
            let display = segment_display(doc, self.line, prev_s, prev_e);
            let target_byte = byte_at_col(display, cur_visual_col);
            self.byte = line_start + prev_s + target_byte;
            tracing::debug!("move_up: -> same line segment {} byte={}", idx - 1, self.byte);
            return;
        }

        // Previous line, last segment.
        if self.line > 0 {
            self.line -= 1;
            let prev_segments = doc.segments_for_line(self.line, max_cols);
            let last = prev_segments.last().unwrap();
            let display = segment_display(doc, self.line, last.0, last.1);
            let target_byte = byte_at_col(display, cur_visual_col);
            self.byte = doc.line_to_byte(self.line) + last.0 + target_byte;
            tracing::debug!(
                "move_up: -> prev line {} segment ({},{}) byte={}",
                self.line,
                last.0,
                last.1,
                self.byte,
            );
        } else {
            tracing::debug!("move_up: at first line, no move");
        }
    }

    pub fn move_down(&mut self, doc: &Document, max_cols: usize) {
        let segments = doc.segments_for_line(self.line, max_cols);
        let line_start = doc.line_to_byte(self.line);
        let byte_in_line = self.byte - line_start;

        let seg_idx = segments
            .iter()
            .rposition(|&(s, e)| byte_in_line >= s && byte_in_line <= e);

        let cur_visual_col = seg_idx.map_or(0, |idx| {
            let (s, e) = segments[idx];
            let display = segment_display(doc, self.line, s, e);
            let byte_in_seg = (byte_in_line - s).min(display.len());
            display[..byte_in_seg].width()
        });

        tracing::debug!(
            "move_down: line={} byte_in_line={} segments={:?} seg_idx={:?} visual_col={}",
            self.line,
            byte_in_line,
            segments,
            seg_idx,
            cur_visual_col,
        );

        // Same line, next segment – keep the visual column.
        if let Some(idx) = seg_idx
            && idx + 1 < segments.len()
        {
            let (next_s, next_e) = segments[idx + 1];
            let display = segment_display(doc, self.line, next_s, next_e);
            let target_byte = byte_at_col(display, cur_visual_col);
            self.byte = line_start + next_s + target_byte;
            tracing::debug!("move_down: -> same line segment {} byte={}", idx + 1, self.byte);
            return;
        }

        // Next line, first segment.
        if self.line + 1 < doc.len_lines() {
            self.line += 1;
            let next_segments = doc.segments_for_line(self.line, max_cols);
            let first = next_segments.first().unwrap();
            let display = segment_display(doc, self.line, first.0, first.1);
            let target_byte = byte_at_col(display, cur_visual_col);
            self.byte = doc.line_to_byte(self.line) + first.0 + target_byte;
            tracing::debug!(
                "move_down: -> next line {} segment ({},{}) byte={}",
                self.line,
                first.0,
                first.1,
                self.byte,
            );
        } else {
            tracing::debug!("move_down: at last line, no move");
        }
    }

    pub fn move_left(&mut self, doc: &Document, _max_cols: usize) {
        let line_start = doc.line_to_byte(self.line);
        let byte_in_line = self.byte - line_start;

        if self.byte > line_start {
            let line = doc.line(self.line);
            if let Some(line_str) = line.as_str() {
                let mut gc = GraphemeCursor::new(byte_in_line, line_str.len(), true);
                if let Ok(Some(prev)) = gc.prev_boundary(line_str, 0) {
                    self.byte = line_start + prev;
                    return;
                }
            }
            self.byte = line_start + byte_in_line.saturating_sub(1);
        } else if self.line > 0 {
            // Jump to end of previous line (before its trailing \n).
            self.line -= 1;
            let line = doc.line(self.line);
            let s = line.as_str().unwrap_or("");
            let eol = s.strip_suffix('\n').unwrap_or(s).len();
            self.byte = doc.line_to_byte(self.line) + eol;
        }
    }

    pub fn move_right(&mut self, doc: &Document, _max_cols: usize) {
        let line_start = doc.line_to_byte(self.line);
        let line = doc.line(self.line);
        let line_len = line.len_bytes();
        let byte_in_line = self.byte - line_start;
        let len_lines = doc.len_lines();

        if byte_in_line < line_len {
            let next = if let Some(line_str) = line.as_str() {
                let mut gc = GraphemeCursor::new(byte_in_line, line_str.len(), true);
                match gc.next_boundary(line_str, 0) {
                    Ok(Some(n)) => n,
                    _ => byte_in_line + 1,
                }
            } else {
                byte_in_line + 1
            };
            tracing::debug!(
                "move_right: cursor_line={} byte_in_line={} line_len={} next={} len_lines={}",
                self.line,
                byte_in_line,
                line_len,
                next,
                len_lines,
            );
            if next >= line_len {
                if self.line + 1 < len_lines {
                    self.line += 1;
                    self.byte = doc.line_to_byte(self.line);
                } else {
                    self.byte = line_start + line_len;
                }
            } else {
                self.byte = line_start + next;
            }
            tracing::debug!(
                "move_right: after -> cursor_line={} cursor_byte={}",
                self.line,
                self.byte,
            );
        } else if self.line + 1 < len_lines {
            self.line += 1;
            self.byte = doc.line_to_byte(self.line);
        }
    }
}
