//! Test utilities for fixture-based testing.
//!
//! # Fixture formats
//!
//! **Single-file** (`===file===`, appears exactly once):
//! ```text
//! ===file===
//! <?php
//! ...
//! ===expect===
//! UndefinedMethod: Method Foo::bar() does not exist
//! ```
//!
//! **Multi-file** (`===file:name===`, one or more):
//! ```text
//! ===file:Base.php===
//! <?php
//! class Base { ... }
//! ===file:Child.php===
//! <?php
//! class Child extends Base { ... }
//! ===expect===
//! Child.php: UndefinedMethod: Method Child::bar() does not exist
//! ```
//!
//! **With config** (optional `===config===` section, must appear before file sections):
//! ```text
//! ===config===
//! php_version=8.1
//! suppress=MissingThrowsDocblock,UnusedFunction
//! stub_file=stubs/helpers.php
//! stub_dir=stubs
//! ===file===
//! <?php
//! ...
//! ===expect===
//! ...
//! ```
//!
//! `stub_file=path` and `stub_dir=path` refer to files/directories already declared
//! with `===file:path===` markers. They are passed to `ProjectAnalyzer::stub_files` /
//! `stub_dirs` and excluded from the analysis file list, so only the non-stub PHP
//! files are analysed. Multiple `stub_file=` and `stub_dir=` lines are allowed.
//!
//! **With Composer/PSR-4**:
//! ```text
//! ===file:composer.json===
//! {"autoload":{"psr-4":{"App\\":"src/"}}}
//! ===file:src/Base.php===
//! <?php
//! namespace App;
//! class Base { ... }
//! ===file:Child.php===
//! <?php
//! class Child extends \App\Base { ... }
//! ===expect===
//! Child.php: UndefinedMethod: Method Child::bar() does not exist
//! ```
//!
//! **With description** (optional `===description===` section, must appear before file sections):
//! ```text
//! ===description===
//! Verify that calling a method on a null variable is reported.
//! ===file===
//! <?php
//! ...
//! ===expect===
//! ...
//! ```
//!
//! **Skipped / WIP fixture** (`===ignore===`, must appear before file sections):
//! ```text
//! ===ignore===
//! TODO: narrowing through loop variables not yet implemented
//! ===file===
//! <?php
//! ...
//! ===expect===
//! ...
//! ```
//!
//! The presence of `===ignore===` causes the generated test to be marked
//! `#[ignore]` at compile time (via `build.rs`), so it shows up as `ignored`
//! rather than `ok` or `FAILED` in test output.
//!
//! # Validation rules
//!
//! - `===file===` (bare, no name) must appear **at most once** per fixture.
//! - `===file===` and `===file:name===` cannot appear in the same fixture.
//! - A fixture with no file section at all fails immediately.
//! - `===config===` must appear **at most once** per fixture.
//! - Every key in `===config===` must be a recognised key (`php_version`,
//!   `suppress`, `stub_file`, `stub_dir`); unknown keys fail the test.
//! - `php_version` is parsed via [`PhpVersion::from_str`] (same parser as the
//!   real CLI config); invalid values fail the test.
//! - `suppress` accepts a comma-separated list of [`IssueKind`] names and
//!   **replaces** the analyzer's default suppression set. When the key is
//!   omitted the runner auto-fills the dead-code group unless the
//!   fixture's `===expect===` references one of those kinds, so dead-code
//!   fixtures need no boilerplate and ordinary fixtures don't get unsolicited
//!   `UnusedFunction` noise from their bare top-level functions.
//! - `stub_file` and `stub_dir` accept a relative path (matching a `===file:===` name).
//! - `===description===` must appear **at most once** and before any file section.
//! - `===ignore===` must appear **at most once** and before any file section.
//!
//! # Expect format
//!
//! Single-file fixtures use `KindName@line:col: message`.
//! Multi-file fixtures use `FileName.php: KindName@line:col: message`.
//!
//! Location assertions (`@line:col`) are **required**. Both line and column must be specified
//! and must match for the issue to be considered a match.
//!
//! Set `UPDATE_FIXTURES=1` to rewrite the expect section with actual output (including locations).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::{project::ProjectAnalyzer, PhpVersion};
use mir_issues::{Issue, IssueKind};

