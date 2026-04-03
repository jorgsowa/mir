/// Per-file analysis result cache backed by a JSON file on disk.
///
/// Cache key: file path.  Cache validity: SHA-256 hash of file content.
/// If the content hash matches what was stored, the cached issues are returned
/// and Pass 2 analysis is skipped for that file.
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

use mir_issues::Issue;

// ---------------------------------------------------------------------------
// Hash helper
// ---------------------------------------------------------------------------

/// Compute the SHA-256 hex digest of `content`.
pub fn hash_content(content: &str) -> String {
    let mut h = Sha256::new();
    h.update(content.as_bytes());
    format!("{:x}", h.finalize())
}

// ---------------------------------------------------------------------------
// CacheEntry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    content_hash: String,
    issues: Vec<Issue>,
}

// ---------------------------------------------------------------------------
// AnalysisCache
// ---------------------------------------------------------------------------

/// Thread-safe, disk-backed cache for per-file analysis results.
pub struct AnalysisCache {
    cache_dir: PathBuf,
    entries: Mutex<HashMap<String, CacheEntry>>,
    dirty: Mutex<bool>,
}

impl AnalysisCache {
    /// Open or create a cache stored under `cache_dir`.
    /// If the directory or cache file do not exist they are created lazily on
    /// the first `flush()` call.
    pub fn open(cache_dir: &Path) -> Self {
        std::fs::create_dir_all(cache_dir).ok();
        let entries = Self::load(cache_dir);
        Self {
            cache_dir: cache_dir.to_path_buf(),
            entries: Mutex::new(entries),
            dirty: Mutex::new(false),
        }
    }

    /// Open the default cache directory: `{project_root}/.mir-cache/`.
    pub fn open_default(project_root: &Path) -> Self {
        Self::open(&project_root.join(".mir-cache"))
    }

    /// Return cached issues for `file_path` if its `content_hash` matches.
    /// Returns `None` if there is no entry or the file has changed.
    pub fn get(&self, file_path: &str, content_hash: &str) -> Option<Vec<Issue>> {
        let entries = self.entries.lock().unwrap();
        entries.get(file_path).and_then(|e| {
            if e.content_hash == content_hash {
                Some(e.issues.clone())
            } else {
                None
            }
        })
    }

    /// Store `issues` for `file_path` with the given `content_hash`.
    pub fn put(&self, file_path: &str, content_hash: String, issues: Vec<Issue>) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(
            file_path.to_string(),
            CacheEntry { content_hash, issues },
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
        let entries = self.entries.lock().unwrap();
        let cache_file = self.cache_dir.join("cache.json");
        if let Ok(json) = serde_json::to_string(&*entries) {
            std::fs::write(cache_file, json).ok();
        }
    }

    // -----------------------------------------------------------------------

    fn load(cache_dir: &Path) -> HashMap<String, CacheEntry> {
        let cache_file = cache_dir.join("cache.json");
        let Ok(bytes) = std::fs::read(&cache_file) else {
            return HashMap::new();
        };
        serde_json::from_slice(&bytes).unwrap_or_default()
    }
}
