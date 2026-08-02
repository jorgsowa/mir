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