static COUNTER: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Fixture configuration
// ---------------------------------------------------------------------------

#[derive(Default)]
struct FixtureConfig {
    php_version: Option<PhpVersion>,
    /// Explicit replacement for the analyzer's default suppression set.
    /// Set from the `suppress=Foo,Bar` config key. When unset, the
    /// analyzer's default suppressions (currently the dead-code group)
    /// apply unchanged. `suppress=` with an empty value clears the set
    /// entirely — that's how a fixture opts in to dead-code reporting.
    suppressed_issue_kinds: Option<rustc_hash::FxHashSet<String>>,
    /// Paths (relative to temp dir) to pass as `analyzer.stub_files`.
    stub_files: Vec<String>,
    /// Paths (relative to temp dir) to pass as `analyzer.stub_dirs`.
    stub_dirs: Vec<String>,
}

// ---------------------------------------------------------------------------
// Public inline-analysis API
// ---------------------------------------------------------------------------

/// Run the full analyzer on an inline PHP string and return all unsuppressed issues.
pub fn check(src: &str) -> Vec<Issue> {
    run_analyzer(&[("test.php", src)], &FixtureConfig::default())
}

/// Analyze a set of named PHP files together, returning all unsuppressed issues.
///
/// Each entry is `(filename, php_source)`. Files are written to a unique temp
/// directory, analyzed together, then cleaned up.
///
/// If a `"composer.json"` entry is included, a `Psr4Map` is built from it.
/// Files under PSR-4-mapped directories are left for lazy discovery and are
/// **not** passed to `analyze()` explicitly.
pub fn check_files(files: &[(&str, &str)]) -> Vec<Issue> {
    run_analyzer(files, &FixtureConfig::default())
}

// ---------------------------------------------------------------------------
// Fixture data types
// ---------------------------------------------------------------------------

/// One expected issue from a `.phpt` fixture's `===expect===` section.
pub(crate) struct ExpectedIssue {
    pub file: Option<String>,
    pub kind_name: String,
    pub message: String,
    pub line: Option<u32>,
    pub col_start: Option<u16>,
}

/// Parsed representation of a `.phpt` fixture.
pub(crate) struct ParsedFixture {
    /// `(filename, content)` pairs — always at least one entry.
    pub files: Vec<(String, String)>,
    pub expected: Vec<ExpectedIssue>,
    pub is_multi: bool,
    /// Optional human-readable description from `===description===`.
    pub description: Option<String>,
    config: FixtureConfig,
}

// ---------------------------------------------------------------------------
// Fixture parsing
// ---------------------------------------------------------------------------

const BARE_FILE: &str = "===file===";
const FILE_PREFIX: &str = "===file:";
const CONFIG_MARKER: &str = "===config===";
const EXPECT_MARKER: &str = "===expect===";
const DESCRIPTION_MARKER: &str = "===description===";
const IGNORE_MARKER: &str = "===ignore===";

