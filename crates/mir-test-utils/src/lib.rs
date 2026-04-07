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
/// Format: `KindName at LINE:COL`
pub struct ExpectedIssue {
    pub kind_name: String,
    pub line: u32,
    pub col: u16,
}

/// Parse a `.phpt` fixture file into `(php_source, expected_issues)`.
///
/// Fixture format:
/// ```text
/// ===source===
/// <?php
/// ...
/// ===expect===
/// UndefinedClass at 3:8
/// UndefinedFunction at 5:4
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

fn parse_expected_line(line: &str, fixture_path: &str) -> ExpectedIssue {
    // Format: "KindName at LINE:COL"
    let parts: Vec<&str> = line.splitn(2, " at ").collect();
    assert_eq!(
        parts.len(),
        2,
        "fixture {}: invalid expect line {:?} — expected \"KindName at LINE:COL\"",
        fixture_path,
        line
    );
    let kind_name = parts[0].trim().to_string();
    let loc: Vec<&str> = parts[1].trim().splitn(2, ':').collect();
    assert_eq!(
        loc.len(),
        2,
        "fixture {}: invalid location {:?} — expected \"LINE:COL\"",
        fixture_path,
        parts[1]
    );
    let line_num = loc[0]
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("fixture {}: invalid line number {:?}", fixture_path, loc[0]));
    let col = loc[1]
        .parse::<u16>()
        .unwrap_or_else(|_| panic!("fixture {}: invalid col {:?}", fixture_path, loc[1]));

    ExpectedIssue {
        kind_name,
        line: line_num,
        col,
    }
}

/// Run a `.phpt` fixture file: parse, analyze, and assert the issues match
/// the `===expect===` section exactly (no missing, no unexpected).
///
/// Called by the [`fixture_test!`] macro.
pub fn run_fixture(path: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path, e));
    let (source, expected) = parse_phpt(&content, path);
    let actual = check(&source);

    let mut failures: Vec<String> = Vec::new();

    for exp in &expected {
        let found = actual.iter().any(|a| {
            a.kind.name() == exp.kind_name
                && a.location.line == exp.line
                && a.location.col_start == exp.col
        });
        if !found {
            failures.push(format!(
                "  MISSING  {} at {}:{}",
                exp.kind_name, exp.line, exp.col
            ));
        }
    }

    for act in &actual {
        let expected_it = expected.iter().any(|e| {
            e.kind_name == act.kind.name()
                && e.line == act.location.line
                && e.col == act.location.col_start
        });
        if !expected_it {
            failures.push(format!(
                "  UNEXPECTED {} at {}:{}  — {}",
                act.kind.name(),
                act.location.line,
                act.location.col_start,
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
            format!(
                "  {} @ line {}, col {} — {}",
                i.kind.name(),
                i.location.line,
                i.location.col_start,
                i.kind.message(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
