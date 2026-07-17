//! Demand-driven inferred return type lookups for body analysis.
//!
//! body-analysis callers use [`inferred_function_return_type_demand`] /
//! [`inferred_method_return_type_demand`] to resolve cross-file inferred
//! return types on demand via the salsa query graph.  No pre-committed
//! singleton is needed.

use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;

use mir_types::Type;

use crate::db::{MirDatabase, SymbolLoc};

thread_local! {
    // Guards against re-entrant demand for a file currently being inferred on
    // this thread. When mutually-referential classes trigger a cycle that salsa
    // hasn't closed yet, the same file can be demanded again before the first
    // inference completes.  Returning None (→ mixed) breaks the recursion and
    // lets the fixpoint converge.
    static INFER_IN_PROGRESS: RefCell<HashSet<Arc<str>>> = RefCell::new(HashSet::new());
}

struct InferGuard(Arc<str>);

impl Drop for InferGuard {
    fn drop(&mut self) {
        INFER_IN_PROGRESS.with(|s| s.borrow_mut().remove(&self.0));
    }
}

/// Demand-driven inferred return type lookup for a function.
///
/// Locates the file that declares `fqn` via the workspace symbol index, then
/// calls [`crate::db::infer_file_return_types`] on that file. Salsa
/// memoizes both queries, so repeated lookups for the same function are free.
/// Returns `None` when the function is unknown or not in the workspace index.
pub fn inferred_function_return_type_demand(db: &dyn MirDatabase, fqn: &str) -> Option<Arc<Type>> {
    let idx = crate::db::workspace_index(db);
    let key = mir_types::Name::new(fqn).ascii_lowercase();
    let sf = match idx.functions.get(&key)? {
        SymbolLoc::Function { file, .. } => *file,
        _ => return None,
    };
    let path = sf.path(db).clone();
    let already_active = INFER_IN_PROGRESS.with(|s| s.borrow().contains(&path));
    if already_active {
        return None;
    }
    INFER_IN_PROGRESS.with(|s| s.borrow_mut().insert(path.clone()));
    let _guard = InferGuard(path);
    let inferred = crate::db::infer_file_return_types(db, sf);
    inferred.functions.get(fqn).cloned()
}

/// Demand-driven inferred return type lookup for a method.
///
/// Locates the file that declares the class via the workspace symbol index,
/// then calls [`crate::db::infer_file_return_types`] on that file.
/// `method_name_lower` must already be ASCII-lowercased (PHP semantics).
/// Returns `None` when the class or method is unknown.
pub fn inferred_method_return_type_demand(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name_lower: &str,
) -> Option<Arc<Type>> {
    let idx = crate::db::workspace_index(db);
    let key = mir_types::Name::new(fqcn).ascii_lowercase();
    let sf = match idx.class_like.get(&key)? {
        SymbolLoc::Class { file, .. }
        | SymbolLoc::Interface { file, .. }
        | SymbolLoc::Trait { file, .. }
        | SymbolLoc::Enum { file, .. } => *file,
        _ => return None,
    };
    let path = sf.path(db).clone();
    let already_active = INFER_IN_PROGRESS.with(|s| s.borrow().contains(&path));
    if already_active {
        return None;
    }
    INFER_IN_PROGRESS.with(|s| s.borrow_mut().insert(path.clone()));
    let _guard = InferGuard(path);
    let inferred = crate::db::infer_file_return_types(db, sf);
    inferred
        .methods
        .get(&(Arc::<str>::from(fqcn), Arc::<str>::from(method_name_lower)))
        .cloned()
}
