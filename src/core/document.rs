use std::path::Path;

use ropey::RopeSlice;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::core::file::File;

#[derive(Debug)]
pub struct Document {
    pub file: File,
}

impl Document {
    pub fn new() -> Self {
        Self { file: File::new() }
    }

    pub fn load_file(&mut self, path: &Path) {
        tracing::debug!("Document::load_file: {}", path.display());
        self.file = File::open(path);
    }

    pub fn segments_for_line(&self, line: usize, max_cols: usize) -> Vec<(usize, usize)> {
        let line_slice = self.file.rope.line(line);
        let owned;
        let s = if let Some(s) = line_slice.as_str() {
            s
        } else {
            owned = line_slice.to_string();
            &owned
        };
        if max_cols == 0 {
            return vec![(0, s.len())];
        }
        let content = s.strip_suffix('\n').unwrap_or(s);
        if content.width() <= max_cols {
            return vec![(0, s.len())];
        }
        let mut segments = Vec::new();
        let mut seg_start = 0;
        let mut col = 0;
        for (byte_idx, c) in content.char_indices() {
            let cw = c.width().unwrap_or(1);
            if col + cw > max_cols && col > 0 {
                segments.push((seg_start, byte_idx));
                seg_start = byte_idx;
                col = 0;
            }
            col += cw;
        }
        segments.push((seg_start, s.len()));
        tracing::trace!(
            "segments_for_line: line={} max_cols={} s.len={} content.width={} segments={:?}",
            line,
            max_cols,
            s.len(),
            content.width(),
            segments,
        );
        segments
    }

    pub fn line_visual_count(&self, line: usize, max_cols: usize) -> usize {
        self.segments_for_line(line, max_cols).len()
    }

    pub fn len_lines(&self) -> usize {
        self.file.rope.len_lines()
    }

    pub fn line_to_byte(&self, line: usize) -> usize {
        self.file.rope.line_to_byte(line)
    }

    pub fn line(&self, line: usize) -> RopeSlice<'_> {
        self.file.rope.line(line)
    }
}
