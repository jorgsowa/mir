use std::path::{Path, PathBuf};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum ComposerError {
    #[error("composer I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("composer JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("composer.json has no autoload section")]
    MissingAutoload,
}

// ---------------------------------------------------------------------------
// Psr4Map
// ---------------------------------------------------------------------------

/// PSR-4 namespace → directory mapping, built from `composer.json`.
///
/// `project_entries` covers `autoload.psr-4` and `autoload-dev.psr-4`.
/// `vendor_entries`  covers `vendor/composer/installed.json` packages.
///
/// Both lists are sorted longest-prefix-first for correct prefix matching.
pub struct Psr4Map {
    project_entries: Vec<(String, PathBuf)>,
    vendor_entries: Vec<(String, PathBuf)>,
    root: PathBuf,
}

impl Psr4Map {
    pub fn from_composer(_root: &Path) -> Result<Self, ComposerError> {
        todo!()
    }

    pub fn project_files(&self) -> Vec<PathBuf> {
        todo!()
    }

    pub fn vendor_files(&self) -> Vec<PathBuf> {
        todo!()
    }

    /// Resolve a fully-qualified class name to a file path using longest-prefix-first matching.
    /// Returns `None` if no prefix matches or the mapped file does not exist on disk.
    pub fn resolve(&self, _fqcn: &str) -> Option<PathBuf> {
        todo!()
    }
}
