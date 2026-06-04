use rustc_hash::FxHashMap;
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

/// PSR-4 / PSR-0 / classmap / files autoload mapping, built from `composer.json`
/// and `vendor/composer/installed.json`.
///
/// `project_entries` covers `autoload.psr-4` / `autoload-dev.psr-4` /
/// `autoload.psr-0` / `autoload-dev.psr-0` for the project itself.
/// `vendor_entries` covers the same keys from each installed package.
/// `project_extra_paths` and `vendor_extra_paths` collect the (prefix-less)
/// `classmap` and `files` entries as raw paths — files are kept as-is, dirs
/// are walked when assembling the file list.
///
/// Both prefix lists are sorted longest-prefix-first for correct prefix matching.
#[derive(Clone)]
pub struct Psr4Map {
    project_entries: Vec<(String, PathBuf)>,
    vendor_entries: Vec<(String, PathBuf)>,
    project_extra_paths: Vec<PathBuf>,
    vendor_extra_paths: Vec<PathBuf>,
    /// Pre-resolved FQCN → file map from `vendor/composer/autoload_classmap.php`.
    /// Covers packages using `classmap:` autoload (non-PSR-4) so they can be
    /// lazy-loaded by FQCN without parsing every classmap directory eagerly.
    /// Key is the FQCN with single backslashes (e.g. `AWS\\CRT\\Auth\\Signing`
    /// in source becomes `AWS\CRT\Auth\Signing` here, matching what PHP code
    /// uses at call sites).
    classmap: FxHashMap<String, PathBuf>,
    /// Files registered via `autoload.files` (project + vendor). These contain
    /// unbound global functions/constants that are NOT FQCN-resolvable, so they
    /// must be eagerly parsed even in lazy mode. Read from
    /// `vendor/composer/autoload_files.php` if present, falling back to the
    /// per-package `installed.json` walk.
    vendor_eager_files: Vec<PathBuf>,
    #[allow(dead_code)] // used by issue #50 (lazy FQCN resolution)
    root: PathBuf,
}

fn ensure_trailing_backslash(prefix: &str) -> String {
    if prefix.ends_with('\\') {
        prefix.to_string()
    } else {
        format!("{prefix}\\")
    }
}

/// Append `(prefix, base.join(dir))` to `entries` for every dir-string in `value`
/// (which may be a JSON string or an array of strings).
fn collect_prefix_dirs(
    value: &serde_json::Value,
    prefix: &str,
    base: &Path,
    entries: &mut Vec<(String, PathBuf)>,
) {
    let pfx = ensure_trailing_backslash(prefix);
    if let Some(d) = value.as_str() {
        entries.push((pfx, base.join(d)));
    } else if let Some(arr) = value.as_array() {
        for item in arr {
            if let Some(d) = item.as_str() {
                entries.push((pfx.clone(), base.join(d)));
            }
        }
    }
}

/// Append every string in `value` (a JSON array) to `out` as `base.join(s)`.
fn collect_path_array(value: &serde_json::Value, base: &Path, out: &mut Vec<PathBuf>) {
    if let Some(arr) = value.as_array() {
        for item in arr {
            if let Some(s) = item.as_str() {
                out.push(base.join(s));
            }
        }
    }
}

fn parse_autoload_section(
    autoload: &serde_json::Value,
    base: &Path,
    entries: &mut Vec<(String, PathBuf)>,
    extras: &mut Vec<PathBuf>,
) {
    if let Some(map) = autoload.get("psr-4").and_then(|v| v.as_object()) {
        for (prefix, dir) in map {
            collect_prefix_dirs(dir, prefix, base, entries);
        }
    }
    // PSR-0 maps prefix → dir similarly to PSR-4. The class-name-to-file
    // resolution differs (underscores in class basename become dirs), but for
    // discovering all .php files in the mapped directories, walking the dir
    // is sufficient. We do NOT add these to `entries` for FQCN resolution
    // because `Psr4Map::resolve` uses PSR-4 semantics — instead we treat the
    // dirs as bulk-scan paths.
    if let Some(map) = autoload.get("psr-0").and_then(|v| v.as_object()) {
        for (_, dir) in map {
            if let Some(d) = dir.as_str() {
                extras.push(base.join(d));
            } else if let Some(arr) = dir.as_array() {
                for item in arr {
                    if let Some(d) = item.as_str() {
                        extras.push(base.join(d));
                    }
                }
            }
        }
    }
    if let Some(cm) = autoload.get("classmap") {
        collect_path_array(cm, base, extras);
    }
    if let Some(files) = autoload.get("files") {
        collect_path_array(files, base, extras);
    }
}

