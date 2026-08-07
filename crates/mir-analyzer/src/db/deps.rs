//! Structural file-dependency edges as a tracked query.
//!
//! [`file_structural_deps`] memoizes the set of files `file` depends on
//! through its *declarations*: defining files of its `use` imports, parent /
//! interface / trait FQCNs, and named-object type hints on properties,
//! params and return types. Body-level bare-FQN references are deliberately
//! excluded — they live in the reference index (a side channel salsa cannot
//! observe) and are merged in by `AnalysisSession::dependency_graph`.
//!
//! Memoization makes the warm `dependency_graph()` rebuild cheap: the query
//! depends on the file's definitions and on the workspace symbol index
//! revision (via `symbol_defining_file`), so it re-runs only when the file's
//! declarations or the symbol→file mapping actually change.

use std::sync::Arc;

use rustc_hash::FxHashSet;

use super::*;
use crate::db::workspace::structural_symbols_from_slice;

/// Files that `file`'s declarations depend on. Sorted for deterministic
/// memo equality. Self-edges are excluded.
#[salsa::tracked]
pub fn file_structural_deps(db: &dyn MirDatabase, file: SourceFile) -> Arc<[Arc<str>]> {
    let path = file.path(db);
    let mut targets: FxHashSet<Arc<str>> = FxHashSet::default();

    let mut add_target = |symbol: &str| {
        if let Some(defining_file) = db.symbol_defining_file(symbol) {
            if defining_file.as_ref() != path.as_ref() {
                targets.insert(defining_file);
            }
        }
    };

    let defs = crate::db::collect_file_definitions(db, file);
    for symbol in structural_symbols_from_slice(&defs.slice, file).iter() {
        add_target(symbol.as_ref());
    }

    let mut sorted: Vec<Arc<str>> = targets.into_iter().collect();
    sorted.sort();
    sorted.into()
}
