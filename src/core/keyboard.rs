use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, Focus};

const SAVE_MOD: KeyModifiers = KeyModifiers::CONTROL;

pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    if matches!(
        (key.modifiers, key.code),
        (
            KeyModifiers::CONTROL,
            KeyCode::Char('c') | KeyCode::Char('C')
        ) | (
            KeyModifiers::CONTROL,
            KeyCode::Char('q') | KeyCode::Char('Q')
        )
    ) {
        tracing::debug!("quit");
        app.running = false;
        return;
    }

    if app.search.visible {
        app.search.handle_key(key, &mut app.editor);
        return;
    }

    if key.modifiers == KeyModifiers::CONTROL
        && matches!(key.code, KeyCode::Char('f') | KeyCode::Char('F'))
    {
        tracing::debug!("open search");
        app.search.toggle();
        return;
    }

    if matches!(key.code, KeyCode::Tab | KeyCode::Char('\t')) {
        app.focus = app.focus.toggle();
        tracing::debug!("focus -> {:?}", app.focus);
        return;
    }

    match app.focus {
        Focus::Explorer => {
            handle_explorer_key(app, key);
        }
        Focus::Editor => {
            app.editor.handle_key(key, SAVE_MOD);
        }
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
