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
use php_ast::owned::{Program, Stmt, StmtKind};
use php_ast::Span;
use php_rs_parser::source_map::SourceMap;
use rustc_hash::FxHashSet;

use crate::body_analysis::BodyAnalyzer;
use crate::db::MirDatabase;
use crate::session::AnalysisSession;
use crate::symbol::{NavigationFact, ResolvedNavigationFact, ResolvedSymbol};

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
        symbol_at(&self.symbols, byte_offset)
    }
}

fn span_contains(span: Span, byte_offset: u32) -> bool {
    span.start <= byte_offset && byte_offset < span.end
}

fn span_len(span: Span) -> u32 {
    span.end.saturating_sub(span.start)
}

fn symbol_at<'a>(symbols: &'a [ResolvedSymbol], byte_offset: u32) -> Option<&'a ResolvedSymbol> {
    if let Some(symbol) = symbols
        .iter()
        .filter(|f| span_contains(f.span, byte_offset))
        .min_by_key(|f| span_len(f.span))
    {
        return Some(symbol);
    }
    symbols
        .iter()
        .filter(|symbol| {
            symbol
                .expr_span
                .is_some_and(|es| span_contains(es, byte_offset))
        })
        .min_by_key(|symbol| symbol.expr_span.map(span_len).unwrap_or(u32::MAX))
}

fn navigation_fact_at<'a>(
    facts: &'a [NavigationFact],
    byte_offset: u32,
) -> Option<&'a NavigationFact> {
    if let Some(fact) = facts
        .iter()
        .filter(|fact| span_contains(fact.span, byte_offset))
        .min_by_key(|fact| span_len(fact.span))
    {
        return Some(fact);
    }
    facts
        .iter()
        .filter(|fact| {
            fact.expr_span
                .is_some_and(|span| span_contains(span, byte_offset))
        })
        .min_by_key(|fact| fact.expr_span.map(span_len).unwrap_or(u32::MAX))
}

fn resolved_navigation_fact_at<'a>(
    facts: &'a [ResolvedNavigationFact],
    byte_offset: u32,
) -> Option<&'a ResolvedNavigationFact> {
    if let Some(fact) = facts
        .iter()
        .filter(|fact| span_contains(fact.span, byte_offset))
        .min_by_key(|fact| span_len(fact.span))
    {
        return Some(fact);
    }
    facts
        .iter()
        .filter(|fact| {
            fact.expr_span
                .is_some_and(|span| span_contains(span, byte_offset))
        })
        .min_by_key(|fact| fact.expr_span.map(span_len).unwrap_or(u32::MAX))
}

fn for_each_navigation_scope<'a>(stmts: &'a [Stmt], f: &mut impl FnMut(&'a Stmt)) {
    for stmt in stmts.iter() {
        visit_navigation_scope_stmt(stmt, f);
    }
}

fn best_navigation_scope_stmt<'a>(program: &'a Program, byte_offset: u32) -> Option<&'a Stmt> {
    let mut best_stmt: Option<&Stmt> = None;
    for_each_navigation_scope(&program.stmts, &mut |stmt| {
        if !span_contains(stmt.span, byte_offset) {
            return;
        }
        if !matches!(
            stmt.kind,
            StmtKind::Function(_)
                | StmtKind::Class(_)
                | StmtKind::Enum(_)
                | StmtKind::Interface(_)
                | StmtKind::Trait(_)
                | StmtKind::Use(_)
        ) {
            return;
        }
        if best_stmt.is_none_or(|best| span_len(stmt.span) <= span_len(best.span)) {
            best_stmt = Some(stmt);
        }
    });
    best_stmt
}

fn visit_navigation_scope_stmt<'a>(stmt: &'a Stmt, f: &mut impl FnMut(&'a Stmt)) {
    use php_ast::owned::NamespaceBody;
    match &stmt.kind {
        StmtKind::If(s) => {
            visit_navigation_scope_stmt(&s.then_branch, f);
            for branch in s.elseif_branches.iter() {
                visit_navigation_scope_stmt(&branch.body, f);
            }
            if let Some(else_branch) = &s.else_branch {
                visit_navigation_scope_stmt(else_branch, f);
            }
        }
        StmtKind::While(s) => visit_navigation_scope_stmt(&s.body, f),
        StmtKind::For(s) => visit_navigation_scope_stmt(&s.body, f),
        StmtKind::Foreach(s) => visit_navigation_scope_stmt(&s.body, f),
        StmtKind::DoWhile(s) => visit_navigation_scope_stmt(&s.body, f),
        StmtKind::Switch(s) => {
            for case in s.body.cases.iter() {
                for inner in case.body.iter() {
                    visit_navigation_scope_stmt(inner, f);
                }
            }
        }
        StmtKind::TryCatch(t) => {
            for inner in t.body.stmts.iter() {
                visit_navigation_scope_stmt(inner, f);
            }
            for catch in t.catches.iter() {
                for inner in catch.body.stmts.iter() {
                    visit_navigation_scope_stmt(inner, f);
                }
            }
            if let Some(finally) = &t.finally {
                for inner in finally.stmts.iter() {
                    visit_navigation_scope_stmt(inner, f);
                }
            }
        }
        StmtKind::Block(b) => {
            for inner in b.stmts.iter() {
                visit_navigation_scope_stmt(inner, f);
            }
        }
        StmtKind::Declare(d) => {
            if let Some(body) = &d.body {
                visit_navigation_scope_stmt(body, f);
            }
        }
        StmtKind::Namespace(ns) => {
            if let NamespaceBody::Braced(block) = &ns.body {
                for inner in block.stmts.iter() {
                    visit_navigation_scope_stmt(inner, f);
                }
            }
        }
        _ => f(stmt),
    }
}

