/// Per-file analysis result cache backed by a binary file on disk.
///
/// Cache key: file path.  Cache validity: BLAKE3 hash of file content.
/// If the content hash matches what was stored, the cached issues are returned
/// and body analysis analysis is skipped for that file.
///
/// Internally, path strings are mapped to compact [`FileId`] integers so the
/// hot-path lookups (`get` / `put`) hash a `u32` instead of a full path string.
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use mir_codebase::{FileId, FileIdMap};
use parking_lot::Mutex;

use serde::{Deserialize, Serialize};

use mir_issues::Issue;

/// Cached analysis result returned on a cache hit: issues and reference location
/// tuples `(symbol_key, line, col_start, col_end)`.
pub type CacheHit = (Vec<Issue>, Vec<(String, u32, u16, u16)>);

// ---------------------------------------------------------------------------
// Hash helper
// ---------------------------------------------------------------------------

/// Compute the BLAKE3 hex digest of `content`.
pub fn hash_content(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex().to_string()
}

// ---------------------------------------------------------------------------
// CacheEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    content_hash: String,
    issues: Vec<Issue>,
    /// Reference locations recorded during body analysis: (symbol_key, line, col_start, col_end).
    /// Stored so that cache hits can replay symbol_reference_locations without re-running
    /// analyze_bodies.
    #[serde(default)]
    reference_locations: Vec<(String, u32, u16, u16)>,
}

// ---------------------------------------------------------------------------
// AnalysisCache
// ---------------------------------------------------------------------------

/// Serialized form of the full cache file.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheFile {
    #[serde(default)]
    entries: HashMap<String, CacheEntry>,
    /// Reverse dependency graph: defining_file → [files that depend on it].
    /// Persisted so that the next run can invalidate dependents before definition collection.
    #[serde(default)]
    reverse_deps: HashMap<String, HashSet<String>>,
}

/// View for serializing cache data without cloning.
#[derive(Serialize)]
struct CacheFileView<'a> {
    entries: &'a HashMap<String, CacheEntry>,
    reverse_deps: &'a HashMap<String, HashSet<String>>,
}

/// Thread-safe, disk-backed cache for per-file analysis results.
pub struct AnalysisCache {
    cache_dir: PathBuf,
    /// Path ↔ FileId mapping; owns the canonical string storage so entry/dep
    /// maps can use 4-byte keys instead of heap-allocated path strings.
    file_id_map: Mutex<FileIdMap>,
    entries: Mutex<HashMap<FileId, CacheEntry>>,
    /// Reverse dependency graph loaded from disk (from the previous run).
    reverse_deps: Mutex<HashMap<FileId, HashSet<FileId>>>,
    dirty: AtomicBool,
}

impl AnalysisCache {
    /// Open or create a cache stored under `cache_dir`.
    /// If the directory or cache file do not exist they are created lazily on
    /// the first `flush()` call.
    pub fn open(cache_dir: &Path) -> Self {
        std::fs::create_dir_all(cache_dir).ok();
        let disk = Self::load(cache_dir);

        // Build a FileIdMap from the on-disk path strings, then convert both
        // maps to use FileId keys for O(1) u32-hash lookups at runtime.
        let mut id_map = FileIdMap::new();
        let entries: HashMap<FileId, CacheEntry> = disk
            .entries
            .into_iter()
            .map(|(path, entry)| (id_map.assign_or_get(&path), entry))
            .collect();
        let reverse_deps: HashMap<FileId, HashSet<FileId>> = disk
            .reverse_deps
            .into_iter()
            .map(|(path, dep_paths)| {
                let id = id_map.assign_or_get(&path);
                let dep_ids = dep_paths.iter().map(|p| id_map.assign_or_get(p)).collect();
                (id, dep_ids)
            })
            .collect();

        Self {
            cache_dir: cache_dir.to_path_buf(),
            file_id_map: Mutex::new(id_map),
            entries: Mutex::new(entries),
            reverse_deps: Mutex::new(reverse_deps),
            dirty: AtomicBool::new(false),
        }
    }

