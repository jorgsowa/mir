//! Per-file analysis entry point for incremental analysis.
//!
//! [`FileAnalyzer`] runs a **single** body-analysis pass against an
//! [`AnalysisSession`] snapshot. In the eager-static-input model the workspace
//! symbol index is built up front by the background indexer
//! ([`AnalysisSession::index_batch`]), so `find_class_like` resolves vendor
//! classes directly — there is no lazy-load / retry loop. The only on-demand
//! work is [`AnalysisSession::priority_index_for_ast`], which faults in the
//! open file's *direct* references if the background walk hasn't reached them
//! yet, keeping warm-up free of transient false positives.
//!
//! For bulk multi-file work, use the session sweeps
//! ([`AnalysisSession::reanalyze_files_cancellable`]) — memoized and
//! cancellable — or the CLI batch pipeline.

use std::sync::Arc;

use mir_issues::Issue;
use php_ast::owned::Program;
use php_rs_parser::source_map::SourceMap;

use crate::body_analysis::BodyAnalyzer;
use crate::db::MirDatabase;
use crate::session::AnalysisSession;
use crate::symbol::ResolvedSymbol;

/// Result of a single-file analysis.
pub struct FileAnalysis {
    pub issues: Vec<Issue>,
    pub symbols: Vec<ResolvedSymbol>,
}

impl FileAnalysis {
    /// Return the innermost resolved symbol whose span contains `byte_offset`,
    /// or `None` if no symbol was recorded at that position.
    ///
    /// Entry point for hover / go-to-definition flows: callers map
    /// (line, column) → byte offset → resolved symbol, then look up the
    /// symbol's definition via [`crate::AnalysisSession::definition_of`] or
    /// type info via [`ResolvedSymbol::resolved_type`].
    pub fn symbol_at(&self, byte_offset: u32) -> Option<&ResolvedSymbol> {
        // Primary: cursor is on an identifier token.
        if let Some(sym) = self
            .symbols
            .iter()
            .filter(|s| s.span.start <= byte_offset && byte_offset < s.span.end)
            .min_by_key(|s| s.span.end - s.span.start)
        {
            return Some(sym);
        }

        // Fallback: cursor is in a call-expression gap (e.g. the whitespace,
        // argument list, or trailing `->` between two chained method calls).
        // Match against the full expression span recorded for call-like
        // symbols and return the innermost (smallest) enclosing call —
        // mirrors `crate::batch::BatchAnalysis::symbol_at`, which already
        // does this; this single-file path fell out of sync with it.
        self.symbols
            .iter()
            .filter(|s| {
                s.expr_span
                    .is_some_and(|es| es.start <= byte_offset && byte_offset < es.end)
            })
            .min_by_key(|s| {
                let es = s.expr_span.unwrap();
                es.end - es.start
            })
    }
}

/// Per-file body analysis analyzer bound to an [`AnalysisSession`]. Cheap to
/// construct — typically held transiently per analysis call.
pub struct FileAnalyzer<'a> {
    session: &'a AnalysisSession,
}

impl<'a> FileAnalyzer<'a> {
    pub fn new(session: &'a AnalysisSession) -> Self {
        Self { session }
    }

    /// Run a single body-analysis pass against a frozen db snapshot.
    ///
    /// `priority_index_for_ast` runs first to fault in any of this file's
    /// direct class references not yet reached by the background indexer; then
    /// one snapshot is analyzed and its reference locations committed. The lock
    /// is not held during analysis, so concurrent edits and reads proceed.
    pub fn analyze(
        &self,
        file: Arc<str>,
        source: &str,
        program: &Program,
        source_map: &SourceMap,
    ) -> FileAnalysis {
        crate::metrics::record_file_analysis();

        // Priority-index the buffer's direct class references so any not yet
        // reached by the background indexer resolve in this single pass (no
        // transient false UndefinedClass during warm-up). Once indexing
        // completes this is a no-op.
        // Capture (text, generation) BEFORE the warm-up: if a concurrent edit
        // swaps the input text mid-flight, the stored Arc no longer matches
        // and the mark is dead on arrival — the safe direction.
        let prepare_generation = self.session.prepare_generation_snapshot();
        let ingested_text = {
            let db = self.session.snapshot_db();
            db.lookup_source_file(file.as_ref())
                .map(|sf| sf.text(&db as &dyn crate::db::MirDatabase).clone())
        };
        self.session
            .prepare_ast_for_analysis(program, file.as_ref());
        // Record the warm-up so later Phase-1 sweeps (references, dependent
        // re-analysis) skip this file's parse + AST walk while its salsa
        // input text is unchanged.
        if let Some(text) = ingested_text.clone() {
            self.session
                .mark_prepared_for_analysis(&file, text, prepare_generation);
        }

        let _scope = crate::metrics::BodyAnalysisScope::new();

        // Generation before the analysis snapshot — after the warm-up, so
        // its lazy loads don't immediately stale the commit; a file add
        // racing the analysis still leaves the mark stale, never fresh.
        let commit_gen = self.session.index_generation();
        // Single pass against a frozen snapshot. With the eager-static-input
        // model the workspace index is complete (or priority-indexed for this
        // file's direct refs), so there are no body-analysis "misses" to fault
        // in — no retry loop, no whole-file re-analysis.
        let db = self.session.snapshot_db();
        let driver = BodyAnalyzer::new(&db, self.session.php_version());
        let (issues, symbols) = driver.analyze_bodies(program, file.clone(), source, source_map);
        // Replace (not append): this pass produced the file's complete
        // reference set, and marking freshness against the pre-analysis text
        // keeps the mark dead-on-arrival if a concurrent edit swapped the
        // input mid-flight (Arc identity no longer matches).
        let resolved = !crate::db::issues_have_unresolved_names(&issues);
        self.session.commit_file_refs(
            &file,
            ingested_text,
            db.take_pending_ref_locs(),
            commit_gen,
            resolved,
        );
        FileAnalysis { issues, symbols }
    }
}