/// Parse a `.phpt` fixture file.
pub(crate) fn parse_phpt(content: &str, path: &str) -> ParsedFixture {
    // --- Locate expect (required, exactly once) ---
    let expect_count = count_occurrences(content, EXPECT_MARKER);
    assert_eq!(
        expect_count, 1,
        "fixture {path}: ===expect=== must appear exactly once, found {expect_count} times"
    );
    let expect_pos = content.find(EXPECT_MARKER).unwrap();
    let header_region = &content[..expect_pos];
    let expect_content = content[expect_pos + EXPECT_MARKER.len()..].trim();

    // --- Validate config section ---
    let config_count = count_occurrences(header_region, CONFIG_MARKER);
    assert!(
        config_count <= 1,
        "fixture {path}: ===config=== must appear at most once, found {config_count} times"
    );

    // --- Validate description section ---
    let description_count = count_occurrences(header_region, DESCRIPTION_MARKER);
    assert!(
        description_count <= 1,
        "fixture {path}: ===description=== must appear at most once, found {description_count} times"
    );

    // --- Validate ignore marker ---
    let ignore_count = count_occurrences(header_region, IGNORE_MARKER);
    assert!(
        ignore_count <= 1,
        "fixture {path}: ===ignore=== must appear at most once, found {ignore_count} times"
    );

    // --- Count and validate file markers ---
    // Config, description, and ignore must appear before any file marker so their
    // text is never silently included in the PHP source of the first file.
    if config_count == 1 {
        if let (Some(cfg_pos), Some(first_file_pos)) = (
            header_region.find(CONFIG_MARKER),
            header_region.find("===file"),
        ) {
            assert!(
                cfg_pos < first_file_pos,
                "fixture {path}: ===config=== must appear before the first ===file=== / ===file:name=== marker"
            );
        }
    }
    if description_count == 1 {
        if let (Some(desc_pos), Some(first_file_pos)) = (
            header_region.find(DESCRIPTION_MARKER),
            header_region.find("===file"),
        ) {
            assert!(
                desc_pos < first_file_pos,
                "fixture {path}: ===description=== must appear before the first ===file=== / ===file:name=== marker"
            );
        }
    }
    if ignore_count == 1 {
        if let (Some(ignore_pos), Some(first_file_pos)) = (
            header_region.find(IGNORE_MARKER),
            header_region.find("===file"),
        ) {
            assert!(
                ignore_pos < first_file_pos,
                "fixture {path}: ===ignore=== must appear before the first ===file=== / ===file:name=== marker"
            );
        }
    }

    // ---
    let bare_count = count_occurrences(header_region, BARE_FILE);
    // FILE_PREFIX ("===file:") won't match BARE_FILE ("===file===") since after
    // "file" one has ':' and the other '='.
    let named_count = count_occurrences(header_region, FILE_PREFIX);

    assert!(
        !(bare_count > 0 && named_count > 0),
        "fixture {path}: cannot mix ===file=== and ===file:name=== markers in the same fixture"
    );
    assert!(
        bare_count > 0 || named_count > 0,
        "fixture {path}: no ===file=== or ===file:name=== section found"
    );
    assert!(
        bare_count <= 1,
        "fixture {path}: ===file=== must appear at most once, found {bare_count} times"
    );

    let is_multi = named_count > 0;

    // --- Extract file content(s) ---
    let files = if is_multi {
        extract_named_files(header_region, path)
    } else {
        let bare_pos = header_region.find(BARE_FILE).unwrap();
        let src = header_region[bare_pos + BARE_FILE.len()..]
            .trim()
            .to_string();
        vec![("test.php".to_string(), src)]
    };

    // --- Parse config section ---
    let config = if config_count == 1 {
        let cfg_pos = header_region.find(CONFIG_MARKER).unwrap();
        let after_cfg = cfg_pos + CONFIG_MARKER.len();
        // Config body ends at the first ===file marker (bare or named).
        let cfg_end = header_region[after_cfg..]
            .find("===file")
            .map(|r| after_cfg + r)
            .unwrap_or(header_region.len());
        let cfg_text = header_region[after_cfg..cfg_end].trim();
        parse_config_section(cfg_text, path)
    } else {
        FixtureConfig::default()
    };

    // --- Parse description section ---
    let description = if description_count == 1 {
        let desc_pos = header_region.find(DESCRIPTION_MARKER).unwrap();
        let after_desc = desc_pos + DESCRIPTION_MARKER.len();
        // Description body ends at the next section marker.
        let desc_end = header_region[after_desc..]
            .find("===")
            .map(|r| after_desc + r)
            .unwrap_or(header_region.len());
        Some(header_region[after_desc..desc_end].trim().to_string())
    } else {
        None
    };

    // --- Parse expect lines ---
    let expected = expect_content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            if is_multi {
                parse_multi_expect_line(l, path)
            } else {
                parse_single_expect_line(l, path)
            }
        })
        .collect();

    ParsedFixture {
        files,
        expected,
        is_multi,
        description,
        config,
    }
}

