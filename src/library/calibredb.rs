//! The documented calibre CLI: `calibredb list` and `calibredb export`.

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("calibredb is not executable: {0}")]
    NotExecutable(String),
    #[error("calibredb timed out after {0} ms")]
    Timeout(u64),
    #[error("calibredb failed: {0}")]
    Failed(String),
    #[error("calibredb output is not valid JSON: {0}")]
    InvalidOutput(String),
    #[error("the exported edition is missing: {0}")]
    MissingExport(String),
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListingBook {
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub authors: Option<String>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub pubdate: Option<String>,
    #[serde(default)]
    pub formats: Option<Vec<String>>,
    #[serde(default)]
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Calibredb {
    binary: String,
    with_library: String,
    timeout: Duration,
}

impl Calibredb {
    pub fn new(binary: impl Into<String>, with_library: impl Into<String>, timeout_ms: u64) -> Self {
        Self {
            binary: binary.into(),
            with_library: with_library.into(),
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    pub fn with_library(&self) -> &str {
        &self.with_library
    }

    /// `calibredb list --with-library <url-or-path> --for-machine --fields all`
    pub async fn list(&self) -> Result<Vec<ListingBook>, LibraryError> {
        let output = self
            .run(&[
                "list",
                "--with-library",
                &self.with_library,
                "--for-machine",
                "--fields",
                "all",
            ])
            .await?;
        serde_json::from_slice::<Vec<ListingBook>>(&output)
            .map_err(|error| LibraryError::InvalidOutput(error.to_string()))
    }

    /// Export one format into `dir` with `--single-dir`; return the file.
    pub async fn export(
        &self,
        work_id: u64,
        format: &str,
        dir: &Path,
    ) -> Result<PathBuf, LibraryError> {
        let id = work_id.to_string();
        self.run(&[
            "export",
            "--with-library",
            &self.with_library,
            "--to-dir",
            &dir.to_string_lossy(),
            "--single-dir",
            "--formats",
            format,
            "--dont-save-cover",
            "--dont-write-opf",
            "--dont-save-extra-files",
            &id,
        ])
        .await?;
        let wanted = format.to_ascii_lowercase();
        let mut found = None;
        let entries = std::fs::read_dir(dir).map_err(|error| {
            LibraryError::MissingExport(format!("{}: {error}", dir.display()))
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|value| value.eq_ignore_ascii_case(&wanted))
            {
                found = Some(path);
                break;
            }
        }
        found.ok_or_else(|| {
            LibraryError::MissingExport(format!(
                "no .{} file in {}",
                wanted,
                dir.display()
            ))
        })
    }

    async fn run(&self, args: &[&str]) -> Result<Vec<u8>, LibraryError> {
        let mut command = tokio::process::Command::new(&self.binary);
        command
            .args(args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);
        let output = tokio::time::timeout(self.timeout, command.output())
            .await
            .map_err(|_| LibraryError::Timeout(self.timeout.as_millis() as u64))?
            .map_err(|error| LibraryError::NotExecutable(error.to_string()))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(LibraryError::Failed(format!(
                "{}: {}",
                self.binary,
                stderr.trim()
            )));
        }
        Ok(output.stdout)
    }
}
