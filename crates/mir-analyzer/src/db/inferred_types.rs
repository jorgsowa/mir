//! Demand-driven inferred return type lookups for body analysis.
//!
//! body-analysis callers use [`inferred_function_return_type_demand`] /
//! [`inferred_method_return_type_demand`] to resolve cross-file inferred
//! return types on demand via the salsa query graph.  No pre-committed
//! singleton is needed.

use std::sync::Arc;

use mir_types::Type;

use crate::body_analysis::InferredTypes;
use crate::db::{Fqcn, MirDatabase, ScopeKey, SourceFile};

/// Fixpoint iterations after which a cycle head's still-changing inferred
/// types widen to `mixed`, so an ever-growing type (`return [$this->f()];`)
/// converges instead of hitting salsa's iteration cap.
const WIDEN_AFTER_ITERATIONS: u32 = 3;

pub(crate) fn should_widen(cycle: &salsa::Cycle) -> bool {
    cycle.iteration() >= WIDEN_AFTER_ITERATIONS
}

/// Replace each entry of `value` absent from `last` with `mixed`.
pub(crate) fn widen_unstable<T: PartialEq>(
    last: &[T],
    value: &mut [T],
    ty: impl Fn(&mut T) -> &mut Type,
) {
    for entry in value.iter_mut() {
        if !last.contains(entry) {
            *ty(entry) = Type::mixed();
        }
    }
}

/// Demand-driven inferred return type lookup for a function.
///
/// Locates the file that declares `fqn` (the symbol index, else on demand),
/// then calls `infer_file_return_types` on that file. Salsa
/// memoizes both queries, so repeated lookups for the same function are free.
/// Returns `None` when the function is unknown.
pub fn inferred_function_return_type_demand(db: &dyn MirDatabase, fqn: &str) -> Option<Arc<Type>> {
    let sf = crate::db::function_loc(db, Fqcn::from_str(db, fqn))?.file();
    function_return_type_in(db, sf, fqn)
}

/// [`inferred_function_return_type_demand`] as seen from `file`: resolves the
/// declaration the same way [`crate::db::find_function_from`] does.
pub fn inferred_function_return_type_from(
    db: &dyn MirDatabase,
    file: &str,
    fqn: &str,
) -> Option<Arc<Type>> {
    let sf = crate::db::function_loc_from(db, file, Fqcn::from_str(db, fqn))?.file();
    function_return_type_in(db, sf, fqn)
}

/// Inferred return type of function `fqn` declared in `sf`, analyzing only
/// its own scope.
fn function_return_type_in(db: &dyn MirDatabase, sf: SourceFile, fqn: &str) -> Option<Arc<Type>> {
    let Some(inferred) = infer_scopes_matching(
        db,
        sf,
        |key| matches!(key, ScopeKey::Function(name, _) if name.as_ref() == fqn),
        None,
    ) else {
        return crate::db::infer_file_return_types(db, sf)
            .functions
            .get(fqn)
            .cloned();
    };
    inferred
        .iter()
        .flat_map(|types| types.functions.iter())
        .rfind(|(name, _)| name.as_ref() == fqn)
        .map(|(_, ty)| mir_codebase::definitions::wrap_var_type(ty.clone()))
}

/// Run `infer_member` on every declaration scope of `sf` that
/// `matches`, in source order. `None` when no scope matches — the symbol is
/// not a top-level declaration (e.g. an anonymous class), so callers fall
/// back to inferring the whole file.
fn infer_scopes_matching(
    db: &dyn MirDatabase,
    sf: SourceFile,
    matches: impl Fn(&ScopeKey) -> bool,
    method: Option<&str>,
) -> Option<Vec<Arc<InferredTypes>>> {
    let method: Option<Arc<str>> = method.map(Arc::from);
    let inferred: Vec<Arc<InferredTypes>> = crate::db::file_scopes(db, sf)
        .iter()
        .filter(|key| matches(key))
        .map(|key| crate::db::infer_member(db, sf, key.clone(), method.clone()).clone())
        .collect();
    (!inferred.is_empty()).then_some(inferred)
}

