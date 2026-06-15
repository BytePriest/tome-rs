use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::app::App;
use crate::app::Focus;
use crate::widget;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let [explorer_area, divider_area, editor_area] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    let visible_lines = editor_area.height.saturating_sub(1) as usize;
    app.editor.adjust_scroll(visible_lines);

    widget::explorer::render_explorer(
        frame,
        explorer_area,
        &app.explorer,
        matches!(app.focus, Focus::Explorer),
    );
    widget::divider::render_divider(frame, divider_area);
    widget::editor::render_editor(
        frame,
        editor_area,
        &app.editor,
        matches!(app.focus, Focus::Editor),
    );
}
