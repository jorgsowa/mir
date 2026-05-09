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

use std::sync::Arc;

use mir_issues::Issue;
use php_ast::ast::Program;
use php_rs_parser::source_map::SourceMap;

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
