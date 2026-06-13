use std::sync::Arc;

use mir_issues::Issue;

use super::*;

// `analyze_file` tracked query: the single value-returning analysis driver.
//
// Issues and reference locations are *returned* from the query rather than
// pushed through salsa accumulators. Accumulators were measured to be the
// wrong vehicle here:
//
//   1. `accumulated_by` performs an untracked read, so any tracked query
//      consuming accumulator output is permanently invalidated and re-runs
//      every revision — derived aggregates can never be built on top.
//   2. Every `accumulated()` call performs a fresh DFS over the query's
//      entire dependency subtree (hundreds of edges per file), which is
//      O(total_edges) per read at workspace scale.
//
// Returning an `Arc<AnalyzeOutput>` makes the memo a plain value: cheap to
// fetch, shareable with downstream aggregates, and validated by salsa's
// regular red-green algorithm.

/// Reference-index entry as produced during analysis.  Mirrors the tuple
/// shape that `Codebase::record_ref` accepts:
///
/// - `symbol_key`: interner-bound string (`"fn:foo"`, `"cls:Foo"`,
///   `"prop:Foo::$bar"`, `"cnst:Foo::BAR"`, `"meth:Foo::bar"` — same keys
///   `Codebase::mark_*_referenced_at` use).
/// - `file`: the file in which the reference appears.
/// - `(line, col_start, col_end)`: span within the file.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefLoc {
    pub symbol_key: Arc<str>,
    pub file: Arc<str>,
    pub line: u32,
    pub col_start: u16,
    pub col_end: u16,
}

/// Singleton salsa input carrying the analysis parameters that aren't
/// already captured by `SourceFile` itself. Lazily created once per
/// database (see `MirDatabase::analyze_config`) so tracked queries that
/// read it get a stable memo key; `MirDbStorage::set_php_version` writes
/// through to it, invalidating dependents on version change.
#[salsa::input]
pub struct AnalyzeFileInput {
    /// Resolved PHP version (`"8.1"`, `"8.2"`, …) used by the analyzer.
    /// Mirrors `ProjectAnalyzer::resolved_php_version`.
    pub php_version: Arc<str>,
}

/// Everything `analyze_file` produces for one file: diagnostics plus the
/// reference-index entries recorded while analyzing its bodies.
///
/// `ref_locs` is sorted + deduplicated so the memo value is deterministic
/// regardless of analysis traversal order — required for salsa backdating
/// (unchanged output ⇒ dependents stay green).
///
/// Per-expression resolved symbols are intentionally NOT part of the memo:
/// a typical file resolves thousands of expressions and retaining them in
/// the salsa cache balloons memory (~50 KiB/file measured on Laravel).
/// Consumers that need symbols call `BodyAnalyzer` directly via
/// `FileAnalyzer`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalyzeOutput {
    pub issues: Arc<[Issue]>,
    pub ref_locs: Arc<[RefLoc]>,
}

unsafe impl salsa::Update for AnalyzeOutput {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

/// Analyze one file: parse-error issues plus full body analysis, assembled
/// from per-scope memos (see [`super::scopes::infer_scope`]).
///
/// Reads the PHP version from the [`AnalyzeFileInput`] singleton (a tracked
/// field read), so the memo key is just `file` and version changes
/// invalidate all memos at once.
///
/// The per-scope merge means this whole-file memo is cheap to recompute:
/// when a dependency change invalidates it, only the scopes actually
/// reached by the change re-execute — the rest return their memoized
/// results during the merge.
#[salsa::tracked]
pub fn analyze_file(db: &dyn MirDatabase, file: SourceFile) -> Arc<AnalyzeOutput> {
    let path = file.path(db);
    let text = file.text(db);

    let mut issues: Vec<Issue> = Vec::new();

    let parsed_file = super::queries::parse_file(db, file);
    let parsed = &*parsed_file.0;

    for err in &parsed.errors {
        if crate::parser::is_spurious_reserved_class_error(err) {
            continue;
        }
        issues.push(crate::parser::parse_error_to_issue(
            err,
            &path,
            &text,
            &parsed.source_map,
        ));
    }

    // Run full analysis unless there are hard parse errors. ForbiddenWarning
    // diagnostics are non-fatal and leave the AST complete, so analysis can
    // still proceed.
    let has_hard_parse_errors = parsed.errors.iter().any(crate::parser::is_hard_parse_error);
    let mut ref_locs: Vec<RefLoc> = Vec::new();
    if !has_hard_parse_errors {
        let (scope_issues, scope_refs) = super::scopes::analyze_file_per_scope(db, file);
        issues.extend(scope_issues);

        // The shared reference index is intentionally NOT mutated from
        // inside the tracked query — consumers decide when to commit the
        // returned locations. Sorted + deduped so the memo value is
        // deterministic (required for backdating).
        ref_locs = scope_refs;
        ref_locs.sort_unstable();
        ref_locs.dedup();
    }

    Arc::new(AnalyzeOutput {
        issues: issues.into(),
        ref_locs: ref_locs.into(),
    })
}
