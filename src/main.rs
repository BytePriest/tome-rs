mod app;
mod modules;
mod ui;
mod widget;

use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let log_file = std::fs::File::create("debug.log")?;
    tracing_subscriber::fmt()
        .with_writer(std::sync::Mutex::new(log_file))
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let _ = std::process::Command::new("stty").args(["-ixon"]).status();
    let terminal = ratatui::init();
    let mut app = app::App::new();
    let result = run(&mut app, terminal);
    ratatui::restore();
    result
}

fn run(app: &mut app::App, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(app, frame))?;
        let key = match event::read() {
            Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => key,
            _ => continue,
        };
        tracing::debug!("key event: {:?} modifiers={:?}", key.code, key.modifiers);
        app.on_key_event(key);
    }
    Ok(())
}
