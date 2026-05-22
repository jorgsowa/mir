//! Salsa-pure storage for Pass-2-inferred return types.
//!
//! One singleton `#[salsa::input] InferredReturnTypes` per database, holding
//! `Arc<FxHashMap>`s keyed by FQN (functions) and `(FQCN, name_lower)` tuples
//! (methods). The input handle is created lazily on first commit and stored on
//! `MirDb::inferred_return_types`. Salsa's ptr_eq update semantics make
//! replacing the maps cheap when their contents are unchanged.
//!
//! Pass-2 callers go through [`inferred_function_return_type`] /
//! [`inferred_method_return_type`].

use std::sync::Arc;

use mir_types::Union;
use rustc_hash::FxHashMap;

use crate::db::{MirDatabase, SymbolLoc};

/// Map of function FQN → inferred return type.
pub type FunctionInferredMap = FxHashMap<Arc<str>, Arc<Union>>;

/// Map of `(FQCN, method_name_lower)` → inferred return type.
pub type MethodInferredMap = FxHashMap<(Arc<str>, Arc<str>), Arc<Union>>;

/// Singleton salsa input holding the post-sweep inferred return types.
///
/// `MirDb::set_inferred_return_types_map` lazily creates this input the
/// first time the inference sweep commits. Subsequent commits replace the
/// inner `Arc<...>` maps; salsa's default `Update` impl for `Arc<T>` uses
/// ptr_eq so unchanged commits don't invalidate downstream queries.
#[salsa::input]
pub struct InferredReturnTypes {
    pub functions: Arc<FunctionInferredMap>,
    pub methods: Arc<MethodInferredMap>,
}

/// Look up the inferred return type for `fqn`. Returns `None` if no
/// inference has been committed yet, or the function isn't in the map.
pub fn inferred_function_return_type(db: &dyn MirDatabase, fqn: &str) -> Option<Arc<Union>> {
    db.inferred_return_types()?.functions(db).get(fqn).cloned()
}

/// Look up the inferred return type for `(fqcn, method_name_lower)`.
/// Caller must pre-lowercase the method name (PHP semantics).
pub fn inferred_method_return_type(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name_lower: &str,
) -> Option<Arc<Union>> {
    let irt = db.inferred_return_types()?;
    irt.methods(db)
        .get(&(Arc::<str>::from(fqcn), Arc::<str>::from(method_name_lower)))
        .cloned()
}

/// Demand-driven version of [`inferred_function_return_type`].
///
/// Locates the file that declares `fqn` via the workspace symbol index, then
/// calls [`crate::db::infer_file_return_types`] on that file. Salsa
/// memoizes both queries, so repeated lookups for the same function are free.
/// Returns `None` when the function is unknown or not in the workspace index.
pub fn inferred_function_return_type_demand(db: &dyn MirDatabase, fqn: &str) -> Option<Arc<Union>> {
    let idx = if let Some(singleton) = db.workspace_symbol_index_singleton() {
        singleton.index(db)
    } else {
        crate::db::workspace_symbol_index(db)
    };
    let fqn_lower = fqn.to_ascii_lowercase();
    let sf = match idx.functions.get(&fqn_lower)? {
        SymbolLoc::Function { file, .. } => *file,
        _ => return None,
    };
    let inferred = crate::db::infer_file_return_types(db, sf);
    inferred.functions.get(fqn).cloned()
}

/// Demand-driven version of [`inferred_method_return_type`].
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
    let idx = if let Some(singleton) = db.workspace_symbol_index_singleton() {
        singleton.index(db)
    } else {
        crate::db::workspace_symbol_index(db)
    };
    let fqcn_lower = fqcn.to_ascii_lowercase();
    let sf = match idx.class_like.get(&fqcn_lower)? {
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
