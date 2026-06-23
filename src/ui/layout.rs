use ratatui::{
    Frame,
    layout::{Constraint, Layout},
};

use crate::app::App;
use crate::app::Focus;
use crate::ui::{divider, editor, explorer, search};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let [explorer_area, divider_area, editor_area] = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(area);

    let (editor_content, search_bar) = if app.search.visible {
        let [content, bar] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(editor_area);
        (content, Some(bar))
    } else {
        (editor_area, None)
    };

    let visible_lines = editor_content.height.saturating_sub(1) as usize;
    app.editor.visible_lines = visible_lines;
    app.editor.available_cols = editor_content.width.saturating_sub(6) as usize;
    tracing::trace!(
        "layout: editor_area.width={} available_cols={} visible_lines={}",
        editor_content.width,
        app.editor.available_cols,
        visible_lines,
    );
    app.editor.adjust_scroll(visible_lines);

    explorer::render_explorer(
        frame,
        explorer_area,
        &app.explorer,
        matches!(app.focus, Focus::Explorer),
    );
    divider::render_divider(frame, divider_area);
    editor::render_editor(
        frame,
        editor_content,
        &app.editor,
        matches!(app.focus, Focus::Editor),
        app.search.current_match(),
        &app.search.input,
    );

    if let Some(area) = search_bar {
        search::render_search(frame, area, &app.search);
    }
}
