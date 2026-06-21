#[derive(Debug, Clone, Copy)]
pub struct Match {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

pub fn find_all(lines: &[String], pattern: &str) -> Vec<Match> {
    if pattern.is_empty() {
        return Vec::new();
    }
    let char_len = pattern.chars().count();
    let mut results = Vec::new();
    for (line_idx, line) in lines.iter().enumerate() {
        let mut byte_offset = 0;
        while let Some(byte_col) = line[byte_offset..].find(pattern) {
            let byte_start = byte_offset + byte_col;
            let char_col = line[..byte_start].chars().count();
            results.push(Match {
                line: line_idx,
                col: char_col,
                len: char_len,
            });
            byte_offset = byte_start + pattern.len();
        }
    }
    results
}
