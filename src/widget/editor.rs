use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthChar;

use crate::modules::editor::EditorState;
use crate::search::engine::Match;

pub fn render_editor(
    frame: &mut Frame,
    area: Rect,
    editor: &EditorState,
    focused: bool,
    search_match: Option<&Match>,
    search_pattern: &str,
) {
    let title = editor
        .file
        .path
        .as_ref()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "untitled".to_string());

    let title_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    const GUTTER_WIDTH: usize = 5;

    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        format!(" {} ", title),
        title_style,
    ))];

    let highlight_style = Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);

    for (i, l) in editor.file.lines.iter().enumerate() {
        let gutter = format!("{:>width$} ", i + 1, width = GUTTER_WIDTH);
        let gutter_span = Span::styled(gutter, Style::default().fg(Color::DarkGray));

        let line_spans = if let Some(m) = search_match {
            if m.line == i && !search_pattern.is_empty() {
                let pattern_char_len = m.len;
                let before_byte = char_idx_to_byte_idx(l, m.col);
                let after_byte = char_idx_to_byte_idx(l, m.col + pattern_char_len);
                vec![
                    gutter_span,
                    Span::raw(&l[..before_byte]),
                    Span::styled(&l[before_byte..after_byte], highlight_style),
                    Span::raw(&l[after_byte..]),
                ]
            } else {
                vec![gutter_span, Span::raw(l)]
            }
        } else {
            vec![gutter_span, Span::raw(l)]
        };

        lines.push(Line::from(line_spans));
    }

    frame.render_widget(
        Paragraph::new(lines).scroll((editor.scroll as u16, 0)),
        area,
    );

    if focused {
        let gutter_width = GUTTER_WIDTH as u16 + 1;
        let col_pos = editor.file.lines[editor.cursor_line]
            .chars()
            .take(editor.cursor_col)
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>();
        let x = area.x + gutter_width + col_pos as u16;
        let y = area.y + 1 + editor.cursor_line as u16 - editor.scroll as u16;
        frame.set_cursor_position((x, y));
    }
}

fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}
