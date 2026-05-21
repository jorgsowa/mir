use std::sync::Arc;

use mir_issues::Issue;

use crate::pass2::Pass2Driver;
use crate::PhpVersion;

use super::*;

// S4 Step 1: analyze_file accumulators + tracked-query skeleton
//
// First step toward S4 (issues + reference locations as Salsa accumulators,
// `analyze_file` as a tracked query).  This step is purely additive:
//
//   1. Defines `IssueAccumulator` and `RefLocAccumulator` salsa accumulator
//      types — push targets for analyzer-emitted issues and reference-index
//      entries during tracked-query evaluation.
//   2. Defines `analyze_file` as a tracked-query stub keyed on a
//      `(SourceFile, AnalyzeFileInput)` pair.  The stub does NOT perform
//      analysis — it accumulates only the parse errors (a strict subset of
//      what `collect_file_definitions` already produces, so semantics are
//      unchanged).  The full analyzer wiring follows in subsequent S4 PRs.
//
// Nothing in this module is wired into the batch (`analyze`) or LSP
// (`re_analyze_file`) paths yet.  Behavior change: zero.

/// Salsa accumulator carrying analyzer-emitted issues.  In the eventual
/// S4 design, every site that today calls `IssueBuffer::add` / `Vec::push`
/// from inside a tracked query will instead call
/// `IssueAccumulator(issue).accumulate(db)`, and `re_analyze_file` will read
/// the accumulated issues for the file with
/// `analyze_file::accumulated::<IssueAccumulator>(db, file, ...)`.
#[salsa::accumulator]
#[derive(Clone, Debug)]
pub struct IssueAccumulator(pub Issue);

/// Reference-index entry as produced during analysis.  Mirrors the tuple
/// shape that `Codebase::record_ref` accepts:
///
/// - `symbol_key`: interner-bound string (`"fn:foo"`, `"cls:Foo"`,
///   `"prop:Foo::$bar"`, `"cnst:Foo::BAR"`, `"meth:Foo::bar"` — same keys
///   `Codebase::mark_*_referenced_at` use).
/// - `file`: the file in which the reference appears.
/// - `(line, col_start, col_end)`: span within the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefLoc {
    pub symbol_key: Arc<str>,
    pub file: Arc<str>,
    pub line: u32,
    pub col_start: u16,
    pub col_end: u16,
}

/// Salsa accumulator carrying reference-index entries.  In the eventual
/// S4 design this replaces the `Codebase::mark_*_referenced_at` side
/// effects: instead of mutating the codebase's reference index inside a
/// tracked query (which Salsa cannot observe), the analyzer pushes
/// `RefLocAccumulator(loc)` and the consumer (LSP / dead-code) reads via
/// `analyze_file::accumulated::<RefLocAccumulator>(db, …)`.
#[salsa::accumulator]
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct RefLocAccumulator(pub RefLoc);

/// Salsa tracked-query input for `analyze_file`.  Carries the analysis
/// parameters that aren't already captured by `SourceFile` itself.  Kept
/// minimal in this PR; subsequent PRs in the S4 chain will extend it as
/// the query body grows to call the full analyzer pipeline.
#[salsa::input]
pub struct AnalyzeFileInput {
    /// Resolved PHP version (`"8.1"`, `"8.2"`, …) used by the analyzer.
    /// Mirrors `ProjectAnalyzer::resolved_php_version`.
    pub php_version: Arc<str>,
}

// S4 Step 3: Lazy inferred-type queries
//
// These tracked queries compute inferred return types on-demand during Pass 2.
// When `Pass2Driver` encounters a function/method call, it reads the inferred
// type via these queries instead of from a pre-computed buffer.
//
// This enables two key optimizations:
// 1. Single-pass execution: inferred types are computed as needed, not upfront
// 2. Incremental caching: if a dependent file doesn't call a function, its
//    inferred type is never computed (Salsa skips the query)

// Helper: collect analysis results via tracked query accumulators

/// Collects all accumulated issues from a set of files analyzed via the
/// `analyze_file` tracked query. Used during batch analysis to read issues
/// that were emitted during tracked-query evaluation.
#[allow(dead_code)]
pub(crate) fn collect_accumulated_issues(
    db: &dyn MirDatabase,
    files: &[(Arc<str>, SourceFile)],
    php_version: &str,
) -> Vec<Issue> {
    let mut all_issues = Vec::new();
    let input = AnalyzeFileInput::new(db, Arc::from(php_version));

    for (_path, file) in files {
        // Call the tracked query to trigger analysis + accumulation
        analyze_file(db, *file, input);

        // Read back the accumulated issues for this file
        let accumulated: Vec<&IssueAccumulator> = analyze_file::accumulated(db, *file, input);
        for acc in accumulated {
            all_issues.push(acc.0.clone());
        }
    }

    all_issues
}

/// Tracked-query skeleton for `analyze_file`.
///
/// **Current behavior (S4 step 2):** parses the file, emits parse-error issues,
/// and calls Pass 2 to analyze function/method bodies. Issues and reference
/// locations are emitted via `IssueAccumulator` and `RefLocAccumulator`.
///
/// This is still a hybrid: inferred types come from the prior
/// `run_inference_sweep` → `commit_inferred_return_types` in the double-pass
/// orchestration. Future S4 PRs will replace that with lazy
/// `inferred_return_type(node)` tracked queries.
#[salsa::tracked]
pub fn analyze_file(db: &dyn MirDatabase, file: SourceFile, input: AnalyzeFileInput) {
    use salsa::Accumulator as _;
    let path = file.path(db);
    let text = file.text(db);

    let arena = crate::arena::create_parse_arena(text.len());
    let parsed = php_rs_parser::parse_arena(&arena, &text);

    for err in &parsed.errors {
        let issue = crate::parser::parse_error_to_issue(err, &path, &text, &parsed.source_map);
        IssueAccumulator(issue).accumulate(db);
    }

    // Run full analysis unless there are hard parse errors. ForbiddenWarning
    // diagnostics are non-fatal and leave the AST complete, so analysis can
    // still proceed.
    let has_hard_parse_errors = parsed.errors.iter().any(crate::parser::is_hard_parse_error);
    if !has_hard_parse_errors {
        use std::str::FromStr as _;
        let php_version =
            PhpVersion::from_str(input.php_version(db).as_ref()).unwrap_or(PhpVersion::LATEST);
        let driver = Pass2Driver::new(db, php_version);
        let (issues, _symbols) = driver.analyze_bodies(
            &parsed.program,
            path.clone(),
            text.as_ref(),
            &parsed.source_map,
        );

        // Emit issues via accumulator
        for issue in issues {
            IssueAccumulator(issue).accumulate(db);
        }

        // Drain reference locations that Pass 2 staged in this db's pending
        // buffer and emit each via the accumulator. The shared
        // reference_locations map is intentionally NOT mutated from inside
        // the tracked query — consumers (`collect_accumulated_issues`,
        // future S5-B FileAnalyzer migration) decide when to commit after
        // reading accumulator output.
        //
        // Per-expression resolved symbols are NOT routed through an
        // accumulator: a typical file resolves thousands of expressions and
        // retaining them in the salsa cache balloons memory (~50 KiB/file
        // measured on Laravel). Consumers that need symbols call
        // `Pass2Driver` directly via `FileAnalyzer`.
        for loc in db.take_pending_ref_locs() {
            RefLocAccumulator(loc).accumulate(db);
        }
    }
}
