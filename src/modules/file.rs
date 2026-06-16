use std::{fs, path::PathBuf};

#[derive(Debug)]
pub struct File {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub dirty: bool,
}

impl File {
    pub fn new() -> Self {
        Self {
            path: None,
            lines: vec![String::new()],
            dirty: false,
        }
    }

    pub fn open(path: &std::path::Path) -> Self {
        tracing::debug!("File::open: {}", path.display());
        let lines = fs::read_to_string(path)
            .map(|content| {
                let mut lines: Vec<_> = content
                    .lines()
                    .map(|l| l.trim_end_matches('\r').to_string())
                    .collect();
                if lines.is_empty() {
                    lines.push(String::new());
                }
                lines.push(String::new());
                lines
            })
            .unwrap_or_else(|_| vec![String::new()]);
        Self {
            path: Some(path.to_path_buf()),
            lines,
            dirty: false,
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        match &self.path {
            Some(path) => {
                let content = self.lines.join("\n");
                tracing::debug!(
                    "File::save: path={} lines={} bytes={}",
                    path.display(),
                    self.lines.len(),
                    content.len(),
                );
                fs::write(path, &content)?;
                tracing::debug!("File::save: write ok");
            }
            None => {
                tracing::warn!("File::save: no path set, nothing to save");
            }
        }
        Ok(())
    }
}
