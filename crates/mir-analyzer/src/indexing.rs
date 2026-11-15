//! Chunked, cancellable workspace-indexing primitives.
//!
//! These types support **eager background indexing**: at session start the
//! consumer enumerates every project + vendor file
//! (see [`crate::composer::Psr4Map::all_vendor_files`]) and pumps them through
//! [`crate::AnalysisSession::index_batch`] in bounded chunks. Each chunk takes
//! one short write window and merges its declarations into the workspace symbol
//! index incrementally, so the analyzer stays responsive (no multi-second
//! freeze) while the index fills, and the input set becomes static afterward —
//! no per-edit churn of the warm cache.
//!
//! The library owns **no** background thread. The consumer drives the pump:
//! an LSP server runs it on a worker thread; a single-threaded wasm host pumps
//! one chunk per `requestIdleCallback`/`setTimeout(0)` tick. Both pass the same
//! API; only [`IndexParallelism`] differs.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Cooperative cancellation flag shared between a consumer's driver and the
/// indexing/analysis calls it makes.
///
/// Salsa's own cancellation is query-granular and does not unwind the
/// plain-Rust body-analysis walk, so long-running loops here check this flag at
/// chunk / file boundaries instead. Snapshot passes also poll it between loop-body passes. On each new edit the consumer should drop
/// the old flag and create a fresh one for the new work rather than reusing a
/// single flag.
#[derive(Clone, Default)]
pub struct IndexCancel(Arc<AtomicBool>);

impl IndexCancel {
    /// A fresh, un-cancelled token.
    pub fn new() -> Self {
        Self::default()
    }

    /// Request cancellation. In-flight chunks finish their current bounded unit
    /// and stop at the next boundary.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

std::thread_local! {
    static ACTIVE_CANCEL: std::cell::RefCell<Option<IndexCancel>> = const { std::cell::RefCell::new(None) };
}

/// Runs `f` with `cancel` polled by statement analysis on this thread.
pub(crate) fn with_active_cancel<R>(cancel: &IndexCancel, f: impl FnOnce() -> R) -> R {
    struct Restore(Option<IndexCancel>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let previous = self.0.take();
            ACTIVE_CANCEL.with(|active| *active.borrow_mut() = previous);
        }
    }
    let previous = ACTIVE_CANCEL.with(|active| active.replace(Some(cancel.clone())));
    let _restore = Restore(previous);
    f()
}

/// Unwinds with [`salsa::Cancelled::Local`], as salsa does, once this thread's active cancel fired.
pub(crate) fn unwind_if_cancelled() {
    let cancelled = ACTIVE_CANCEL.with(|active| {
        active
            .borrow()
            .as_ref()
            .is_some_and(IndexCancel::is_cancelled)
    });
    if cancelled {
        std::panic::resume_unwind(Box::new(salsa::Cancelled::Local));
    }
}

/// How an [`crate::AnalysisSession::index_batch`] call parses the files in a
/// chunk. `Sequential` is required on wasm (no threads / no rayon); `Rayon`
/// parallelises the parse across the global thread pool on native consumers.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IndexParallelism {
    Sequential,
    Rayon,
}

/// Result of one [`crate::AnalysisSession::index_batch`] call.
#[derive(Clone, Copy, Debug, Default)]
pub struct IndexBatchOutcome {
    /// Files newly registered as salsa inputs by this batch (already-registered
    /// paths are updated in place and not counted).
    pub registered: usize,
    /// `true` if the cancel flag was observed; the batch may be partial.
    pub cancelled: bool,
    /// The workspace generation epoch after this batch (see
    /// [`crate::AnalysisSession::index_generation`]). The consumer records this
    /// alongside published diagnostics; when it later advances, affected open
    /// files become candidates for re-analysis.
    pub generation: u64,
}
