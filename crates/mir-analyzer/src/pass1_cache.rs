//! Per-file Pass 1 definition cache.
//!
//! Snapshots all definitions produced by the combined pre-index +
//! definition-collection pass and persists them to `{cache_dir}/pass1.bin`
//! using bincode.  On subsequent runs, unchanged files skip parsing and
//! definition collection entirely.
//!
//! # Correctness notes
//!
//! * Snapshots are built **before** `Codebase::finalize()` so that the
//!   derived `all_methods` / `all_parents` fields in `ClassStorage` are
//!   intentionally empty.  `finalize()` recomputes them from the replayed
//!   data exactly as it does for freshly-parsed files.
//! * Global PHP constants (`StmtKind::Const` / `define()`) are not
//!   included in snapshots because `DefinitionCollector` does not add them
//!   to `symbol_to_file`; this matches the existing behaviour of
//!   `Codebase::remove_file_definitions`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use mir_codebase::storage::{
    ClassStorage, EnumStorage, FunctionStorage, InterfaceStorage, TraitStorage,
};
use mir_codebase::Codebase;
use mir_issues::Issue;
use mir_types::Union;

// ---------------------------------------------------------------------------
// Pass1Snapshot
// ---------------------------------------------------------------------------

/// All Pass 1 definitions produced for a single PHP file.
///
/// Serialized to/from disk; replayed into the codebase on a cache hit to skip
/// parsing and definition collection for that file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pass1Snapshot {
    pub content_hash: String,
    /// Declared namespace, from the pre-index pass.
    pub namespace: Option<String>,
    /// `use`-alias imports, from the pre-index pass.
    pub imports: HashMap<String, String>,
    pub classes: Vec<(Arc<str>, ClassStorage)>,
    pub interfaces: Vec<(Arc<str>, InterfaceStorage)>,
    pub traits: Vec<(Arc<str>, TraitStorage)>,
    pub enums: Vec<(Arc<str>, EnumStorage)>,
    pub functions: Vec<(Arc<str>, FunctionStorage)>,
    /// `@var`-annotated global variables defined in this file.
    pub global_vars: Vec<(Arc<str>, Union)>,
    /// Parse errors emitted for this file.
    pub parse_errors: Vec<Issue>,
    /// Issues from definition collection (typically empty).
    pub definition_issues: Vec<Issue>,
}

impl Pass1Snapshot {
    /// Write all stored definitions back into `codebase`.
    ///
    /// Call this before `Codebase::finalize()` so that derived fields
    /// (`all_methods`, `all_parents`) are recomputed from the replayed data.
    pub fn replay(&self, codebase: &Codebase, file: &Arc<str>) {
        if let Some(ref ns) = self.namespace {
            codebase.file_namespaces.insert(file.clone(), ns.clone());
        }
        if !self.imports.is_empty() {
            codebase
                .file_imports
                .insert(file.clone(), self.imports.clone());
        }
        for (fqcn, s) in &self.classes {
            codebase.known_symbols.insert(fqcn.clone());
            codebase.symbol_to_file.insert(fqcn.clone(), file.clone());
            codebase.classes.insert(fqcn.clone(), s.clone());
        }
        for (fqcn, s) in &self.interfaces {
            codebase.known_symbols.insert(fqcn.clone());
            codebase.symbol_to_file.insert(fqcn.clone(), file.clone());
            codebase.interfaces.insert(fqcn.clone(), s.clone());
        }
        for (fqcn, s) in &self.traits {
            codebase.known_symbols.insert(fqcn.clone());
            codebase.symbol_to_file.insert(fqcn.clone(), file.clone());
            codebase.traits.insert(fqcn.clone(), s.clone());
        }
        for (fqcn, s) in &self.enums {
            codebase.known_symbols.insert(fqcn.clone());
            codebase.symbol_to_file.insert(fqcn.clone(), file.clone());
            codebase.enums.insert(fqcn.clone(), s.clone());
        }
        for (fqn, s) in &self.functions {
            codebase.known_symbols.insert(fqn.clone());
            codebase.symbol_to_file.insert(fqn.clone(), file.clone());
            codebase.functions.insert(fqn.clone(), s.clone());
        }
        for (name, ty) in &self.global_vars {
            codebase.register_global_var(file, name.clone(), ty.clone());
        }
    }
}

// ---------------------------------------------------------------------------
// build_snapshot
// ---------------------------------------------------------------------------