fn parse_config_section(text: &str, path: &str) -> FixtureConfig {
    let mut config = FixtureConfig::default();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once('=').unwrap_or_else(|| {
            panic!("fixture {path}: invalid config line {line:?} — expected key=value")
        });
        match key.trim() {
            "php_version" => {
                let v = value.trim().parse::<PhpVersion>().unwrap_or_else(|e| {
                    panic!("fixture {path}: invalid php_version: {e}")
                });
                config.php_version = Some(v);
            }
            "suppress" => {
                let set = config.suppressed_issue_kinds.get_or_insert_with(Default::default);
                for name in value.split(',') {
                    let trimmed = name.trim();
                    if !trimmed.is_empty() {
                        set.insert(trimmed.to_string());
                    }
                }
            }
            "stub_file" => {
                config.stub_files.push(value.trim().to_string());
            }
            "stub_dir" => {
                config.stub_dirs.push(value.trim().to_string());
            }
            other => panic!(
                "fixture {path}: unknown config key {other:?} — valid keys: php_version, suppress, stub_file, stub_dir"
            ),
        }
    }
    config
}

fn extract_named_files(region: &str, path: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut search_from = 0;

    while let Some(marker_rel) = region[search_from..].find(FILE_PREFIX) {
        let marker_abs = search_from + marker_rel;
        let after_prefix = marker_abs + FILE_PREFIX.len();

        let close_rel = region[after_prefix..]
            .find("===")
            .unwrap_or_else(|| panic!("fixture {path}: unclosed ===file: marker"));

        let file_name = region[after_prefix..after_prefix + close_rel].to_string();
        let content_start = after_prefix + close_rel + "===".len();

        let content_end = region[content_start..]
            .find(FILE_PREFIX)
            .map(|r| content_start + r)
            .unwrap_or(region.len());

        let file_content = region[content_start..content_end].trim().to_string();
        files.push((file_name, file_content));
        search_from = content_end;
    }

    files
}

fn parse_single_expect_line(line: &str, path: &str) -> ExpectedIssue {
    let parts: Vec<&str> = line.splitn(2, ": ").collect();
    let (kind_part, message) = match parts.len() {
        2 => (parts[0], parts[1].trim().to_string()),
        1 => (parts[0], String::new()),
        _ => panic!("fixture {path}: invalid expect line {line:?}"),
    };

    let (kind_name, line_col) = if let Some(at_pos) = kind_part.find('@') {
        (
            kind_part[..at_pos].trim().to_string(),
            Some(&kind_part[at_pos + 1..]),
        )
    } else {
        (kind_part.trim().to_string(), None)
    };

    let (line_num, col_start) = if let Some(loc) = line_col {
        let loc_parts: Vec<&str> = loc.split(':').collect();
        if loc_parts.len() == 2 {
            let l = loc_parts[0]
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("fixture {path}: invalid line number in {line:?}"));
            let c = loc_parts[1]
                .parse::<u16>()
                .unwrap_or_else(|_| panic!("fixture {path}: invalid column number in {line:?}"));
            (Some(l), Some(c))
        } else {
            panic!("fixture {path}: invalid location format in {line:?} — expected \"@line:col\"");
        }
    } else {
        (None, None)
    };

    ExpectedIssue {
        file: None,
        kind_name,
        message,
        line: line_num,
        col_start,
    }
}

