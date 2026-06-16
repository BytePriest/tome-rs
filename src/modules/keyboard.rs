use crate::app::{App, Focus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

const SAVE_MOD: KeyModifiers = KeyModifiers::CONTROL;

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
            tracing::debug!("quit");
            app.running = false;
        }
        (_, KeyCode::Tab) | (_, KeyCode::Char('\t')) => {
            app.focus = app.focus.toggle();
            tracing::debug!("focus -> {:?}", app.focus);
        }
        _ => match app.focus {
            Focus::Explorer => {
                tracing::debug!("dispatch to explorer");
                handle_explorer_key(app, key);
            }
            Focus::Editor => {
                tracing::debug!("dispatch to editor, save_mod={:?}", SAVE_MOD);
                app.editor.handle_key(key, SAVE_MOD);
            }
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
