use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use mir_analyzer::project::ProjectAnalyzer;
use mir_issues::{Issue, IssueKind};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Run the full analyzer on an inline PHP string.
/// Creates a unique temp file, analyzes it, deletes it, and returns all
/// unsuppressed issues.
pub fn check(src: &str) -> Vec<Issue> {
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp: PathBuf = std::env::temp_dir().join(format!("mir_test_{}.php", id));
    std::fs::write(&tmp, src)
        .unwrap_or_else(|e| panic!("failed to write temp PHP file {}: {}", tmp.display(), e));
    let result = ProjectAnalyzer::new().analyze(std::slice::from_ref(&tmp));
    std::fs::remove_file(&tmp).ok();
    result
        .issues
        .into_iter()
        .filter(|i| !i.suppressed)
        .collect()
}

// ---------------------------------------------------------------------------
// Fixture-based test support
// ---------------------------------------------------------------------------

/// One expected issue from a `.phpt` fixture's `===expect===` section.
///
/// Format: `KindName: snippet`
pub struct ExpectedIssue {
    pub kind_name: String,
    pub snippet: String,
}

/// Parse a `.phpt` fixture file into `(php_source, expected_issues)`.
///
/// Fixture format:
/// ```text
/// ===source===
/// <?php
/// ...
/// ===expect===
/// UndefinedClass: UnknownClass
/// UndefinedFunction: foo()
/// ```
/// An empty `===expect===` section means no issues are expected.
pub fn parse_phpt(content: &str, path: &str) -> (String, Vec<ExpectedIssue>) {
    let source_marker = "===source===";
    let expect_marker = "===expect===";

    let source_pos = content
        .find(source_marker)
        .unwrap_or_else(|| panic!("fixture {} missing ===source=== section", path));
    let expect_pos = content
        .find(expect_marker)
        .unwrap_or_else(|| panic!("fixture {} missing ===expect=== section", path));

    assert!(
        source_pos < expect_pos,
        "fixture {}: ===source=== must come before ===expect===",
        path
    );

    let source = content[source_pos + source_marker.len()..expect_pos]
        .trim()
        .to_string();
    let expect_section = content[expect_pos + expect_marker.len()..].trim();

    let expected: Vec<ExpectedIssue> = expect_section
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| parse_expected_line(l, path))
        .collect();

    (source, expected)
}

/// Extract only the source section from a fixture file (used in UPDATE_FIXTURES mode
/// to avoid parsing potentially stale/old-format expect sections).
fn parse_phpt_source_only(content: &str, path: &str) -> String {
    let source_marker = "===source===";
    let expect_marker = "===expect===";

    let source_pos = content
        .find(source_marker)
        .unwrap_or_else(|| panic!("fixture {} missing ===source=== section", path));
    let expect_pos = content
        .find(expect_marker)
        .unwrap_or_else(|| panic!("fixture {} missing ===expect=== section", path));

    content[source_pos + source_marker.len()..expect_pos]
        .trim()
        .to_string()
}

fn parse_expected_line(line: &str, fixture_path: &str) -> ExpectedIssue {
    // Format: "KindName: snippet"
    let parts: Vec<&str> = line.splitn(2, ": ").collect();
    assert_eq!(
        parts.len(),
        2,
        "fixture {}: invalid expect line {:?} — expected \"KindName: snippet\"",
        fixture_path,
        line
    );
    ExpectedIssue {
        kind_name: parts[0].trim().to_string(),
        snippet: parts[1].trim().to_string(),
    }
}

