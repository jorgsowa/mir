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

use parking_lot::Mutex;
use rayon::prelude::*;

use crate::db::MirDb;
use crate::php_version::PhpVersion;

/// Shared database holder with stub tracking. Owned by both ProjectAnalyzer and
/// AnalysisSession, providing a common point for their database operations.
pub struct SharedDb {
    /// Salsa database (source file handles live inside MirDb.source_files).
    pub salsa: Mutex<MirDb>,
    /// Stubs that have been ingested (for idempotency).
    pub loaded_stubs: Mutex<HashSet<&'static str>>,
    /// Whether user stubs have been ingested.
    pub user_stubs_loaded: std::sync::atomic::AtomicBool,
}

impl SharedDb {
    pub fn new() -> Self {
        Self {
            salsa: Mutex::new(MirDb::default()),
            loaded_stubs: Mutex::new(HashSet::new()),
            user_stubs_loaded: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Acquire a cheap clone of the salsa db for read-only queries.
    /// The lock is held only for the duration of the clone.
    pub fn snapshot_db(&self) -> MirDb {
        let guard = self.salsa.lock();
        guard.clone()
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

        let mut guard = self.salsa.lock();
        let mut loaded = self.loaded_stubs.lock();
        // Filter again under the lock to avoid double-ingestion races, then
        // bulk-ingest so the Arc::make_mut clones amortize over the batch
        // instead of paying per slice.
        let to_ingest: Vec<&mir_codebase::storage::StubSlice> = slices
            .iter()
            .filter_map(|(path, slice)| {
                if loaded.insert(*path) {
                    Some(slice)
                } else {
                    None
                }
            })
            .collect();
        guard.ingest_stub_slices(to_ingest.iter().copied());
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

        let slices = crate::stubs::user_stub_slices(files, dirs);
        let mut guard = self.salsa.lock();
        guard.ingest_stub_slices(slices.iter());
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
    ) -> crate::db::FileDefinitions {
        use mir_issues::Issue;

        // ---- Phase 1: parse + collect outside the lock ---------------------
        let arena = crate::arena::create_parse_arena(source.len());
        let parsed = php_rs_parser::parse(&arena, source);

        let mut all_issues: Vec<Issue> = parsed
            .errors
            .iter()
            .map(|err| {
                Issue::new(
                    mir_issues::IssueKind::ParseError {
                        message: err.to_string(),
                    },
                    mir_issues::Location {
                        file: file.clone(),
                        line: 1,
                        line_end: 1,
                        col_start: 0,
                        col_end: 0,
                    },
                )
            })
            .collect();

        let collector = crate::collector::DefinitionCollector::new_for_slice(
            file.clone(),
            source,
            &parsed.source_map,
        );
        let (slice, collector_issues) = collector.collect_slice(&parsed.program);
        all_issues.extend(collector_issues);

        let file_defs = crate::db::FileDefinitions {
            slice: Arc::new(slice),
            issues: Arc::new(all_issues),
        };

        // ---- Phase 2: register the salsa input + ingest under the lock -----
        // We hold the lock only for the two cheap writes; the expensive parse
        // and AST walk above ran lock-free.
        {
            let mut guard = self.salsa.lock();
            guard.upsert_source_file(file.clone(), Arc::from(source));
            guard.ingest_stub_slice(&file_defs.slice);
        }

        file_defs
    }
}

impl Default for SharedDb {
    fn default() -> Self {
        Self::new()
    }
}
