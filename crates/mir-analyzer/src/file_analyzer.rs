//! Per-file analysis entry point for incremental analysis.
//!
//! [`FileAnalyzer`] runs single-pass Pass 2 against an [`AnalysisSession`] and
//! returns issues + resolved symbols for one file. Unlike
//! [`crate::ProjectAnalyzer::re_analyze_file`], it does **not** run the
//! inference-only Pass 2 sweep — that's a batch concern. For cross-file
//! inferred return types, schedule a project-wide inference sweep on idle.
//!
//! Caller is responsible for parsing the file and passing owned AST.
//! The session must already have Pass 1 state for any files whose definitions
//! this analysis depends on; call [`AnalysisSession::ingest_file`] first.
//!
//! For batch multi-file analysis, use [`BatchFileAnalyzer::analyze_batch`]
//! which parallelizes analysis across multiple pre-parsed files.

use std::sync::Arc;

use mir_issues::Issue;
use php_ast::owned::Program;
use php_rs_parser::source_map::SourceMap;
use rayon::prelude::*;

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
        self.symbols
            .iter()
            .filter(|s| s.span.start <= byte_offset && byte_offset < s.span.end)
            .min_by_key(|s| s.span.end - s.span.start)
    }
}

/// Per-file Pass 2 analyzer bound to an [`AnalysisSession`]. Cheap to
/// construct — typically held transiently per analysis call.
pub struct FileAnalyzer<'a> {
    session: &'a AnalysisSession,
}

impl<'a> FileAnalyzer<'a> {
    pub fn new(session: &'a AnalysisSession) -> Self {
        Self { session }
    }

    /// Run Pass 2 against a db snapshot.
    ///
    /// Pass 2 runs against a cloned db snapshot — the lock is not held during
    /// analysis, so concurrent edits and reads on the session proceed without
    /// blocking on this call. PSR-4-mapped classes referenced in the AST are
    /// pre-loaded before Pass 2 so `find_class_like` resolves them in a single
    /// pass via the salsa query graph.
    pub fn analyze(
        &self,
        file: Arc<str>,
        source: &str,
        program: &Program,
        source_map: &SourceMap,
    ) -> FileAnalysis {
        crate::metrics::record_file_analysis();
        self.session.ensure_essential_stubs();
        self.session
            .prepare_ast_for_analysis(program, file.as_ref());

        let _scope = crate::metrics::BodyAnalysisScope::new();
        let db = self.session.snapshot_db();
        let driver = BodyAnalyzer::new(&db, self.session.php_version());
        let (issues, symbols) = driver.analyze_bodies(program, file, source, source_map);
        self.session
            .commit_ref_locs_batch(db.take_pending_ref_locs());
        FileAnalysis { issues, symbols }
    }
}

/// Batch file analyzer for parallel multi-file analysis.
///
/// `BatchFileAnalyzer` processes pre-parsed files in parallel using rayon,
/// making it efficient for analyzing many files at once (e.g., cold-start analysis).
pub struct BatchFileAnalyzer<'a> {
    session: &'a AnalysisSession,
}

/// A pre-parsed file ready for batch analysis.
pub struct ParsedFile {
    pub(crate) file: Arc<str>,
    pub(crate) source: Arc<str>,
    pub(crate) program: Program,
    pub(crate) source_map: SourceMap,
}

impl ParsedFile {
    /// File path this `ParsedFile` represents.
    pub fn file(&self) -> &Arc<str> {
        &self.file
    }

    /// Source text for this file.
    pub fn source(&self) -> &Arc<str> {
        &self.source
    }

    /// Create a `ParsedFile` from an owned program and source map.
    pub fn new(file: Arc<str>, source: Arc<str>, program: Program, source_map: SourceMap) -> Self {
        Self {
            file,
            source,
            program,
            source_map,
        }
    }
}

impl<'a> BatchFileAnalyzer<'a> {
    pub fn new(session: &'a AnalysisSession) -> Self {
        Self { session }
    }

    /// Analyze multiple pre-parsed files in parallel.
    ///
    /// Each rayon worker gets its own cloned database snapshot, so concurrent
    /// analysis proceeds without lock contention on the session.
    pub fn analyze_batch(&self, files: Vec<ParsedFile>) -> Vec<(Arc<str>, FileAnalysis)> {
        self.session.ensure_essential_stubs();

        // First pass: collect all ASTs and auto-discover stubs.
        files.iter().for_each(|file| {
            self.session.ensure_stubs_for_ast(&file.program);
        });

        // Second pass: analyze files in parallel.
        // Each rayon worker gets its own database clone (Salsa is Send but !Sync).
        let db = self.session.snapshot_db();
        let results: Vec<(Arc<str>, FileAnalysis, Vec<crate::db::RefLoc>)> = files
            .into_par_iter()
            .map_with(db, |db, file| {
                let driver = BodyAnalyzer::new(db as &dyn MirDatabase, self.session.php_version());
                let (issues, symbols) = driver.analyze_bodies(
                    &file.program,
                    file.file.clone(),
                    &file.source,
                    &file.source_map,
                );
                let pending = db.take_pending_ref_locs();
                let analysis = FileAnalysis { issues, symbols };
                (file.file, analysis, pending)
            })
            .collect();
        let mut all_ref_locs = Vec::new();
        let mut out = Vec::with_capacity(results.len());
        for (file, analysis, ref_locs) in results {
            all_ref_locs.extend(ref_locs);
            out.push((file, analysis));
        }
        self.session.commit_ref_locs_batch(all_ref_locs);
        out
    }
}
