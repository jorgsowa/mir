use super::*;

impl AnalysisSession {
    /// Returns `true` if a function with `fqn` is registered and active in
    /// the codebase. Case-insensitive lookup with optional leading backslash.
    pub fn contains_function(&self, fqn: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::function_exists(&db, fqn)
    }

    /// Returns `true` if a class / interface / trait / enum with `fqcn` is
    /// registered and active in the codebase.
    pub fn contains_class(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::class_exists(&db, fqcn)
    }

    /// Returns `true` if `class` has a method named `name` registered. Method
    /// names are matched case-insensitively (PHP method dispatch semantics).
    pub fn contains_method(&self, class: &str, name: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::has_method_in_chain(&db, class, name)
    }

    /// Resolve `fqcn` via the configured [`crate::ClassResolver`] and ingest
    /// the mapped file. The session keeps a negative cache so repeated calls
    /// for an unresolvable name don't re-hit the resolver; the cache is
    /// invalidated on any [`Self::ingest_file`] / [`Self::invalidate_file`].
    ///
    /// This is the LSP-friendly entry point: the analyzer never touches
    /// `vendor/` on its own, but consumers can ask it to resolve individual
    /// symbols on demand. Designed to be called when a diagnostic would
    /// otherwise report `UndefinedClass`.
    ///
    /// Returns a [`crate::LoadOutcome`] distinguishing
    /// already-loaded / freshly-loaded / not-resolvable. Use
    /// [`crate::LoadOutcome::is_loaded`] when only success matters.
    pub fn load_class(&self, fqcn: &str) -> crate::LoadOutcome {
        if self.contains_class(fqcn) {
            return crate::LoadOutcome::AlreadyLoaded;
        }
        if self.unresolvable_fqcns.read().contains_key(fqcn) {
            return crate::LoadOutcome::NotResolvable;
        }
        if self.try_resolve_and_ingest(fqcn) {
            crate::LoadOutcome::Loaded
        } else {
            // Cache the failure with the resolver-mapped path (if any) so
            // future file edits can selectively evict.
            let resolved_path: Option<Arc<str>> = self
                .resolver
                .as_ref()
                .and_then(|r| r.resolve(fqcn))
                .map(|p| Arc::from(p.to_string_lossy().as_ref()));
            let key: Arc<str> = Arc::from(fqcn);
            let mut cache = self.unresolvable_fqcns.write();
            if cache.len() >= UNRESOLVABLE_CACHE_CAP {
                cache.clear();
            }
            cache.insert(key, resolved_path);
            crate::LoadOutcome::NotResolvable
        }
    }

    /// Inner load path: resolver lookup + ingest, no caching. Returns `true`
    /// iff `fqcn` ends up registered. Failure buckets are recorded for
    /// telemetry.
    fn try_resolve_and_ingest(&self, fqcn: &str) -> bool {
        use crate::metrics::{record_lazy_load_failure, LazyLoadFailure};
        let Some(resolver) = &self.resolver else {
            record_lazy_load_failure(LazyLoadFailure::NoResolver, fqcn);
            return false;
        };
        let Some(path) = resolver.resolve(fqcn) else {
            record_lazy_load_failure(LazyLoadFailure::ResolverNone, fqcn);
            return false;
        };
        let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
        // Prefer in-memory text from a prior `set_file_text` /
        // `set_workspace_files` call; fall back to disk. This makes the LSP's
        // unsaved-edit buffer authoritative over the on-disk content for the
        // same path.
        let src: Arc<str> = match self.source_of(&file) {
            Some(text) => text,
            None => match self.source_provider.read(&path.to_string_lossy()) {
                Some(text) => text,
                None => {
                    record_lazy_load_failure(LazyLoadFailure::SourceUnreadable, fqcn);
                    return false;
                }
            },
        };
        self.ingest_file(file, src);
        if self.contains_class(fqcn) {
            true
        } else {
            record_lazy_load_failure(LazyLoadFailure::IngestThenMissing, fqcn);
            false
        }
    }

    /// Evict every negative-cache entry whose stored resolver-mapped path
    /// equals `file`. FQCNs cached as never-resolvable (path `None`) are left
    /// alone — no source-text change can make them resolvable.
    pub(super) fn evict_unresolvable_for_file(&self, file: &str) {
        let mut cache = self.unresolvable_fqcns.write();
        if cache.is_empty() {
            return;
        }
        cache.retain(|_fqcn, path| path.as_deref() != Some(file));
    }

    /// Bulk variant of [`Self::evict_unresolvable_for_file`]. One `HashSet`
    /// build + one pass over the cache; no resolver calls.
    pub(super) fn evict_unresolvable_for_files(&self, files: &[Arc<str>]) {
        let mut cache = self.unresolvable_fqcns.write();
        if cache.is_empty() {
            return;
        }
        let registered: HashSet<&str> = files.iter().map(|f| f.as_ref()).collect();
        cache.retain(|_fqcn, path| match path {
            Some(p) => !registered.contains(p.as_ref()),
            None => true,
        });
    }
}
