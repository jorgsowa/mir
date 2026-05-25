//! Demand-driven inferred return type lookups for body analysis.
//!
//! body-analysis callers use [`inferred_function_return_type_demand`] /
//! [`inferred_method_return_type_demand`] to resolve cross-file inferred
//! return types on demand via the salsa query graph.  No pre-committed
//! singleton is needed.

use std::sync::Arc;

use mir_types::Union;

use crate::db::{MirDatabase, SymbolLoc};

/// Demand-driven inferred return type lookup for a function.
///
/// Locates the file that declares `fqn` via the workspace symbol index, then
/// calls [`crate::db::infer_file_return_types`] on that file. Salsa
/// memoizes both queries, so repeated lookups for the same function are free.
/// Returns `None` when the function is unknown or not in the workspace index.
pub fn inferred_function_return_type_demand(db: &dyn MirDatabase, fqn: &str) -> Option<Arc<Union>> {
    let idx = crate::db::workspace_index(db);
    let key = mir_types::Symbol::new(fqn).ascii_lowercase();
    let sf = match idx.functions.get(&key)? {
        SymbolLoc::Function { file, .. } => *file,
        _ => return None,
    };
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
) -> Option<Arc<Union>> {
    let idx = crate::db::workspace_index(db);
    let key = mir_types::Symbol::new(fqcn).ascii_lowercase();
    let sf = match idx.class_like.get(&key)? {
        SymbolLoc::Class { file, .. }
        | SymbolLoc::Interface { file, .. }
        | SymbolLoc::Trait { file, .. }
        | SymbolLoc::Enum { file, .. } => *file,
        _ => return None,
    };
    let inferred = crate::db::infer_file_return_types(db, sf);
    inferred
        .methods
        .get(&(Arc::<str>::from(fqcn), Arc::<str>::from(method_name_lower)))
        .cloned()
}
