use crossterm::event::KeyEvent;

use crate::core::editor::EditorState;
use crate::core::explorer::ExplorerState;
use crate::core::keyboard::handle_key_event;
use crate::core::search::SearchState;

//记录当前在左侧侧边栏还是右侧编辑区
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
    pub search: SearchState,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            focus: Focus::Explorer,
            explorer: ExplorerState::new("."),
            editor: EditorState::new(),
            search: SearchState::new(),
        }
    }

    pub fn on_key_event(&mut self, key: KeyEvent) {
        handle_key_event(self, key);
    }
}
