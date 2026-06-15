use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
};

pub fn render_divider(frame: &mut Frame, area: Rect) {
    let lines: Vec<Line> = (0..area.height).map(|_| Line::from("│")).collect();
    frame.render_widget(
        Paragraph::new(lines).style(Style::default().fg(Color::DarkGray)),
        area,
    );
}
