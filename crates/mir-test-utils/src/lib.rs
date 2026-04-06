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
