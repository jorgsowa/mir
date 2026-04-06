# PSR-4 Autoload Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `Psr4Map` and `ProjectAnalyzer::from_composer()` so the CLI (and LSP) can discover PHP files from `composer.json` instead of requiring explicit paths.

**Architecture:** A new `composer.rs` module in `mir-analyzer` owns all JSON parsing and file discovery. `ProjectAnalyzer` gets a `psr4` field and a `from_composer()` constructor. The CLI detects `composer.json` in cwd when no paths are given and routes through the new constructor.

**Tech Stack:** Rust, `serde_json` (already a dependency), `std::fs` for file I/O.

---

## File Map

| File | Change |
|------|--------|
| `crates/mir-analyzer/src/composer.rs` | **Create** — `Psr4Map`, `ComposerError`, all parsing logic |
| `crates/mir-analyzer/src/project.rs` | **Modify** — add `psr4` field, update constructors, add `from_composer()` |
| `crates/mir-analyzer/src/lib.rs` | **Modify** — add `pub mod composer; pub use composer::Psr4Map;` |
| `crates/mir-cli/src/main.rs` | **Modify** — auto-detect `composer.json` in cwd |

---

## Task 1: `ComposerError` and `Psr4Map` skeleton

**Files:**
- Create: `crates/mir-analyzer/src/composer.rs`
- Modify: `crates/mir-analyzer/src/lib.rs`

- [ ] **Step 1: Create `composer.rs` with the error type and empty struct**

Create `crates/mir-analyzer/src/composer.rs`:

```rust
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
```

- [ ] **Step 2: Register the module in `lib.rs`**

In `crates/mir-analyzer/src/lib.rs`, add after the existing `pub mod` lines:

```rust
pub mod composer;
pub use composer::Psr4Map;
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo check -p mir-analyzer 2>&1
```

Expected: compiles with warnings about `todo!()` and unused fields, no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/mir-analyzer/src/composer.rs crates/mir-analyzer/src/lib.rs
git commit -m "feat: add Psr4Map skeleton and ComposerError"
```

---

## Task 2: Parse `composer.json` project entries

**Files:**
- Modify: `crates/mir-analyzer/src/composer.rs`

- [ ] **Step 1: Write the failing tests**

Add at the bottom of `crates/mir-analyzer/src/composer.rs`:

```rust
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

        // All three prefixes present
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

        // First entry must be the longer prefix
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
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p mir-analyzer composer 2>&1
```

Expected: compile error or panics on `todo!()`.

- [ ] **Step 3: Implement `from_composer` for project entries**

Replace the `from_composer` `todo!()` body in `crates/mir-analyzer/src/composer.rs`:

```rust
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

    // autoload.psr-4
    if let Some(map) = value.pointer("/autoload/psr-4").and_then(|v| v.as_object()) {
        for (prefix, dir) in map {
            if let Some(d) = dir.as_str() {
                project_entries.push((ensure_trailing_backslash(prefix), root.join(d)));
            }
        }
    }
    // autoload-dev.psr-4
    if let Some(map) = value.pointer("/autoload-dev/psr-4").and_then(|v| v.as_object()) {
        for (prefix, dir) in map {
            if let Some(d) = dir.as_str() {
                project_entries.push((ensure_trailing_backslash(prefix), root.join(d)));
            }
        }
    }

    // Sort longest-prefix-first
    project_entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    // Vendor entries parsed separately
    let vendor_entries = parse_vendor_entries(root);

    Ok(Psr4Map {
        project_entries,
        vendor_entries,
        root: root.to_path_buf(),
    })
}
```

Add these helpers at module level (outside `impl`):

```rust
/// Ensure namespace prefix ends with `\`.
fn ensure_trailing_backslash(prefix: &str) -> String {
    if prefix.ends_with('\\') {
        prefix.to_string()
    } else {
        format!("{}\\", prefix)
    }
}

