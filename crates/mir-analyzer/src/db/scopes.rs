//! Per-scope tracked inference.
//!
//! [`infer_scope`] memoizes body analysis at the granularity of one
//! top-level declaration (function or class-like) plus two file-frame
//! scopes, instead of one whole-file memo. The payoff is cross-file: when a
//! dependency's signature changes, a dependent file re-validates each scope
//! memo independently — scopes that don't touch the changed symbol stay
//! green and skip re-execution, so re-analysis cost scales with what the
//! change actually reaches, not with file size.
//!
//! Scope granularity is the *declaration*, not the method: intra-file
//! invalidation is file-granular either way (every scope depends on
//! `parse_file`), so per-method keys would only multiply memo count without
//! finer invalidation. On-demand type inference is the exception:
//! [`infer_member`] analyzes just the one method a caller needs, since a
//! caller usually needs one or two methods of a class and analyzing all of
//! them costs far more than the extra memos. Closures and anonymous classes stay inline in their
//! enclosing scope — their analysis input is the flow-sensitive variable
//! environment at their syntactic position, which is an output of analyzing
//! the enclosing body.
//!
//! Dispatch calls the *same* `BodyAnalyzer::analyze_*_decl` methods that
//! `analyze_bodies` calls, so per-scope output is byte-identical to the
//! whole-file walk; [`analyze_file_per_scope`] reassembles the file result
//! in `analyze_bodies`' emission order (header → declarations in source
//! order → top-level executable statements).

use std::sync::Arc;

use mir_issues::Issue;

use rustc_hash::FxHashSet;

use crate::body_analysis::{BodyAnalyzer, InferredTypes};

use super::*;

/// One memoization unit of per-scope inference.
///
/// `Function` / `ClassLike` carry the resolved FQN plus an occurrence index
/// (nth declaration with that FQN in the file, almost always 0) so duplicate
/// declarations — which `analyze_bodies` analyzes individually — each get
/// their own scope.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ScopeKey {
    /// Duplicate-declaration check + `use`-statement casing checks.
    FileHeader,
    /// One top-level (or braced-namespace) function declaration.
    Function(Arc<str>, u32),
    /// One top-level (or braced-namespace) class / interface / trait / enum
    /// declaration: class frame plus all member bodies in member order.
    ClassLike(Arc<str>, u32),
    /// Top-level executable statements in global scope.
    FileExec,
}

/// Everything one scope's analysis produces that salsa memoizes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeInferenceResult {
    pub issues: Arc<[Issue]>,
    pub ref_locs: Arc<[RefLoc]>,
    pub(crate) inferred_types: Arc<crate::body_analysis::InferredTypes>,
}

impl ScopeInferenceResult {
    fn empty() -> Arc<Self> {
        Arc::new(Self {
            issues: Arc::from([]),
            ref_locs: Arc::from([]),
            inferred_types: Arc::new(crate::body_analysis::InferredTypes::default()),
        })
    }
}

/// Declaration scopes of `file` in source order (file-frame scopes are
/// implicit). Drives [`analyze_file_per_scope`]'s merge order.
#[salsa::tracked]
pub fn file_scopes(db: &dyn MirDatabase, file: SourceFile) -> Arc<[ScopeKey]> {
    let path = file.path(db);
    let parsed_file = super::queries::parse_file(db, file);
    let parsed = &*parsed_file.0;

    let mut keys: Vec<ScopeKey> = Vec::new();
    let mut occurrences: rustc_hash::FxHashMap<(bool, Arc<str>), u32> =
        rustc_hash::FxHashMap::default();
    for_each_top_level_decl(&parsed.program.stmts, &mut |stmt| {
        use php_ast::owned::StmtKind;
        let (is_fn, name) = match &stmt.kind {
            StmtKind::Function(decl) => (true, decl.name.as_deref().unwrap_or("")),
            StmtKind::Class(decl) => (
                false,
                decl.name
                    .as_ref()
                    .and_then(|i| i.as_deref())
                    .unwrap_or("<anonymous>"),
            ),
            StmtKind::Enum(decl) => (false, decl.name.as_deref().unwrap_or("<anonymous>")),
            StmtKind::Interface(decl) => (false, decl.name.as_deref().unwrap_or("<anonymous>")),
            StmtKind::Trait(decl) => (false, decl.name.as_deref().unwrap_or("")),
            _ => return,
        };
        let fqn: Arc<str> = Arc::from(resolve_name(db, path.as_ref(), name));
        let occ = occurrences.entry((is_fn, fqn.clone())).or_insert(0);
        let key = if is_fn {
            ScopeKey::Function(fqn, *occ)
        } else {
            ScopeKey::ClassLike(fqn, *occ)
        };
        *occ += 1;
        keys.push(key);
    });
    keys.into()
}

