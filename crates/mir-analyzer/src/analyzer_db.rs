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

use crate::db::MirDbStorage;
use crate::php_version::PhpVersion;

/// Shared database holder with stub tracking. Owned by both ProjectAnalyzer and
/// AnalysisSession, providing a common point for their database operations.
pub struct AnalyzerDb {
    /// Salsa database, owned by the single writer; readers use `snapshot_db()`.
    pub(crate) salsa: MirDbStorage,
    /// Stubs that have been ingested (for idempotency).
    pub(crate) loaded_stubs: HashSet<&'static str>,
    /// Whether user stubs have been ingested.
    user_stubs_loaded: bool,
    /// Optional definition-collection disk cache. When `Some`, `collect_and_ingest_file`
    /// (the per-file LSP path) consults the cache before parsing and writes
    /// back on misses. Wired in by [`Self::attach_cache_dir`].
    pub(crate) stub_cache: Option<Arc<crate::stub_cache::StubSliceCache>>,
}

pub(crate) struct CollectedIngest {
    pub file_defs: crate::db::FileDefinitions,
    pub parsed: Option<php_rs_parser::ParseResult>,
}

/// Output of [`AnalyzerDb::prepare_ingest`], registered by [`AnalyzerDb::commit_ingest`].
pub(crate) struct PreparedIngest {
    file: Arc<str>,
    source: Arc<str>,
    durability: salsa::Durability,
    collected: CollectedIngest,
}

impl AnalyzerDb {
    pub fn new() -> Self {
        Self {
            salsa: MirDbStorage::default(),
            loaded_stubs: HashSet::new(),
            user_stubs_loaded: false,
            stub_cache: None,
        }
    }

    /// Attach a persistent definition cache stored under `cache_dir`. Future
    /// calls to [`Self::collect_and_ingest_file`] will consult the cache
    /// before parsing and write back on misses. The target PHP version is
    /// passed per call so the same cache directory remains usable across
    /// version changes (entries from other versions become misses).
    pub(crate) fn attach_cache_dir(&mut self, cache_dir: &std::path::Path) {
        let cache = Arc::new(crate::stub_cache::StubSliceCache::open(cache_dir));
        // Wire cache into the salsa db so collect_file_definitions can use it.
        self.salsa.set_stub_cache(cache.clone());
        self.stub_cache = Some(cache);
    }

    /// Number of [`crate::db::SourceFile`] inputs registered in salsa.
    /// Used by upstream cache-attach guards to detect "wire the cache
    /// before ingesting" violations.
    pub fn source_file_count(&self) -> usize {
        self.salsa.source_file_count()
    }

    /// Cheap read-only clone of the salsa storage; never blocks.
    pub fn snapshot_db(&self) -> MirDbStorage {
        self.salsa.clone()
    }

    /// Look up an existing [`crate::db::SourceFile`] handle by path.
    pub fn lookup_source_file(&self, path: &str) -> Option<crate::db::SourceFile> {
        use crate::db::MirDatabase as _;
        self.salsa.lookup_source_file(path)
    }