fn parse_multi_expect_line(line: &str, path: &str) -> ExpectedIssue {
    let parts: Vec<&str> = line.splitn(3, ": ").collect();
    assert!(
        parts.len() >= 2,
        "fixture {path}: invalid multi-file expect line {line:?} — expected \"FileName.php: KindName[@line:col][ : message]\""
    );

    let kind_part = parts[1];
    let message = if parts.len() >= 3 {
        parts[2].trim().to_string()
    } else {
        String::new()
    };

    let (kind_name, line_col) = if let Some(at_pos) = kind_part.find('@') {
        (
            kind_part[..at_pos].trim().to_string(),
            Some(&kind_part[at_pos + 1..]),
        )
    } else {
        (kind_part.trim().to_string(), None)
    };

    let (line_num, col_start) = if let Some(loc) = line_col {
        let loc_parts: Vec<&str> = loc.split(':').collect();
        if loc_parts.len() == 2 {
            let l = loc_parts[0]
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("fixture {path}: invalid line number in {line:?}"));
            let c = loc_parts[1]
                .parse::<u16>()
                .unwrap_or_else(|_| panic!("fixture {path}: invalid column number in {line:?}"));
            (Some(l), Some(c))
        } else {
            panic!("fixture {path}: invalid location format in {line:?} — expected \"@line:col\"");
        }
    } else {
        (None, None)
    };

    ExpectedIssue {
        file: Some(parts[0].trim().to_string()),
        kind_name,
        message,
        line: line_num,
        col_start,
    }
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        count += 1;
        start += pos + needle.len();
    }
    count
}

// ---------------------------------------------------------------------------
// Fixture runner
// ---------------------------------------------------------------------------

/// Run a `.phpt` fixture file and assert issues match the `===expect===` section.
///
/// Set `UPDATE_FIXTURES=1` to rewrite the expect section with actual output.
pub fn run_fixture(path: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"));

    let mut fixture = parse_phpt(&content, path);
    // Auto-suppression: fixtures that don't expect any dead-code diagnostic
    // get the dead-code group silently suppressed, so authors don't have to
    // sprinkle boilerplate `suppress=` lines on every fixture whose example
    // code happens to declare an uncalled global function. Fixtures that
    // explicitly set `suppress=...` keep their replacement semantics; this
    // only fills in the default.
    if fixture.config.suppressed_issue_kinds.is_none() {
        let dead = crate::project::dead_code_issue_kinds();
        let expects_dead_code = fixture
            .expected
            .iter()
            .any(|e| dead.contains(&e.kind_name.as_str()));
        if !expects_dead_code {
            fixture.config.suppressed_issue_kinds =
                Some(dead.iter().map(|s| (*s).to_string()).collect());
        }
    }
    let file_refs: Vec<(&str, &str)> = fixture
        .files
        .iter()
        .map(|(n, s)| (n.as_str(), s.as_str()))
        .collect();
    let actual = run_analyzer(&file_refs, &fixture.config);

    if std::env::var("UPDATE_FIXTURES").as_deref() == Ok("1") {
        rewrite_fixture(path, &content, &actual, fixture.is_multi);
        return;
    }

    assert_fixture(path, &fixture, &actual);
}

// ---------------------------------------------------------------------------
// Core analyzer runner
// ---------------------------------------------------------------------------

