/// Per-file analysis result cache backed by a JSON file on disk.
///
/// Cache key: file path.  Cache validity: BLAKE3 hash of file content.
/// If the content hash matches what was stored, the cached issues are returned
/// and Pass 2 analysis is skipped for that file.
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

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
    /// Reference locations recorded during Pass 2: (symbol_key, line, col_start, col_end).
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
    /// Persisted so that the next run can invalidate dependents before Pass 1.
    #[serde(default)]
    reverse_deps: HashMap<String, HashSet<String>>,
}

/// Thread-safe, disk-backed cache for per-file analysis results.
pub struct AnalysisCache {
    cache_dir: PathBuf,
    entries: Mutex<HashMap<String, CacheEntry>>,
    /// Reverse dependency graph loaded from disk (from the previous run).
    reverse_deps: Mutex<HashMap<String, HashSet<String>>>,
    dirty: Mutex<bool>,
}

impl AnalysisCache {
    /// Open or create a cache stored under `cache_dir`.
    /// If the directory or cache file do not exist they are created lazily on
    /// the first `flush()` call.
    pub fn open(cache_dir: &Path) -> Self {
        std::fs::create_dir_all(cache_dir).ok();
        let file = Self::load(cache_dir);
        Self {
            cache_dir: cache_dir.to_path_buf(),
            entries: Mutex::new(file.entries),
            reverse_deps: Mutex::new(file.reverse_deps),
            dirty: Mutex::new(false),
        }
    }

    /// Open the default cache directory: `{project_root}/.mir-cache/`.
    pub fn open_default(project_root: &Path) -> Self {
        Self::open(&project_root.join(".mir-cache"))
    }

    /// Return cached issues and reference locations for `file_path` if its
    /// `content_hash` matches. Returns `None` if there is no entry or the file
    /// has changed. The second element of the tuple is the list of
    /// `(symbol_key, line, col_start, col_end)` entries to replay into
    /// `Codebase::symbol_reference_locations`.
    pub fn get(&self, file_path: &str, content_hash: &str) -> Option<CacheHit> {
        let entries = self.entries.lock().unwrap();
        entries.get(file_path).and_then(|e| {
            if e.content_hash == content_hash {
                Some((e.issues.clone(), e.reference_locations.clone()))
            } else {
                None
            }
        })
    }

    /// Store `issues` and `reference_locations` for `file_path` with the given
    /// `content_hash`. `reference_locations` is a list of
    /// `(symbol_key, line, col_start, col_end)` recorded during Pass 2.
    pub fn put(
        &self,
        file_path: &str,
        content_hash: String,
        issues: Vec<Issue>,
        reference_locations: Vec<(String, u32, u16, u16)>,
    ) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(
            file_path.to_string(),
            CacheEntry {
                content_hash,
                issues,
                reference_locations,
            },
        );
        *self.dirty.lock().unwrap() = true;
    }

    /// Persist the in-memory cache to `{cache_dir}/cache.json`.
    /// This is a no-op if nothing changed since the last flush.
    pub fn flush(&self) {
        let dirty = {
            let mut d = self.dirty.lock().unwrap();
            let was = *d;
            *d = false;
            was
        };
        if !dirty {
            return;
        }
        let cache_file = self.cache_dir.join("cache.json");
        let file = CacheFile {
            entries: self.entries.lock().unwrap().clone(),
            reverse_deps: self.reverse_deps.lock().unwrap().clone(),
        };
        if let Ok(json) = serde_json::to_string(&file) {
            std::fs::write(cache_file, json).ok();
        }
    }

    /// Replace the reverse dependency graph (called after each Pass 1).
    pub fn set_reverse_deps(&self, deps: HashMap<String, HashSet<String>>) {
        *self.reverse_deps.lock().unwrap() = deps;
        *self.dirty.lock().unwrap() = true;
    }

    /// BFS from each changed file through the reverse dep graph.
    /// Evicts every reachable dependent's cache entry.
    /// Returns the number of entries evicted.
    pub fn evict_with_dependents(&self, changed_files: &[String]) -> usize {
        // Phase 1: collect all dependents to evict via BFS (lock held only here).
        let to_evict: Vec<String> = {
            let deps = self.reverse_deps.lock().unwrap();
            let mut visited: HashSet<String> = changed_files.iter().cloned().collect();
            let mut queue: std::collections::VecDeque<String> =
                changed_files.iter().cloned().collect();
            let mut result = Vec::new();

            while let Some(file) = queue.pop_front() {
                if let Some(dependents) = deps.get(&file) {
                    for dep in dependents {
                        if visited.insert(dep.clone()) {
                            queue.push_back(dep.clone());
                            result.push(dep.clone());
                        }
                    }
                }
            }
            result
        };

        // Phase 2: evict (reverse_deps lock released above, entries lock taken per file).
        let count = to_evict.len();
        for file in &to_evict {
            self.evict(file);
        }
        count
    }

    /// Remove a single file's cache entry.
    pub fn evict(&self, file_path: &str) {
        let mut entries = self.entries.lock().unwrap();
        entries.remove(file_path);
        *self.dirty.lock().unwrap() = true;
    }

    // -----------------------------------------------------------------------

    fn load(cache_dir: &Path) -> CacheFile {
        let cache_file = cache_dir.join("cache.json");
        let Ok(bytes) = std::fs::read(&cache_file) else {
            return CacheFile::default();
        };
        serde_json::from_slice(&bytes).unwrap_or_default()
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

        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
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

        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
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

        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
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

        let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
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
