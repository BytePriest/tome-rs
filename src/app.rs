use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{fs, path::PathBuf};

#[derive(Debug, Default)]
pub enum Focus {
    #[default]
    Explorer,
    Editor,
}

#[derive(Debug, Clone)]
pub(crate) struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    fn new_dir(name: String, path: PathBuf, expanded: bool) -> Self {
        let mut node = Self {
            name,
            path,
            is_dir: true,
            expanded,
            children: Vec::new(),
        };
        if expanded {
            node.load_children();
        }
        node
    }

    fn new_file(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            is_dir: false,
            expanded: false,
            children: Vec::new(),
        }
    }

    fn load_children(&mut self) {
        let entries = read_dir_entries(&self.path);
        self.children = entries
            .into_iter()
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let path = e.path();
                if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    TreeNode::new_dir(name, path, false)
                } else {
                    TreeNode::new_file(name, path)
                }
            })
            .collect();
    }
}

#[derive(Debug, Clone)]
pub(crate) struct VisibleEntry {
    pub depth: usize,
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
}

#[derive(Debug)]
pub struct ExplorerState {
    pub root: TreeNode,
    pub visible_entries: Vec<VisibleEntry>,
    pub selected: usize,
}

#[derive(Debug)]
pub struct EditorState {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub scroll: usize,
    pub file_path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub focus: Focus,
    pub explorer: ExplorerState,
    pub editor: EditorState,
}

impl ExplorerState {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let path = path.canonicalize().unwrap_or(path);
        let root_name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        let root = TreeNode::new_dir(root_name, path, false);
        let mut state = Self {
            root,
            selected: 0,
            visible_entries: Vec::new(),
        };
        state.rebuild_visible();
        state
    }

    fn rebuild_visible(&mut self) {
        self.visible_entries.clear();
        collect_visible(&self.root, 0, &mut self.visible_entries);
    }

    pub fn navigate_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn navigate_down(&mut self) {
        if self.selected + 1 < self.visible_entries.len() {
            self.selected += 1;
        }
    }

    pub fn toggle_selected(&mut self, editor: &mut EditorState) {
        let Some(entry) = self.visible_entries.get(self.selected).cloned() else {
            return;
        };

        if entry.is_dir {
            self.toggle_dir(&entry.path);
        } else {
            open_file_in_editor(&entry.path, editor);
        }
    }

    fn toggle_dir(&mut self, path: &std::path::Path) {
        toggle_tree_node(&mut self.root, path);
        self.rebuild_visible();
    }
}

fn read_dir_entries(path: &std::path::Path) -> Vec<fs::DirEntry> {
    let mut entries: Vec<_> = fs::read_dir(path)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name != "." && name != ".."
        })
        .collect();
    entries.sort_by_key(|e| {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        (!is_dir, e.file_name())
    });
    entries
}

fn toggle_tree_node(node: &mut TreeNode, path: &std::path::Path) -> bool {
    if node.path == path {
        node.expanded = !node.expanded;
        if node.expanded {
            node.load_children();
        }
        return true;
    }
    for child in &mut node.children {
        if toggle_tree_node(child, path) {
            return true;
        }
    }
    false
}

fn collect_visible(node: &TreeNode, depth: usize, result: &mut Vec<VisibleEntry>) {
    result.push(VisibleEntry {
        depth,
        name: node.name.clone(),
        path: node.path.clone(),
        is_dir: node.is_dir,
        expanded: node.expanded,
    });
    if node.is_dir && node.expanded {
        for child in &node.children {
            collect_visible(child, depth + 1, result);
        }
    }
}

fn open_file_in_editor(path: &std::path::Path, editor: &mut EditorState) {
    if let Ok(content) = fs::read_to_string(path) {
        let lines: Vec<String> = content
            .lines()
            .map(|l| l.trim_end_matches('\r').to_string())
            .collect();
        editor.lines = if lines.is_empty() {
            vec![String::new()]
        } else {
            lines
        };
        editor.cursor_line = 0;
        editor.cursor_col = 0;
        editor.scroll = 0;
        editor.file_path = Some(path.to_path_buf());
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            scroll: 0,
            file_path: None,
        }
    }

    fn clamp_cursor(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_col > line_len {
            self.cursor_col = line_len;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.clamp_cursor();
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.clamp_cursor();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
        }
    }

    pub fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.lines[self.cursor_line].insert(self.cursor_col, c);
        self.cursor_col += 1;
    }

    pub fn insert_newline(&mut self) {
        let remainder = self.lines[self.cursor_line].split_off(self.cursor_col);
        self.lines.insert(self.cursor_line + 1, remainder);
        self.cursor_line += 1;
        self.cursor_col = 0;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_col > 0 {
            self.lines[self.cursor_line].remove(self.cursor_col - 1);
            self.cursor_col = self.cursor_col.saturating_sub(1);
        } else if self.cursor_line > 0 {
            let prev_len = self.lines[self.cursor_line - 1].len();
            let rest = self.lines.remove(self.cursor_line);
            self.lines[self.cursor_line - 1].push_str(&rest);
            self.cursor_line -= 1;
            self.cursor_col = prev_len;
        }
    }

    pub fn adjust_scroll(&mut self, visible_lines: usize) {
        if visible_lines == 0 {
            return;
        }
        if self.cursor_line < self.scroll {
            self.scroll = self.cursor_line;
        } else if self.cursor_line >= self.scroll + visible_lines {
            self.scroll = self.cursor_line - visible_lines + 1;
        }
    }
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
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                self.running = false;
            }
            (_, KeyCode::Tab) | (_, KeyCode::Char('\t')) => self.toggle_focus(),
            _ => match self.focus {
                Focus::Explorer => self.handle_explorer_key(key),
                Focus::Editor => self.handle_editor_key(key),
            },
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Explorer => Focus::Editor,
            Focus::Editor => Focus::Explorer,
        };
    }

    fn handle_explorer_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.explorer.navigate_up(),
            KeyCode::Down => self.explorer.navigate_down(),
            KeyCode::Enter => self.explorer.toggle_selected(&mut self.editor),
            _ => {}
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up => self.editor.move_up(),
            KeyCode::Down => self.editor.move_down(),
            KeyCode::Left => self.editor.move_left(),
            KeyCode::Right => self.editor.move_right(),
            KeyCode::Enter => self.editor.insert_newline(),
            KeyCode::Backspace => self.editor.delete_char(),
            KeyCode::Char(c) => self.editor.insert_char(c),
            _ => {}
        }
    }
}