    /// Open the default cache directory: `{project_root}/.mir-cache/`.
    pub fn open_default(project_root: &Path) -> Self {
        Self::open(&project_root.join(".mir-cache"))
    }

    /// Directory the cache was opened from. Useful for callers that want to
    /// open a sibling cache (e.g. the definition-collection `StubSliceCache`) under the
    /// same root.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Return cached issues and reference locations for `file_path` if its
    /// `content_hash` matches. Returns `None` if there is no entry or the file
    /// has changed. The second element of the tuple is the list of
    /// `(symbol_key, line, col_start, col_end)` entries to replay into
    /// the salsa db via `MirDatabase::replay_reference_locations`.
    pub fn get(&self, file_path: &str, content_hash: &str) -> Option<CacheHit> {
        let id = self.file_id_map.lock().get(file_path)?;
        let entries = self.entries.lock();
        entries.get(&id).and_then(|e| {
            if e.content_hash == content_hash {
                Some((e.issues.clone(), e.reference_locations.clone()))
            } else {
                None
            }
        })
    }

    /// Store `issues` and `reference_locations` for `file_path` with the given
    /// `content_hash`. `reference_locations` is a list of
    /// `(symbol_key, line, col_start, col_end)` recorded during body analysis.
    pub fn put(
        &self,
        file_path: &str,
        content_hash: String,
        issues: Vec<Issue>,
        reference_locations: Vec<(String, u32, u16, u16)>,
    ) {
        let id = self.file_id_map.lock().assign_or_get(file_path);
        let mut entries = self.entries.lock();
        entries.insert(
            id,
            CacheEntry {
                content_hash,
                issues,
                reference_locations,
            },
        );
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Persist the in-memory cache to `{cache_dir}/cache.bin`.
    /// This is a no-op if nothing changed since the last flush.
    pub fn flush(&self) {
        let was_dirty = self.dirty.swap(false, Ordering::Relaxed);
        if !was_dirty {
            return;
        }
        let cache_file = self.cache_dir.join("cache.bin");
        let id_map = self.file_id_map.lock();
        let entries_guard = self.entries.lock();
        let deps_guard = self.reverse_deps.lock();

        // Resolve FileIds back to path strings for the on-disk format.
        let entries: HashMap<String, CacheEntry> = entries_guard
            .iter()
            .filter_map(|(&id, entry)| id_map.path(id).map(|p| (p.to_string(), entry.clone())))
            .collect();
        let reverse_deps: HashMap<String, HashSet<String>> = deps_guard
            .iter()
            .filter_map(|(&id, dep_ids)| {
                let path = id_map.path(id)?;
                let dep_paths: HashSet<String> = dep_ids
                    .iter()
                    .filter_map(|&dep_id| id_map.path(dep_id))
                    .map(|s| s.to_string())
                    .collect();
                Some((path.to_string(), dep_paths))
            })
            .collect();

        let view = CacheFileView {
            entries: &entries,
            reverse_deps: &reverse_deps,
        };
        if let Ok(bytes) = bincode::serialize(&view) {
            std::fs::write(cache_file, bytes).ok();
        }
    }

    /// Replace the reverse dependency graph (called after each definition collection).
    pub fn set_reverse_deps(&self, deps: HashMap<String, HashSet<String>>) {
        let mut id_map = self.file_id_map.lock();
        let converted: HashMap<FileId, HashSet<FileId>> = deps
            .into_iter()
            .map(|(path, dep_paths)| {
                let id = id_map.assign_or_get(&path);
                let dep_ids = dep_paths.iter().map(|p| id_map.assign_or_get(p)).collect();
                (id, dep_ids)
            })
            .collect();
        drop(id_map);
        *self.reverse_deps.lock() = converted;
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Update the reverse-dep graph for a single file in place.
    ///
    /// `new_targets` is the set of files `file` depends on (its imports'
    /// defining files plus parent / interfaces / traits' defining files).
    /// This removes `file` from every existing dependent set, then inserts it
    /// into each of `new_targets`' dependent sets — preserving the invariant
    /// that the graph reflects the file's *current* outgoing edges.
    ///
    /// Used by `AnalysisSession::ingest_file` to keep cross-file invalidation
    /// correct without rebuilding the whole graph on every edit.
    pub fn update_reverse_deps_for_file(&self, file: &str, new_targets: &HashSet<String>) {
        let file_id = self.file_id_map.lock().assign_or_get(file);
        let target_ids: Vec<FileId> = {
            let mut id_map = self.file_id_map.lock();
            new_targets
                .iter()
                .map(|t| id_map.assign_or_get(t))
                .collect()
        };

        let mut deps = self.reverse_deps.lock();
        for dependents in deps.values_mut() {
            dependents.remove(&file_id);
        }
        deps.retain(|_, dependents| !dependents.is_empty());
        for target_id in target_ids {
            if target_id != file_id {
                deps.entry(target_id).or_default().insert(file_id);
            }
        }

        self.dirty.store(true, Ordering::Relaxed);
    }

    /// BFS from each changed file through the reverse dep graph.
    /// Evicts every reachable dependent's cache entry.
    /// Returns the number of entries evicted.
    pub fn evict_with_dependents(&self, changed_files: &[String]) -> usize {
        // Resolve paths to FileIds; skip unknown files (no cache entry → nothing to evict).
        let seed_ids: Vec<FileId> = {
            let id_map = self.file_id_map.lock();
            changed_files.iter().filter_map(|p| id_map.get(p)).collect()
        };
        if seed_ids.is_empty() {
            return 0;
        }

        // Phase 1: collect all dependents to evict via BFS (lock held only here).
        let to_evict: Vec<FileId> = {
            let deps = self.reverse_deps.lock();
            let mut visited: HashSet<FileId> = seed_ids.iter().copied().collect();
            let mut queue: std::collections::VecDeque<FileId> = seed_ids.iter().copied().collect();
            let mut result = Vec::new();

            while let Some(id) = queue.pop_front() {
                if let Some(dependents) = deps.get(&id) {
                    for &dep_id in dependents {
                        if visited.insert(dep_id) {
                            queue.push_back(dep_id);
                            result.push(dep_id);
                        }
                    }
                }
            }
            result
        };

        // Phase 2: evict (reverse_deps lock released above, entries lock taken per file).
        let count = to_evict.len();
        let mut entries = self.entries.lock();
        for id in &to_evict {
            entries.remove(id);
        }
        if count > 0 {
            self.dirty.store(true, Ordering::Relaxed);
        }
        count
    }

    /// Remove a single file's cache entry.
    pub fn evict(&self, file_path: &str) {
        let Some(id) = self.file_id_map.lock().get(file_path) else {
            return;
        };
        let mut entries = self.entries.lock();
        if entries.remove(&id).is_some() {
            self.dirty.store(true, Ordering::Relaxed);
        }
    }

    // -----------------------------------------------------------------------

    fn load(cache_dir: &Path) -> CacheFile {
        // Primary: bincode format
        if let Ok(bytes) = std::fs::read(cache_dir.join("cache.bin")) {
            if let Ok(file) = bincode::deserialize::<CacheFile>(&bytes) {
                return file;
            }
        }
        // Fallback: legacy JSON format (migrate on next flush)
        if let Ok(bytes) = std::fs::read(cache_dir.join("cache.json")) {
            if let Ok(file) = serde_json::from_slice::<CacheFile>(&bytes) {
                return file;
            }
        }
        CacheFile::default()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_cache(dir: &TempDir) -> AnalysisCache {
        AnalysisCache::open(dir.path())
    }

    fn seed(cache: &AnalysisCache, file: &str) {
        cache.put(file, "hash".to_string(), vec![], vec![]);
    }

    #[test]
    fn evict_with_dependents_linear_chain() {
        // reverse_deps: A → [B], B → [C]
        // Changing A must evict B and C.
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        seed(&cache, "A");
        seed(&cache, "B");
        seed(&cache, "C");

        let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
        deps.entry("A".into()).or_default().insert("B".into());
        deps.entry("B".into()).or_default().insert("C".into());
        cache.set_reverse_deps(deps);

        let evicted = cache.evict_with_dependents(&["A".to_string()]);

        assert_eq!(evicted, 2, "B and C should be evicted");
        assert!(cache.get("A", "hash").is_some(), "A itself is not evicted");
        assert!(cache.get("B", "hash").is_none(), "B should be evicted");
        assert!(cache.get("C", "hash").is_none(), "C should be evicted");
    }

    #[test]
    fn evict_with_dependents_diamond() {
        // reverse_deps: A → [B, C], B → [D], C → [D]
        // D should be evicted exactly once (visited set prevents double-eviction).
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        seed(&cache, "A");
        seed(&cache, "B");
        seed(&cache, "C");
        seed(&cache, "D");

        let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
        deps.entry("A".into()).or_default().insert("B".into());
        deps.entry("A".into()).or_default().insert("C".into());
        deps.entry("B".into()).or_default().insert("D".into());
        deps.entry("C".into()).or_default().insert("D".into());
        cache.set_reverse_deps(deps);

        let evicted = cache.evict_with_dependents(&["A".to_string()]);

        assert_eq!(evicted, 3, "B, C, D each evicted once");
        assert!(cache.get("D", "hash").is_none());
    }

    #[test]
    fn evict_with_dependents_cycle_safety() {
        // reverse_deps: A → [B], B → [A]  (circular)
        // Must not loop forever; B should be evicted.
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        seed(&cache, "A");
        seed(&cache, "B");

        let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
        deps.entry("A".into()).or_default().insert("B".into());
        deps.entry("B".into()).or_default().insert("A".into());
        cache.set_reverse_deps(deps);

        let evicted = cache.evict_with_dependents(&["A".to_string()]);

        // B is a dependent of A; A is the seed (not counted as "evicted dependent")
        assert_eq!(evicted, 1);
        assert!(cache.get("B", "hash").is_none());
    }

    #[test]
    fn evict_with_dependents_unrelated_file_untouched() {
        // Changing C should not evict B (which depends on A, not C).
        let dir = TempDir::new().unwrap();
        let cache = make_cache(&dir);
        seed(&cache, "A");
        seed(&cache, "B");
        seed(&cache, "C");

        let mut deps: HashMap<String, HashSet<String>> = HashMap::default();
        deps.entry("A".into()).or_default().insert("B".into());
        cache.set_reverse_deps(deps);

        let evicted = cache.evict_with_dependents(&["C".to_string()]);

        assert_eq!(evicted, 0);
        assert!(
            cache.get("B", "hash").is_some(),
            "B unrelated, should survive"
        );
    }

    #[test]
    fn old_cache_without_reference_locations_deserializes_to_empty() {
        // Cache entries written before the reference_locations field was added
        // must still be readable. The #[serde(default)] attribute covers this,
        // but we verify it explicitly so a future refactor can't silently break it.
        let dir = TempDir::new().unwrap();
        let cache_file = dir.path().join("cache.json");

        // Write a cache file in the old format (no reference_locations field).
        std::fs::write(
            &cache_file,
            r#"{"entries":{"a.php":{"content_hash":"abc","issues":[]}},"reverse_deps":{}}"#,
        )
        .unwrap();

        let cache = AnalysisCache::open(dir.path());
        let hit = cache
            .get("a.php", "abc")
            .expect("old cache entry should deserialize successfully");

        assert!(hit.0.is_empty(), "no issues");
        assert!(
            hit.1.is_empty(),
            "reference_locations should default to empty vec, not fail"
        );
    }
}
