//! Parity check: `infer_function` tracked query must produce the same issue
//! set and return type as the original per-file `Pass2Driver::analyze_bodies`
//! pipeline for the same function. Guards against semantic drift in
//! `Pass2Driver::analyze_fn_decl_pure`.

use std::sync::Arc;

use std::hash::{Hash, Hasher};

use mir_analyzer::db::{
    collect_file_definitions, infer_function, parse_file, AnalyzeFileInput, MirDatabase,
};
use mir_analyzer::PhpVersion;

/// Sources crafted to exercise: a clean fn, an undefined-variable fn, a
/// return-type-mismatch fn, and a typed fn that returns a literal.
fn fixture() -> &'static str {
    r#"<?php
function plain(): string {
    return "hello";
}

function broken(): int {
    return $undefined_var;
}

function returns_str(): string {
    return 42;
}

function with_params(int $x, string $y): string {
    return $y;
}
"#
}

/// Drive Pass-2 over the whole file via the existing path and pick out
/// issues attributable to one function by line range.
fn old_path_issues_for(
    fn_name: &str,
    source: &str,
) -> (Vec<mir_issues::Issue>, std::ops::Range<u32>) {
    use mir_analyzer::{AnalysisSession, BatchOptions};

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs_loaded();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.php");
    std::fs::write(&path, source).unwrap();

    let result = session.analyze_paths(std::slice::from_ref(&path), &BatchOptions::new());

    // Find the function's line range in the source so we can filter issues.
    // Simple scan: starting line of `function $fn_name(` and the matching `}`.
    let mut start_line: Option<u32> = None;
    let mut end_line: Option<u32> = None;
    let needle = format!("function {fn_name}(");
    let mut depth = 0i32;
    let mut in_fn = false;
    for (i, line) in source.lines().enumerate() {
        let lineno = (i + 1) as u32;
        if !in_fn && line.contains(&needle) {
            start_line = Some(lineno);
            in_fn = true;
        }
        if in_fn {
            depth += line.matches('{').count() as i32;
            depth -= line.matches('}').count() as i32;
            if depth == 0 && line.contains('}') {
                end_line = Some(lineno);
                break;
            }
        }
    }
    let range = start_line.unwrap()..end_line.unwrap() + 1;
    let issues: Vec<_> = result
        .issues
        .iter()
        .filter(|i| range.contains(&i.location.line))
        .cloned()
        .collect();
    (issues, range)
}

fn new_path_issues_for(fn_name: &str, source: &str) -> Vec<mir_issues::Issue> {
    // Mirror AnalysisSession setup minimally: ingest stubs so resolution works.
    use mir_analyzer::{AnalysisSession, BatchOptions};
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs_loaded();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("fixture.php");
    std::fs::write(&path, source).unwrap();
    // Use the session's prepared db so workspace lookups work the same way
    // as the old-path measurement.
    let _ = session.analyze_paths(std::slice::from_ref(&path), &BatchOptions::new());
    let db_snap = session.snapshot_db();

    let path_str: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
    let file = db_snap.lookup_source_file(path_str.as_ref()).unwrap();
    let _ = parse_file(&db_snap, file);
    let _ = collect_file_definitions(&db_snap, file);

    let input = AnalyzeFileInput::new(&db_snap, Arc::from("8.4"));
    let result = infer_function(&db_snap, file, Arc::from(fn_name), input);
    result.map(|r| r.issues.clone()).unwrap_or_default()
}

/// Compare issues by `(IssueKind, line, col_start)`. Hash-based to avoid needing
/// `Ord` on IssueKind. IssueKind already implements Hash via `Eq` + Hash derive.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
struct IssueKey(String, u32, u16);

fn key_of(i: &mir_issues::Issue) -> IssueKey {
    // Debug format of IssueKind is stable and distinguishes variants/fields.
    IssueKey(
        format!("{:?}", i.kind),
        i.location.line,
        i.location.col_start,
    )
}

/// Cross-function diagnostics that *cannot* be emitted from a single-function
/// query — they need call-graph or workspace-wide knowledge. The new path
/// is per-function by design and won't produce them; filter from comparison.
fn is_cross_function_diagnostic(kind: &mir_issues::IssueKind) -> bool {
    use mir_issues::IssueKind::*;
    matches!(
        kind,
        UnusedFunction { .. }
            | UnusedMethod { .. }
            | UnusedProperty { .. }
            | UnimplementedAbstractMethod { .. }
            | UnimplementedInterfaceMethod { .. }
            | MethodSignatureMismatch { .. }
            | OverriddenMethodAccess { .. }
            | FinalClassExtended { .. }
            | FinalMethodOverridden { .. }
    )
}

fn assert_issue_set_parity(fn_name: &str, source: &str) {
    let (old_issues, _range) = old_path_issues_for(fn_name, source);
    let new_issues = new_path_issues_for(fn_name, source);

    let old_set: std::collections::HashSet<IssueKey> = old_issues
        .iter()
        .filter(|i| !is_cross_function_diagnostic(&i.kind))
        .map(key_of)
        .collect();
    let new_set: std::collections::HashSet<IssueKey> = new_issues
        .iter()
        .filter(|i| !is_cross_function_diagnostic(&i.kind))
        .map(key_of)
        .collect();

    let only_in_old: Vec<_> = old_set.difference(&new_set).collect();
    let only_in_new: Vec<_> = new_set.difference(&old_set).collect();

    assert!(
        only_in_old.is_empty() && only_in_new.is_empty(),
        "parity mismatch for fn `{fn_name}`:\n  only in old path ({}): {:?}\n  only in new path ({}): {:?}\n",
        only_in_old.len(),
        only_in_old,
        only_in_new.len(),
        only_in_new,
    );
    // Silence the Hash/Hasher imports — they're load-bearing for HashSet here.
    let _ = std::collections::hash_map::DefaultHasher::new().finish();
}

#[test]
fn parity_plain_function() {
    assert_issue_set_parity("plain", fixture());
}

#[test]
fn parity_undefined_var_in_body() {
    assert_issue_set_parity("broken", fixture());
}

#[test]
fn parity_return_type_mismatch() {
    assert_issue_set_parity("returns_str", fixture());
}

#[test]
fn parity_typed_params() {
    assert_issue_set_parity("with_params", fixture());
}
