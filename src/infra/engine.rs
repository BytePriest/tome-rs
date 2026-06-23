use ropey::Rope;

#[derive(Debug, Clone, Copy)]
pub struct Match {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

pub fn find_all(rope: &Rope, pattern: &str) -> Vec<Match> {
    if pattern.is_empty() {
        return Vec::new();
    }
    let byte_len = pattern.len();
    let mut results = Vec::new();
    for (line_idx, line_slice) in rope.lines().enumerate() {
        let owned;
        let line_str = if let Some(s) = line_slice.as_str() {
            s
        } else {
            owned = line_slice.to_string();
            &owned
        };
        let mut byte_offset = 0;
        while let Some(byte_col) = line_str[byte_offset..].find(pattern) {
            let byte_start = byte_offset + byte_col;
            results.push(Match {
                line: line_idx,
                col: byte_start,
                len: byte_len,
            });
            byte_offset = byte_start + pattern.len();
        }
    }
    results
}
