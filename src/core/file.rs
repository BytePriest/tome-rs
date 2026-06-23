use std::{fs, path::PathBuf};

use ropey::Rope;

#[derive(Debug)]
pub struct File {
    pub path: Option<PathBuf>,
    pub rope: Rope,
    pub dirty: bool,
}

impl File {
    pub fn new() -> Self {
        Self {
            path: None,
            rope: Rope::new(),
            dirty: false,
        }
    }

    pub fn open(path: &std::path::Path) -> Self {
        tracing::debug!("File::open: {}", path.display());
        let content = fs::read_to_string(path).unwrap_or_default();
        let rope = Rope::from_str(&content);
        Self {
            path: Some(path.to_path_buf()),
            rope,
            dirty: false,
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            let content = self.rope.to_string();
            tracing::debug!(
                "File::save: path={} lines={} bytes={}",
                path.display(),
                self.rope.len_lines(),
                content.len(),
            );
            fs::write(path, &content)?;
            tracing::debug!("File::save: write ok");
        } else {
            tracing::warn!("File::save: no path set, nothing to save");
        }
        Ok(())
    }
}
