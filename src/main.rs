mod app;
mod ui;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = app::App::new();
    let result = run(&mut app, terminal);
    ratatui::restore();
    result
}

fn run(app: &mut app::App, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(app, frame))?;
        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.on_key_event(key);
        }
    }
    Ok(())
}