/// Demand-driven inferred return type lookup for a method.
///
/// Locates the file that declares the class (the symbol index, else on
/// demand), then analyzes just that method of the class via
/// `infer_member`. `method_name_lower` must already be
/// ASCII-lowercased (PHP semantics). Returns `None` when the class or method
/// is unknown.
pub fn inferred_method_return_type_demand(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name_lower: &str,
) -> Option<Arc<Type>> {
    let sf = crate::db::class_like_loc(db, Fqcn::from_str(db, fqcn))?.file();
    let Some(inferred) = infer_scopes_matching(
        db,
        sf,
        |key| matches!(key, ScopeKey::ClassLike(name, _) if name.as_ref() == fqcn),
        Some(method_name_lower),
    ) else {
        return crate::db::infer_file_return_types(db, sf)
            .methods
            .get(&(Arc::<str>::from(fqcn), Arc::<str>::from(method_name_lower)))
            .cloned();
    };
    inferred
        .iter()
        .flat_map(|types| types.methods.iter())
        .rfind(|(class, name, _)| {
            class.as_ref() == fqcn && name.eq_ignore_ascii_case(method_name_lower)
        })
        .map(|(_, _, ty)| mir_codebase::definitions::wrap_var_type(ty.clone()))
}

/// Demand-driven inferred property type lookup, for a property with no
/// native type hint and no `@var` docblock (see `PropertyDef::ty`).
///
/// Locates the file that declares `fqcn` (the symbol index, else on demand),
/// then analyzes the constructors of every class-like declared in that file
/// via `infer_member`, collecting the union of types they
/// directly assign to the property (a subclass constructor in the same file
/// may assign it too). `fqcn` must be the property's DECLARING class (e.g.
/// `find_property_in_chain`'s owner), and `name` is case-sensitive (PHP
/// property names are). Returns `None` when the class is unknown or nothing
/// was inferred for it.
pub fn inferred_property_type_demand(
    db: &dyn MirDatabase,
    fqcn: &str,
    name: &str,
) -> Option<Arc<Type>> {
    let sf = crate::db::class_like_loc(db, Fqcn::from_str(db, fqcn))?.file();
    let declared_here = crate::db::file_scopes(db, sf)
        .iter()
        .any(|key| matches!(key, ScopeKey::ClassLike(class, _) if class.as_ref() == fqcn));
    let inferred = declared_here
        .then(|| {
            infer_scopes_matching(
                db,
                sf,
                |key| matches!(key, ScopeKey::ClassLike(..)),
                Some("__construct"),
            )
        })
        .flatten();
    let Some(inferred) = inferred else {
        return crate::db::infer_file_return_types(db, sf)
            .properties
            .get(&(Arc::<str>::from(fqcn), Arc::<str>::from(name)))
            .cloned();
    };
    let mut merged: Option<Type> = None;
    for (_, _, ty) in inferred
        .iter()
        .flat_map(|types| types.properties.iter())
        .filter(|(class, prop, _)| class.as_ref() == fqcn && prop.as_ref() == name)
    {
        match &mut merged {
            Some(existing) => existing.merge_with(ty),
            None => merged = Some(ty.clone()),
        }
    }
    merged.map(mir_codebase::definitions::wrap_var_type)
}

/// Stubs and other Composer packages' code aren't summarized.
pub(crate) fn summarizable(db: &dyn MirDatabase, callee: SourceFile, from: &str) -> bool {
    let path = callee.path(db);
    !crate::stubs::is_stub_path(path.as_ref()) && !crate::taint::is_dependency_of(path, from)
}

/// Taint summary of the user function `fqn` as seen from `file`; `None` for
/// stubs, dependencies, unknown functions, and bodies that move no taint.
pub fn function_taint_summary(
    db: &dyn MirDatabase,
    file: &str,
    fqn: &str,
) -> Option<crate::taint::TaintSummary> {
    let sf = crate::db::function_loc_from(db, file, Fqcn::from_str(db, fqn))?.file();
    if !summarizable(db, sf, file) {
        return None;
    }
    infer_scopes_matching(
        db,
        sf,
        |key| matches!(key, ScopeKey::Function(name, _) if name.as_ref() == fqn),
        None,
    )?
    .iter()
    .flat_map(|types| types.function_taint.iter())
    .rfind(|(name, _)| name.as_ref() == fqn)
    .map(|(_, summary)| summary.clone())
}

/// Taint summary of method `method_name_lower` declared on `owner_fqcn`, called from `file`.
pub fn method_taint_summary(
    db: &dyn MirDatabase,
    file: &str,
    owner_fqcn: &str,
    method_name_lower: &str,
) -> Option<crate::taint::TaintSummary> {
    let sf = crate::db::class_like_loc(db, Fqcn::from_str(db, owner_fqcn))?.file();
    if !summarizable(db, sf, file) {
        return None;
    }
    infer_scopes_matching(
        db,
        sf,
        |key| matches!(key, ScopeKey::ClassLike(name, _) if name.as_ref() == owner_fqcn),
        Some(method_name_lower),
    )?
    .iter()
    .flat_map(|types| types.method_taint.iter())
    .rfind(|(class, name, _)| {
        class.as_ref() == owner_fqcn && name.eq_ignore_ascii_case(method_name_lower)
    })
    .map(|(_, _, summary)| summary.clone())
}