/// Parse a Composer-generated `autoload_classmap.php` or `autoload_files.php`.
///
/// The format is mechanically generated and stable:
///
/// ```text
/// <?php
/// $vendorDir = dirname(__DIR__);
/// $baseDir = dirname($vendorDir);
/// return array(
///     'KEY' => $vendorDir . '/relative/path.php',
///     'OTHER' => $baseDir . '/other/path.php',
/// );
/// ```
///
/// Returns `(key, absolute_path)` pairs. Unparseable lines are silently
/// skipped — a stale or hand-edited file degrades gracefully.
///
/// `key` is returned with PHP-escape-sequence handling for backslashes (`\\` → `\`).
fn parse_composer_autoload_array(
    content: &str,
    vendor_dir: &Path,
    base_dir: &Path,
) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        // Find `'KEY' => $VAR . 'PATH'` or `"KEY" => $VAR . "PATH"`.
        let (key, rest) = match extract_quoted(line) {
            Some(p) => p,
            None => continue,
        };
        let rest = rest.trim_start();
        let rest = match rest.strip_prefix("=>") {
            Some(r) => r.trim_start(),
            None => continue,
        };
        let (var, rest) = match rest.strip_prefix('$') {
            Some(r) => {
                let end = r
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or(r.len());
                (&r[..end], &r[end..])
            }
            None => continue,
        };
        let rest = rest.trim_start();
        let rest = match rest.strip_prefix('.') {
            Some(r) => r.trim_start(),
            None => continue,
        };
        let (path_frag, _) = match extract_quoted(rest) {
            Some(p) => p,
            None => continue,
        };
        let base = match var {
            "vendorDir" => vendor_dir,
            "baseDir" => base_dir,
            _ => continue,
        };
        // path_frag begins with '/' relative to the chosen base.
        let path_rel = path_frag.trim_start_matches('/');
        out.push((key, base.join(path_rel)));
    }
    out
}

/// Pull a PHP single- or double-quoted string from the start of `s`, decoding
/// `\\` → `\` and `\'` / `\"` → the corresponding quote. Returns `(decoded, tail)`
/// where `tail` is the slice after the closing quote.
fn extract_quoted(s: &str) -> Option<(String, &str)> {
    let mut it = s.char_indices();
    let (_, quote) = it.next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escape = false;
    for (i, ch) in it {
        if escape {
            out.push(ch);
            escape = false;
        } else if ch == '\\' {
            escape = true;
        } else if ch == quote {
            return Some((out, &s[i + ch.len_utf8()..]));
        } else {
            out.push(ch);
        }
    }
    None
}

fn parse_vendor(root: &Path, entries: &mut Vec<(String, PathBuf)>, extras: &mut Vec<PathBuf>) {
    let installed_path = root.join("vendor/composer/installed.json");
    let content = match std::fs::read_to_string(&installed_path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return,
    };

    let packages = if let Some(arr) = value.get("packages").and_then(|v| v.as_array()) {
        arr.clone()
    } else if let Some(arr) = value.as_array() {
        arr.clone()
    } else {
        return;
    };

    let vendor_dir = root.join("vendor");

    for pkg in &packages {
        let pkg_name = pkg.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let pkg_dir = vendor_dir.join(pkg_name);
        if let Some(autoload) = pkg.get("autoload") {
            parse_autoload_section(autoload, &pkg_dir, entries, extras);
        }
    }
}

/// Read `vendor/composer/autoload_classmap.php`. Returns an empty map if the
/// file is absent or unreadable — callers fall back to the project-defined
/// PSR-4 entries which still cover the typical case.
///
/// Uses lossy UTF-8 decoding because some real-world classmap files contain
/// stray Latin-1 bytes (e.g. Laravel's includes a `\xa9` key from a vendor
/// package). Lossy decoding only affects the malformed FQCN itself — every
/// other entry parses correctly.
fn read_classmap(vendor_dir: &Path, base_dir: &Path) -> FxHashMap<String, PathBuf> {
    let path = vendor_dir.join("composer/autoload_classmap.php");
    let Ok(bytes) = std::fs::read(&path) else {
        return FxHashMap::default();
    };
    let content = String::from_utf8_lossy(&bytes);
    parse_composer_autoload_array(&content, vendor_dir, base_dir)
        .into_iter()
        .collect()
}