/// Parse vendor entries from `vendor/composer/installed.json`.
/// Returns an empty vec if the file does not exist (vendor not installed).
fn parse_vendor_entries(root: &Path) -> Vec<(String, PathBuf)> {
    let installed_path = root.join("vendor/composer/installed.json");
    let content = match std::fs::read_to_string(&installed_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(), // silently ignore missing file
    };
    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // Composer v2: { "packages": [...] }  — v1: top-level array
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
            // Package install path: vendor/<name>
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
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p mir-analyzer composer::tests::parse_project_entries composer::tests::longest_prefix_first composer::tests::missing_autoload_section_is_error 2>&1
```

Expected: all 3 pass.

- [ ] **Step 5: Commit**

```bash
git add crates/mir-analyzer/src/composer.rs
git commit -m "feat: parse composer.json project and vendor PSR-4 entries"
```

---

## Task 3: Vendor entry tests (`installed.json` v1 and v2)

**Files:**
- Modify: `crates/mir-analyzer/src/composer.rs` (tests only)

- [ ] **Step 1: Add vendor tests to the `tests` module**

Append inside the `#[cfg(test)] mod tests` block:

```rust
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
        // Create the vendor package src dir so files() won't error
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
        // No vendor/composer/installed.json — must not error
        let map = Psr4Map::from_composer(&root).unwrap();
        assert!(map.vendor_entries.is_empty());
    }
```

- [ ] **Step 2: Run tests**

```bash
cargo test -p mir-analyzer composer::tests::composer_v2_installed composer::tests::composer_v1_installed composer::tests::missing_installed_json 2>&1
```

Expected: all 3 pass.

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/src/composer.rs
git commit -m "test: add installed.json v1/v2 and missing-file tests"
```

---

## Task 4: `project_files()` and `vendor_files()`

**Files:**
- Modify: `crates/mir-analyzer/src/composer.rs`
- Modify: `crates/mir-analyzer/src/project.rs` (expose `collect_php_files` as `pub(crate)`)

- [ ] **Step 1: Expose `collect_php_files` in `project.rs`**

In `crates/mir-analyzer/src/project.rs`, find `fn collect_php_files` and change visibility:

```rust
pub(crate) fn collect_php_files(dir: &Path, out: &mut Vec<PathBuf>) {
```

- [ ] **Step 2: Write failing tests for `project_files()`**

Append inside the `#[cfg(test)] mod tests` block in `composer.rs`:

```rust
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
```

- [ ] **Step 3: Run test to verify it fails**

```bash
cargo test -p mir-analyzer composer::tests::project_files_returns_php_files 2>&1
```

Expected: panic on `todo!()`.

- [ ] **Step 4: Implement `project_files()` and `vendor_files()`**

Replace the two `todo!()` bodies in `crates/mir-analyzer/src/composer.rs`:

```rust
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
```

- [ ] **Step 5: Run tests**

```bash
cargo test -p mir-analyzer composer 2>&1
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add crates/mir-analyzer/src/composer.rs crates/mir-analyzer/src/project.rs
git commit -m "feat: implement project_files() and vendor_files() on Psr4Map"
```

---

## Task 5: `resolve()`

**Files:**
- Modify: `crates/mir-analyzer/src/composer.rs`

- [ ] **Step 1: Write failing tests**

Append inside `#[cfg(test)] mod tests`:

```rust
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
        let result = map.resolve("App\\Models\\User"); // file doesn't exist
        assert!(result.is_none());
    }

    #[test]
    fn boundary_check() {
        let root = make_temp_project("boundary_check");
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            root.join("composer.json"),
            r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
        )
        .unwrap();

        let map = Psr4Map::from_composer(&root).unwrap();
        // "App\" must NOT match "Application\Foo"
        let result = map.resolve("Application\\Foo");
        assert!(result.is_none(), "App\\ prefix must not match Application\\Foo");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test -p mir-analyzer composer::tests::resolve_existing_file composer::tests::resolve_missing_file composer::tests::boundary_check 2>&1
```

Expected: panic on `todo!()`.

- [ ] **Step 3: Implement `resolve()`**

Replace the `resolve` `todo!()` body:

```rust
pub fn resolve(&self, fqcn: &str) -> Option<PathBuf> {
    // Search project entries first, then vendor entries
    for (prefix, dir) in self.project_entries.iter().chain(self.vendor_entries.iter()) {
        // Boundary-safe match: fqcn must start with the full prefix (which already ends with \)
        if fqcn.starts_with(prefix.as_str()) {
            let relative = &fqcn[prefix.len()..];
            let file_path = dir.join(relative.replace('\\', "/")).with_extension("php");
            if file_path.exists() {
                return Some(file_path);
            }
        }
    }
    None
}
```

- [ ] **Step 4: Run tests**

```bash
cargo test -p mir-analyzer composer 2>&1
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add crates/mir-analyzer/src/composer.rs
git commit -m "feat: implement Psr4Map::resolve() with boundary-safe prefix matching"
```

---

## Task 6: `ProjectAnalyzer` changes

**Files:**
- Modify: `crates/mir-analyzer/src/project.rs`

- [ ] **Step 1: Add `psr4` field and update constructors**

In `crates/mir-analyzer/src/project.rs`, update the struct and both constructors:

```rust
pub struct ProjectAnalyzer {
    pub codebase: Arc<Codebase>,
    pub cache: Option<AnalysisCache>,
    pub on_file_done: Option<Arc<dyn Fn() + Send + Sync>>,
    pub psr4: Option<Arc<crate::composer::Psr4Map>>,
    stubs_loaded: std::sync::atomic::AtomicBool,
}
```

Update `new()`:

```rust
pub fn new() -> Self {
    Self {
        codebase: Arc::new(Codebase::new()),
        cache: None,
        on_file_done: None,
        psr4: None,
        stubs_loaded: std::sync::atomic::AtomicBool::new(false),
    }
}
```

Update `with_cache()`:

```rust
pub fn with_cache(cache_dir: &Path) -> Self {
    Self {
        codebase: Arc::new(Codebase::new()),
        cache: Some(AnalysisCache::open(cache_dir)),
        on_file_done: None,
        psr4: None,
        stubs_loaded: std::sync::atomic::AtomicBool::new(false),
    }
}
```

- [ ] **Step 2: Add `from_composer()` constructor**

Add after `with_cache()`:

```rust
/// Create a `ProjectAnalyzer` from a project root containing `composer.json`.
/// Returns the analyzer (with `psr4` set) and the `Psr4Map` so callers can
/// call `map.project_files()` / `map.vendor_files()`.
pub fn from_composer(
    root: &Path,
) -> Result<(Self, crate::composer::Psr4Map), crate::composer::ComposerError> {
    let map = crate::composer::Psr4Map::from_composer(root)?;
    let psr4 = Arc::new(crate::composer::Psr4Map::from_composer(root)?);
    // Note: map and psr4 are two separate parses of the same files. This is
    // intentional — map is returned to the caller for file listing, psr4 is
    // stored on the analyzer for future lazy resolution (issue #50).
    // Both parses are fast (JSON reads) so the duplication is acceptable.
    let analyzer = Self {
        codebase: Arc::new(Codebase::new()),
        cache: None,
        on_file_done: None,
        psr4: Some(psr4),
        stubs_loaded: std::sync::atomic::AtomicBool::new(false),
    };
    Ok((analyzer, map))
}
```

- [ ] **Step 3: Verify it compiles**

```bash
cargo check -p mir-analyzer 2>&1
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add crates/mir-analyzer/src/project.rs
git commit -m "feat: add psr4 field and ProjectAnalyzer::from_composer()"
```

---

## Task 7: CLI auto-detection

**Files:**
- Modify: `crates/mir-cli/src/main.rs`

- [ ] **Step 1: Inline the composer branch**

The cleanest approach is to add an early-return branch before the existing path discovery. In `main()`, after the rayon thread pool setup (around line 155), insert this block and then update the existing `let paths:` line to use `cwd`:

Replace the entire body of `main()` after the config loading block with:

```rust
    // Configure rayon thread pool
    if let Some(n) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // --- Composer auto-detection -------------------------------------------
    if cli.paths.is_empty() && cwd.join("composer.json").exists() {
        let (mut analyzer, map) = match ProjectAnalyzer::from_composer(&cwd) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("mir: composer error: {}", e);
                std::process::exit(2);
            }
        };

        let vendor_files = map.vendor_files();
        let files = map.project_files();

        if files.is_empty() {
            if !cli.quiet {
                eprintln!("No PHP files found via composer.json.");
            }
            process::exit(0);
        }

        if !cli.quiet {
            eprintln!(
                "{} Analyzing {} file{} (from composer.json)...",
                "mir".bold().green(),
                files.len(),
                if files.len() == 1 { "" } else { "s" },
            );
        }

        analyzer.load_stubs();

        if !vendor_files.is_empty() {
            if !cli.quiet {
                eprintln!("mir: scanning {} vendor files for types...", vendor_files.len());
            }
            analyzer.collect_types_only(&vendor_files);
        }

        let show_progress = !cli.no_progress && !cli.quiet && matches!(cli.format, OutputFormat::Text);
        let start = std::time::Instant::now();
        if show_progress {
            let pb = Arc::new(
                ProgressBar::new(files.len() as u64).with_style(
                    ProgressStyle::with_template(
                        "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files {elapsed_precise}",
                    )
                    .unwrap_or_else(|_| ProgressStyle::default_bar())
                    .progress_chars("=> "),
                ),
            );
            let pb2 = pb.clone();
            analyzer.on_file_done = Some(Arc::new(move || { pb2.inc(1); }));
            let result = analyzer.analyze(&files);
            let elapsed = start.elapsed();
            pb.finish_and_clear();
            let baseline = load_baseline(&cli, &config);
            run_output(&cli, &config, &files, result, baseline, elapsed);
        } else {
            let result = analyzer.analyze(&files);
            let elapsed = start.elapsed();
            let baseline = load_baseline(&cli, &config);
            run_output(&cli, &config, &files, result, baseline, elapsed);
        }
        return;
    }

    // --- Existing explicit-path discovery (unchanged) ----------------------
    let paths: Vec<PathBuf> = if cli.paths.is_empty() {
        vec![cwd.clone()]
    } else {
        cli.paths.clone()
    };
    // ... rest of existing main() body unchanged ...
```

- [ ] **Step 2: Build and verify**

```bash
cargo build -p mir-cli 2>&1
```

Expected: builds cleanly.

- [ ] **Step 3: Commit**

```bash
git add crates/mir-cli/src/main.rs
git commit -m "feat: auto-detect composer.json in CLI when no paths given"
```

---

## Task 8: Full build and test run

- [ ] **Step 1: Run all tests**

```bash
cargo test --workspace 2>&1
```

Expected: all tests pass.

- [ ] **Step 2: Build release binary**

```bash
cargo build --release 2>&1
```

Expected: no errors or warnings about unused fields.

- [ ] **Step 3: Smoke test against app-server if available**

```bash
cd /Users/adamspychala/dev/app-server && /Users/adamspychala/Projects/mir/target/release/mir 2>&1 | head -20
```

Expected: mir discovers and analyzes files via composer.json, reports issues.
