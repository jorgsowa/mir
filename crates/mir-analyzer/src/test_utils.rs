//! Test utilities for fixture-based testing.
//!
//! Provides helpers to run `.phpt` fixture files against the analyzer
//! and compare actual vs expected issues.
//!
//! # Fixture formats
//!
//! **Single-file** (original format, 250 existing fixtures):
//! ```text
//! ===source===
//! <?php
//! ...
//! ===expect===
//! UndefinedMethod: Method Foo::bar() does not exist
//! ```
//!
//! **Multi-file** (cross-file scenarios):
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
//! In multi-file fixtures every expect line is prefixed with the originating
//! filename (`Name.php: Kind: message`) for unambiguous attribution.
//!
//! **Multi-file with Composer/PSR-4**:
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
//! When `composer.json` is present, a `Psr4Map` is built from it. Files under
//! PSR-4-mapped directories (e.g. `src/`) are written to disk but **not**
//! passed to `analyze()` — they must be discovered lazily, exactly as they
//! would be in a real project.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::project::ProjectAnalyzer;
use mir_issues::{Issue, IssueKind};

static COUNTER: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Single-file inline analysis
// ---------------------------------------------------------------------------

/// Run the full analyzer on an inline PHP string and return all unsuppressed issues.
pub fn check(src: &str) -> Vec<Issue> {
    check_with_opts(src, false)
}

/// Like [`check`] but also runs the dead-code detector
/// (`UnusedMethod`, `UnusedProperty`).
pub fn check_dead_code(src: &str) -> Vec<Issue> {
    check_with_opts(src, true)
}

fn check_with_opts(src: &str, find_dead_code: bool) -> Vec<Issue> {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp: PathBuf = std::env::temp_dir().join(format!("mir_test_{}.php", id));
    std::fs::write(&tmp, src)
        .unwrap_or_else(|e| panic!("failed to write temp PHP file {}: {}", tmp.display(), e));
    let mut analyzer = ProjectAnalyzer::new();
    analyzer.find_dead_code = find_dead_code;
    let result = analyzer.analyze(std::slice::from_ref(&tmp));
    let tmp_str = tmp.to_string_lossy().into_owned();
    std::fs::remove_file(&tmp).ok();
    result
        .issues
        .into_iter()
        .filter(|i| !i.suppressed)
        // When dead-code analysis is enabled the analyzer walks the entire
        // codebase (including PHP stubs).  Filter to issues originating from
        // the test file only so that stub-side false positives don't pollute
        // the fixture output.
        .filter(|i| !find_dead_code || i.location.file.as_ref() == tmp_str.as_str())
        .collect()
}

// ---------------------------------------------------------------------------
// Multi-file inline analysis
// ---------------------------------------------------------------------------

/// Analyze a set of named PHP files together, returning all unsuppressed issues.
///
/// Each entry is `(filename, php_source)`. Files are written to a unique temp
/// directory, analyzed together, then cleaned up.
///
/// If a `"composer.json"` entry is included, a `Psr4Map` is built from it.
/// Files under PSR-4-mapped directories are left for lazy discovery and are
/// **not** passed to `analyze()` explicitly.
pub fn check_files(files: &[(&str, &str)]) -> Vec<Issue> {
    check_files_with_opts(files, false)
}

/// Like [`check_files`] but also enables the dead-code detector.
pub fn check_files_dead_code(files: &[(&str, &str)]) -> Vec<Issue> {
    check_files_with_opts(files, true)
}