fn run_analyzer(files: &[(&str, &str)], config: &FixtureConfig) -> Vec<Issue> {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_dir = std::env::temp_dir().join(format!("mir_fixture_{id}"));
    std::fs::create_dir_all(&tmp_dir)
        .unwrap_or_else(|e| panic!("failed to create temp dir {}: {e}", tmp_dir.display()));

    let paths: Vec<PathBuf> = files
        .iter()
        .map(|(name, src)| {
            let path = tmp_dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .unwrap_or_else(|e| panic!("failed to create dir for {name}: {e}"));
            }
            std::fs::write(&path, src).unwrap_or_else(|e| panic!("failed to write {name}: {e}"));
            path
        })
        .collect();

    let tmp_dir_str = tmp_dir.to_string_lossy().into_owned();

    let mut analyzer = ProjectAnalyzer::new();
    if let Some(explicit) = &config.suppressed_issue_kinds {
        analyzer.suppressed_issue_kinds = explicit.clone();
    }
    if let Some(version) = config.php_version {
        analyzer = analyzer.with_php_version(version);
    }

    // Register user stub files and directories from the fixture config.
    for stub_file in &config.stub_files {
        analyzer.stub_files.push(tmp_dir.join(stub_file));
    }
    for stub_dir in &config.stub_dirs {
        analyzer.stub_dirs.push(tmp_dir.join(stub_dir));
    }

    // Build a set of paths that belong to user stubs so they are excluded from
    // the list of files passed to `analyze()` (stubs are loaded separately).
    let stub_file_set: HashSet<PathBuf> =
        config.stub_files.iter().map(|f| tmp_dir.join(f)).collect();
    let stub_dir_set: Vec<PathBuf> = config.stub_dirs.iter().map(|d| tmp_dir.join(d)).collect();
    let is_stub = |p: &PathBuf| -> bool {
        stub_file_set.contains(p) || stub_dir_set.iter().any(|d| p.starts_with(d))
    };

    let has_composer = files.iter().any(|(name, _)| *name == "composer.json");
    let explicit_paths: Vec<PathBuf> = if has_composer {
        match crate::composer::Psr4Map::from_composer(&tmp_dir) {
            Ok(psr4) => {
                let psr4 = Arc::new(psr4);
                let psr4_files: HashSet<PathBuf> = psr4.project_files().into_iter().collect();
                let explicit: Vec<PathBuf> = paths
                    .iter()
                    .filter(|p| p.extension().map(|e| e == "php").unwrap_or(false))
                    .filter(|p| !psr4_files.contains(*p) && !is_stub(p))
                    .cloned()
                    .collect();
                analyzer.psr4 = Some(psr4);
                explicit
            }
            Err(_) => php_files_only(&paths)
                .into_iter()
                .filter(|p| !is_stub(p))
                .collect(),
        }
    } else {
        php_files_only(&paths)
            .into_iter()
            .filter(|p| !is_stub(p))
            .collect()
    };

    // Re-borrow the analyzer's effective suppression set so the stub-side
    // filter below knows whether the dead-code pass actually ran.
    let dead_code_enabled = crate::project::dead_code_issue_kinds()
        .iter()
        .any(|k| !analyzer.suppressed_issue_kinds.contains(*k));

    let result = analyzer.analyze(&explicit_paths);
    std::fs::remove_dir_all(&tmp_dir).ok();

    result
        .issues
        .into_iter()
        .filter(|i| !i.suppressed)
        // When dead-code analysis runs, the analyzer walks the entire
        // codebase including stubs. Filter to issues from the temp directory
        // only so stub-side false positives don't pollute fixture output.
        .filter(|i| {
            !dead_code_enabled || i.location.file.as_ref().starts_with(tmp_dir_str.as_str())
        })
        .collect()
}

fn php_files_only(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|p| p.extension().map(|e| e == "php").unwrap_or(false))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Fixture assertion
// ---------------------------------------------------------------------------

fn assert_fixture(path: &str, fixture: &ParsedFixture, actual: &[Issue]) {
    let mut failures: Vec<String> = Vec::new();

    for exp in &fixture.expected {
        if exp.line.is_none() || exp.col_start.is_none() {
            failures.push(format!(
                "  MISSING LOCATION  {}: expected issue must include @line:col (e.g., {}@1:1: {})",
                exp.kind_name, exp.kind_name, exp.message
            ));
        }
        if !actual.iter().any(|a| issue_matches(a, exp)) {
            failures.push(format!(
                "  MISSING  {}",
                fmt_expected(exp, fixture.is_multi)
            ));
        }
    }

    for act in actual {
        if !fixture.expected.iter().any(|e| issue_matches(act, e)) {
            failures.push(format!(
                "  UNEXPECTED {}",
                fmt_actual(act, fixture.is_multi)
            ));
        }
    }

    if !failures.is_empty() {
        let desc = fixture
            .description
            .as_deref()
            .map(|d| format!("\n\nDescription: {d}"))
            .unwrap_or_default();
        panic!(
            "fixture {path} FAILED:{desc}\n{}\n\nTo fix: ensure all expected issues have @line:col locations, then run: UPDATE_FIXTURES=1 cargo test --lib fixture\n\nAll actual issues:\n{}",
            failures.join("\n"),
            fmt_issues(actual, fixture.is_multi)
        );
    }
}

