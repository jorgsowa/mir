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
//! finer invalidation. Closures and anonymous classes stay inline in their
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

use crate::body_analysis::BodyAnalyzer;

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
}

unsafe impl salsa::Update for ScopeInferenceResult {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
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

/// Per-scope tracked inference: run body analysis for one scope of `file`
/// and memoize its issues + reference locations.
///
/// Returns an empty result for hard parse errors (parse-error issues are
/// emitted by `analyze_file`, not per scope) and for scope keys that don't
/// resolve to a declaration in this file.
///
/// `lru = 4096` bounds the memo table: keys embed the resolved FQN, so a
/// rename storm would otherwise mint a permanent memo per historical name.
#[salsa::tracked(lru = 4096)]
pub fn infer_scope(
    db: &dyn MirDatabase,
    file: SourceFile,
    scope: ScopeKey,
) -> Arc<ScopeInferenceResult> {
    let path = file.path(db);
    let text = file.text(db);
    let parsed_file = super::queries::parse_file(db, file);
    let parsed = &*parsed_file.0;

    let empty = || {
        Arc::new(ScopeInferenceResult {
            issues: Arc::from([]),
            ref_locs: Arc::from([]),
        })
    };

    if parsed.errors.iter().any(crate::parser::is_hard_parse_error) {
        return empty();
    }

    let php_version = super::queries::db_php_version(db);
    let driver = BodyAnalyzer::new(db, php_version);

    let mut issues: Vec<Issue> = Vec::new();
    let mut symbols: Vec<crate::symbol::ResolvedSymbol> = Vec::new();

    // Isolate this scope's refs in a fresh staging frame so nested on-demand
    // inference on the same handle can't leak into (or consume) them.
    db.push_ref_loc_frame();

    match &scope {
        ScopeKey::FileHeader => {
            crate::body_analysis::check_duplicate_declarations(
                &parsed.program.stmts,
                &path,
                text.as_ref(),
                &parsed.source_map,
                &mut issues,
            );
            check_use_decls(
                &parsed.program.stmts,
                db,
                &path,
                text.as_ref(),
                &parsed.source_map,
                &mut issues,
            );
        }
        ScopeKey::FileExec => {
            driver.analyze_global_exec(
                &parsed.program,
                &path,
                text.as_ref(),
                &parsed.source_map,
                &mut issues,
                &mut symbols,
            );
        }
        ScopeKey::Function(fqn, occ) | ScopeKey::ClassLike(fqn, occ) => {
            let want_fn = matches!(scope, ScopeKey::Function(..));
            let mut seen: u32 = 0;
            let mut found = false;
            for_each_top_level_decl(&parsed.program.stmts, &mut |stmt| {
                use php_ast::owned::StmtKind;
                if found {
                    return;
                }
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
                        &path,
                        text.as_ref(),
                        &parsed.source_map,
                        &mut issues,
                        &mut symbols,
                    ),
                    StmtKind::Class(decl) => driver.analyze_class_decl(
                        decl,
                        &path,
                        text.as_ref(),
                        &parsed.source_map,
                        &mut issues,
                        &mut symbols,
                        &Default::default(),
                    ),
                    StmtKind::Enum(decl) => driver.analyze_enum_decl(
                        decl,
                        &path,
                        text.as_ref(),
                        &parsed.source_map,
                        &mut issues,
                        &mut symbols,
                    ),
                    StmtKind::Interface(decl) => driver.analyze_interface_decl(
                        decl,
                        &path,
                        text.as_ref(),
                        &parsed.source_map,
                        &mut issues,
                        &Default::default(),
                        &mut symbols,
                    ),
                    StmtKind::Trait(decl) => driver.analyze_trait_decl(
                        decl,
                        &path,
                        text.as_ref(),
                        &parsed.source_map,
                        &mut issues,
                        &mut symbols,
                    ),
                    _ => {}
                }
            });
        }
    }

    let ref_locs = db.pop_ref_loc_frame();

    Arc::new(ScopeInferenceResult {
        issues: issues.into(),
        ref_locs: ref_locs.into(),
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
) {
    use php_ast::owned::StmtKind;
    for stmt in stmts.iter() {
        match &stmt.kind {
            StmtKind::Use(use_decl) => {
                crate::body_analysis::check_use_decl_casing(
                    use_decl, db, file, source, source_map, issues, None,
                );
            }
            StmtKind::Namespace(ns) => {
                if let php_ast::owned::NamespaceBody::Braced(inner) = &ns.body {
                    check_use_decls(&inner.stmts, db, file, source, source_map, issues);
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

    let mut merge = |r: Arc<ScopeInferenceResult>| {
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
