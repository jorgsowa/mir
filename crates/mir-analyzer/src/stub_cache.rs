/// Disk-backed cache for parsed stub definitions.
///
/// Stub loading (phpstorm-stubs + user stubs) costs 20–50 ms per run. For
/// repeated invocations — watch mode, LSP server, CI — this overhead is
/// unnecessary when nothing has changed. This module serializes the stub
/// portion of a `Codebase` to disk after the first load, and restores it on
/// subsequent runs when the cache key matches.
///
/// Cache key captures everything that affects stub output:
/// - mir crate version (changes when phpstorm-stubs are updated in a release)
/// - Target PHP version
/// - Enabled extension set
/// - Content hashes of user-provided stub files and directories
///
/// Cache file: `{cache_dir}/stub-cache.json`.
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use mir_codebase::storage::{
    ClassStorage, EnumStorage, FunctionStorage, InterfaceStorage, TraitStorage,
};
use mir_codebase::Codebase;
use mir_types::Union;

// ---------------------------------------------------------------------------
// Snapshot
// ---------------------------------------------------------------------------

/// Serializable copy of the stub-only entries in a `Codebase`.
/// Captured right after `load_stubs_configured()`, before any user-code Pass 1.
/// `all_parents` fields are intentionally empty here — `Codebase::finalize()`
/// rebuilds them after user code is added.
#[derive(Serialize, Deserialize)]
pub struct StubSnapshot {
    classes: HashMap<String, ClassStorage>,
    interfaces: HashMap<String, InterfaceStorage>,
    traits: HashMap<String, TraitStorage>,
    enums: HashMap<String, EnumStorage>,
    functions: HashMap<String, FunctionStorage>,
    constants: HashMap<String, Union>,
    known_symbols: Vec<String>,
    symbol_to_file: HashMap<String, String>,
}

/// Capture every entry currently in `codebase` into a serializable snapshot.
pub fn capture(codebase: &Codebase) -> StubSnapshot {
    StubSnapshot {
        classes: codebase
            .classes
            .iter()
            .map(|e| (e.key().to_string(), e.value().clone()))
            .collect(),
        interfaces: codebase
            .interfaces
            .iter()
            .map(|e| (e.key().to_string(), e.value().clone()))
            .collect(),
        traits: codebase
            .traits
            .iter()
            .map(|e| (e.key().to_string(), e.value().clone()))
            .collect(),
        enums: codebase
            .enums
            .iter()
            .map(|e| (e.key().to_string(), e.value().clone()))
            .collect(),
        functions: codebase
            .functions
            .iter()
            .map(|e| (e.key().to_string(), e.value().clone()))
            .collect(),
        constants: codebase
            .constants
            .iter()
            .map(|e| (e.key().to_string(), e.value().clone()))
            .collect(),
        known_symbols: codebase
            .known_symbols
            .iter()
            .map(|e| e.key().to_string())
            .collect(),
        symbol_to_file: codebase
            .symbol_to_file
            .iter()
            .map(|e| (e.key().to_string(), e.value().to_string()))
            .collect(),
    }
}

/// Inject a snapshot into an empty `Codebase` (before any user-code Pass 1).
pub fn apply(codebase: &Codebase, snap: StubSnapshot) {
    for (k, v) in snap.classes {
        codebase.classes.insert(Arc::from(k.as_str()), v);
    }
    for (k, v) in snap.interfaces {
        codebase.interfaces.insert(Arc::from(k.as_str()), v);
    }
    for (k, v) in snap.traits {
        codebase.traits.insert(Arc::from(k.as_str()), v);
    }
    for (k, v) in snap.enums {
        codebase.enums.insert(Arc::from(k.as_str()), v);
    }
    for (k, v) in snap.functions {
        codebase.functions.insert(Arc::from(k.as_str()), v);
    }
    for (k, v) in snap.constants {
        codebase.constants.insert(Arc::from(k.as_str()), v);
    }
    for sym in snap.known_symbols {
        codebase.known_symbols.insert(Arc::from(sym.as_str()));
    }
    for (k, v) in snap.symbol_to_file {
        codebase
            .symbol_to_file
            .insert(Arc::from(k.as_str()), Arc::from(v.as_str()));
    }
}

// ---------------------------------------------------------------------------
// Cache key
// ---------------------------------------------------------------------------

/// Compute a deterministic cache key that covers all inputs to stub loading.
pub fn cache_key(
    php_version: Option<(u8, u8)>,
    enabled_extensions: Option<&HashSet<String>>,
    stub_files: &[PathBuf],
    stub_dirs: &[PathBuf],
) -> String {
    let mut h = Sha256::new();

    // Crate version — changes whenever phpstorm-stubs are updated in a release.
    h.update(env!("CARGO_PKG_VERSION").as_bytes());
    h.update(b"|");

    // PHP version gate.
    match php_version {
        Some((maj, min)) => h.update(format!("php={}.{}", maj, min).as_bytes()),
        None => h.update(b"php=any"),
    }
    h.update(b"|");

    // Enabled extension set (sorted for determinism).
    match enabled_extensions {
        None => h.update(b"ext=all"),
        Some(set) => {
            let mut exts: Vec<&str> = set.iter().map(String::as_str).collect();
            exts.sort_unstable();
            h.update(b"ext=");
            h.update(exts.join(",").as_bytes());
        }
    }
    h.update(b"|");

    // User stub files sorted by path for determinism.
    let mut direct_files: Vec<PathBuf> = stub_files.to_vec();
    for dir in stub_dirs {
        collect_php_files_sorted(dir, &mut direct_files);
    }
    direct_files.sort_unstable();
    direct_files.dedup();

    for path in &direct_files {
        if let Ok(content) = std::fs::read_to_string(path) {
            h.update(path.to_string_lossy().as_bytes());
            h.update(b":");
            h.update(content.as_bytes());
            h.update(b"|");
        }
    }

    h.finalize().iter().fold(String::new(), |mut acc, b| {
        use std::fmt::Write;
        write!(acc, "{:02x}", b).unwrap();
        acc
    })
}

fn collect_php_files_sorted(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort_unstable();
    for path in paths {
        if path.is_dir() {
            collect_php_files_sorted(&path, out);
        } else if path.extension().is_some_and(|e| e == "php") {
            out.push(path);
        }
    }
}

// ---------------------------------------------------------------------------
// Disk I/O
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct CacheFile {
    key: String,
    snapshot: StubSnapshot,
}

fn cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("stub-cache.json")
}

/// Try to load a valid snapshot from disk.
/// Returns `None` if the cache file is absent, corrupt, or the key has changed.
pub fn load(cache_dir: &Path, key: &str) -> Option<StubSnapshot> {
    let data = std::fs::read_to_string(cache_path(cache_dir)).ok()?;
    let file: CacheFile = serde_json::from_str(&data).ok()?;
    if file.key == key {
        Some(file.snapshot)
    } else {
        None
    }
}

/// Persist a stub snapshot to disk. Silently ignores write errors.
pub fn save(cache_dir: &Path, key: &str, snapshot: StubSnapshot) {
    let file = CacheFile {
        key: key.to_string(),
        snapshot,
    };
    if let Ok(json) = serde_json::to_string(&file) {
        let _ = std::fs::write(cache_path(cache_dir), json);
    }
}
