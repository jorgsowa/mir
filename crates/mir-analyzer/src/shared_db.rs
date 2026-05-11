//! Shared database and analysis operations for both ProjectAnalyzer and AnalysisSession.
//!
//! This module consolidates the common patterns both APIs need:
//! - Database management (Salsa cloning, snapshots)
//! - Stub loading and ingestion
//! - File definition collection
//!
//! By extracting these into a single place, both APIs benefit from the same code
//! paths and behavior, eliminating duplication and reducing maintenance burden.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use rayon::prelude::*;
use salsa::Setter as _;

use crate::db::{collect_file_definitions, MirDb, SourceFile};
use crate::php_version::PhpVersion;

/// Shared database holder with stub tracking. Owned by both ProjectAnalyzer and
/// AnalysisSession, providing a common point for their database operations.
pub struct SharedDb {
    /// Salsa database and registered source files.
    pub salsa: Mutex<(MirDb, HashMap<Arc<str>, SourceFile>)>,
    /// Stubs that have been ingested (for idempotency).
    pub loaded_stubs: Mutex<HashSet<&'static str>>,
    /// Whether user stubs have been ingested.
    pub user_stubs_loaded: std::sync::atomic::AtomicBool,
}

impl SharedDb {
    pub fn new() -> Self {
        Self {
            salsa: Mutex::new((MirDb::default(), HashMap::new())),
            loaded_stubs: Mutex::new(HashSet::new()),
            user_stubs_loaded: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Acquire a cheap clone of the salsa db for read-only queries.
    /// The lock is held only for the duration of the clone.
    pub fn snapshot_db(&self) -> MirDb {
        let guard = self.salsa.lock();
        guard.0.clone()
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
        for (path, slice) in slices {
            if loaded.insert(path) {
                guard.0.ingest_stub_slice(&slice);
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

        let slices = crate::stubs::user_stub_slices(files, dirs);
        let mut guard = self.salsa.lock();
        for slice in slices {
            guard.0.ingest_stub_slice(&slice);
        }
        self.user_stubs_loaded
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Collect definitions from a file and ingest its stub slice.
    /// Used by both ProjectAnalyzer and AnalysisSession during file ingestion.
    pub fn collect_and_ingest_file(
        &self,
        file: Arc<str>,
        source: &str,
    ) -> crate::db::FileDefinitions {
        let file_defs = {
            let mut guard = self.salsa.lock();
            let (ref mut db, ref mut files) = *guard;
            let salsa_file = match files.get(&file) {
                Some(&sf) => {
                    if sf.text(db).as_ref() != source {
                        sf.set_text(db).to(Arc::from(source));
                    }
                    sf
                }
                None => {
                    let sf = SourceFile::new(db, file.clone(), Arc::from(source));
                    files.insert(file.clone(), sf);
                    sf
                }
            };
            collect_file_definitions(db, salsa_file)
        };

        {
            let mut guard = self.salsa.lock();
            guard.0.ingest_stub_slice(&file_defs.slice);
        }

        file_defs
    }
}

impl Default for SharedDb {
    fn default() -> Self {
        Self::new()
    }
}