/// Walk top-level declarations in source order, recursing into braced
/// namespaces — the exact stmt set `BodyAnalyzer::analyze_top_level_stmts`
/// dispatches on.
fn for_each_top_level_decl<'a>(
    stmts: &'a [php_ast::owned::Stmt],
    f: &mut impl FnMut(&'a php_ast::owned::Stmt),
) {
    use php_ast::owned::StmtKind;
    for stmt in stmts.iter() {
        match &stmt.kind {
            StmtKind::Function(_)
            | StmtKind::Class(_)
            | StmtKind::Enum(_)
            | StmtKind::Interface(_)
            | StmtKind::Trait(_) => f(stmt),
            StmtKind::Namespace(ns) => {
                if let php_ast::owned::NamespaceBody::Braced(inner) = &ns.body {
                    for_each_top_level_decl(&inner.stmts, f);
                }
            }
            _ => {}
        }
    }
}

/// Like [`for_each_top_level_decl`], but also hands each declaration the
/// `class_exists` guards (`if (!class_exists(X)) { throw ...; }`) seen
/// before it at its nesting level — the same set `analyze_top_level_stmts`
/// threads through, so a scope analyzed on its own skips the same
/// UndefinedClass checks as the whole-file walk.
fn for_each_top_level_decl_guarded<'a>(
    db: &dyn MirDatabase,
    file: &Arc<str>,
    stmts: &'a [php_ast::owned::Stmt],
    f: &mut impl FnMut(&'a php_ast::owned::Stmt, &FxHashSet<Arc<str>>),
) {
    use php_ast::owned::StmtKind;
    let mut guards: FxHashSet<Arc<str>> = FxHashSet::default();
    for stmt in stmts.iter() {
        match &stmt.kind {
            StmtKind::Function(_)
            | StmtKind::Class(_)
            | StmtKind::Enum(_)
            | StmtKind::Interface(_)
            | StmtKind::Trait(_) => f(stmt, &guards),
            StmtKind::Namespace(ns) => {
                if let php_ast::owned::NamespaceBody::Braced(inner) = &ns.body {
                    for_each_top_level_decl_guarded(db, file, &inner.stmts, f);
                }
            }
            _ => {}
        }
        let _scope = crate::db::ns_scope::enter(db, file, stmt.span.start);
        crate::body_analysis::accumulate_class_exists_guard(stmt, db, file, &mut guards);
    }
}

/// Per-scope tracked inference: run body analysis for one scope of `file`
/// and memoize its issues + reference locations.
///
/// Returns an empty result for hard parse errors (parse-error issues are
/// emitted by `analyze_file`, not per scope) and for scope keys that don't
/// resolve to a declaration in this file.
///
/// No `lru`: salsa 0.28 throws `PropagatedPanic` when a fixpoint query's evicted
/// memo is recomputed after revalidation (see `memo_bounds`).
///
/// Cross-file inferred-type demands can re-enter a scope still on the stack
/// (`A::f` → `B::g` → `A::h`); salsa then fixpoint-iterates from an empty
/// result, so the memo is independent of which scope was demanded first.
#[salsa::tracked(cycle_fn = infer_scope_cycle, cycle_initial = infer_scope_initial)]
pub fn infer_scope(
    db: &dyn MirDatabase,
    file: SourceFile,
    scope: ScopeKey,
) -> Arc<ScopeInferenceResult> {
    crate::recursion::ensure_phase_stack(|| infer_scope_body(db, file, scope))
}

