//! Abstraction over how the analyzer obtains file source text it doesn't yet
//! have in its in-memory salsa inputs.
//!
//! Today the analyzer occasionally needs to fault in a file (lazy-load of a
//! referenced class). The default implementation [`FsSourceProvider`] reads
//! disk; LSPs swap in a VFS-backed provider so unsaved editor buffers
//! authoritatively override the on-disk content.
//!
//! Boundary: the analyzer never invents file paths. Paths come from one of
//! - a class resolver (PSR-4, classmap) it was configured with, or
//! - `AnalysisSession::set_file_text` registrations from the consumer.
//!
//! Consumers can therefore reason about exactly which paths the analyzer
//! might ask for, and serve them from whatever source they prefer.

use std::sync::Arc;

/// Read a file's source text on demand. Returns `None` if the path is
/// unreadable or doesn't exist — the analyzer treats that as "this class is
/// unresolvable" and (negative-)caches the failure.
pub trait SourceProvider: Send + Sync {
    fn read(&self, path: &str) -> Option<Arc<str>>;

    /// Hint that `paths` will be read soon, so they may be fetched concurrently.
    fn prefetch(&self, _paths: &[String]) {}

    /// Drop hinted text that was never read.
    fn clear_prefetch(&self) {}
}

/// Reads source text from the local filesystem. The default provider for
/// CLI batch contexts.
#[derive(Default)]
pub struct FsSourceProvider {
    prefetched: parking_lot::Mutex<rustc_hash::FxHashMap<String, Option<Arc<str>>>>,
}

impl FsSourceProvider {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SourceProvider for FsSourceProvider {
    fn read(&self, path: &str) -> Option<Arc<str>> {
        if let Some(text) = self.prefetched.lock().remove(path) {
            return text;
        }
        std::fs::read_to_string(path).ok().map(Arc::from)
    }

    // Opens right after a burst of file opens are slow on macOS but scale across threads.
    fn prefetch(&self, paths: &[String]) {
        use rayon::prelude::*;
        let fetched: Vec<(String, Option<Arc<str>>)> = paths
            .par_iter()
            .map(|path| {
                (
                    path.clone(),
                    std::fs::read_to_string(path).ok().map(Arc::from),
                )
            })
            .collect();
        self.prefetched.lock().extend(fetched);
    }

    fn clear_prefetch(&self) {
        self.prefetched.lock().clear();
    }
}
