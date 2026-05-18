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
}

/// Reads source text from the local filesystem. The default provider for
/// CLI batch contexts.
pub struct FsSourceProvider;

impl SourceProvider for FsSourceProvider {
    fn read(&self, path: &str) -> Option<Arc<str>> {
        std::fs::read_to_string(path).ok().map(Arc::from)
    }
}