fn infer_scope_body(
    db: &dyn MirDatabase,
    file: SourceFile,
    scope: ScopeKey,
) -> Arc<ScopeInferenceResult> {
    let prepared = super::queries::prepare_analysis_file(db, file);
    let path = prepared.path;

    if prepared.has_hard_parse_errors {
        return ScopeInferenceResult::empty();
    }

    let driver = BodyAnalyzer::new(db, prepared.php_version);
    let mut driver = driver;
    driver.collect_inferred_types = true;
    driver.collect_symbols = false;

    let mut issues: Vec<Issue> = Vec::new();

    // Isolate this scope's refs in a fresh staging frame so nested on-demand
    // inference on the same handle can't leak into (or consume) them.
    db.push_ref_loc_frame();

    analyze_scope(db, &driver, &prepared, &scope, &mut issues);

    let ref_locs = db.pop_ref_loc_frame();
    let inferred_types = Arc::new(driver.take_inferred_types());
    crate::metrics::record_scope_analysis(path.as_ref(), 0);
    db.note_work(crate::db::Work::ScopeAnalysis, 1);
    crate::metrics::record_infer_scope_retained(
        issues.len(),
        ref_locs.len(),
        inferred_types.functions.len(),
        inferred_types.methods.len(),
        inferred_types.properties.len(),
    );

    Arc::new(ScopeInferenceResult {
        issues: issues.into(),
        ref_locs: ref_locs.into(),
        inferred_types,
    })
}

/// Inference-only analysis of part of one scope, backing the on-demand
/// inferred return and property type lookups in [`super::inferred_types`].
///
/// `method` (ASCII-lowercased) limits a class-like scope to that one method;
/// `None` analyzes the whole scope. A caller needing one method's return
/// type therefore analyzes that method alone, not every method of its class
/// as [`infer_scope`] would, and the analysis records neither issues nor
/// references. Cycles fixpoint-iterate and widen like [`infer_scope`].
#[salsa::tracked(cycle_fn = infer_member_cycle, cycle_initial = infer_member_initial)]
pub(crate) fn infer_member(
    db: &dyn MirDatabase,
    file: SourceFile,
    scope: ScopeKey,
    method: Option<Arc<str>>,
) -> Arc<InferredTypes> {
    crate::recursion::ensure_phase_stack(|| {
        let prepared = super::queries::prepare_analysis_file(db, file);
        if prepared.has_hard_parse_errors {
            return Arc::new(InferredTypes::default());
        }
        let mut driver = BodyAnalyzer::new_inference_only(db, prepared.php_version);
        driver.collect_symbols = false;
        driver.only_method = method;

        let mut issues: Vec<Issue> = Vec::new();
        db.push_ref_loc_frame();
        analyze_scope(db, &driver, &prepared, &scope, &mut issues);
        db.pop_ref_loc_frame();
        crate::metrics::record_scope_analysis(prepared.path.as_ref(), 0);
        db.note_work(crate::db::Work::ScopeAnalysis, 1);
        Arc::new(driver.take_inferred_types())
    })
}

fn infer_member_initial(
    _db: &dyn MirDatabase,
    _id: salsa::Id,
    _file: SourceFile,
    _scope: ScopeKey,
    _method: Option<Arc<str>>,
) -> Arc<InferredTypes> {
    Arc::new(InferredTypes::default())
}

