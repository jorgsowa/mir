//! Shared database and analysis operations for both ProjectAnalyzer and AnalysisSession.
//!
//! This module consolidates the common patterns both APIs need:
//! - Database management (Salsa cloning, snapshots)
//! - Stub loading and ingestion
//! - File definition collection
//!
//! By extracting these into a single place, both APIs benefit from the same code
//! paths and behavior, eliminating duplication and reducing maintenance burden.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::db::MirDatabase;
use parking_lot::{Mutex, RwLock};
use rayon::prelude::*;

use crate::db::MirDb;
use crate::php_version::PhpVersion;

/// Newtype that allows `RwLock<MirDbRw>` in `SharedDb`.
///
/// SAFETY: Under the *read* lock only these operations are performed on the
/// shared `&MirDb`:
///
///   1. `clone()` — increments `Arc` refcounts; allocates a fresh `ZalsaLocal`
///      for the clone without touching the original `ZalsaLocal`.
///   2. `source_file_count()` — reads `self.source_files.len()`, a plain
///      `HashMap` field, no Salsa involvement.
///   3. `replay_reference_locations()` — writes into `Mutex<HashMap<_>>` fields
///      (`file_references`, `reference_locations`), no `ZalsaLocal` access.
///
/// None of these touch the `RefCell<QueryStack>` inside `ZalsaLocal`, so
/// concurrent read-lock holders are data-race-free. Under the *write* lock
/// access is exclusive, so there is no aliasing.
///
/// Callers MUST NOT call Salsa input-field getters (e.g. `node.text(db)`,
/// `node.is_interface(db)`) on a shared `&MirDbRw` under the read lock —
/// those write to `ZalsaLocal`. Use `snapshot_db()` for all other reads.
pub(crate) struct MirDbRw(MirDb);

unsafe impl Sync for MirDbRw {}

impl std::ops::Deref for MirDbRw {
    type Target = MirDb;
    fn deref(&self) -> &MirDb {
        &self.0
    }
}

impl std::ops::DerefMut for MirDbRw {
    fn deref_mut(&mut self) -> &mut MirDb {
        &mut self.0
    }
}

/// Shared database holder with stub tracking. Owned by both ProjectAnalyzer and
/// AnalysisSession, providing a common point for their database operations.
pub struct SharedDb {
    /// Salsa database (source file handles live inside MirDb.source_files).
    /// RwLock: multiple concurrent snapshot_db() reads; exclusive for writes.
    pub salsa: RwLock<MirDbRw>,
    /// Stubs that have been ingested (for idempotency).
    pub loaded_stubs: Mutex<HashSet<&'static str>>,
    /// Whether user stubs have been ingested.
    pub user_stubs_loaded: std::sync::atomic::AtomicBool,
    /// Optional Pass-1 disk cache. When `Some`, `collect_and_ingest_file`
    /// (the per-file LSP path) consults the cache before parsing and writes
    /// back on misses. Wired in by [`Self::with_cache_dir`].
    pub(crate) stub_cache: Option<Arc<crate::stub_cache::StubSliceCache>>,
}

impl SharedDb {
    pub fn new() -> Self {
        let mut db = MirDb::default();
        // Pre-create the WorkspaceRevision salsa input so workspace_symbol_index
        // always reads it and salsa properly invalidates it on first file add.
        // Without this, querying workspace_symbol_index before any file is
        // ingested memoizes an empty result that salsa can never invalidate
        // (because the query never read the revision during that execution).
        db.init_workspace_revision();
        Self {
            salsa: RwLock::new(MirDbRw(db)),
            loaded_stubs: Mutex::new(HashSet::new()),
            user_stubs_loaded: std::sync::atomic::AtomicBool::new(false),
            stub_cache: None,
        }
    }

    /// Attach a persistent Pass-1 cache stored under `cache_dir`. Future
    /// calls to [`Self::collect_and_ingest_file`] will consult the cache
    /// before parsing and write back on misses. The target PHP version is
    /// passed per call so the same cache directory remains usable across
    /// version changes (entries from other versions become misses).
    pub fn with_cache_dir(mut self, cache_dir: &std::path::Path) -> Self {
        let cache = Arc::new(crate::stub_cache::StubSliceCache::open(cache_dir));
        // Wire cache into the salsa db so collect_file_definitions can use it.
        self.salsa.write().set_stub_cache(cache.clone());
        self.stub_cache = Some(cache);
        self
    }