/// Construct a `Pass1Snapshot` for `file` from the codebase after Pass 1.
///
/// `fqcns` is every FQCN defined in this file — obtained from the reverse
/// of `Codebase::symbol_to_file`.  Must be called after the parallel Pass 1
/// completes but **before** `Codebase::finalize()`.
pub fn build_snapshot(
    codebase: &Codebase,
    file: &Arc<str>,
    content_hash: String,
    fqcns: &[Arc<str>],
    parse_errors: Vec<Issue>,
    definition_issues: Vec<Issue>,
) -> Pass1Snapshot {
    let namespace = codebase
        .file_namespaces
        .get(file.as_ref())
        .map(|n| n.clone());
    let imports = codebase
        .file_imports
        .get(file.as_ref())
        .map(|i| i.clone())
        .unwrap_or_default();

    let mut classes = Vec::new();
    let mut interfaces = Vec::new();
    let mut traits = Vec::new();
    let mut enums = Vec::new();
    let mut functions = Vec::new();

    for fqcn in fqcns {
        if let Some(s) = codebase.classes.get(fqcn.as_ref()) {
            classes.push((fqcn.clone(), s.clone()));
        } else if let Some(s) = codebase.interfaces.get(fqcn.as_ref()) {
            interfaces.push((fqcn.clone(), s.clone()));
        } else if let Some(s) = codebase.traits.get(fqcn.as_ref()) {
            traits.push((fqcn.clone(), s.clone()));
        } else if let Some(s) = codebase.enums.get(fqcn.as_ref()) {
            enums.push((fqcn.clone(), s.clone()));
        } else if let Some(s) = codebase.functions.get(fqcn.as_ref()) {
            functions.push((fqcn.clone(), s.clone()));
        }
    }

    let gvar_names = codebase.file_global_vars_for_file(file);
    let global_vars: Vec<(Arc<str>, Union)> = gvar_names
        .iter()
        .filter_map(|name| {
            codebase
                .global_vars
                .get(name.as_ref())
                .map(|ty| (name.clone(), ty.clone()))
        })
        .collect();

    Pass1Snapshot {
        content_hash,
        namespace,
        imports,
        classes,
        interfaces,
        traits,
        enums,
        functions,
        global_vars,
        parse_errors,
        definition_issues,
    }
}

// ---------------------------------------------------------------------------
// Pass1Status
// ---------------------------------------------------------------------------

/// Whether a file was a Pass 1 cache hit or miss.
pub enum Pass1Status {
    Hit,
    /// Cache miss — carries the SHA-256 content hash for snapshot building.
    Miss(String),
}

// ---------------------------------------------------------------------------
// Pass1Cache
// ---------------------------------------------------------------------------

/// Disk-backed store for per-file Pass 1 snapshots.
///
/// Uses `RwLock<HashMap>` so that many rayon threads can perform cache lookups
/// concurrently, while the single-threaded post-processing step writes new
/// snapshots exclusively after the parallel pass completes.
pub struct Pass1Cache {
    cache_path: PathBuf,
    entries: RwLock<HashMap<String, Pass1Snapshot>>,
    dirty: AtomicBool,
}

impl Pass1Cache {
    /// Open (or create) a cache backed by `{cache_dir}/pass1.bin`.
    pub fn open(cache_dir: &Path) -> Self {
        let cache_path = cache_dir.join("pass1.bin");
        let entries = Self::load_from_disk(&cache_path);
        Self {
            cache_path,
            entries: RwLock::new(entries),
            dirty: AtomicBool::new(false),
        }
    }

    fn load_from_disk(path: &Path) -> HashMap<String, Pass1Snapshot> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => return HashMap::new(),
        };
        let config = bincode::config::standard();
        match bincode::serde::decode_from_slice::<HashMap<String, Pass1Snapshot>, _>(&bytes, config)
        {
            Ok((map, _)) => map,
            // Corrupt or format-changed cache — start fresh.
            Err(_) => HashMap::new(),
        }
    }

    /// Return a snapshot if `file` is cached with a matching `content_hash`.
    pub fn get(&self, file: &str, content_hash: &str) -> Option<Pass1Snapshot> {
        let entries = self.entries.read().unwrap();
        entries.get(file).and_then(|s| {
            if s.content_hash == content_hash {
                Some(s.clone())
            } else {
                None
            }
        })
    }

    /// Store a snapshot, replacing any previous entry for `file`.
    pub fn put(&self, file: &str, snapshot: Pass1Snapshot) {
        self.entries
            .write()
            .unwrap()
            .insert(file.to_string(), snapshot);
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Persist the in-memory cache to `{cache_dir}/pass1.bin`.
    /// No-op when nothing has changed since the last flush.
    pub fn flush(&self) {
        if !self.dirty.load(Ordering::Relaxed) {
            return;
        }
        let config = bincode::config::standard();
        let entries = self.entries.read().unwrap();
        if let Ok(bytes) = bincode::serde::encode_to_vec(&*entries, config) {
            std::fs::write(&self.cache_path, bytes).ok();
            self.dirty.store(false, Ordering::Relaxed);
        }
    }
}
