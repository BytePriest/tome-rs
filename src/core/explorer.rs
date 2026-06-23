use std::{fs, path::PathBuf};

use crate::core::editor::EditorState;

#[derive(Debug, Clone)]
pub struct TreeNode {
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
pub struct VisibleEntry {
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
            editor.load_file(&entry.path);
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
