use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::modules::explorer::ExplorerState;

pub fn render_explorer(frame: &mut Frame, area: Rect, explorer: &ExplorerState, _focused: bool) {
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
