// m4-rs include module — file inclusion and search path.
//
// Placeholder. Includes are a parity surface (M4.INCLUDE.1).
// Currently not claimed.

use std::path::{Path, PathBuf};

/// Include search path for `include` and `sinclude`.
pub struct IncludePath {
    pub directories: Vec<PathBuf>,
}

impl IncludePath {
    pub fn new() -> Self {
        Self {
            directories: vec![PathBuf::from(".")],
        }
    }

    pub fn add(&mut self, dir: &Path) {
        self.directories.push(dir.to_path_buf());
    }

    pub fn resolve(&self, filename: &str) -> Option<PathBuf> {
        for dir in &self.directories {
            let candidate = dir.join(filename);
            if candidate.exists() {
                return Some(candidate);
            }
        }
        None
    }
}

impl Default for IncludePath {
    fn default() -> Self {
        Self::new()
    }
}
