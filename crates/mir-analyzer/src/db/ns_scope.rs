//! Selects the active `namespace` block while analyzing a declaration in a
//! file with several, so `file_namespace`/`file_imports` answer per block.

use std::cell::RefCell;
use std::sync::Arc;

use mir_codebase::NamespaceBlock;

use super::MirDatabase;

thread_local! {
    static ACTIVE: RefCell<Option<(Arc<str>, u32)>> = const { RefCell::new(None) };
}

pub(crate) struct NsScopeGuard(Option<Option<(Arc<str>, u32)>>);

impl Drop for NsScopeGuard {
    fn drop(&mut self) {
        if let Some(prev) = self.0.take() {
            ACTIVE.with(|a| *a.borrow_mut() = prev);
        }
    }
}

/// Resolves names at byte `offset` of `file` against its enclosing namespace
/// block until the guard drops. No-op for files with at most one namespace.
pub(crate) fn enter(db: &dyn MirDatabase, file: &Arc<str>, offset: u32) -> NsScopeGuard {
    if db.file_namespace_blocks(file).is_empty() {
        return NsScopeGuard(None);
    }
    NsScopeGuard(Some(
        ACTIVE.with(|a| a.borrow_mut().replace((file.clone(), offset))),
    ))
}

/// The block of `file` selected by the innermost active guard, if any.
pub(super) fn active_block(
    file: &str,
    blocks: impl FnOnce() -> Arc<[NamespaceBlock]>,
) -> Option<NamespaceBlock> {
    let offset = ACTIVE.with(|a| match &*a.borrow() {
        Some((f, off)) if f.as_ref() == file => Some(*off),
        _ => None,
    })?;
    blocks()
        .iter()
        .find(|b| b.start <= offset && offset < b.end)
        .cloned()
}