/// Run a `.phpt` fixture file: parse, analyze, and assert the issues match
/// the `===expect===` section exactly (no missing, no unexpected).
///
/// If the environment variable `UPDATE_FIXTURES` is set to `1`, the fixture
/// file is rewritten with the actual issues instead of asserting.
///
/// Called by the auto-generated test functions in `build.rs`.
pub fn run_fixture(path: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path, e));

    if std::env::var("UPDATE_FIXTURES").as_deref() == Ok("1") {
        let source = parse_phpt_source_only(&content, path);
        let actual = check(&source);
        rewrite_fixture(path, &content, &actual);
        return;
    }

    let (source, expected) = parse_phpt(&content, path);
    let actual = check(&source);

    let mut failures: Vec<String> = Vec::new();

    for exp in &expected {
        let found = actual.iter().any(|a| {
            a.kind.name() == exp.kind_name && a.snippet.as_deref() == Some(exp.snippet.as_str())
        });
        if !found {
            failures.push(format!("  MISSING  {}: {}", exp.kind_name, exp.snippet));
        }
    }

    for act in &actual {
        let expected_it = expected.iter().any(|e| {
            e.kind_name == act.kind.name() && Some(e.snippet.as_str()) == act.snippet.as_deref()
        });
        if !expected_it {
            let snippet = act.snippet.as_deref().unwrap_or("<no snippet>");
            failures.push(format!(
                "  UNEXPECTED {}: {}  — {}",
                act.kind.name(),
                snippet,
                act.kind.message(),
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "fixture {} FAILED:\n{}\n\nAll actual issues:\n{}",
            path,
            failures.join("\n"),
            fmt_issues(&actual)
        );
    }
}

/// Rewrite the fixture file's `===expect===` section with the actual issues.
/// Preserves the `===source===` section unchanged.
fn rewrite_fixture(path: &str, content: &str, actual: &[Issue]) {
    let source_marker = "===source===";
    let expect_marker = "===expect===";

    let source_pos = content.find(source_marker).expect("missing ===source===");
    let expect_pos = content.find(expect_marker).expect("missing ===expect===");

    let source_section = &content[source_pos..expect_pos];

    let mut new_content = String::new();
    new_content.push_str(source_section);
    new_content.push_str(expect_marker);
    new_content.push('\n');

    // Sort issues by (line, col, kind) for deterministic output.
    let mut sorted: Vec<&Issue> = actual.iter().collect();
    sorted.sort_by_key(|i| (i.location.line, i.location.col_start, i.kind.name()));

    for issue in sorted {
        let snippet = issue.snippet.as_deref().unwrap_or("<no snippet>");
        new_content.push_str(&format!("{}: {}\n", issue.kind.name(), snippet));
    }

    std::fs::write(path, &new_content)
        .unwrap_or_else(|e| panic!("failed to write fixture {}: {}", path, e));
}

/// Generate a `#[test]` function that runs a `.phpt` fixture file.
///
/// The path is relative to the crate's `tests/fixtures/` directory.
///
/// # Example
/// ```rust,ignore
/// fixture_test!(new_unknown_class, "undefined_class/new_unknown_class.phpt");
/// ```
#[macro_export]
macro_rules! fixture_test {
    ($name:ident, $path:expr) => {
        #[test]
        fn $name() {
            mir_test_utils::run_fixture(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/",
                $path
            ));
        }
    };
}

// ---------------------------------------------------------------------------
// Assertion helpers (used by inline tests)
// ---------------------------------------------------------------------------

/// Assert that `issues` contains at least one issue with the exact `IssueKind`
/// at `line` and `col_start`. Panics with the full issue list on failure.
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
            fmt_issues(issues),
        );
    }
}

/// Assert that `issues` contains at least one issue whose `kind.name()` equals
/// `kind_name`, at `line` and `col_start`. Use this when the exact IssueKind
/// field values are complex (e.g. type-format strings in InvalidArgument).
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
            fmt_issues(issues),
        );
    }
}

/// Assert that `issues` contains no issue whose `kind.name()` equals `kind_name`.
/// Panics with the matching issues on failure.
pub fn assert_no_issue(issues: &[Issue], kind_name: &str) {
    let found: Vec<_> = issues
        .iter()
        .filter(|i| i.kind.name() == kind_name)
        .collect();
    if !found.is_empty() {
        panic!(
            "Expected no {} issues, but found:\n{}",
            kind_name,
            fmt_issues(&found.into_iter().cloned().collect::<Vec<_>>()),
        );
    }
}

fn fmt_issues(issues: &[Issue]) -> String {
    if issues.is_empty() {
        return "  (none)".to_string();
    }
    issues
        .iter()
        .map(|i| {
            let snippet = i.snippet.as_deref().unwrap_or("<no snippet>");
            format!("  {}: {}  — {}", i.kind.name(), snippet, i.kind.message(),)
        })
        .collect::<Vec<_>>()
        .join("\n")
}
