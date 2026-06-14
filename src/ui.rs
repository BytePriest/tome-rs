use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, EditorState, ExplorerState, Focus};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let [explorer_area, divider_area, editor_area] = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    let visible_lines = editor_area.height.saturating_sub(1) as usize;
    app.editor.adjust_scroll(visible_lines);

    render_explorer(
        frame,
        explorer_area,
        &app.explorer,
        matches!(app.focus, Focus::Explorer),
    );
    render_divider(frame, divider_area);
    render_editor(
        frame,
        editor_area,
        &app.editor,
        matches!(app.focus, Focus::Editor),
    );
}

fn render_divider(frame: &mut Frame, area: Rect) {
    let lines: Vec<Line> = (0..area.height).map(|_| Line::from("│")).collect();
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}

fn render_explorer(frame: &mut Frame, area: Rect, explorer: &ExplorerState, _focused: bool) {
    let mut lines: Vec<Line> = Vec::new();

    for (i, entry) in explorer.visible_entries.iter().enumerate() {
        let is_selected = i == explorer.selected;
        let indent = "  ".repeat(entry.depth);

        let icon = if entry.is_dir {
            if entry.expanded { "▼" } else { "▶" }
        } else {
            " "
        };

        let display = if entry.is_dir {
            format!("{}{} {}/", indent, icon, entry.name)
        } else {
            format!("{}{} {}", indent, icon, entry.name)
        };

        let mut style = Style::default();
        if entry.is_dir {
            style = style.fg(Color::Cyan);
        }
        if is_selected {
            style = style.bg(Color::DarkGray).add_modifier(Modifier::BOLD);
        }

        lines.push(Line::from(Span::styled(display, style)));
    }

    frame.render_widget(Paragraph::new(lines), area);
}

fn render_editor(frame: &mut Frame, area: Rect, editor: &EditorState, focused: bool) {
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

    let mut lines: Vec<Line> = vec![Line::from(Span::styled(
        format!(" {} ", title),
        title_style,
    ))];

    for l in &editor.lines {
        lines.push(Line::from(Span::raw(l)));
    }

    frame.render_widget(
        Paragraph::new(lines).scroll((editor.scroll as u16, 0)),
        area,
    );

    if focused {
        let x = area.x + editor.cursor_col as u16;
        let y = area.y + 1 + editor.cursor_line as u16 - editor.scroll as u16;
        frame.set_cursor_position((x, y));
    }
}
