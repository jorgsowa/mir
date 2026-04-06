use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ComposerError {
    Io(std::io::Error),
    Json(serde_json::Error),
    MissingAutoload,
}

impl std::fmt::Display for ComposerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComposerError::Io(e) => write!(f, "composer I/O error: {}", e),
            ComposerError::Json(e) => write!(f, "composer JSON error: {}", e),
            ComposerError::MissingAutoload => write!(f, "composer.json has no autoload section"),
        }
    }
}

impl From<std::io::Error> for ComposerError {
    fn from(e: std::io::Error) -> Self {
        ComposerError::Io(e)
    }
}

impl From<serde_json::Error> for ComposerError {
    fn from(e: serde_json::Error) -> Self {
        ComposerError::Json(e)
    }
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
