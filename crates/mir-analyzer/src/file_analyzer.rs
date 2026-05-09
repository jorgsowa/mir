//! Per-file analysis entry point for incremental analysis.
//!
//! [`FileAnalyzer`] runs single-pass Pass 2 against an [`AnalysisSession`] and
//! returns issues + resolved symbols for one file. Unlike
//! [`crate::ProjectAnalyzer::re_analyze_file`], it does **not** run the
//! inference-only Pass 2 sweep — that's a batch concern. For cross-file
//! inferred return types, schedule a project-wide inference sweep on idle.
//!
//! Caller is responsible for parsing the file (so they keep ownership of the
//! arena and AST). The session must already have Pass 1 state for any files
//! whose definitions this analysis depends on; call
//! [`AnalysisSession::ingest_file`] first.
//!
//! For batch multi-file analysis, use [`BatchFileAnalyzer::analyze_batch`]
//! which parallelizes analysis across multiple pre-parsed files.

use std::sync::Arc;

use mir_issues::Issue;
use php_ast::ast::Program;
use php_rs_parser::source_map::SourceMap;
use rayon::prelude::*;

use crate::db::MirDatabase;
use crate::pass2::Pass2Driver;
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

    /// Single-pass Pass 2. Returns issues and per-expression resolved symbols.
    ///
    /// Pass 2 runs against a cloned db snapshot — the lock is not held during
    /// analysis, so concurrent edits and reads on the session proceed without
    /// blocking on this call.
    ///
    /// Stub loading: ensures the session's essentials are loaded, then auto-
    /// discovers any extension stubs (`imagecreate` → gd, `ReflectionClass` →
    /// Reflection, …) referenced by `source` and lazy-ingests them. This
    /// keeps essentials-only sessions correct without callers having to
    /// enumerate stubs by hand. Call `ensure_all_stubs_loaded` once if the
    /// consumer prefers eager loading instead.
    pub fn analyze(
        &self,
        file: Arc<str>,
        source: &str,
        program: &Program<'_, '_>,
        source_map: &SourceMap,
    ) -> FileAnalysis {
        self.session.ensure_essential_stubs_loaded();
        self.session.ensure_stubs_for_ast(program);
        let db = self.session.snapshot_db();
        let driver = Pass2Driver::new(&db, self.session.php_version());
        let (issues, symbols) = driver.analyze_bodies(program, file, source, source_map);
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
    pub file: Arc<str>,
    pub source: Arc<str>,
    pub program: *const Program<'static, 'static>,
    pub source_map: *const SourceMap,
}

// SAFETY: ParsedFile contains pointers to owned AST and source_map that are kept
// alive by the parser and owned by the caller. Analysis only reads these, never mutates.
unsafe impl Send for ParsedFile {}
unsafe impl Sync for ParsedFile {}

impl<'a> BatchFileAnalyzer<'a> {
    pub fn new(session: &'a AnalysisSession) -> Self {
        Self { session }
    }

    /// Analyze multiple pre-parsed files in parallel.
    ///
    /// Each file must already have its AST and source_map computed and kept alive
    /// by the caller. This function processes all files in parallel using rayon.
    ///
    /// Each rayon worker gets its own cloned database snapshot, so concurrent
    /// analysis proceeds without lock contention on the session.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring that the Program and SourceMap pointers
    /// remain valid for the duration of this call.
    pub fn analyze_batch(&self, files: Vec<ParsedFile>) -> Vec<(Arc<str>, FileAnalysis)> {
        self.session.ensure_essential_stubs_loaded();

        // First pass: collect all ASTs and auto-discover stubs.
        files.iter().for_each(|file| {
            // SAFETY: Caller guarantees pointer validity.
            let program = unsafe { &*file.program };
            self.session.ensure_stubs_for_ast(program);
        });

        // Second pass: analyze files in parallel.
        // Each rayon worker gets its own database clone (Salsa is Send but !Sync).
        let db = self.session.snapshot_db();
        files
            .into_par_iter()
            .map_with(db, |db, file| {
                // SAFETY: Caller guarantees pointer validity.
                let program = unsafe { &*file.program };
                let source_map = unsafe { &*file.source_map };
                let driver = Pass2Driver::new(db as &dyn MirDatabase, self.session.php_version());
                let (issues, symbols) =
                    driver.analyze_bodies(program, file.file.clone(), &file.source, source_map);
                let analysis = FileAnalysis { issues, symbols };
                (file.file, analysis)
            })
            .collect()
    }
}