fn infer_member_cycle(
    _db: &dyn MirDatabase,
    cycle: &salsa::Cycle,
    last: &Arc<InferredTypes>,
    value: Arc<InferredTypes>,
    _file: SourceFile,
    _scope: ScopeKey,
    _method: Option<Arc<str>>,
) -> Arc<InferredTypes> {
    if !super::inferred_types::should_widen(cycle) {
        return value;
    }
    use super::inferred_types::widen_unstable;
    let mut next = (*value).clone();
    widen_unstable(&last.functions, &mut next.functions, |e| &mut e.1);
    widen_unstable(&last.methods, &mut next.methods, |e| &mut e.2);
    widen_unstable(&last.properties, &mut next.properties, |e| &mut e.2);
    Arc::new(next)
}

/// Dispatch `scope` of the prepared file to the `BodyAnalyzer::analyze_*`
/// method the whole-file walk would call for it.
fn analyze_scope(
    db: &dyn MirDatabase,
    driver: &BodyAnalyzer<'_>,
    prepared: &super::queries::PreparedAnalysisFile<'_>,
    scope: &ScopeKey,
    issues: &mut Vec<Issue>,
) {
    let path = prepared.path;
    let text = prepared.text;
    let parsed = prepared.parse_result();
    match scope {
        ScopeKey::FileHeader => {
            crate::body_analysis::check_duplicate_declarations(
                &parsed.program.stmts,
                path,
                text.as_ref(),
                &parsed.source_map,
                issues,
            );
            check_use_decls(
                &parsed.program.stmts,
                db,
                path,
                text.as_ref(),
                &parsed.source_map,
                issues,
                true,
            );
        }
        ScopeKey::FileExec => {
            driver.analyze_global_exec(
                &parsed.program,
                path,
                text.as_ref(),
                &parsed.source_map,
                issues,
                None,
            );
        }
        ScopeKey::Function(fqn, occ) | ScopeKey::ClassLike(fqn, occ) => {
            let want_fn = matches!(scope, ScopeKey::Function(..));
            let mut seen: u32 = 0;
            let mut found = false;
            for_each_top_level_decl_guarded(
                db,
                path,
                &parsed.program.stmts,
                &mut |stmt, guards| {
                    use php_ast::owned::StmtKind;
                    if found {
                        return;
                    }
                    let _scope = crate::db::ns_scope::enter(db, path, stmt.span.start);
                    let (is_fn, name) = match &stmt.kind {
                        StmtKind::Function(decl) => (true, decl.name.as_deref().unwrap_or("")),
                        StmtKind::Class(decl) => (
                            false,
                            decl.name
                                .as_ref()
                                .and_then(|i| i.as_deref())
                                .unwrap_or("<anonymous>"),
                        ),
                        StmtKind::Enum(decl) => {
                            (false, decl.name.as_deref().unwrap_or("<anonymous>"))
                        }
                        StmtKind::Interface(decl) => {
                            (false, decl.name.as_deref().unwrap_or("<anonymous>"))
                        }
                        StmtKind::Trait(decl) => (false, decl.name.as_deref().unwrap_or("")),
                        _ => return,
                    };
                    if is_fn != want_fn || resolve_name(db, path.as_ref(), name) != fqn.as_ref() {
                        return;
                    }
                    if seen != *occ {
                        seen += 1;
                        return;
                    }
                    found = true;
                    match &stmt.kind {
                        StmtKind::Function(decl) => driver.analyze_fn_decl(
                            decl,
                            path,
                            text.as_ref(),
                            &parsed.source_map,
                            issues,
                            None,
                        ),
                        StmtKind::Class(decl) => driver.analyze_class_decl(
                            decl,
                            path,
                            text.as_ref(),
                            &parsed.source_map,
                            issues,
                            None,
                            guards,
                        ),
                        StmtKind::Enum(decl) => driver.analyze_enum_decl(
                            decl,
                            path,
                            text.as_ref(),
                            &parsed.source_map,
                            issues,
                            None,
                        ),
                        StmtKind::Interface(decl) => driver.analyze_interface_decl(
                            decl,
                            path,
                            text.as_ref(),
                            &parsed.source_map,
                            issues,
                            guards,
                            None,
                        ),
                        StmtKind::Trait(decl) => driver.analyze_trait_decl(
                            decl,
                            path,
                            text.as_ref(),
                            &parsed.source_map,
                            issues,
                            None,
                        ),
                        _ => {}
                    }
                },
            );
        }
    }
}

