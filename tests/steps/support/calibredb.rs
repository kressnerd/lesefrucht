//! Real `calibredb list` invocation used by the non-mutation assertions.

use std::path::Path;

pub fn list(binary: &str, library: &Path) -> String {
    std::process::Command::new(binary)
        .args([
            "list",
            "--with-library",
            &library.to_string_lossy(),
            "--for-machine",
            "--fields",
            "all",
        ])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
}