    /// Number of [`crate::db::SourceFile`] inputs registered in salsa.
    /// Used by upstream cache-attach guards to detect "wire the cache
    /// before ingesting" violations.
    pub fn source_file_count(&self) -> usize {
        self.salsa.read().source_file_count()
    }

    /// Acquire a cheap clone of the salsa db for read-only queries.
    /// Multiple callers may snapshot concurrently; the read lock is held
    /// only for the duration of the clone.
    pub fn snapshot_db(&self) -> MirDb {
        let guard = self.salsa.read();
        (**guard).clone()
    }

    /// Ingest multiple stub paths in parallel then serially under the lock.
    /// Idempotent — already-loaded stubs are skipped.
    pub fn ingest_stub_paths(&self, paths: &[&'static str], php_version: PhpVersion) {
        // Identify needed paths (filter to those not yet loaded).
        let needed: Vec<&'static str> = {
            let loaded = self.loaded_stubs.lock();
            paths
                .iter()
                .copied()
                .filter(|p| !loaded.contains(p))
                .collect()
        };

        if needed.is_empty() {
            return;
        }

        // Parse in parallel; ingest serially under write lock.
        let slices: Vec<(&'static str, mir_codebase::storage::StubSlice)> = needed
            .par_iter()
            .filter_map(|&path| {
                crate::stubs::stub_content_for_path(path).map(|content| {
                    let slice =
                        crate::stubs::stub_slice_from_source(path, content, Some(php_version));
                    (path, slice)
                })
            })
            .collect();

        let mut guard = self.salsa.write();
        let mut loaded = self.loaded_stubs.lock();
        // Filter again under the lock to avoid double-ingestion races, then
        // bulk-ingest so the Arc::make_mut clones amortize over the batch
        // instead of paying per slice.
        for (path, _slice) in &slices {
            if loaded.insert(*path) {
                // Register as a SourceFile so the pull path (workspace_symbol_index
                // → collect_file_definitions) can index built-in PHP classes.
                // HIGH durability: built-in stubs never change within a session.
                if let Some(content) = crate::stubs::stub_content_for_path(path) {
                    guard.upsert_source_file_with_durability(
                        Arc::from(*path),
                        Arc::from(content),
                        salsa::Durability::HIGH,
                    );
                }
            }
        }
    }

    /// Ingest user stub slices from configured files and directories.
    pub fn ingest_user_stubs(&self, files: &[PathBuf], dirs: &[PathBuf]) {
        if files.is_empty() && dirs.is_empty() {
            return;
        }

        let was_loaded = self
            .user_stubs_loaded
            .load(std::sync::atomic::Ordering::Relaxed);
        if was_loaded {
            return;
        }

        // Collect paths + raw source so we can register SourceFile inputs.
        let mut all_paths: Vec<PathBuf> = files.to_vec();
        for dir in dirs {
            crate::stubs::collect_stub_dir_paths(dir, &mut all_paths);
        }
        let path_sources: Vec<(PathBuf, String)> = all_paths
            .into_iter()
            .filter_map(|p| std::fs::read_to_string(&p).ok().map(|s| (p, s)))
            .collect();

        let mut guard = self.salsa.write();
        // Register each user stub as a SourceFile so workspace_symbol_index
        // can index its functions, classes, etc. via the pull path.
        // Also mark each path as a user stub so user stubs take priority
        // over native stubs for the same symbol in workspace_symbol_index.
        for (path, source) in &path_sources {
            let path_arc: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
            guard.upsert_source_file(path_arc.clone(), Arc::from(source.as_str()));
            guard.register_user_stub_path(path_arc);
        }
        self.user_stubs_loaded
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Collect definitions from a file and ingest its stub slice.
    /// Used by both ProjectAnalyzer and AnalysisSession during file ingestion.
    ///
    /// **Lock discipline:** parsing and definition collection happen *outside*
    /// the salsa write lock — they don't need the db beyond reading the source
    /// text we already have in hand. Only the salsa input update and the slice
    /// ingestion happen under the lock. This lets concurrent readers (e.g. an
    /// LSP serving hover requests on a snapshot) proceed in parallel with the
    /// expensive parse step.
    pub fn collect_and_ingest_file(
        &self,
        file: Arc<str>,
        source: &str,
        php_version: PhpVersion,
    ) -> crate::db::FileDefinitions {
        use mir_issues::Issue;

        let php_v = php_version.cache_byte();

        // ---- Phase 0: cache lookup before parsing --------------------------
        // On a hit, we avoid the arena alloc, parse, and definition-collection
        // walk entirely — the dominant cost on cold sessions. Parse-error
        // issues aren't cached (they're reported through Pass 2 anyway for
        // project files), so a hit returns an empty issues list.

        // Always compute the content hash — needed for both cache paths and
        // for priming the in-process parse cache that collect_file_definitions
        // checks to avoid re-parsing in the same session.
        let content_hash = crate::stub_cache::hash_source(source);

        // Vendor and user-stub files won't change within a session; project
        // files may be edited repeatedly. HIGH durability tells salsa it can
        // skip re-verifying vendor SourceFiles when only project files change,
        // reducing O(N_total_files) verification to O(N_project_files) on
        // every incremental edit.
        let durability = if file.contains("/vendor/") || file.contains("\\vendor\\") {
            salsa::Durability::HIGH
        } else {
            salsa::Durability::LOW
        };

        // Check in-process parse cache first (fastest path, avoids even disk I/O).
        {
            let guard = self.salsa.read();
            let cached = guard
                .parse_cache()
                .get(&content_hash)
                .map(|r| Arc::clone(&*r));
            drop(guard);
            if let Some(cached) = cached {
                crate::metrics::record_stub_cache_hit();
                let slice_arc = if cached.file.as_deref() == Some(&*file) {
                    // Path matches — share the Arc directly (no data clone needed).
                    cached
                } else {
                    // Different path — fix the `file` field.
                    let mut owned = (*cached).clone();
                    owned.file = Some(file.clone());
                    Arc::new(owned)
                };
                let file_defs = crate::db::FileDefinitions {
                    slice: slice_arc,
                    issues: Arc::new(Vec::new()),
                };
                let mut write_guard = self.salsa.write();
                write_guard.upsert_source_file_with_durability(
                    file.clone(),
                    Arc::from(source),
                    durability,
                );
                return file_defs;
            }
        }

        let cache_hit = self.stub_cache.as_ref().and_then(|cache| {
            let mut slice = cache.get(&file, &content_hash, php_v)?;
            crate::stub_cache::prepare_for_ingest(&mut slice);
            Some(slice)
        });

        if let Some(slice) = cache_hit {
            crate::metrics::record_stub_cache_hit();
            let slice_arc = Arc::new(slice);
            // Prime the in-process cache so later collect_file_definitions calls hit.
            self.salsa
                .read()
                .prime_parse_cache(content_hash, slice_arc.clone());
            let file_defs = crate::db::FileDefinitions {
                slice: slice_arc,
                issues: Arc::new(Vec::new()),
            };
            let mut guard = self.salsa.write();
            guard.upsert_source_file_with_durability(file.clone(), Arc::from(source), durability);
            return file_defs;
        }
        crate::metrics::record_stub_cache_miss();

        // ---- Phase 1: parse + collect outside the lock ---------------------
        let parsed = php_rs_parser::parse(source);

        let has_hard_parse_errors = parsed.errors.iter().any(crate::parser::is_hard_parse_error);

        let mut all_issues: Vec<Issue> = parsed
            .errors
            .iter()
            .map(|err| crate::parser::parse_error_to_issue(err, &file, source, &parsed.source_map))
            .collect();

        let collector = crate::collector::DefinitionCollector::new_for_slice(
            file.clone(),
            source,
            &parsed.source_map,
        );
        let (mut slice, collector_issues) = collector.collect_slice(&parsed.program);
        let has_collector_issues = !collector_issues.is_empty();
        all_issues.extend(collector_issues);
        mir_codebase::storage::deduplicate_params_in_slice(&mut slice);

        let slice_arc = Arc::new(slice);

        // Write to the caches on a clean parse so future lookups hit.
        if !has_hard_parse_errors && !has_collector_issues {
            // In-process cache: prevents re-parsing in the same session.
            self.salsa
                .read()
                .prime_parse_cache(content_hash, Arc::clone(&slice_arc));
            // Disk cache: prevents re-parsing in future sessions.
            if let Some(cache) = &self.stub_cache {
                cache.put(&file, &content_hash, php_v, &slice_arc);
            }
        }

        let file_defs = crate::db::FileDefinitions {
            slice: slice_arc,
            issues: Arc::new(all_issues),
        };

        // ---- Phase 2: register the salsa input under the write lock --
        // The expensive parse and AST walk above ran lock-free.
        {
            let mut guard = self.salsa.write();
            guard.upsert_source_file_with_durability(file.clone(), Arc::from(source), durability);
        }

        file_defs
    }
}

impl Default for SharedDb {
    fn default() -> Self {
        Self::new()
    }
}