fn infer_scope_initial(
    _db: &dyn MirDatabase,
    _id: salsa::Id,
    _file: SourceFile,
    _scope: ScopeKey,
) -> Arc<ScopeInferenceResult> {
    ScopeInferenceResult::empty()
}

fn infer_scope_cycle(
    _db: &dyn MirDatabase,
    cycle: &salsa::Cycle,
    last: &Arc<ScopeInferenceResult>,
    value: Arc<ScopeInferenceResult>,
    _file: SourceFile,
    _scope: ScopeKey,
) -> Arc<ScopeInferenceResult> {
    if !super::inferred_types::should_widen(cycle) {
        return value;
    }
    use super::inferred_types::widen_unstable;
    let (prev, mut next) = (&last.inferred_types, (*value.inferred_types).clone());
    widen_unstable(&prev.functions, &mut next.functions, |e| &mut e.1);
    widen_unstable(&prev.methods, &mut next.methods, |e| &mut e.2);
    widen_unstable(&prev.properties, &mut next.properties, |e| &mut e.2);
    Arc::new(ScopeInferenceResult {
        issues: value.issues.clone(),
        ref_locs: value.ref_locs.clone(),
        inferred_types: Arc::new(next),
    })
}

/// Run `check_use_decl_casing` for every top-level `use` statement, in
/// source order (recursing into braced namespaces) — mirrors the `Use` arm
/// of `analyze_top_level_stmts`.
fn check_use_decls(
    stmts: &[php_ast::owned::Stmt],
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<Issue>,
    is_full: bool,
) {
    use php_ast::owned::StmtKind;
    for stmt in stmts.iter() {
        let _scope = crate::db::ns_scope::enter(db, file, stmt.span.start);
        match &stmt.kind {
            StmtKind::Use(use_decl) => {
                crate::body_analysis::check_use_decl_casing(
                    use_decl, db, file, source, source_map, issues, None, None, None, false,
                    is_full, true,
                );
            }
            StmtKind::Namespace(ns) => {
                if let php_ast::owned::NamespaceBody::Braced(inner) = &ns.body {
                    check_use_decls(&inner.stmts, db, file, source, source_map, issues, is_full);
                }
            }
            _ => {}
        }
    }
}

/// Reassemble whole-file analysis output from per-scope memos in
/// `analyze_bodies`' emission order: file header (duplicates + use casing),
/// declarations in source order, top-level executable statements.
///
/// Ref locs are concatenated unsorted; the caller (`analyze_file`)
/// sorts + dedups for memo determinism.
pub fn analyze_file_per_scope(db: &dyn MirDatabase, file: SourceFile) -> (Vec<Issue>, Vec<RefLoc>) {
    let mut issues: Vec<Issue> = Vec::new();
    let mut ref_locs: Vec<RefLoc> = Vec::new();

    let mut merge = |r: &ScopeInferenceResult| {
        issues.extend(r.issues.iter().cloned());
        ref_locs.extend(r.ref_locs.iter().cloned());
    };

    merge(infer_scope(db, file, ScopeKey::FileHeader));
    for key in file_scopes(db, file).iter() {
        merge(infer_scope(db, file, key.clone()));
    }
    merge(infer_scope(db, file, ScopeKey::FileExec));

    (issues, ref_locs)
}
