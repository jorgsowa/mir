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

fn ensure_trailing_backslash(prefix: &str) -> String {
    if prefix.ends_with('\\') {
        prefix.to_string()
    } else {
        format!("{}\\", prefix)
    }
}

fn parse_vendor_entries(root: &Path) -> Vec<(String, PathBuf)> {
    let installed_path = root.join("vendor/composer/installed.json");
    let content = match std::fs::read_to_string(&installed_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let packages = if let Some(arr) = value.get("packages").and_then(|v| v.as_array()) {
        arr.clone()
    } else if let Some(arr) = value.as_array() {
        arr.clone()
    } else {
        return Vec::new();
    };

    let vendor_dir = root.join("vendor");
    let mut entries: Vec<(String, PathBuf)> = Vec::new();

    for pkg in &packages {
        if let Some(map) = pkg.pointer("/autoload/psr-4").and_then(|v| v.as_object()) {
            let pkg_name = pkg.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let pkg_dir = vendor_dir.join(pkg_name);
            for (prefix, dir) in map {
                if let Some(d) = dir.as_str() {
                    entries.push((ensure_trailing_backslash(prefix), pkg_dir.join(d)));
                }
            }
        }
    }

    entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    entries
}

impl Psr4Map {
    pub fn from_composer(root: &Path) -> Result<Self, ComposerError> {
        let composer_path = root.join("composer.json");
        let content = std::fs::read_to_string(&composer_path)?;
        let value: serde_json::Value = serde_json::from_str(&content)?;

        let has_autoload = value.get("autoload").is_some()
            || value.get("autoload-dev").is_some();
        if !has_autoload {
            return Err(ComposerError::MissingAutoload);
        }

        let mut project_entries: Vec<(String, PathBuf)> = Vec::new();

        if let Some(map) = value.pointer("/autoload/psr-4").and_then(|v| v.as_object()) {
            for (prefix, dir) in map {
                if let Some(d) = dir.as_str() {
                    project_entries.push((ensure_trailing_backslash(prefix), root.join(d)));
                }
            }
        }
        if let Some(map) = value.pointer("/autoload-dev/psr-4").and_then(|v| v.as_object()) {
            for (prefix, dir) in map {
                if let Some(d) = dir.as_str() {
                    project_entries.push((ensure_trailing_backslash(prefix), root.join(d)));
                }
            }
        }

        project_entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        let vendor_entries = parse_vendor_entries(root);

        Ok(Psr4Map {
            project_entries,
            vendor_entries,
            root: root.to_path_buf(),
        })
    }

    pub fn project_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for (_, dir) in &self.project_entries {
            crate::project::collect_php_files(dir, &mut out);
        }
        out
    }

    pub fn vendor_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for (_, dir) in &self.vendor_entries {
            crate::project::collect_php_files(dir, &mut out);
        }
        out
    }

    /// Resolve a fully-qualified class name to a file path using longest-prefix-first matching.
    /// Returns `None` if no prefix matches or the mapped file does not exist on disk.
    pub fn resolve(&self, _fqcn: &str) -> Option<PathBuf> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_temp_project(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mir_psr4_{}", name));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parse_project_entries() {
        let root = make_temp_project("parse_project_entries");
        fs::write(
            root.join("composer.json"),
            r#"{
                "autoload": {
                    "psr-4": { "App\\": "src/", "App\\Models\\": "src/models/" }
                },
                "autoload-dev": {
                    "psr-4": { "Tests\\": "tests/" }
                }
            }"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();

        let prefixes: Vec<&str> = map.project_entries.iter().map(|(p, _)| p.as_str()).collect();
        assert!(prefixes.contains(&"App\\Models\\"), "missing App\\Models\\");
        assert!(prefixes.contains(&"App\\"), "missing App\\");
        assert!(prefixes.contains(&"Tests\\"), "missing Tests\\");
    }

    #[test]
    fn longest_prefix_first() {
        let root = make_temp_project("longest_prefix_first");
        fs::write(
            root.join("composer.json"),
            r#"{
                "autoload": {
                    "psr-4": { "App\\": "src/", "App\\Models\\": "src/models/" }
                }
            }"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();

        assert_eq!(map.project_entries[0].0, "App\\Models\\");
    }

    #[test]
    fn missing_autoload_section_is_error() {
        let root = make_temp_project("missing_autoload");
        fs::write(root.join("composer.json"), r#"{ "name": "my/pkg" }"#).unwrap();

        let result = Psr4Map::from_composer(&root);
        assert!(
            matches!(result, Err(ComposerError::MissingAutoload)),
            "expected MissingAutoload error"
        );
    }

    #[test]
    fn composer_v2_installed() {
        let root = make_temp_project("composer_v2");
        fs::write(root.join("composer.json"), r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#).unwrap();

        let vendor_dir = root.join("vendor/composer");
        fs::create_dir_all(&vendor_dir).unwrap();
        fs::write(
            vendor_dir.join("installed.json"),
            r#"{
                "packages": [
                    {
                        "name": "vendor/pkg",
                        "autoload": { "psr-4": { "Vendor\\Pkg\\": "src/" } }
                    }
                ]
            }"#,
        )
        .unwrap();
        fs::create_dir_all(root.join("vendor/vendor/pkg/src")).unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let prefixes: Vec<&str> = map.vendor_entries.iter().map(|(p, _)| p.as_str()).collect();
        assert!(prefixes.contains(&"Vendor\\Pkg\\"), "missing Vendor\\Pkg\\");
    }

    #[test]
    fn composer_v1_installed() {
        let root = make_temp_project("composer_v1");
        fs::write(root.join("composer.json"), r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#).unwrap();

        let vendor_dir = root.join("vendor/composer");
        fs::create_dir_all(&vendor_dir).unwrap();
        fs::write(
            vendor_dir.join("installed.json"),
            r#"[
                {
                    "name": "vendor/pkg",
                    "autoload": { "psr-4": { "Vendor\\Pkg\\": "src/" } }
                }
            ]"#,
        )
        .unwrap();
        fs::create_dir_all(root.join("vendor/vendor/pkg/src")).unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let prefixes: Vec<&str> = map.vendor_entries.iter().map(|(p, _)| p.as_str()).collect();
        assert!(prefixes.contains(&"Vendor\\Pkg\\"), "missing Vendor\\Pkg\\");
    }

    #[test]
    fn missing_installed_json() {
        let root = make_temp_project("missing_installed");
        fs::write(root.join("composer.json"), r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#).unwrap();
        let map = Psr4Map::from_composer(&root).unwrap();
        assert!(map.vendor_entries.is_empty());
    }

    #[test]
    fn project_files_returns_php_files() {
        let root = make_temp_project("project_files");
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("Foo.php"), "<?php class Foo {}").unwrap();
        fs::write(src.join("README.md"), "not php").unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let files = map.project_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Foo.php"));
    }
}