/// Read `vendor/composer/autoload_files.php`. Falls back to walking
/// `installed.json` if the generated file is absent — covers projects that
/// haven't run `composer dump-autoload --optimize`.
fn read_files_autoload(vendor_dir: &Path, base_dir: &Path) -> Vec<PathBuf> {
    let path = vendor_dir.join("composer/autoload_files.php");
    if let Ok(bytes) = std::fs::read(&path) {
        let content = String::from_utf8_lossy(&bytes);
        return parse_composer_autoload_array(&content, vendor_dir, base_dir)
            .into_iter()
            .map(|(_, p)| p)
            .filter(|p| p.is_file())
            .collect();
    }
    // Fallback: walk installed.json packages for autoload.files entries only.
    let installed_path = vendor_dir.join("composer/installed.json");
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
    let mut out = Vec::new();
    for pkg in &packages {
        let pkg_name = pkg.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let pkg_dir = vendor_dir.join(pkg_name);
        if let Some(files) = pkg.get("autoload").and_then(|a| a.get("files")) {
            collect_path_array(files, &pkg_dir, &mut out);
        }
    }
    let _ = base_dir; // unused in fallback path (paths are package-relative)
    out.into_iter().filter(|p| p.is_file()).collect()
}

impl Psr4Map {
    pub fn from_composer(root: &Path) -> Result<Self, ComposerError> {
        let composer_path = root.join("composer.json");
        let content = std::fs::read_to_string(&composer_path)?;
        let value: serde_json::Value = serde_json::from_str(&content)?;

        let has_autoload = value.get("autoload").is_some() || value.get("autoload-dev").is_some();
        if !has_autoload {
            return Err(ComposerError::MissingAutoload);
        }

        let mut project_entries: Vec<(String, PathBuf)> = Vec::new();
        let mut project_extra_paths: Vec<PathBuf> = Vec::new();

        if let Some(autoload) = value.get("autoload") {
            parse_autoload_section(
                autoload,
                root,
                &mut project_entries,
                &mut project_extra_paths,
            );
        }
        if let Some(autoload) = value.get("autoload-dev") {
            parse_autoload_section(
                autoload,
                root,
                &mut project_entries,
                &mut project_extra_paths,
            );
        }

        project_entries.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

        let mut vendor_entries: Vec<(String, PathBuf)> = Vec::new();
        let mut vendor_extra_paths: Vec<PathBuf> = Vec::new();
        parse_vendor(root, &mut vendor_entries, &mut vendor_extra_paths);
        vendor_entries.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

        // Read composer-generated FQCN → file map from autoload_classmap.php.
        // When present this is the source of truth for non-PSR-4 vendor classes,
        // letting lazy-mode resolve them without parsing whole classmap dirs.
        let vendor_dir = root.join("vendor");
        let classmap = read_classmap(&vendor_dir, root);

        // Eager-load list = autoload.files entries (project + vendor). These
        // hold unbound globals (functions, constants, polyfills) that the lazy
        // FQCN-based resolver cannot reach.
        let vendor_eager_files = read_files_autoload(&vendor_dir, root);

        Ok(Psr4Map {
            project_entries,
            vendor_entries,
            project_extra_paths,
            vendor_extra_paths,
            classmap,
            vendor_eager_files,
            root: root.to_path_buf(),
        })
    }

