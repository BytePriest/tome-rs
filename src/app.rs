use crossterm::event::KeyEvent;

use crate::modules::editor::EditorState;
use crate::modules::explorer::ExplorerState;
use crate::modules::keyboard::handle_key_event;

#[derive(Debug, Default)]
pub enum Focus {
    #[default]
    Explorer,
    Editor,
}

impl Focus {
    pub fn toggle(&self) -> Self {
        match self {
            Focus::Explorer => Focus::Editor,
            Focus::Editor => Focus::Explorer,
        }
    }
}

pub struct App {
    pub running: bool,
    pub focus: Focus,
    pub explorer: ExplorerState,
    pub editor: EditorState,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            focus: Focus::Explorer,
            explorer: ExplorerState::new("."),
            editor: EditorState::new(),
        }
    }

    pub fn on_key_event(&mut self, key: KeyEvent) {
        handle_key_event(self, key);
    }
}
