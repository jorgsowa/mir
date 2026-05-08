// Test utilities for common setup patterns.
// Reduces boilerplate and provides informative error messages.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

/// Create a temporary directory, panicking with context if creation fails.
pub fn create_temp_dir(context: &str) -> TempDir {
    TempDir::new().unwrap_or_else(|_| panic!("failed to create temporary directory ({})", context))
}

/// Write content to a file within a TempDir, panicking with context if write fails.
pub fn write_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let path = dir.path().join(filename);
    fs::write(&path, content)
        .unwrap_or_else(|_| panic!("failed to write file {} ({})", filename, path.display()));
    path
}

/// Convert a Path to a string slice, panicking with context if conversion fails.
pub fn path_to_str(path: &Path) -> &str {
    path.to_str()
        .unwrap_or_else(|| panic!("path contains invalid UTF-8: {}", path.display()))
}

/// Convert a Path to an Arc<str>, useful for APIs expecting Arc<str>.
#[allow(dead_code)]
pub fn path_to_arc_str(path: &Path) -> Arc<str> {
    Arc::from(path_to_str(path))
}

/// Convert a Path to an Arc<str>, useful for APIs expecting Arc<str>.
#[allow(dead_code)]
pub fn pathbuf_to_arc_str(path: &Path) -> Arc<str> {
    path_to_arc_str(path)
}

/// Create an analyzer slice from a single path for analyze() calls.
#[allow(dead_code)]
pub fn to_analyze_slice(path: &Path) -> Vec<PathBuf> {
    vec![path.to_path_buf()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_dir() {
        let dir = create_temp_dir("test");
        assert!(dir.path().exists());
    }

    #[test]
    fn test_write_file() {
        let dir = create_temp_dir("test_write");
        let file = write_file(&dir, "test.txt", "hello");
        assert!(file.exists());
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_path_to_str() {
        let dir = create_temp_dir("test_path");
        let file = write_file(&dir, "test.txt", "");
        let s = path_to_str(&file);
        assert!(!s.is_empty());
    }
}