fn resolve_scope_symbols(
    db: &dyn MirDatabase,
    php_version: crate::PhpVersion,
    file: Arc<str>,
    source: &str,
    program: &Program,
    source_map: &SourceMap,
    byte_offset: u32,
    capture_symbol_types: bool,
    codebase_symbols_only: bool,
) -> Vec<ResolvedSymbol> {
    let best_stmt = best_navigation_scope_stmt(program, byte_offset);

    let mut driver = match best_stmt.map(|stmt| &stmt.kind) {
        Some(
            StmtKind::Function(_)
            | StmtKind::Class(_)
            | StmtKind::Enum(_)
            | StmtKind::Interface(_)
            | StmtKind::Trait(_),
        ) => BodyAnalyzer::new_inference_only(db, php_version),
        _ => BodyAnalyzer::new_inference_only(db, php_version),
    };
    driver.capture_symbol_types = capture_symbol_types;
    driver.codebase_symbols_only = codebase_symbols_only;
    driver.record_reference_locations = false;
    let mut issues = Vec::new();
    let mut symbols = Vec::new();
    let guards: FxHashSet<Arc<str>> = FxHashSet::default();

    match best_stmt.map(|stmt| &stmt.kind) {
        Some(StmtKind::Function(decl)) => {
            driver.analyze_fn_decl(
                decl,
                &file,
                source,
                source_map,
                &mut issues,
                Some(&mut symbols),
            );
        }
        Some(StmtKind::Class(decl)) => {
            driver.analyze_class_decl(
                decl,
                &file,
                source,
                source_map,
                &mut issues,
                Some(&mut symbols),
                &guards,
            );
        }
        Some(StmtKind::Enum(decl)) => {
            driver.analyze_enum_decl(
                decl,
                &file,
                source,
                source_map,
                &mut issues,
                Some(&mut symbols),
            );
        }
        Some(StmtKind::Interface(decl)) => {
            driver.analyze_interface_decl(
                decl,
                &file,
                source,
                source_map,
                &mut issues,
                &guards,
                Some(&mut symbols),
            );
        }
        Some(StmtKind::Trait(decl)) => {
            driver.analyze_trait_decl(
                decl,
                &file,
                source,
                source_map,
                &mut issues,
                Some(&mut symbols),
            );
        }
        Some(StmtKind::Use(use_decl)) => {
            let mut resolved_navigation_facts = Vec::new();
            crate::body_analysis::check_use_decl_casing(
                use_decl,
                db,
                &file,
                source,
                source_map,
                &mut issues,
                None,
                None,
                Some(&mut resolved_navigation_facts),
                false,
                true,
                driver.record_reference_locations,
            );
            if let Some(fact) = resolved_navigation_fact_at(&resolved_navigation_facts, byte_offset)
            {
                return vec![fact.clone().into_resolved_symbol(file.clone())];
            }
        }
        _ => {
            driver.analyze_global_exec(
                program,
                &file,
                source,
                source_map,
                &mut issues,
                Some(&mut symbols),
            );
        }
    }

    symbols
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
        self.analyze_with_symbols(file, source, program, source_map, true)
    }

    /// Run the open-file diagnostics path without retaining a whole-file
    /// `ResolvedSymbol` list.
    ///
    /// This is the preferred hot path for editor diagnostics: body analysis
    /// still walks the entire file and commits references, but per-expression
    /// symbol payloads are skipped and navigation can use [`Self::resolve_at`]
    /// on demand.
    pub fn analyze_diagnostics_only(
        &self,
        file: Arc<str>,
        source: &str,
        program: &Program,
        source_map: &SourceMap,
    ) -> FileAnalysis {
        self.analyze_with_symbols(file, source, program, source_map, false)
    }

    fn analyze_with_symbols(
        &self,
        file: Arc<str>,
        source: &str,
        program: &Program,
        source_map: &SourceMap,
        collect_symbols: bool,
    ) -> FileAnalysis {
        crate::metrics::record_file_analysis();
        // Reconcile mirror-only writes with the symbol-index singleton before
        // resolution runs against it (no-op when nothing is pending).
        self.session.settle_workspace_index();

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
        let mut driver = BodyAnalyzer::new(&db, self.session.php_version());
        driver.collect_symbols = collect_symbols;
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

    /// Resolve the symbol at `byte_offset` in the session's current text for
    /// `file` by analyzing only the containing scope.
    ///
    /// This is the targeted navigation path for open files: it warms direct
    /// references, snapshots the current ingested text/AST, selects the
    /// smallest containing file-scope declaration (or `use` item / top-level
    /// exec region), and runs symbol recording only for that scope.
    pub fn resolve_at(&self, file: Arc<str>, byte_offset: u32) -> Option<ResolvedSymbol> {
        self.resolve_at_with_symbol_types(file, byte_offset, true)
    }

    pub(crate) fn resolve_name_at(&self, file: Arc<str>, byte_offset: u32) -> Option<crate::Name> {
        self.session.settle_workspace_index();
        self.session.prepare_file_for_analysis(&file);

        let db = self.session.snapshot_db();
        let sf = db.lookup_source_file(file.as_ref())?;
        let prepared = crate::db::prepare_analysis_file(&db, sf);
        if prepared.has_hard_parse_errors {
            return None;
        }
        let parsed = prepared.parse_result();
        if let Some(name) = self.resolve_name_at_via_compact_facts(
            &db,
            &file,
            prepared.text.as_ref(),
            &parsed.program,
            &parsed.source_map,
            byte_offset,
        ) {
            crate::metrics::record_name_at_compact_hit();
            return Some(name);
        }
        crate::metrics::record_name_at_fallback_walk();
        let symbols = resolve_scope_symbols(
            &db,
            self.session.php_version(),
            file,
            prepared.text.as_ref(),
            &parsed.program,
            &parsed.source_map,
            byte_offset,
            false,
            true,
        );
        symbol_at(&symbols, byte_offset).and_then(ResolvedSymbol::to_symbol)
    }

    fn resolve_at_with_symbol_types(
        &self,
        file: Arc<str>,
        byte_offset: u32,
        capture_symbol_types: bool,
    ) -> Option<ResolvedSymbol> {
        self.session.settle_workspace_index();
        self.session.prepare_file_for_analysis(&file);

        let db = self.session.snapshot_db();
        let sf = db.lookup_source_file(file.as_ref())?;
        let prepared = crate::db::prepare_analysis_file(&db, sf);
        if prepared.has_hard_parse_errors {
            return None;
        }
        let parsed = prepared.parse_result();
        if let Some(symbol) = self.resolve_at_via_compact_facts(
            &db,
            &file,
            prepared.text.as_ref(),
            &parsed.program,
            &parsed.source_map,
            byte_offset,
            capture_symbol_types,
        ) {
            crate::metrics::record_resolve_at_compact_hit();
            return Some(symbol);
        }
        crate::metrics::record_resolve_at_fallback_walk();
        let symbols = resolve_scope_symbols(
            &db,
            self.session.php_version(),
            file,
            prepared.text.as_ref(),
            &parsed.program,
            &parsed.source_map,
            byte_offset,
            capture_symbol_types,
            false,
        );
        symbol_at(&symbols, byte_offset).cloned()
    }

    fn resolve_name_at_via_compact_facts(
        &self,
        db: &dyn MirDatabase,
        file: &Arc<str>,
        source: &str,
        program: &Program,
        source_map: &SourceMap,
        byte_offset: u32,
    ) -> Option<crate::Name> {
        let best_stmt = best_navigation_scope_stmt(program, byte_offset);
        let mut issues = Vec::new();
        let guards: FxHashSet<Arc<str>> = FxHashSet::default();
        let mut driver = match best_stmt.map(|stmt| &stmt.kind) {
            Some(
                StmtKind::Function(_)
                | StmtKind::Class(_)
                | StmtKind::Enum(_)
                | StmtKind::Interface(_)
                | StmtKind::Trait(_),
            ) => BodyAnalyzer::new_inference_only(db, self.session.php_version()),
            _ => BodyAnalyzer::new_inference_only(db, self.session.php_version()),
        };
        driver.collect_symbols = false;
        driver.capture_symbol_types = false;
        driver.codebase_symbols_only = true;
        driver.record_reference_locations = false;
        driver.collect_navigation_facts = true;

        match best_stmt.map(|stmt| &stmt.kind) {
            Some(StmtKind::Function(decl)) => {
                driver.analyze_fn_decl(decl, file, source, source_map, &mut issues, None);
            }
            Some(StmtKind::Class(decl)) => {
                driver.analyze_class_decl(
                    decl,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    None,
                    &guards,
                );
            }
            Some(StmtKind::Enum(decl)) => {
                driver.analyze_enum_decl(decl, file, source, source_map, &mut issues, None);
            }
            Some(StmtKind::Interface(decl)) => {
                driver.analyze_interface_decl(
                    decl,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    &guards,
                    None,
                );
            }
            Some(StmtKind::Trait(decl)) => {
                driver.analyze_trait_decl(decl, file, source, source_map, &mut issues, None);
            }
            Some(StmtKind::Use(use_decl)) => {
                let mut navigation_facts = Vec::new();
                crate::body_analysis::check_use_decl_casing(
                    use_decl,
                    db,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    None,
                    Some(&mut navigation_facts),
                    None,
                    false,
                    true,
                    false,
                );
                return navigation_fact_at(&navigation_facts, byte_offset)
                    .map(|fact| fact.name.clone());
            }
            _ => {
                driver.analyze_global_exec(program, file, source, source_map, &mut issues, None);
            }
        }

        let facts = driver.take_navigation_facts();
        navigation_fact_at(&facts, byte_offset).map(|fact| fact.name.clone())
    }

    fn resolve_at_via_compact_facts(
        &self,
        db: &dyn MirDatabase,
        file: &Arc<str>,
        source: &str,
        program: &Program,
        source_map: &SourceMap,
        byte_offset: u32,
        capture_symbol_types: bool,
    ) -> Option<ResolvedSymbol> {
        let mut issues = Vec::new();
        let guards: FxHashSet<Arc<str>> = FxHashSet::default();
        let best_stmt = best_navigation_scope_stmt(program, byte_offset);
        let mut driver = match best_stmt.map(|stmt| &stmt.kind) {
            Some(
                StmtKind::Function(_)
                | StmtKind::Class(_)
                | StmtKind::Enum(_)
                | StmtKind::Interface(_)
                | StmtKind::Trait(_),
            ) => BodyAnalyzer::new_inference_only(db, self.session.php_version()),
            _ => BodyAnalyzer::new_inference_only(db, self.session.php_version()),
        };
        driver.collect_symbols = false;
        driver.capture_symbol_types = capture_symbol_types;
        driver.codebase_symbols_only = false;
        driver.record_reference_locations = false;
        driver.collect_navigation_facts = false;
        driver.collect_resolved_navigation_facts = true;

        match best_stmt.map(|stmt| &stmt.kind) {
            Some(StmtKind::Function(decl)) => {
                driver.analyze_fn_decl(decl, file, source, source_map, &mut issues, None);
            }
            Some(StmtKind::Class(decl)) => {
                driver.analyze_class_decl(
                    decl,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    None,
                    &guards,
                );
            }
            Some(StmtKind::Enum(decl)) => {
                driver.analyze_enum_decl(decl, file, source, source_map, &mut issues, None);
            }
            Some(StmtKind::Interface(decl)) => {
                driver.analyze_interface_decl(
                    decl,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    &guards,
                    None,
                );
            }
            Some(StmtKind::Trait(decl)) => {
                driver.analyze_trait_decl(decl, file, source, source_map, &mut issues, None);
            }
            Some(StmtKind::Use(use_decl)) => {
                let mut resolved_navigation_facts = Vec::new();
                crate::body_analysis::check_use_decl_casing(
                    use_decl,
                    db,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    None,
                    None,
                    Some(&mut resolved_navigation_facts),
                    false,
                    true,
                    false,
                );
                if let Some(fact) =
                    resolved_navigation_fact_at(&resolved_navigation_facts, byte_offset)
                {
                    return Some(fact.clone().into_resolved_symbol(file.clone()));
                }
            }
            _ => {
                driver.analyze_global_exec(program, file, source, source_map, &mut issues, None);
            }
        }

        let facts = driver.take_resolved_navigation_facts();
        resolved_navigation_fact_at(&facts, byte_offset)
            .cloned()
            .map(|fact| fact.into_resolved_symbol(file.clone()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use super::FileAnalyzer;
    use crate::{symbol::ReferenceKind, AnalysisSession, PhpVersion, ResolvedSymbol};

    fn session_for_source(path: &str, source: &str) -> (AnalysisSession, Arc<str>) {
        let session = AnalysisSession::new(PhpVersion::LATEST);
        let file: Arc<str> = Arc::from(path);
        session.ingest_file(file.clone(), Arc::from(source));
        (session, file)
    }

    fn analyze_diagnostics_only(session: &AnalysisSession, file: Arc<str>, source: &str) {
        let parsed = php_rs_parser::parse(source);
        assert!(
            parsed.errors.is_empty(),
            "parser errors in test source: {:?}",
            parsed.errors
        );

        let analysis = FileAnalyzer::new(session).analyze_diagnostics_only(
            file,
            source,
            &parsed.program,
            &parsed.source_map,
        );
        assert!(
            analysis.symbols.is_empty(),
            "diagnostics-only analysis should not retain whole-file symbols"
        );
    }

    fn median(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[samples.len() / 2]
    }

    fn metric_section_lines<'a>(dump: &'a str, label: &str) -> Vec<&'a str> {
        let Some((_, rest)) = dump.split_once(label) else {
            return Vec::new();
        };
        rest.lines()
            .skip(1)
            .take_while(|line| line.starts_with("    "))
            .collect()
    }

    fn assert_resolve_at_compact_without_fallback(
        session: &AnalysisSession,
        file: &Arc<str>,
        offset: u32,
        expected: impl FnOnce(&ResolvedSymbol),
    ) {
        crate::metrics::test_reset();
        let symbol = session
            .resolve_at(file.as_ref(), offset)
            .expect("resolve_at should find a symbol at the chosen offset");
        expected(&symbol);

        let dump = crate::metrics::dump().expect("metrics enabled in tests");
        assert!(
            dump.contains("resolve_at path      : compact 1  fallback 0"),
            "expected compact resolve_at path without fallback, got:\n{dump}"
        );
        assert!(
            dump.contains("symbols allocated    : 0"),
            "compact resolve_at should not allocate legacy symbols, got:\n{dump}"
        );
    }

    #[test]
    fn name_at_records_compact_path_without_fallback() {
        crate::metrics::test_reset();

        let src = "<?php\nfunction helper(): void {}\nfunction caller(): void { helper(); }\n";
        let (session, file) = session_for_source("/proj/name_at_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.rfind("helper();").unwrap() as u32;
        let name = session
            .name_at(file.as_ref(), offset)
            .expect("name_at should resolve helper()");

        assert_eq!(name, crate::Name::function("helper"));

        let dump = crate::metrics::dump().expect("metrics enabled in tests");
        assert!(
            dump.contains("name_at path         : compact 1  fallback 0"),
            "expected compact name_at path without fallback, got:\n{dump}"
        );
        assert!(
            dump.contains("symbols allocated    : 0"),
            "compact name_at should not allocate legacy symbols, got:\n{dump}"
        );
    }

    #[test]
    fn name_at_top_level_exec_uses_compact_path_without_fallback() {
        crate::metrics::test_reset();

        let src = "<?php\nfunction helper(): void {}\nhelper();\n";
        let (session, file) = session_for_source("/proj/name_at_top_level_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("helper();").unwrap() as u32;
        let name = session
            .name_at(file.as_ref(), offset)
            .expect("name_at should resolve top-level helper()");

        assert_eq!(name, crate::Name::function("helper"));

        let dump = crate::metrics::dump().expect("metrics enabled in tests");
        assert!(
            dump.contains("name_at path         : compact 1  fallback 0"),
            "expected compact top-level name_at path without fallback, got:\n{dump}"
        );
        assert!(
            dump.contains("symbols allocated    : 0"),
            "compact top-level name_at should not allocate legacy symbols, got:\n{dump}"
        );
    }

    #[test]
    fn resolve_at_records_compact_path_without_fallback() {
        let src = "<?php\nclass Box { public int $value = 0; }\nfunction read(Box $box): void { $box->value; }\n";
        let (session, file) = session_for_source("/proj/resolve_at_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("$box->value").unwrap() as u32 + "$box".len() as u32;
        assert_resolve_at_compact_without_fallback(&session, &file, offset, |symbol| {
            assert!(matches!(symbol.kind, ReferenceKind::Receiver));
        });
    }

    #[test]
    fn resolve_at_top_level_exec_uses_compact_path_without_fallback() {
        let src = "<?php\nfunction helper(): void {}\nhelper();\n";
        let (session, file) = session_for_source("/proj/resolve_top_level_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("helper();").unwrap() as u32;
        assert_resolve_at_compact_without_fallback(&session, &file, offset, |symbol| {
            assert!(matches!(
                &symbol.kind,
                ReferenceKind::FunctionCall(name) if name.as_ref() == "helper"
            ));
        });
    }

    #[test]
    fn resolve_at_method_call_uses_compact_path_without_fallback() {
        let src = "<?php\nclass Dep { public function next(): int { return 1; } }\nfunction run(Dep $dep): int { return $dep->next(); }\n";
        let (session, file) = session_for_source("/proj/resolve_method_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("->next()").unwrap() as u32 + 2;
        assert_resolve_at_compact_without_fallback(&session, &file, offset, |symbol| {
            assert!(matches!(
                &symbol.kind,
                ReferenceKind::MethodCall { class, method }
                    if class.as_ref() == "Dep" && method.as_ref() == "next"
            ));
        });
    }

    #[test]
    fn resolve_at_type_hint_uses_compact_path_without_fallback() {
        let src = "<?php\nclass Dep {}\nfunction run(Dep $dep): Dep { return $dep; }\n";
        let (session, file) = session_for_source("/proj/resolve_type_hint_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("run(Dep").unwrap() as u32 + "run(".len() as u32;
        assert_resolve_at_compact_without_fallback(&session, &file, offset, |symbol| {
            assert!(matches!(
                &symbol.kind,
                ReferenceKind::ClassReference(name) if name.as_ref() == "Dep"
            ));
        });
    }

    #[test]
    fn resolve_at_use_import_uses_compact_path_without_fallback() {
        let session = AnalysisSession::new(PhpVersion::LATEST);
        let dep_file: Arc<str> = Arc::from("/proj/Dep.php");
        let main_file: Arc<str> = Arc::from("/proj/Main.php");
        let dep_src = "<?php\nnamespace App;\nclass Dep {}\n";
        let main_src = "<?php\nuse App\\Dep;\nfunction run(): Dep { return new Dep(); }\n";
        session.ingest_file(dep_file, Arc::from(dep_src));
        session.ingest_file(main_file.clone(), Arc::from(main_src));
        analyze_diagnostics_only(&session, main_file.clone(), main_src);

        let offset = main_src.find("App\\Dep").unwrap() as u32 + "App\\".len() as u32;
        assert_resolve_at_compact_without_fallback(&session, &main_file, offset, |symbol| {
            assert!(matches!(
                &symbol.kind,
                ReferenceKind::UseImport(inner)
                    if matches!(
                        inner.as_ref(),
                        ReferenceKind::ClassReference(name) if name.as_ref() == "App\\Dep"
                    )
            ));
        });
    }

    #[test]
    fn resolve_at_catch_clause_type_uses_compact_path_without_fallback() {
        let src = "<?php\nfinal class MyException extends \\Exception {}\nfunction run(): void {\n    try {\n        throw new MyException();\n    } catch (MyException $e) {\n    }\n}\n";
        let (session, file) = session_for_source("/proj/resolve_catch_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.rfind("catch (MyException").unwrap() as u32 + "catch (".len() as u32;
        assert_resolve_at_compact_without_fallback(&session, &file, offset, |symbol| {
            assert!(matches!(
                &symbol.kind,
                ReferenceKind::ClassReference(name) if name.as_ref() == "MyException"
            ));
        });
    }

    #[test]
    fn resolve_at_static_property_write_class_uses_compact_path_without_fallback() {
        let src = "<?php\nclass Counter {\n    public static int $count = 0;\n}\nfunction bump(): void {\n    Counter::$count = Counter::$count + 1;\n}\n";
        let (session, file) = session_for_source("/proj/resolve_static_prop_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("Counter::$count =").unwrap() as u32;
        assert_resolve_at_compact_without_fallback(&session, &file, offset, |symbol| {
            assert!(matches!(
                &symbol.kind,
                ReferenceKind::ClassReference(name) if name.as_ref() == "Counter"
            ));
        });
    }

    #[test]
    fn hover_at_uses_resolve_at_targeted_path() {
        crate::metrics::test_reset();

        let src = "<?php\nclass Dep {}\nfunction run(Dep $dep): Dep { return $dep; }\n";
        let (session, file) = session_for_source("/proj/hover_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("run(Dep").unwrap() as u32 + "run(".len() as u32;
        let hover = session
            .hover_at(file.as_ref(), offset)
            .expect("hover_at should resolve the Dep type hint");

        assert_eq!(hover.ty.to_string(), "Dep");

        let dump = crate::metrics::dump().expect("metrics enabled in tests");
        assert!(
            dump.contains("resolve_at path      : compact 1  fallback 0"),
            "hover_at should resolve through the compact resolve_at path, got:\n{dump}"
        );
        assert!(
            dump.contains("name_at path         : compact 0  fallback 0"),
            "hover_at should not rely on name_at anymore, got:\n{dump}"
        );
    }

    #[test]
    fn definition_at_uses_resolve_at_targeted_path() {
        crate::metrics::test_reset();

        let src = "<?php\nclass Dep {}\nfunction run(Dep $dep): Dep { return $dep; }\n";
        let (session, file) = session_for_source("/proj/definition_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("run(Dep").unwrap() as u32 + "run(".len() as u32;
        let loc = session
            .definition_at(file.as_ref(), offset)
            .expect("definition_at should resolve the Dep type hint");

        assert_eq!(loc.file.as_ref(), file.as_ref());

        let dump = crate::metrics::dump().expect("metrics enabled in tests");
        assert!(
            dump.contains("resolve_at path      : compact 1  fallback 0"),
            "definition_at should resolve through the compact resolve_at path, got:\n{dump}"
        );
        assert!(
            dump.contains("name_at path         : compact 0  fallback 0"),
            "definition_at should not rely on name_at anymore, got:\n{dump}"
        );
    }

    #[test]
    fn references_at_uses_name_at_compact_path_without_fallback() {
        crate::metrics::test_reset();

        let src = "<?php\nfunction helper(): void {}\nfunction caller(): void { helper(); }\n";
        let (session, file) = session_for_source("/proj/references_metrics.php", src);
        analyze_diagnostics_only(&session, file.clone(), src);

        let offset = src.find("helper();").unwrap() as u32 + 1;
        let refs = session
            .references_at(
                file.as_ref(),
                offset,
                std::slice::from_ref(&file),
                false,
                crate::ReferenceIncludes::Plain,
            )
            .expect("references_at should resolve helper()");

        assert!(
            refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
            "references_at should include the helper() call site; got {refs:?}"
        );

        let dump = crate::metrics::dump().expect("metrics enabled in tests");
        assert!(
            dump.contains("name_at path         : compact 1  fallback 0"),
            "references_at should resolve through the compact name_at path, got:\n{dump}"
        );
        assert!(
            dump.contains("resolve_at path      : compact 0  fallback 0"),
            "references_at should not rely on resolve_at, got:\n{dump}"
        );
    }

    #[test]
    #[ignore]
    fn perf_report_post_diagnostics_navigation_micro() {
        crate::metrics::test_reset();

        let src = "<?php
class Dep {
    public function next(int $value): int { return $value + 1; }
}
class Box {
    public int $value = 0;
}
function helper(Dep $dep, int $value): int {
    return $dep->next($value);
}
function read_value(int $value): int {
    return $value;
}
function read_box(Box $box): int {
    return $box->value;
}
";
        let (session, file) = session_for_source("/proj/perf_nav.php", src);
        let import_session = AnalysisSession::new(PhpVersion::LATEST);
        let dep_file: Arc<str> = Arc::from("/proj/import/Dep.php");
        let main_file: Arc<str> = Arc::from("/proj/import/Main.php");
        let dep_src = "<?php\nnamespace App;\nclass Dep {}\n";
        let main_src = "<?php\nuse App\\Dep;\nfunction run(): Dep { return new Dep(); }\n";
        import_session.ingest_file(dep_file, Arc::from(dep_src));
        import_session.ingest_file(main_file.clone(), Arc::from(main_src));

        let parsed = php_rs_parser::parse(src);
        assert!(parsed.errors.is_empty(), "fixture must parse cleanly");
        let main_parsed = php_rs_parser::parse(main_src);
        assert!(
            main_parsed.errors.is_empty(),
            "import fixture must parse cleanly"
        );

        let t0 = Instant::now();
        let analysis = FileAnalyzer::new(&session).analyze_diagnostics_only(
            file.clone(),
            src,
            &parsed.program,
            &parsed.source_map,
        );
        let same_file_diagnostics_time = t0.elapsed();
        assert!(analysis.symbols.is_empty());

        let t0 = Instant::now();
        let main_analysis = FileAnalyzer::new(&import_session).analyze_diagnostics_only(
            main_file.clone(),
            main_src,
            &main_parsed.program,
            &main_parsed.source_map,
        );
        let import_diagnostics_time = t0.elapsed();
        assert!(main_analysis.symbols.is_empty());

        let method_name_offset = src.find("next($value)").unwrap() as u32;
        let method_resolve_offset = src.find("->next($value)").unwrap() as u32 + 2;
        let variable_resolve_offset =
            src.rfind("return $value;").unwrap() as u32 + "return ".len() as u32 + 1;
        let type_hint_offset = src.find("helper(Dep").unwrap() as u32 + "helper(".len() as u32;
        let receiver_gap_offset = src.find("$box->value").unwrap() as u32 + "$box".len() as u32;
        let import_resolve_offset =
            main_src.find("App\\Dep").unwrap() as u32 + "App\\".len() as u32;
        let hover_offset = src.find("Dep $dep").unwrap() as u32;
        let references_offset = method_name_offset;

        let t0 = Instant::now();
        let first_name = session.name_at(file.as_ref(), method_name_offset);
        let first_name_time = t0.elapsed();
        assert_eq!(first_name, Some(crate::Name::method("Dep", "next")));

        let t0 = Instant::now();
        let first_method_resolve = session.resolve_at(file.as_ref(), method_resolve_offset);
        let first_method_resolve_time = t0.elapsed();
        assert!(matches!(
            first_method_resolve.as_ref().map(|s| &s.kind),
            Some(ReferenceKind::MethodCall { class, method })
                if class.as_ref() == "Dep" && method.as_ref() == "next"
        ));

        let t0 = Instant::now();
        let first_variable_resolve = session.resolve_at(file.as_ref(), variable_resolve_offset);
        let first_variable_resolve_time = t0.elapsed();
        assert!(matches!(
            first_variable_resolve.as_ref().map(|s| &s.kind),
            Some(ReferenceKind::Variable(name)) if name.as_ref() == "value"
        ));

        let t0 = Instant::now();
        let first_type_hint_resolve = session.resolve_at(file.as_ref(), type_hint_offset);
        let first_type_hint_resolve_time = t0.elapsed();
        assert!(matches!(
            first_type_hint_resolve.as_ref().map(|s| &s.kind),
            Some(ReferenceKind::ClassReference(name)) if name.as_ref() == "Dep"
        ));

        let t0 = Instant::now();
        let first_receiver_gap_resolve = session.resolve_at(file.as_ref(), receiver_gap_offset);
        let first_receiver_gap_resolve_time = t0.elapsed();
        assert!(matches!(
            first_receiver_gap_resolve.as_ref().map(|s| &s.kind),
            Some(ReferenceKind::Receiver)
        ));

        let t0 = Instant::now();
        let first_import_resolve =
            import_session.resolve_at(main_file.as_ref(), import_resolve_offset);
        let first_import_resolve_time = t0.elapsed();
        assert!(matches!(
            first_import_resolve.as_ref().map(|s| &s.kind),
            Some(ReferenceKind::UseImport(inner))
                if matches!(
                    inner.as_ref(),
                    ReferenceKind::ClassReference(name) if name.as_ref() == "App\\Dep"
                )
        ));

        let t0 = Instant::now();
        let first_hover = session.hover_at(file.as_ref(), hover_offset);
        let first_hover_time = t0.elapsed();
        assert!(
            first_hover.is_ok(),
            "hover_at should succeed on Dep type hint"
        );

        let t0 = Instant::now();
        let first_definition = session.definition_at(file.as_ref(), hover_offset);
        let first_definition_time = t0.elapsed();
        assert!(
            first_definition.is_ok(),
            "definition_at should succeed on Dep type hint"
        );

        let t0 = Instant::now();
        let first_references = session.references_at(
            file.as_ref(),
            references_offset,
            std::slice::from_ref(&file),
            false,
            crate::ReferenceIncludes::Plain,
        );
        let first_references_time = t0.elapsed();
        assert!(
            first_references
                .as_ref()
                .is_ok_and(|refs| refs.iter().any(|(f, _)| f.as_ref() == file.as_ref())),
            "references_at should succeed on helper-like method call"
        );

        const ITERS: usize = 100;
        let mut name_samples = Vec::with_capacity(ITERS);
        let mut method_resolve_samples = Vec::with_capacity(ITERS);
        let mut variable_resolve_samples = Vec::with_capacity(ITERS);
        let mut type_hint_resolve_samples = Vec::with_capacity(ITERS);
        let mut receiver_gap_resolve_samples = Vec::with_capacity(ITERS);
        let mut import_resolve_samples = Vec::with_capacity(ITERS);
        let mut hover_samples = Vec::with_capacity(ITERS);
        let mut definition_samples = Vec::with_capacity(ITERS);
        let mut references_samples = Vec::with_capacity(ITERS);

        for _ in 0..ITERS {
            let t0 = Instant::now();
            let name = session.name_at(file.as_ref(), method_name_offset);
            name_samples.push(t0.elapsed());
            assert_eq!(name, Some(crate::Name::method("Dep", "next")));

            let t0 = Instant::now();
            let resolve = session.resolve_at(file.as_ref(), method_resolve_offset);
            method_resolve_samples.push(t0.elapsed());
            assert!(matches!(
                resolve.as_ref().map(|s| &s.kind),
                Some(ReferenceKind::MethodCall { class, method })
                    if class.as_ref() == "Dep" && method.as_ref() == "next"
            ));

            let t0 = Instant::now();
            let resolve = session.resolve_at(file.as_ref(), variable_resolve_offset);
            variable_resolve_samples.push(t0.elapsed());
            assert!(matches!(
                resolve.as_ref().map(|s| &s.kind),
                Some(ReferenceKind::Variable(name)) if name.as_ref() == "value"
            ));

            let t0 = Instant::now();
            let resolve = session.resolve_at(file.as_ref(), type_hint_offset);
            type_hint_resolve_samples.push(t0.elapsed());
            assert!(matches!(
                resolve.as_ref().map(|s| &s.kind),
                Some(ReferenceKind::ClassReference(name)) if name.as_ref() == "Dep"
            ));

            let t0 = Instant::now();
            let resolve = session.resolve_at(file.as_ref(), receiver_gap_offset);
            receiver_gap_resolve_samples.push(t0.elapsed());
            assert!(matches!(
                resolve.as_ref().map(|s| &s.kind),
                Some(ReferenceKind::Receiver)
            ));

            let t0 = Instant::now();
            let resolve = import_session.resolve_at(main_file.as_ref(), import_resolve_offset);
            import_resolve_samples.push(t0.elapsed());
            assert!(matches!(
                resolve.as_ref().map(|s| &s.kind),
                Some(ReferenceKind::UseImport(inner))
                    if matches!(
                        inner.as_ref(),
                        ReferenceKind::ClassReference(name) if name.as_ref() == "App\\Dep"
                    )
            ));

            let t0 = Instant::now();
            let hover = session.hover_at(file.as_ref(), hover_offset);
            hover_samples.push(t0.elapsed());
            assert!(hover.is_ok());

            let t0 = Instant::now();
            let definition = session.definition_at(file.as_ref(), hover_offset);
            definition_samples.push(t0.elapsed());
            assert!(definition.is_ok());

            let t0 = Instant::now();
            let references = session.references_at(
                file.as_ref(),
                references_offset,
                std::slice::from_ref(&file),
                false,
                crate::ReferenceIncludes::Plain,
            );
            references_samples.push(t0.elapsed());
            assert!(references
                .as_ref()
                .is_ok_and(|refs| refs.iter().any(|(f, _)| f.as_ref() == file.as_ref())));
        }

        let repeat_name_p50 = median(&mut name_samples);
        let repeat_method_resolve_p50 = median(&mut method_resolve_samples);
        let repeat_variable_resolve_p50 = median(&mut variable_resolve_samples);
        let repeat_type_hint_resolve_p50 = median(&mut type_hint_resolve_samples);
        let repeat_receiver_gap_resolve_p50 = median(&mut receiver_gap_resolve_samples);
        let repeat_import_resolve_p50 = median(&mut import_resolve_samples);
        let repeat_hover_p50 = median(&mut hover_samples);
        let repeat_definition_p50 = median(&mut definition_samples);
        let repeat_references_p50 = median(&mut references_samples);
        let dump = crate::metrics::dump().expect("metrics enabled in tests");
        let expected_resolve_calls = 7 * (ITERS + 1);
        let expected_name_calls = 2 * (ITERS + 1);

        assert!(
            dump.contains(&format!(
                "resolve_at path      : compact {expected_resolve_calls}  fallback 0"
            )),
            "expanded probe should stay fully on the compact resolve_at path, got:\n{dump}"
        );
        assert!(
            dump.contains(&format!(
                "name_at path         : compact {expected_name_calls}  fallback 0"
            )),
            "expanded probe should stay fully on the compact name_at path, got:\n{dump}"
        );

        println!();
        println!("post_diagnostics_navigation_micro");
        println!(
            "  diagnostics_only same-file : {} us",
            same_file_diagnostics_time.as_micros()
        );
        println!(
            "  diagnostics_only import    : {} us",
            import_diagnostics_time.as_micros()
        );
        println!(
            "  first name_at method       : {} us",
            first_name_time.as_micros()
        );
        println!(
            "  first resolve_at method    : {} us",
            first_method_resolve_time.as_micros()
        );
        println!(
            "  first resolve_at variable  : {} us",
            first_variable_resolve_time.as_micros()
        );
        println!(
            "  first resolve_at type_hint : {} us",
            first_type_hint_resolve_time.as_micros()
        );
        println!(
            "  first resolve_at receiver  : {} us",
            first_receiver_gap_resolve_time.as_micros()
        );
        println!(
            "  first resolve_at import    : {} us",
            first_import_resolve_time.as_micros()
        );
        println!(
            "  first hover_at             : {} us",
            first_hover_time.as_micros()
        );
        println!(
            "  first definition_at        : {} us",
            first_definition_time.as_micros()
        );
        println!(
            "  first references_at        : {} us",
            first_references_time.as_micros()
        );
        println!(
            "  repeat name_at method p50  : {} us",
            repeat_name_p50.as_micros()
        );
        println!(
            "  repeat resolve method p50  : {} us",
            repeat_method_resolve_p50.as_micros()
        );
        println!(
            "  repeat resolve variable p50: {} us",
            repeat_variable_resolve_p50.as_micros()
        );
        println!(
            "  repeat resolve hint p50    : {} us",
            repeat_type_hint_resolve_p50.as_micros()
        );
        println!(
            "  repeat resolve recv p50    : {} us",
            repeat_receiver_gap_resolve_p50.as_micros()
        );
        println!(
            "  repeat resolve import p50  : {} us",
            repeat_import_resolve_p50.as_micros()
        );
        println!(
            "  repeat hover p50           : {} us",
            repeat_hover_p50.as_micros()
        );
        println!(
            "  repeat definition p50      : {} us",
            repeat_definition_p50.as_micros()
        );
        println!(
            "  repeat references p50      : {} us",
            repeat_references_p50.as_micros()
        );
        println!("{dump}");
    }

    #[test]
    #[ignore]
    fn perf_report_edit_locality_micro() {
        crate::metrics::test_reset();

        let session = AnalysisSession::new(PhpVersion::LATEST);
        let base_file: Arc<str> = Arc::from("/proj/edit/Base.php");
        let dep_a_file: Arc<str> = Arc::from("/proj/edit/DepA.php");
        let dep_b_file: Arc<str> = Arc::from("/proj/edit/DepB.php");
        let leaf_file: Arc<str> = Arc::from("/proj/edit/Leaf.php");

        let base_src = "<?php
class Base {
    public function id(int $n): int { return $n; }
}
";
        let dep_a_src = "<?php
class DepA extends Base {
    public function run(int $n): int { return $this->id($n) + 1; }
}
";
        let dep_b_src = "<?php
class DepB extends Base {
    public function run(int $n): int { return $this->id($n) + 2; }
}
";
        let leaf_src = "<?php
function leaf_value(int $n): int { return $n + 1; }
";

        session.ingest_file(base_file.clone(), Arc::from(base_src));
        session.ingest_file(dep_a_file.clone(), Arc::from(dep_a_src));
        session.ingest_file(dep_b_file.clone(), Arc::from(dep_b_src));
        session.ingest_file(leaf_file.clone(), Arc::from(leaf_src));

        let dep_a_parsed = php_rs_parser::parse(dep_a_src);
        let dep_b_parsed = php_rs_parser::parse(dep_b_src);
        let leaf_parsed = php_rs_parser::parse(leaf_src);
        assert!(dep_a_parsed.errors.is_empty());
        assert!(dep_b_parsed.errors.is_empty());
        assert!(leaf_parsed.errors.is_empty());

        let _ = FileAnalyzer::new(&session).analyze_diagnostics_only(
            dep_a_file.clone(),
            dep_a_src,
            &dep_a_parsed.program,
            &dep_a_parsed.source_map,
        );
        let _ = FileAnalyzer::new(&session).analyze_diagnostics_only(
            dep_b_file.clone(),
            dep_b_src,
            &dep_b_parsed.program,
            &dep_b_parsed.source_map,
        );
        let _ = FileAnalyzer::new(&session).analyze_diagnostics_only(
            leaf_file.clone(),
            leaf_src,
            &leaf_parsed.program,
            &leaf_parsed.source_map,
        );

        const ITERS: usize = 25;
        let mut leaf_ingest_samples = Vec::with_capacity(ITERS);
        let mut leaf_reanalyze_samples = Vec::with_capacity(ITERS);
        let mut base_ingest_samples = Vec::with_capacity(ITERS);
        let mut base_reanalyze_samples = Vec::with_capacity(ITERS);

        // Ignore the one-time warm-up passes above: the steady-state loops
        // below are what we want to compare for edit locality.
        crate::metrics::test_reset();

        for i in 0..ITERS {
            let leaf_edit = format!(
                "<?php\nfunction leaf_value(int $n): int {{ return $n + 1; }}\n// leaf edit {i}\n"
            );
            let t0 = Instant::now();
            session.ingest_file(leaf_file.clone(), Arc::from(leaf_edit.as_str()));
            leaf_ingest_samples.push(t0.elapsed());

            let t0 = Instant::now();
            let leaf_results = session.reanalyze_dependents(leaf_file.as_ref());
            leaf_reanalyze_samples.push(t0.elapsed());
            assert!(
                leaf_results.is_empty(),
                "leaf file should have no dependents; got {:?}",
                leaf_results
                    .iter()
                    .map(|(file, _)| file.as_ref())
                    .collect::<Vec<_>>()
            );
        }

        let leaf_dump = crate::metrics::dump().expect("metrics enabled in tests");
        assert!(
            leaf_dump.contains("whole-file walks     : 0"),
            "leaf edits should not trigger body walks outside the edited file path, got:\n{leaf_dump}"
        );
        assert!(
            leaf_dump.contains("scopes analyzed      : 0"),
            "leaf edits with no dependents should not trigger per-scope reanalysis, got:\n{leaf_dump}"
        );

        crate::metrics::test_reset();

        for i in 0..ITERS {
            let base_edit = format!(
                "<?php\nclass Base {{\n    public function id(int $n): int {{ return $n; }}\n}}\n// base edit {i}\n"
            );
            let t0 = Instant::now();
            session.ingest_file(base_file.clone(), Arc::from(base_edit.as_str()));
            base_ingest_samples.push(t0.elapsed());

            let t0 = Instant::now();
            let base_results = session.reanalyze_dependents(base_file.as_ref());
            base_reanalyze_samples.push(t0.elapsed());
            let base_files: Vec<&str> =
                base_results.iter().map(|(file, _)| file.as_ref()).collect();
            assert!(
                base_files.contains(&dep_a_file.as_ref())
                    && base_files.contains(&dep_b_file.as_ref()),
                "base edit should reanalyze both dependents; got {base_files:?}"
            );
        }

        let leaf_ingest_p50 = median(&mut leaf_ingest_samples);
        let leaf_reanalyze_p50 = median(&mut leaf_reanalyze_samples);
        let base_ingest_p50 = median(&mut base_ingest_samples);
        let base_reanalyze_p50 = median(&mut base_reanalyze_samples);
        let base_dump = crate::metrics::dump().expect("metrics enabled in tests");
        let body_walks = metric_section_lines(&base_dump, "  top body walks/file:");
        let scope_walks = metric_section_lines(&base_dump, "  top scopes analyzed/file:");

        assert!(
            base_dump.contains("whole-file walks     : 0"),
            "base edit loop should avoid whole-file body walks after migrating inferred types to per-scope queries, got:\n{base_dump}"
        );
        assert!(
            body_walks.is_empty(),
            "base edit loop should not report any whole-file body-walk metrics anymore, got:\n{base_dump}"
        );
        assert!(
            scope_walks.iter().any(|line| line.contains(base_file.as_ref()))
                && scope_walks.iter().any(|line| line.contains(dep_a_file.as_ref()))
                && scope_walks.iter().any(|line| line.contains(dep_b_file.as_ref())),
            "base edit loop should stay entirely on per-scope Salsa queries for the edited file and both dependents, got:\n{base_dump}"
        );

        println!();
        println!("edit_locality_micro");
        println!(
            "  leaf ingest p50          : {} us",
            leaf_ingest_p50.as_micros()
        );
        println!(
            "  leaf reanalyze p50       : {} us",
            leaf_reanalyze_p50.as_micros()
        );
        println!(
            "  base ingest p50          : {} us",
            base_ingest_p50.as_micros()
        );
        println!(
            "  base reanalyze p50       : {} us",
            base_reanalyze_p50.as_micros()
        );
        println!("  leaf metrics:");
        println!("{leaf_dump}");
        println!("  base metrics:");
        println!("{base_dump}");
    }
}