    pub fn project_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for (_, dir) in &self.project_entries {
            crate::batch::collect_php_files(dir, &mut out);
        }
        for path in &self.project_extra_paths {
            collect_php_path(path, &mut out);
        }
        out
    }

    pub fn vendor_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for (_, dir) in &self.vendor_entries {
            crate::batch::collect_php_files(dir, &mut out);
        }
        for path in &self.vendor_extra_paths {
            collect_php_path(path, &mut out);
        }
        out
    }

    /// Resolve a fully-qualified class name to a file path using longest-prefix-first matching.
    /// Returns `None` if no prefix matches or the mapped file does not exist on disk.
    ///
    /// Resolution order:
    /// 1. PSR-4 project entries (longest-prefix-first).
    /// 2. PSR-4 vendor entries (longest-prefix-first).
    /// 3. Classmap from `vendor/composer/autoload_classmap.php` — exact FQCN match.
    ///
    /// PSR-4 wins over classmap because Composer's runtime resolver uses the
    /// same order; this matches what the PHP code being analyzed actually sees.
    pub fn resolve(&self, fqcn: &str) -> Option<PathBuf> {
        let key = fqcn.trim_start_matches('\\');
        for (prefix, dir) in self
            .project_entries
            .iter()
            .chain(self.vendor_entries.iter())
        {
            if key.starts_with(prefix.as_str()) {
                let relative = &key[prefix.len()..];
                let file_path = dir.join(relative.replace('\\', "/")).with_extension("php");
                if file_path.exists() {
                    return Some(file_path);
                }
            }
        }
        if let Some(path) = self.classmap.get(key) {
            if path.exists() {
                return Some(path.clone());
            }
        }
        None
    }

    /// Vendor files that must be eagerly parsed in lazy mode: `autoload.files`
    /// entries from composer. These hold globals (functions, constants,
    /// polyfills) that the FQCN-based lazy resolver cannot reach because they
    /// have no namespace mapping.
    ///
    /// Returns just the existing `.php` files; missing entries (stale generated
    /// file) are dropped.
    pub fn vendor_eager_files(&self) -> Vec<PathBuf> {
        self.vendor_eager_files.clone()
    }

    /// Every vendor file the analyzer should index eagerly, for the
    /// rust-analyzer-style static-input model: the union of
    ///
    /// 1. [`Self::vendor_files`] — PSR-4 / PSR-0 walked directories + extra paths,
    /// 2. classmap file targets — packages that use `classmap:` autoload (no
    ///    PSR-4 prefix), which [`Self::vendor_files`] does NOT walk, and
    /// 3. [`Self::vendor_eager_files`] — `autoload.files` globals.
    ///
    /// Deduplicated by path. Non-existent classmap targets (stale generated
    /// file) are skipped. This is the work-list a consumer feeds to the chunked
    /// background indexer ([`crate::AnalysisSession::index_batch`]).
    pub fn all_vendor_files(&self) -> Vec<PathBuf> {
        let mut seen: rustc_hash::FxHashSet<PathBuf> = rustc_hash::FxHashSet::default();
        let mut out = Vec::new();
        let mut push = |p: PathBuf, out: &mut Vec<PathBuf>| {
            if seen.insert(p.clone()) {
                out.push(p);
            }
        };
        for p in self.vendor_files() {
            push(p, &mut out);
        }
        // Classmap-only packages are not covered by vendor_files() (which walks
        // only PSR-4/PSR-0 dirs + extra paths). Include their file targets.
        for p in self.classmap.values() {
            if p.is_file() {
                push(p.clone(), &mut out);
            }
        }
        for p in self.vendor_eager_files() {
            push(p, &mut out);
        }
        out
    }

    /// Number of FQCN entries known to the classmap. Used by callers that want
    /// to log/verify the classmap loaded successfully.
    pub fn classmap_len(&self) -> usize {
        self.classmap.len()
    }
}

