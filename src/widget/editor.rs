use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};
use unicode_width::UnicodeWidthChar;

use crate::modules::editor::EditorState;

pub fn render_editor(frame: &mut Frame, area: Rect, editor: &EditorState, focused: bool) {
    let title = editor
        .file_path
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

    for (i, l) in editor.lines.iter().enumerate() {
        let gutter = format!("{:>width$} ", i + 1, width = GUTTER_WIDTH);
        lines.push(Line::from(vec![
            Span::styled(gutter, Style::default().fg(Color::DarkGray)),
            Span::raw(l),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines).scroll((editor.scroll as u16, 0)),
        area,
    );

    if focused {
        let gutter_width = GUTTER_WIDTH as u16 + 1;
        let col_pos = editor.lines[editor.cursor_line]
            .chars()
            .take(editor.cursor_col)
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>();
        let x = area.x + gutter_width + col_pos as u16;
        let y = area.y + 1 + editor.cursor_line as u16 - editor.scroll as u16;
        frame.set_cursor_position((x, y));
    }
}