fn issue_matches(actual: &Issue, expected: &ExpectedIssue) -> bool {
    if actual.kind.name() != expected.kind_name {
        return false;
    }
    if actual.kind.message() != expected.message.as_str() {
        return false;
    }
    if let Some(expected_file) = &expected.file {
        let actual_basename = Path::new(actual.location.file.as_ref())
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        if actual_basename.as_ref() != expected_file.as_str() {
            return false;
        }
    }
    if let Some(line) = expected.line {
        if actual.location.line != line {
            return false;
        }
    }
    if let Some(col) = expected.col_start {
        if actual.location.col_start != col {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// UPDATE_FIXTURES rewrite
// ---------------------------------------------------------------------------

fn rewrite_fixture(path: &str, content: &str, actual: &[Issue], is_multi: bool) {
    // Preserve everything before ===expect=== and rewrite only the expect section.
    let exp_pos = content
        .find(EXPECT_MARKER)
        .expect("fixture missing ===expect===");

    let mut out = content[..exp_pos].to_string();
    out.push_str(EXPECT_MARKER);
    out.push('\n');

    let mut sorted: Vec<&Issue> = actual.iter().collect();
    if is_multi {
        sorted.sort_by_key(|i| {
            let basename = Path::new(i.location.file.as_ref())
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            (
                basename,
                i.location.line,
                i.location.col_start,
                i.kind.name(),
            )
        });
        for issue in sorted {
            let basename = Path::new(issue.location.file.as_ref())
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            out.push_str(&format!(
                "{}: {}@{}:{}: {}\n",
                basename,
                issue.kind.name(),
                issue.location.line,
                issue.location.col_start,
                issue.kind.message()
            ));
        }
    } else {
        sorted.sort_by_key(|i| (i.location.line, i.location.col_start, i.kind.name()));
        for issue in sorted {
            out.push_str(&format!(
                "{}@{}:{}: {}\n",
                issue.kind.name(),
                issue.location.line,
                issue.location.col_start,
                issue.kind.message()
            ));
        }
    }

    std::fs::write(path, &out).unwrap_or_else(|e| panic!("failed to write fixture {path}: {e}"));
}

// ---------------------------------------------------------------------------
// Assertion helpers (used by inline tests)
// ---------------------------------------------------------------------------

/// Assert that `issues` contains at least one issue with the exact `IssueKind`
/// at `line` and `col_start`.
pub fn assert_issue(issues: &[Issue], kind: IssueKind, line: u32, col_start: u16) {
    let found = issues
        .iter()
        .any(|i| i.kind == kind && i.location.line == line && i.location.col_start == col_start);
    if !found {
        panic!(
            "Expected issue {:?} at line {line}, col {col_start}.\nActual issues:\n{}",
            kind,
            fmt_issues(issues, false),
        );
    }
}

/// Assert that `issues` contains at least one issue whose `kind.name()` equals
/// `kind_name` at `line` and `col_start`.
pub fn assert_issue_kind(issues: &[Issue], kind_name: &str, line: u32, col_start: u16) {
    let found = issues.iter().any(|i| {
        i.kind.name() == kind_name && i.location.line == line && i.location.col_start == col_start
    });
    if !found {
        panic!(
            "Expected issue {kind_name} at line {line}, col {col_start}.\nActual issues:\n{}",
            fmt_issues(issues, false),
        );
    }
}

/// Assert that `issues` contains no issue whose `kind.name()` equals `kind_name`.
pub fn assert_no_issue(issues: &[Issue], kind_name: &str) {
    let found: Vec<_> = issues
        .iter()
        .filter(|i| i.kind.name() == kind_name)
        .collect();
    if !found.is_empty() {
        panic!(
            "Expected no {kind_name} issues, but found:\n{}",
            fmt_issues(&found.into_iter().cloned().collect::<Vec<_>>(), false),
        );
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn fmt_expected(exp: &ExpectedIssue, is_multi: bool) -> String {
    let kind_with_loc = if let (Some(line), Some(col)) = (exp.line, exp.col_start) {
        format!("{}@{}:{}", exp.kind_name, line, col)
    } else {
        exp.kind_name.clone()
    };

    if is_multi {
        if let Some(f) = &exp.file {
            return format!("{}: {}: {}", f, kind_with_loc, exp.message);
        }
    }
    format!("{}: {}", kind_with_loc, exp.message)
}

fn fmt_actual(act: &Issue, is_multi: bool) -> String {
    if is_multi {
        let basename = Path::new(act.location.file.as_ref())
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        return format!(
            "{}: {}@{}:{}: {}",
            basename,
            act.kind.name(),
            act.location.line,
            act.location.col_start,
            act.kind.message()
        );
    }
    format!(
        "{}@{}:{}: {}",
        act.kind.name(),
        act.location.line,
        act.location.col_start,
        act.kind.message()
    )
}

fn fmt_issues(issues: &[Issue], is_multi: bool) -> String {
    if issues.is_empty() {
        return "  (none)".to_string();
    }
    issues
        .iter()
        .map(|i| format!("  {}", fmt_actual(i, is_multi)))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Fixture parser validation tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod parser_validation {
    use super::{parse_phpt, ParsedFixture};

    fn p(content: &str) -> ParsedFixture {
        parse_phpt(content, "<test>")
    }

    #[test]
    #[should_panic(expected = "===file=== must appear at most once")]
    fn duplicate_bare_file_marker() {
        p("===file===\n<?php\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "cannot mix ===file=== and ===file:name===")]
    fn mixed_bare_and_named_markers() {
        p("===file===\n<?php\n===file:Other.php===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===config=== must appear at most once")]
    fn duplicate_config_section() {
        p("===config===\nsuppress=Foo\n===config===\nsuppress=Bar\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "unknown config key")]
    fn unknown_config_key() {
        p("===config===\nfoo=bar\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "invalid php_version")]
    fn invalid_php_version() {
        p("===config===\nphp_version=banana\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===config=== must appear before the first ===file===")]
    fn config_after_file_marker() {
        p("===file===\n<?php\n===config===\nsuppress=Foo\n===expect===\n");
    }

    #[test]
    fn valid_config_is_accepted() {
        p("===config===\nphp_version=8.1\nsuppress=Foo,Bar\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===description=== must appear at most once")]
    fn duplicate_description_section() {
        p("===description===\nfoo\n===description===\nbar\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===description=== must appear before the first ===file===")]
    fn description_after_file_marker() {
        p("===file===\n<?php\n===description===\nfoo\n===expect===\n");
    }

    #[test]
    fn valid_description_is_accepted() {
        let f = p("===description===\nChecks null method call.\n===file===\n<?php\n===expect===\n");
        assert_eq!(f.description.as_deref(), Some("Checks null method call."));
    }

    #[test]
    #[should_panic(expected = "===ignore=== must appear at most once")]
    fn duplicate_ignore_marker() {
        p("===ignore===\n===ignore===\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===ignore=== must appear before the first ===file===")]
    fn ignore_after_file_marker() {
        p("===file===\n<?php\n===ignore===\n===expect===\n");
    }

    #[test]
    fn valid_ignore_is_accepted() {
        let f = p("===ignore===\nTODO: not yet implemented\n===file===\n<?php\n===expect===\n");
        assert!(f.description.is_none());
    }
}