fn check_files_with_opts(files: &[(&str, &str)], find_dead_code: bool) -> Vec<Issue> {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_dir = std::env::temp_dir().join(format!("mir_multi_{}", id));
    std::fs::create_dir_all(&tmp_dir)
        .unwrap_or_else(|e| panic!("failed to create temp dir {}: {}", tmp_dir.display(), e));

    let paths: Vec<PathBuf> = files
        .iter()
        .map(|(name, src)| {
            let path = tmp_dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .unwrap_or_else(|e| panic!("failed to create dir for {}: {}", name, e));
            }
            std::fs::write(&path, src)
                .unwrap_or_else(|e| panic!("failed to write {}: {}", name, e));
            path
        })
        .collect();

    let tmp_dir_str = tmp_dir.to_string_lossy().into_owned();

    let mut analyzer = ProjectAnalyzer::new();
    analyzer.find_dead_code = find_dead_code;

    // When composer.json is present, build a Psr4Map and exclude PSR-4-mapped
    // files from the explicit analysis list so they are discovered lazily.
    let has_composer = files.iter().any(|(name, _)| *name == "composer.json");
    let explicit_paths: Vec<PathBuf> = if has_composer {
        match crate::composer::Psr4Map::from_composer(&tmp_dir) {
            Ok(psr4) => {
                let psr4 = Arc::new(psr4);
                let psr4_files: HashSet<PathBuf> = psr4.project_files().into_iter().collect();
                let explicit: Vec<PathBuf> = paths
                    .iter()
                    .filter(|p| p.extension().map(|e| e == "php").unwrap_or(false))
                    .filter(|p| !psr4_files.contains(*p))
                    .cloned()
                    .collect();
                analyzer.psr4 = Some(psr4);
                explicit
            }
            Err(_) => php_files_only(&paths),
        }
    } else {
        php_files_only(&paths)
    };

    let result = analyzer.analyze(&explicit_paths);
    std::fs::remove_dir_all(&tmp_dir).ok();

    result
        .issues
        .into_iter()
        .filter(|i| !i.suppressed)
        .filter(|i| !find_dead_code || i.location.file.as_ref().starts_with(tmp_dir_str.as_str()))
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
// Fixture data types
// ---------------------------------------------------------------------------

/// One expected issue from a `.phpt` fixture's `===expect===` section.
///
/// - In single-file fixtures `file` is `None`; matching ignores origin.
/// - In multi-file fixtures `file` is `Some("Name.php")`; the issue must
///   originate from that file (matched by basename).
pub struct ExpectedIssue {
    pub file: Option<String>,
    pub kind_name: String,
    pub message: String,
}

/// Parsed representation of a `.phpt` fixture.
pub struct ParsedFixture {
    /// `(filename, content)` pairs — always at least one entry.
    pub files: Vec<(String, String)>,
    pub expected: Vec<ExpectedIssue>,
    pub is_multi: bool,
}

// ---------------------------------------------------------------------------
// Fixture parsing
// ---------------------------------------------------------------------------

/// Parse a `.phpt` fixture file.
///
/// Auto-detects single-file (`===source===`) vs multi-file (`===file:===`) format.
pub fn parse_phpt(content: &str, path: &str) -> ParsedFixture {
    if content.contains("===file:") {
        parse_multi_file(content, path)
    } else {
        parse_single_file(content, path)
    }
}

fn parse_single_file(content: &str, path: &str) -> ParsedFixture {
    const SOURCE: &str = "===source===";
    const EXPECT: &str = "===expect===";

    let src_pos = content
        .find(SOURCE)
        .unwrap_or_else(|| panic!("fixture {} missing ===source=== section", path));
    let exp_pos = content
        .find(EXPECT)
        .unwrap_or_else(|| panic!("fixture {} missing ===expect=== section", path));

    assert!(
        src_pos < exp_pos,
        "fixture {}: ===source=== must come before ===expect===",
        path
    );

    let source = content[src_pos + SOURCE.len()..exp_pos].trim().to_string();
    let expect_section = content[exp_pos + EXPECT.len()..].trim();

    let expected = expect_section
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| parse_single_expect_line(l, path))
        .collect();

    ParsedFixture {
        files: vec![("_test.php".to_string(), source)],
        expected,
        is_multi: false,
    }
}