/// Collect `.php` files from `path`. If `path` is a file, push it directly
/// (when it has a `.php` extension); if it is a directory, walk it.
fn collect_php_path(path: &Path, out: &mut Vec<PathBuf>) {
    let Ok(meta) = std::fs::metadata(path) else {
        return;
    };
    if meta.is_file() {
        if path.extension().and_then(|e| e.to_str()) == Some("php") {
            out.push(path.to_path_buf());
        }
    } else if meta.is_dir() {
        crate::batch::collect_php_files(path, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn make_temp_project(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mir_psr4_{name}"));
        let _ = fs::remove_dir_all(&dir);
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

        let prefixes: Vec<&str> = map
            .project_entries
            .iter()
            .map(|(p, _)| p.as_str())
            .collect();
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
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();

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
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();

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
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();
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

    #[test]
    fn resolve_existing_file() {
        let root = make_temp_project("resolve_existing");
        let models = root.join("src/models");
        fs::create_dir_all(&models).unwrap();
        fs::write(models.join("User.php"), "<?php class User {}").unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\Models\\":"src/models/","App\\":"src/"}}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let result = map.resolve("App\\Models\\User");
        assert!(result.is_some(), "expected a resolved path");
        assert!(result.unwrap().ends_with("User.php"));
    }

    #[test]
    fn resolve_missing_file() {
        let root = make_temp_project("resolve_missing");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let result = map.resolve("App\\Models\\User");
        assert!(result.is_none());
    }

    #[test]
    fn boundary_check() {
        let root = make_temp_project("boundary_check");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        // "App\" must NOT match "Application\Foo"
        let result = map.resolve("Application\\Foo");
        assert!(
            result.is_none(),
            "App\\ prefix must not match Application\\Foo"
        );
    }

    #[test]
    fn array_valued_psr4_dirs() {
        let root = make_temp_project("array_dirs");
        let src = root.join("src");
        let lib = root.join("lib");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&lib).unwrap();
        fs::write(src.join("Foo.php"), "<?php class Foo {}").unwrap();
        fs::write(lib.join("Bar.php"), "<?php class Bar {}").unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":["src/","lib/"]}}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        // Both dirs should be in project_entries
        assert_eq!(
            map.project_entries.len(),
            2,
            "expected 2 entries for array-valued dir"
        );
        let files = map.project_files();
        assert_eq!(files.len(), 2, "expected Foo.php and Bar.php");
    }

    // -----------------------------------------------------------------------
    // classmap / files / psr-0 — vendor and project
    // -----------------------------------------------------------------------

    #[test]
    fn project_classmap_dir_is_collected() {
        let root = make_temp_project("project_classmap");
        let lib = root.join("lib");
        fs::create_dir_all(&lib).unwrap();
        fs::write(lib.join("Legacy.php"), "<?php class Legacy {}").unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"classmap":["lib/"]}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let files = map.project_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Legacy.php"));
    }

    #[test]
    fn project_files_autoload_is_collected() {
        let root = make_temp_project("project_files_autoload");
        fs::write(root.join("helpers.php"), "<?php function my_helper() {}").unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"files":["helpers.php"]}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let files = map.project_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("helpers.php"));
    }

    #[test]
    fn project_psr0_dir_is_collected() {
        let root = make_temp_project("project_psr0");
        let lib = root.join("legacy");
        fs::create_dir_all(&lib).unwrap();
        fs::write(lib.join("Old.php"), "<?php class Old {}").unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-0":{"":"legacy/"}}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let files = map.project_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Old.php"));
    }

    #[test]
    fn vendor_classmap_is_collected() {
        let root = make_temp_project("vendor_classmap");
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();
        let vendor_dir = root.join("vendor/composer");
        fs::create_dir_all(&vendor_dir).unwrap();
        fs::write(
            vendor_dir.join("installed.json"),
            r#"{
                "packages": [{
                    "name": "vendor/pkg",
                    "autoload": { "classmap": ["src/"] }
                }]
            }"#,
        )
        .unwrap();
        let pkg_src = root.join("vendor/vendor/pkg/src");
        fs::create_dir_all(&pkg_src).unwrap();
        fs::write(pkg_src.join("Legacy.php"), "<?php class Legacy {}").unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let files = map.vendor_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Legacy.php"));
    }

    #[test]
    fn vendor_files_autoload_is_collected() {
        let root = make_temp_project("vendor_files_autoload");
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();
        let vendor_dir = root.join("vendor/composer");
        fs::create_dir_all(&vendor_dir).unwrap();
        fs::write(
            vendor_dir.join("installed.json"),
            r#"{
                "packages": [{
                    "name": "vendor/pkg",
                    "autoload": { "files": ["bootstrap.php"] }
                }]
            }"#,
        )
        .unwrap();
        let pkg_dir = root.join("vendor/vendor/pkg");
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(
            pkg_dir.join("bootstrap.php"),
            "<?php function pkg_bootstrap() {}",
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let files = map.vendor_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("bootstrap.php"));
    }

    #[test]
    fn vendor_psr0_is_collected() {
        let root = make_temp_project("vendor_psr0");
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();
        let vendor_dir = root.join("vendor/composer");
        fs::create_dir_all(&vendor_dir).unwrap();
        fs::write(
            vendor_dir.join("installed.json"),
            r#"{
                "packages": [{
                    "name": "vendor/pkg",
                    "autoload": { "psr-0": { "Old_": "src/" } }
                }]
            }"#,
        )
        .unwrap();
        let pkg_src = root.join("vendor/vendor/pkg/src/Old");
        fs::create_dir_all(&pkg_src).unwrap();
        fs::write(pkg_src.join("Thing.php"), "<?php class Old_Thing {}").unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        let files = map.vendor_files();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("Thing.php"));
    }
}
