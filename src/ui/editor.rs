use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::core::editor::EditorState;
use crate::infra::Match;

pub fn render_editor(
    frame: &mut Frame,
    area: Rect,
    editor: &EditorState,
    focused: bool,
    search_match: Option<&Match>,
    search_pattern: &str,
) {
    let title = editor
        .document
        .file
        .path
        .as_ref()
        .and_then(|p| {
            std::env::current_dir()
                .ok()
                .and_then(|cwd| p.strip_prefix(&cwd).ok())
                .or(Some(p))
                .map(|p| p.display().to_string())
        })
        .unwrap_or_else(|| "untitled".to_string());
    let title = if editor.document.file.dirty {
        format!("{title}*")
    } else {
        title
    };

    let title_style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    };

    // visual_row: 0-based index of visual lines within the document, excluding title bar.
    // The title bar occupies the first terminal row (y = area.y), document starts at y + 1.
    const GUTTER_WIDTH: usize = 5;

    let mut rendered_lines: Vec<Line> = vec![Line::from(Span::styled(
        format!(" {} ", title),
        title_style,
    ))];

    let highlight_style = Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD);

    let max_visual = editor.visible_lines;
    let cursor_vrow = editor.cursor.visual_row(&editor.document, editor.available_cols);
    let cursor_vcol = editor.cursor.visual_col(&editor.document, editor.available_cols);
    let mut visual_row = 0usize;

    for (logical_line, line_slice) in editor.document.file.rope.lines().enumerate() {
        let line_owned = line_slice.to_string();
        let segments = editor
            .document
            .segments_for_line(logical_line, editor.available_cols);

        for (seg_idx, &(seg_start, seg_end)) in segments.iter().enumerate() {
            let segment_content = &line_owned[seg_start..seg_end];
            let display_text = segment_content.strip_suffix('\n').unwrap_or(segment_content);

            let gutter = if seg_idx == 0 {
                format!("{:>width$} ", logical_line + 1, width = GUTTER_WIDTH)
            } else {
                // must match the width of the initial gutter: GUTTER_WIDTH digits + space
                format!("{:>width$} ", "", width = GUTTER_WIDTH)
            };
            let gutter_span = Span::styled(gutter, Style::default().fg(Color::DarkGray));

            if visual_row >= editor.scroll && rendered_lines.len() - 1 < max_visual {
                let line_spans = if let Some(m) = search_match {
                    if m.line == logical_line
                        && !search_pattern.is_empty()
                        && m.col >= seg_start
                        && m.col < seg_end
                    {
                        let match_end = m.col + m.len;
                        let before = display_text[..m.col - seg_start].to_string();
                        let matched =
                            display_text[m.col - seg_start..match_end - seg_start].to_string();
                        let after = display_text[match_end - seg_start..].to_string();
                        vec![
                            gutter_span,
                            Span::raw(before),
                            Span::styled(matched, highlight_style),
                            Span::raw(after),
                        ]
                    } else {
                        vec![gutter_span, Span::raw(display_text.to_string())]
                    }
                } else {
                    vec![gutter_span, Span::raw(display_text.to_string())]
                };
                rendered_lines.push(Line::from(line_spans));
            }

            visual_row += 1;
        }
        if visual_row > editor.scroll + max_visual + 1 {
            break;
        }
    }

    frame.render_widget(Paragraph::new(rendered_lines), area);

    if focused {
        let relative_row = cursor_vrow.saturating_sub(editor.scroll);
        if relative_row < max_visual && cursor_vrow >= editor.scroll {
            // GUTTER_WIDTH + 1 space after line number
            let gutter_visual_width = GUTTER_WIDTH as u16 + 1;
            let x = area.x + gutter_visual_width + cursor_vcol as u16;
            let y = area.y + 1 + relative_row as u16;
            frame.set_cursor_position((x, y));
        }
    }
}