fn parse_multi_file(content: &str, path: &str) -> ParsedFixture {
    const FILE_PREFIX: &str = "===file:";
    const MARKER_CLOSE: &str = "===";
    const EXPECT: &str = "===expect===";

    let expect_pos = content
        .find(EXPECT)
        .unwrap_or_else(|| panic!("fixture {} missing ===expect=== section", path));

    let files_region = &content[..expect_pos];
    let expect_section = content[expect_pos + EXPECT.len()..].trim();

    let mut files: Vec<(String, String)> = Vec::new();
    let mut search_from = 0;

    while let Some(marker_rel) = files_region[search_from..].find(FILE_PREFIX) {
        let marker_abs = search_from + marker_rel;
        let after_prefix = marker_abs + FILE_PREFIX.len();

        // Locate the closing === of the marker: "===file:Base.php==="
        let close_rel = files_region[after_prefix..]
            .find(MARKER_CLOSE)
            .unwrap_or_else(|| panic!("fixture {}: unclosed ===file: marker", path));

        let file_name = files_region[after_prefix..after_prefix + close_rel].to_string();
        let content_start = after_prefix + close_rel + MARKER_CLOSE.len();

        // Content runs to the next ===file: marker (or end of files_region).
        let content_end = files_region[content_start..]
            .find(FILE_PREFIX)
            .map(|r| content_start + r)
            .unwrap_or(files_region.len());

        let file_content = files_region[content_start..content_end].trim().to_string();
        files.push((file_name, file_content));

        search_from = content_end;
    }

    assert!(
        !files.is_empty(),
        "fixture {}: no ===file:Name=== sections found",
        path
    );

    let expected = expect_section
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| parse_multi_expect_line(l, path))
        .collect();

    ParsedFixture {
        files,
        expected,
        is_multi: true,
    }
}

fn parse_single_expect_line(line: &str, fixture_path: &str) -> ExpectedIssue {
    let parts: Vec<&str> = line.splitn(2, ": ").collect();
    assert_eq!(
        parts.len(),
        2,
        "fixture {}: invalid expect line {:?} — expected \"KindName: message\"",
        fixture_path,
        line
    );
    ExpectedIssue {
        file: None,
        kind_name: parts[0].trim().to_string(),
        message: parts[1].trim().to_string(),
    }
}

fn parse_multi_expect_line(line: &str, fixture_path: &str) -> ExpectedIssue {
    // Format: "FileName.php: KindName: message"
    let parts: Vec<&str> = line.splitn(3, ": ").collect();
    assert_eq!(
        parts.len(),
        3,
        "fixture {}: invalid multi-file expect line {:?} — expected \"FileName.php: KindName: message\"",
        fixture_path,
        line
    );
    ExpectedIssue {
        file: Some(parts[0].trim().to_string()),
        kind_name: parts[1].trim().to_string(),
        message: parts[2].trim().to_string(),
    }
}

// ---------------------------------------------------------------------------
// Fixture runners
// ---------------------------------------------------------------------------

/// Run a `.phpt` fixture file and assert issues match the `===expect===` section.
///
/// Supports single-file (`===source===`), multi-file (`===file:===`), and
/// multi-file with Composer (`===file:composer.json===`).
///
/// Set `UPDATE_FIXTURES=1` to rewrite the expect section with actual output.
pub fn run_fixture(path: &str) {
    run_fixture_with_opts(path, false);
}

/// Like [`run_fixture`] but also enables the dead-code detector.
pub fn run_fixture_dead_code(path: &str) {
    run_fixture_with_opts(path, true);
}

fn run_fixture_with_opts(path: &str, find_dead_code: bool) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path, e));

    if std::env::var("UPDATE_FIXTURES").as_deref() == Ok("1") {
        update_fixture(path, &content, find_dead_code);
        return;
    }

    let fixture = parse_phpt(&content, path);

    let actual = if fixture.is_multi {
        let file_refs: Vec<(&str, &str)> = fixture
            .files
            .iter()
            .map(|(n, s)| (n.as_str(), s.as_str()))
            .collect();
        check_files_with_opts(&file_refs, find_dead_code)
    } else {
        check_with_opts(&fixture.files[0].1, find_dead_code)
    };

    assert_fixture(path, &fixture, &actual);
}

