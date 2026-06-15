use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Focus};

const SAVE_MOD: KeyModifiers = if cfg!(target_os = "macos") {
    KeyModifiers::META
} else {
    KeyModifiers::CONTROL
};

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
            app.running = false;
        }
        (_, KeyCode::Tab) | (_, KeyCode::Char('\t')) => app.focus = app.focus.toggle(),
        _ => match app.focus {
            Focus::Explorer => handle_explorer_key(app, key),
            Focus::Editor => app.editor.handle_key(key, SAVE_MOD),
        },
    }
}

fn handle_explorer_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up => app.explorer.navigate_up(),
        KeyCode::Down => app.explorer.navigate_down(),
        KeyCode::Enter => app.explorer.toggle_selected(&mut app.editor),
        _ => {}
    }
}