    /// Ingest multiple stub paths. Idempotent — already-loaded stubs are skipped.
    pub fn ingest_stub_paths(&mut self, paths: &[&'static str]) {
        for &path in paths {
            if !self.loaded_stubs.insert(path) {
                continue;
            }
            // Register as a SourceFile so the pull path (workspace_symbol_index
            // → collect_file_definitions) can index built-in PHP symbols.
            // Version filtering happens in collect_file_definitions_uncached via
            // db.php_version_str() / .with_php_version().
            // HIGH durability: built-in stubs never change within a session.
            if let Some(content) = crate::stubs::stub_content_for_path(path) {
                self.salsa.upsert_source_file_with_durability(
                    Arc::from(path),
                    Arc::from(content),
                    salsa::Durability::HIGH,
                );
            }
        }
    }

    /// Ingest user stub slices from configured files and directories.
    pub fn ingest_user_stubs(&mut self, files: &[PathBuf], dirs: &[PathBuf]) {
        if self.user_stubs_loaded || (files.is_empty() && dirs.is_empty()) {
            return;
        }

        // Collect paths + raw source so we can register SourceFile inputs.
        let mut all_paths: Vec<PathBuf> = files.to_vec();
        for dir in dirs {
            crate::stubs::collect_stub_dir_paths(dir, &mut all_paths);
        }
        let path_sources: Vec<(PathBuf, String)> = all_paths
            .into_iter()
            .filter_map(|p| match std::fs::read_to_string(&p) {
                Ok(s) => Some((p, s)),
                Err(e) => {
                    eprintln!(
                        "mir: warning: failed to read user stub {}: {e}",
                        p.display()
                    );
                    None
                }
            })
            .collect();

        // Register each user stub as a SourceFile so workspace_symbol_index
        // can index its functions, classes, etc. via the pull path.
        // Also mark each path as a user stub so user stubs take priority
        // over native stubs for the same symbol in workspace_symbol_index.
        for (path, source) in &path_sources {
            let path_arc: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
            // HIGH durability: user stubs are loaded once and never change within
            // a session (guarded by user_stubs_loaded). This lets salsa skip
            // O(N_user_stubs) dep-verification on every project-file edit.
            self.salsa.upsert_source_file_with_durability(
                path_arc.clone(),
                Arc::from(source.as_str()),
                salsa::Durability::HIGH,
            );
            self.salsa.register_user_stub_path(path_arc);
        }
        self.user_stubs_loaded = true;
    }

    /// Collect definitions from a file and ingest its stub slice.
    /// Used by both ProjectAnalyzer and AnalysisSession during file ingestion.
    pub fn collect_and_ingest_file(
        &mut self,
        file: Arc<str>,
        source: &str,
        php_version: PhpVersion,
    ) -> crate::db::FileDefinitions {
        self.collect_and_ingest_file_with_parsed(file, source, php_version)
            .file_defs
    }

    pub(crate) fn collect_and_ingest_file_with_parsed(
        &mut self,
        file: Arc<str>,
        source: &str,
        php_version: PhpVersion,
    ) -> CollectedIngest {
        let snapshot = self.snapshot_db();
        let prepared = Self::prepare_ingest(
            &snapshot,
            self.stub_cache.as_ref(),
            file,
            source,
            php_version,
        );
        // A salsa write waits for every other storage clone to drop.
        drop(snapshot);
        self.commit_ingest(prepared)
    }

    /// Register a [`PreparedIngest`]'s salsa input.
    pub(crate) fn commit_ingest(&mut self, prepared: PreparedIngest) -> CollectedIngest {
        self.salsa.upsert_source_file_with_durability(
            prepared.file,
            prepared.source,
            prepared.durability,
        );
        prepared.collected
    }

    /// Parse (or cache-hit) `source` and collect its definitions without
    /// writing salsa; safe to run concurrently on separate snapshots.
    pub(crate) fn prepare_ingest(
        db_snapshot: &MirDbStorage,
        stub_cache: Option<&Arc<crate::stub_cache::StubSliceCache>>,
        file: Arc<str>,
        source: &str,
        php_version: PhpVersion,
    ) -> PreparedIngest {
        use mir_issues::Issue;

        let php_v = php_version.cache_byte();

        // ---- Phase 0: cache lookup before parsing --------------------------
        // On a hit, we avoid the arena alloc, parse, and definition-collection
        // walk entirely — the dominant cost on cold sessions. Parse-error
        // issues aren't cached (they're reported through body analysis anyway for
        // project files), so a hit returns an empty issues list.

        // Always compute the content hash — needed for both cache paths and
        // for priming the in-process parse cache that collect_file_definitions
        // checks to avoid re-parsing in the same session.
        let content_hash = crate::stub_cache::hash_source(source);
        let source_arc: Arc<str> = Arc::from(source);

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
        let cached = db_snapshot.parse_cache().get(&content_hash, php_v);
        if let Some(cached) = cached {
            crate::metrics::record_stub_cache_hit();
            let same_path = cached.slice.file.as_deref() == Some(&*file);
            let slice_arc = if same_path {
                cached.slice
            } else {
                let mut owned = (*cached.slice).clone();
                owned.file = Some(file.clone());
                Arc::new(owned)
            };
            let issues = if same_path {
                cached.issues
            } else {
                Arc::new(crate::parse_cache::patch_issue_locations(
                    &cached.issues,
                    &file,
                ))
            };
            let file_defs = crate::db::FileDefinitions {
                slice: slice_arc,
                issues,
            };
            return PreparedIngest {
                file,
                source: source_arc,
                durability,
                collected: CollectedIngest {
                    file_defs,
                    parsed: None,
                },
            };
        }

        let cache_hit = stub_cache.and_then(|cache| {
            let (mut slice, issues) = cache.get(&file, &content_hash, php_v)?;
            crate::stub_cache::prepare_for_ingest(&mut slice);
            Some((slice, issues))
        });

        if let Some((slice, issues)) = cache_hit {
            crate::metrics::record_stub_cache_hit();
            let slice_arc = Arc::new(slice);
            let issues_arc = Arc::new(issues);
            // Prime the in-process cache so later collect_file_definitions calls hit.
            db_snapshot.prime_parse_cache(
                content_hash,
                php_v,
                slice_arc.clone(),
                issues_arc.clone(),
            );
            let file_defs = crate::db::FileDefinitions {
                slice: slice_arc,
                issues: issues_arc,
            };
            return PreparedIngest {
                file,
                source: source_arc,
                durability,
                collected: CollectedIngest {
                    file_defs,
                    parsed: None,
                },
            };
        }
        crate::metrics::record_stub_cache_miss();

        let parsed = php_rs_parser::parse(source);

        let has_hard_parse_errors = parsed.errors.iter().any(crate::parser::is_hard_parse_error);

        let mut all_issues: Vec<Issue> = parsed
            .errors
            .iter()
            .filter(|err| !crate::parser::is_spurious_reserved_class_error(err))
            .map(|err| crate::parser::parse_error_to_issue(err, &file, source, &parsed.source_map))
            .collect();

        let collector = crate::collector::DefinitionCollector::new_for_slice(
            file.clone(),
            source,
            &parsed.source_map,
        );
        let (mut slice, collector_issues) = collector.collect_slice(&parsed.program);
        all_issues.extend(collector_issues);
        mir_codebase::definitions::deduplicate_params_in_slice(&mut slice);

        let slice_arc = Arc::new(slice);
        let issues_arc = Arc::new(all_issues);

        // Write to the caches as long as the AST parsed cleanly. Collector
        // diagnostics (docblock warnings, etc.) leave the slice complete and
        // valid, so they should not block caching — see the matching comment
        // in `db::queries::collect_file_definitions_uncached`.
        if !has_hard_parse_errors {
            // In-process cache: prevents re-parsing in the same session, and
            // preserves these exact issues for a later hit (re-ingesting this
            // same file unchanged, or a different consumer querying it).
            db_snapshot.prime_parse_cache(
                content_hash,
                php_v,
                Arc::clone(&slice_arc),
                Arc::clone(&issues_arc),
            );
            // Disk cache: prevents re-parsing in future sessions.
            if let Some(cache) = stub_cache {
                cache.put(&file, &content_hash, php_v, &slice_arc, &issues_arc);
            }
        }

        let file_defs = crate::db::FileDefinitions {
            slice: slice_arc,
            issues: issues_arc,
        };

        PreparedIngest {
            file,
            source: source_arc,
            durability,
            collected: CollectedIngest {
                file_defs,
                parsed: Some(parsed),
            },
        }
    }
}

impl Default for AnalyzerDb {
    fn default() -> Self {
        Self::new()
    }
}
