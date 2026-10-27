//! Normalized edition text; character offsets are indices into these files.

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct TextStore {
    dir: PathBuf,
}

impl TextStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn write(&self, key: &str, text: &str) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(&self.dir)?;
        let path = self.path_for(key);
        std::fs::write(&path, text)?;
        Ok(path)
    }

    pub fn read(&self, key: &str) -> std::io::Result<String> {
        std::fs::read_to_string(self.path_for(key))
    }

    pub fn path_for(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{key}.txt"))
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }
}
