//! Regression test: collector-phase issues (found while building a file's
//! declaration slice, before body analysis or cross-file class checks ever
//! run) must be reachable through a public `AnalysisSession` API, the same
//! way `FileAnalyzer::analyze` and `class_issues` already expose their own
//! issue sources.
//!
//! `BackedEnumCaseTypeMismatch` is detected correctly by the collector
//! (`collector/enum.rs`) and flows into `collect_file_definitions(..).issues`,
//! but until `collector_issues` existed there was no way for a consumer to
//! read that field — `FileAnalyzer::analyze` only runs body analysis, and
//! `class_issues` only runs `ClassAnalyzer`'s inheritance/override checks.

mod common;

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion};

const ENUM_WITH_MISMATCHED_CASE: &str = "<?php
enum Suit: string {
    case Hearts = 1;
    case Spades = 'spades';
}
";

#[test]
fn collector_issues_surfaces_backed_enum_case_type_mismatch() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("<test>");
    session.ingest_file(file.clone(), Arc::from(ENUM_WITH_MISMATCHED_CASE));

    let issues = session.collector_issues(&[file]);
    assert!(
        issues
            .iter()
            .any(|i| i.kind.name() == "BackedEnumCaseTypeMismatch"),
        "collector_issues must surface BackedEnumCaseTypeMismatch; got {:?}",
        issues.iter().map(|i| i.kind.name()).collect::<Vec<_>>()
    );
}

/// Documents the gap this fix closes: neither of the two pre-existing issue
/// sources sees the collector's checks, so a caller merging only those two
/// (as php-lsp's `get_semantic_issues_salsa` used to) drops the diagnostic.
#[test]
fn body_analysis_alone_misses_backed_enum_case_type_mismatch() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("<test>");
    session.ingest_file(file.clone(), Arc::from(ENUM_WITH_MISMATCHED_CASE));

    let parsed = php_rs_parser::parse(ENUM_WITH_MISMATCHED_CASE);
    assert!(parsed.errors.is_empty(), "unexpected parse errors: {:?}", parsed.errors);
    let body_issues = FileAnalyzer::new(&session).analyze(
        file.clone(),
        ENUM_WITH_MISMATCHED_CASE,
        &parsed.program,
        &parsed.source_map,
    );
    assert!(
        !body_issues
            .issues
            .iter()
            .any(|i| i.kind.name() == "BackedEnumCaseTypeMismatch"),
        "this test documents that FileAnalyzer::analyze alone does NOT see \
         collector-phase issues — if this starts failing, the doc comment \
         on AnalysisSession::collector_issues needs updating, not this assert"
    );

    let class_issues = session.class_issues(&[file]);
    assert!(
        !class_issues
            .iter()
            .any(|i| i.kind.name() == "BackedEnumCaseTypeMismatch"),
        "class_issues covers ClassAnalyzer, not the collector — documents the same gap"
    );
}

/// Regression guard for a bug in an earlier version of this fix: `ingest_file`
/// eagerly computes `FileDefinitions` via `collect_and_ingest_file`, which
/// primes an in-process, content-hash-keyed parse cache. Re-ingesting the
/// *same* file with *unchanged* content (an LSP client can legitimately
/// re-request diagnostics with no edit in between — a `textDocument/diagnostic`
/// pull, or `didSave` without a prior `didChange`) used to hit that cache and
/// get back an empty issues `Vec`, silently losing the diagnostic on the
/// second call.
#[test]
fn collector_issues_survive_a_second_ingest_of_unchanged_content() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("<test>");

    session.ingest_file(file.clone(), Arc::from(ENUM_WITH_MISMATCHED_CASE));
    session.ingest_file(file.clone(), Arc::from(ENUM_WITH_MISMATCHED_CASE));

    let issues = session.collector_issues(&[file]);
    assert!(
        issues
            .iter()
            .any(|i| i.kind.name() == "BackedEnumCaseTypeMismatch"),
        "issues must survive a second ingest_file call with unchanged content; got {:?}",
        issues.iter().map(|i| i.kind.name()).collect::<Vec<_>>()
    );
}

/// Two distinct files with byte-identical content share the in-process parse
/// cache entry (keyed by content hash, not file). Each file's own issues must
/// still come back with `location.file` pointing at ITS path, not whichever
/// file happened to be ingested first.
#[test]
fn collector_issues_for_two_files_sharing_identical_content_point_at_their_own_path() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file_a: Arc<str> = Arc::from("/proj/a.php");
    let file_b: Arc<str> = Arc::from("/proj/b.php");

    session.ingest_file(file_a.clone(), Arc::from(ENUM_WITH_MISMATCHED_CASE));
    session.ingest_file(file_b.clone(), Arc::from(ENUM_WITH_MISMATCHED_CASE));

    let issues_a = session.collector_issues(&[file_a.clone()]);
    let issues_b = session.collector_issues(&[file_b.clone()]);
    assert_eq!(issues_a.len(), 1, "got {issues_a:?}");
    assert_eq!(issues_b.len(), 1, "got {issues_b:?}");
    assert_eq!(issues_a[0].location.file, file_a);
    assert_eq!(issues_b[0].location.file, file_b);
}

/// A file registered only via `set_file_text` (the lazy/bulk-registration
/// path used for workspace-scan population, never through `ingest_file`)
/// must still surface collector issues — `collector_issues` is a plain
/// snapshot read, indifferent to which write path populated the db, exactly
/// like its siblings `class_issues`/`document_symbols`.
#[test]
fn collector_issues_works_for_a_file_only_ever_set_via_set_file_text() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/lazy.php");

    session.set_file_text(file.clone(), Arc::from(ENUM_WITH_MISMATCHED_CASE));

    let issues = session.collector_issues(&[file]);
    assert!(
        issues
            .iter()
            .any(|i| i.kind.name() == "BackedEnumCaseTypeMismatch"),
        "collector_issues must work for a file only ever registered via \
         set_file_text; got {:?}",
        issues.iter().map(|i| i.kind.name()).collect::<Vec<_>>()
    );
}