fn assert_fixture(path: &str, fixture: &ParsedFixture, actual: &[Issue]) {
    let mut failures: Vec<String> = Vec::new();

    for exp in &fixture.expected {
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
        panic!(
            "fixture {} FAILED:\n{}\n\nAll actual issues:\n{}",
            path,
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
    true
}

// ---------------------------------------------------------------------------
// UPDATE_FIXTURES rewrite
// ---------------------------------------------------------------------------

fn update_fixture(path: &str, content: &str, find_dead_code: bool) {
    if content.contains("===file:") {
        let fixture = parse_multi_file(content, path);
        let file_refs: Vec<(&str, &str)> = fixture
            .files
            .iter()
            .map(|(n, s)| (n.as_str(), s.as_str()))
            .collect();
        let actual = check_files_with_opts(&file_refs, find_dead_code);
        rewrite_fixture_multi(path, content, &actual);
    } else {
        let source = extract_source_section(content, path);
        let actual = check_with_opts(&source, find_dead_code);
        rewrite_fixture_single(path, content, &actual);
    }
}

fn extract_source_section(content: &str, path: &str) -> String {
    const SOURCE: &str = "===source===";
    const EXPECT: &str = "===expect===";
    let src_pos = content
        .find(SOURCE)
        .unwrap_or_else(|| panic!("fixture {} missing ===source===", path));
    let exp_pos = content
        .find(EXPECT)
        .unwrap_or_else(|| panic!("fixture {} missing ===expect===", path));
    content[src_pos + SOURCE.len()..exp_pos].trim().to_string()
}

fn rewrite_fixture_single(path: &str, content: &str, actual: &[Issue]) {
    const SOURCE: &str = "===source===";
    const EXPECT: &str = "===expect===";

    let src_pos = content.find(SOURCE).expect("missing ===source===");
    let exp_pos = content.find(EXPECT).expect("missing ===expect===");

    let mut out = content[src_pos..exp_pos].to_string();
    out.push_str(EXPECT);
    out.push('\n');

    let mut sorted: Vec<&Issue> = actual.iter().collect();
    sorted.sort_by_key(|i| (i.location.line, i.location.col_start, i.kind.name()));

    for issue in sorted {
        out.push_str(&format!(
            "{}: {}\n",
            issue.kind.name(),
            issue.kind.message()
        ));
    }

    std::fs::write(path, &out)
        .unwrap_or_else(|e| panic!("failed to write fixture {}: {}", path, e));
}

fn rewrite_fixture_multi(path: &str, content: &str, actual: &[Issue]) {
    const EXPECT: &str = "===expect===";

    let exp_pos = content.find(EXPECT).expect("missing ===expect===");

    let mut out = content[..exp_pos].to_string();
    out.push_str(EXPECT);
    out.push('\n');

    let mut sorted: Vec<&Issue> = actual.iter().collect();
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
            "{}: {}: {}\n",
            basename,
            issue.kind.name(),
            issue.kind.message()
        ));
    }

    std::fs::write(path, &out)
        .unwrap_or_else(|e| panic!("failed to write fixture {}: {}", path, e));
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
            "Expected issue {:?} at line {}, col {}.\nActual issues:\n{}",
            kind,
            line,
            col_start,
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
            "Expected issue {} at line {}, col {}.\nActual issues:\n{}",
            kind_name,
            line,
            col_start,
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
            "Expected no {} issues, but found:\n{}",
            kind_name,
            fmt_issues(&found.into_iter().cloned().collect::<Vec<_>>(), false),
        );
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn fmt_expected(exp: &ExpectedIssue, is_multi: bool) -> String {
    if is_multi {
        if let Some(f) = &exp.file {
            return format!("{}: {}: {}", f, exp.kind_name, exp.message);
        }
    }
    format!("{}: {}", exp.kind_name, exp.message)
}

fn fmt_actual(act: &Issue, is_multi: bool) -> String {
    if is_multi {
        let basename = Path::new(act.location.file.as_ref())
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        return format!("{}: {}: {}", basename, act.kind.name(), act.kind.message());
    }
    format!("{}: {}", act.kind.name(), act.kind.message())
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
