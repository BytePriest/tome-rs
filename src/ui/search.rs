use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthStr;

use crate::core::search::SearchState;

fn toggle_button(label: &str, shortcut: &str, active: bool) -> Span<'static> {
    if active {
        Span::styled(
            format!(" {label} {shortcut} "),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!(" {label} {shortcut} "),
            Style::default().fg(Color::DarkGray),
        )
    }
}

pub fn render_search(frame: &mut Frame, area: Rect, search: &SearchState) {
    if !search.visible {
        return;
    }

    let (current, total) = search.match_count();
    let hint = search.hint.as_deref();
    let suffix = if let Some(hint) = hint {
        Span::styled(format!(" {hint}"), Style::default().fg(Color::Red))
    } else if total > 0 {
        Span::styled(
            format!(" {current}/{total}"),
            Style::default().fg(Color::DarkGray),
        )
    } else if !search.input.is_empty() {
        Span::styled(
            " 0 matches".to_string(),
            Style::default().fg(Color::DarkGray),
        )
    } else {
        Span::raw("")
    };

    let content = Line::from(vec![
        Span::raw(format!(" Find: {}", search.input)),
        toggle_button("区分大小写", "alt+c", search.case_sensitive),
        toggle_button("精确匹配", "alt+w", search.whole_word),
        toggle_button("替换", "alt+r", search.replace_mode),
        suffix,
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
