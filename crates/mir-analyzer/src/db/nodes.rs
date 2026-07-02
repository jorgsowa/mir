use std::sync::Arc;

use mir_codebase::StubSlice;
use mir_issues::Issue;

// SourceFile input (S1)

/// Source file registered as a Salsa input.
/// Setting `text` on an existing `SourceFile` is the single write that drives
/// all downstream query invalidation.
#[salsa::input]
pub struct SourceFile {
    pub path: Arc<str>,
    pub text: Arc<str>,
}

// FileDefinitions (S1)

/// Result of the `collect_file_definitions` tracked query.
#[derive(Clone, Debug)]
pub struct FileDefinitions {
    pub slice: Arc<StubSlice>,
    pub issues: Arc<Vec<Issue>>,
}

impl FileDefinitions {
    /// FQ names this file defines (classes, interfaces, traits, enums,
    /// functions, constants, global vars). Lets callers that already hold a
    /// `FileDefinitions` derive the symbol set without re-running the
    /// `collect_file_definitions` salsa query.
    pub(crate) fn defined_symbols(&self) -> rustc_hash::FxHashSet<Arc<str>> {
        let mut out = rustc_hash::FxHashSet::default();
        for c in self.slice.classes.iter() {
            out.insert(c.fqcn.clone());
        }
        for i in self.slice.interfaces.iter() {
            out.insert(i.fqcn.clone());
        }
        for t in self.slice.traits.iter() {
            out.insert(t.fqcn.clone());
        }
        for e in self.slice.enums.iter() {
            out.insert(e.fqcn.clone());
        }
        for f in self.slice.functions.iter() {
            out.insert(f.fqn.clone());
        }
        for (name, _) in self.slice.constants.iter() {
            out.insert(name.clone());
        }
        for (name, _) in self.slice.global_vars.iter() {
            let gname: Arc<str> = Arc::from(name.strip_prefix('$').unwrap_or(name.as_ref()));
            out.insert(gname);
        }
        out
    }
}

impl PartialEq for FileDefinitions {
    fn eq(&self, other: &Self) -> bool {
        // Structural comparison on the slice lets salsa skip workspace_symbol_index
        // rebuilds when file text changed but definitions are unchanged
        // (e.g. comment edits, whitespace-only saves in LSP).
        // Issues are deliberately excluded: diagnostics changing doesn't affect
        // symbol lookups and shouldn't force the workspace index to rebuild.
        if Arc::ptr_eq(&self.slice, &other.slice) {
            return true;
        }
        *self.slice == *other.slice
    }
}

// SAFETY: FileDefinitions contains Arc pointers and Vec, which are Move-safe.
// The pointer passed to maybe_update is provided by Salsa and points to
// properly aligned and initialized memory. We have exclusive write access
// through the mutable pointer (Salsa guarantees this). The in-place update
// is safe because we own both the old and new values.
//
// Optimization: Use PartialEq to skip downstream recomputation when definitions
// haven't changed (e.g., no-op file saves in LSP). This is especially valuable
// in incremental scenarios where many files are unchanged.
unsafe impl salsa::Update for FileDefinitions {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false; // Content unchanged; Salsa skips dependent queries
        }
        *old = new_val;
        true
    }
}

// Ancestors return type (S2)

/// The computed ancestor list for a class or interface.
///
/// Uses content equality so Salsa's cycle-convergence check can detect
/// fixpoints correctly (two empty lists from different iterations are equal).
#[derive(Clone, Debug, Default)]
pub struct Ancestors(pub Vec<Arc<str>>);

impl PartialEq for Ancestors {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(&other.0)
                .all(|(a, b)| a.as_ref() == b.as_ref())
    }
}

// SAFETY: Ancestors contains Arc pointers, which are Move-safe.
// The pointer passed to maybe_update is provided by Salsa and points to
// properly aligned and initialized memory. We dereference it to check equality
// and conditionally update. Salsa guarantees exclusive write access through
// the mutable pointer. The comparison is safe because we're comparing valid
// initialized values.
unsafe impl salsa::Update for Ancestors {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}
