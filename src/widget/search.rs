use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

use crate::modules::search::SearchState;

pub fn render_search(frame: &mut Frame, area: Rect, search: &SearchState) {
    if !search.visible {
        return;
    }

    let (current, total) = search.match_count();
    let counter = if total > 0 {
        format!(" {}/{}", current, total)
    } else if !search.input.is_empty() {
        " 0 matches".to_string()
    } else {
        String::new()
    };

    let content = Line::from(vec![
        Span::raw(format!(" Find: {}", search.input)),
        Span::styled(counter, Style::default().fg(Color::DarkGray)),
    ]);

    frame.render_widget(
        Paragraph::new(content).style(
            Style::default()
                .bg(Color::Rgb(0x2a, 0x2a, 0x2a))
                .fg(Color::White),
        ),
        area,
    );

    let text_before = &search.input[..search
        .input
        .char_indices()
        .nth(search.cursor)
        .map(|(i, _)| i)
        .unwrap_or(search.input.len())];
    let display_width = " Find: ".width() + text_before.width();
    let cursor_x = area.x + display_width as u16;
    let cursor_y = area.y;
    frame.set_cursor_position((cursor_x, cursor_y));
}
