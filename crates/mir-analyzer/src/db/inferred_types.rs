//! Demand-driven inferred return type lookups for body analysis.
//!
//! body-analysis callers use [`inferred_function_return_type_demand`] /
//! [`inferred_method_return_type_demand`] to resolve cross-file inferred
//! return types on demand via the salsa query graph.  No pre-committed
//! singleton is needed.

use std::sync::Arc;

use mir_types::Type;

use crate::db::{Fqcn, MirDatabase};

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
    let inferred = crate::db::infer_file_return_types(db, sf);
    inferred.functions.get(fqn).cloned()
}

/// Demand-driven inferred return type lookup for a method.
///
/// Locates the file that declares the class (the symbol index, else on
/// demand), then calls `infer_file_return_types` on that file.
/// `method_name_lower` must already be ASCII-lowercased (PHP semantics).
/// Returns `None` when the class or method is unknown.
pub fn inferred_method_return_type_demand(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name_lower: &str,
) -> Option<Arc<Type>> {
    let sf = crate::db::class_like_loc(db, Fqcn::from_str(db, fqcn))?.file();
    let inferred = crate::db::infer_file_return_types(db, sf);
    inferred
        .methods
        .get(&(Arc::<str>::from(fqcn), Arc::<str>::from(method_name_lower)))
        .cloned()
}

/// Demand-driven inferred property type lookup, for a property with no
/// native type hint and no `@var` docblock (see `PropertyDef::ty`).
///
/// Locates the file that declares `fqcn` (the symbol index, else on demand),
/// then calls `infer_file_return_types` on that file — the same
/// inference-only body-analysis pass also collects the union of types
/// directly assigned to the property in `fqcn`'s own constructor. `fqcn`
/// must be the property's DECLARING class (e.g. `find_property_in_chain`'s
/// owner), and `name` is case-sensitive (PHP property names are). Returns
/// `None` when the class is unknown or nothing was inferred for it.
pub fn inferred_property_type_demand(
    db: &dyn MirDatabase,
    fqcn: &str,
    name: &str,
) -> Option<Arc<Type>> {
    let sf = crate::db::class_like_loc(db, Fqcn::from_str(db, fqcn))?.file();
    let inferred = crate::db::infer_file_return_types(db, sf);
    inferred
        .properties
        .get(&(Arc::<str>::from(fqcn), Arc::<str>::from(name)))
        .cloned()
}
