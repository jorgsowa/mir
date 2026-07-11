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
use std::sync::OnceLock;

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

/// BLAKE3 hex digest of `source` with the body of every **declared-return**
/// function and method removed.
///
/// This is the cache's *cross-file surface* key: a dependent file's diagnostics
/// are a function of what it can observe of this file, and a declared-return
/// callable exposes only its declared return type (the analyzer prefers the
/// declared type over the body-inferred one). So edits confined to such a body
/// leave this digest — and every dependent's result — unchanged, and don't need
/// to cascade re-analysis.
///
/// Bodies of callables *without* a declared return type are deliberately kept:
/// their inferred return type is body-derived and observable across files
/// (`infer_file_return_types`), so a change there must still cascade. Keeping a
/// body that could have been stripped only ever over-cascades (re-analyzes a
/// dependent needlessly) — never serves a stale result — so this is sound by
/// construction even if a future callable form is missed here.
pub fn surface_fingerprint(source: &str, program: &php_ast::owned::Program) -> String {
    use php_ast::owned::visitor::{
        walk_owned_class_member, walk_owned_program, walk_owned_stmt, OwnedVisitor,
    };
    use php_ast::owned::{ClassMember, ClassMemberKind, Stmt, StmtKind};
    use std::ops::ControlFlow;

    struct BodySpans {
        spans: Vec<(u32, u32)>,
    }
    impl OwnedVisitor for BodySpans {
        fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
            if let StmtKind::Function(f) = &stmt.kind {
                if f.return_type.is_some() {
                    self.spans.push((f.body.span.start, f.body.span.end));
                }
            }
            walk_owned_stmt(self, stmt)
        }
        fn visit_class_member(&mut self, member: &ClassMember) -> ControlFlow<()> {
            if let ClassMemberKind::Method(m) = &member.kind {
                if m.return_type.is_some() {
                    if let Some(body) = &m.body {
                        self.spans.push((body.span.start, body.span.end));
                    }
                }
            }
            walk_owned_class_member(self, member)
        }
    }

    let mut collector = BodySpans { spans: Vec::new() };
    let _ = walk_owned_program(&mut collector, program);
    collector.spans.sort_unstable();

    let bytes = source.as_bytes();
    let mut hasher = blake3::Hasher::new();
    let mut cursor = 0usize;
    for (start, end) in collector.spans {
        let start = (start as usize).min(bytes.len());
        let end = (end as usize).min(bytes.len());
        // Skip malformed or nested (already-covered) spans; never walk backwards.
        if start < cursor || end < start {
            continue;
        }
        hasher.update(&bytes[cursor..start]);
        hasher.update(b"\x00body\x00");
        cursor = end;
    }
    hasher.update(&bytes[cursor..]);
    hasher.finalize().to_hex().to_string()
}

/// Memoized identity of the running mir build. Computed once per process.
///
/// This is the hash of the running executable's own bytes. Any new mir build —
/// a released version bump, a different commit, or a local rebuild that changed
/// analysis logic — produces different bytes and therefore a different
/// fingerprint, so an old build's cache is never reused (the embedded stub set
/// is part of the binary too, so this subsumes stub-set changes). A bare
/// `CARGO_PKG_VERSION` would only catch explicit version bumps, leaving every
/// same-version rebuild serving stale results.
///
/// Falls back to `CARGO_PKG_VERSION` + the bundled stub catalogue if the
/// executable can't be read (unusual sandboxes) — still correct for the inputs
/// that change most often, just blind to pure-logic changes within a version.
fn build_fingerprint() -> u64 {
    static FP: OnceLock<u64> = OnceLock::new();
    *FP.get_or_init(|| {
        let exe_bytes = std::env::current_exe().and_then(std::fs::read).ok();
        compute_build_fingerprint(exe_bytes.as_deref())
    })
}

fn compute_build_fingerprint(exe_bytes: Option<&[u8]>) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
    hasher.update(&[0]);
    match exe_bytes {
        // Primary: the build's own bytes uniquely identify it.
        Some(bytes) => {
            hasher.update(bytes);
        }
        // Fallback: the bundled stub set (paths + contents).
        None => {
            for (path, content) in crate::stubs::stub_files() {
                hasher.update(path.as_bytes());
                hasher.update(&[0]);
                hasher.update(content.as_bytes());
                hasher.update(&[0]);
            }
        }
    }
    let bytes = hasher.finalize();
    u64::from_le_bytes(bytes.as_bytes()[..8].try_into().unwrap())
}

/// Epoch the on-disk cache is validated against. Folds together every input
/// that affects a file's analysis result *besides the file's own content*:
/// the build fingerprint (the running mir binary's identity, which includes
/// the crate version + embedded stub set) and the target PHP version.
///
/// A per-file cache entry's validity is otherwise keyed only on the file's
/// content hash, which silently assumes those global inputs are unchanged
/// between runs. They aren't:
///
/// - Bundled stubs change (e.g. vendoring `redis`/`memcached`, or a
///   phpstorm-stubs bump) → files referencing a now-resolvable built-in keep
///   a stale `UndefinedClass`.
/// - The target PHP version changes (`--php-version`, or an edited
///   `composer.json` constraint) → version-gated symbols (`@since`/`@removed`,
///   LanguageLevelTypeAware params) resolve differently, but the file's bytes
///   are identical so the stale result is served.
///
/// - User-configured stubs change (`user_stub_fp`, computed by
///   [`crate::stubs::user_stub_fingerprint`]) → same as bundled stubs, but for
///   custom stub files/dirs the project supplies.
///
/// Mixing them all into a cache-wide epoch invalidates the whole on-disk cache
/// on the next run when any of them change, instead of serving stale diagnostics.
fn cache_epoch(php_version: u8, user_stub_fp: u64) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&build_fingerprint().to_le_bytes());
    hasher.update(&[php_version]);
    hasher.update(&user_stub_fp.to_le_bytes());
    let bytes = hasher.finalize();
    u64::from_le_bytes(bytes.as_bytes()[..8].try_into().unwrap())
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
    /// Digest of this file's cross-file surface (see [`surface_fingerprint`]).
    /// Dependents are evicted only when this changes between runs — a body-only
    /// edit to a declared-return callable leaves it untouched. Empty for entries
    /// written before this field existed; an empty stored value is treated as
    /// "unknown" and conservatively cascades.
    #[serde(default)]
    surface_hash: String,
}

// ---------------------------------------------------------------------------
// AnalysisCache
// ---------------------------------------------------------------------------

/// Serialized form of the full cache file.
#[derive(Debug, Default, Serialize, Deserialize)]
struct CacheFile {
    /// Analyzer epoch the entries were written under (see [`cache_epoch`]).
    /// `load` discards the whole file when this doesn't match the running
    /// binary's epoch. Defaults to 0 for legacy files written before this
    /// field existed — 0 never matches a real epoch, so they're discarded
    /// (the safe choice). Kept first so a bincode read of a legacy layout
    /// reinterprets its leading bytes here rather than mid-struct.
    #[serde(default)]
    version: u64,
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
    version: u64,
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
    /// Epoch this cache validates against (build fingerprint + PHP version).
    /// Written into the on-disk file and re-checked on the next `load`.
    epoch: u64,
    dirty: AtomicBool,
}

impl AnalysisCache {
    /// Open or create a cache stored under `cache_dir`.
    /// If the directory or cache file do not exist they are created lazily on
    /// the first `flush()` call.
    /// `user_stub_fp` is [`crate::stubs::user_stub_fingerprint`] for the
    /// session's user stubs (0 when none are configured).
    pub fn open(cache_dir: &Path, php_version: u8, user_stub_fp: u64) -> Self {
        std::fs::create_dir_all(cache_dir).ok();
        let epoch = cache_epoch(php_version, user_stub_fp);
        let disk = Self::load(cache_dir, epoch);

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
            epoch,
            dirty: AtomicBool::new(false),
        }
    }

    /// Open the default cache directory: `{project_root}/.mir-cache/`.
    pub fn open_default(project_root: &Path, php_version: u8, user_stub_fp: u64) -> Self {
        Self::open(&project_root.join(".mir-cache"), php_version, user_stub_fp)
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

    /// Return the paths of every file that currently has a cache entry.
    /// Used to detect files that were analyzed in a previous run but have since
    /// been deleted, so their dependents can be invalidated.
    pub fn cached_files(&self) -> Vec<String> {
        let id_map = self.file_id_map.lock();
        let entries = self.entries.lock();
        entries
            .keys()
            .filter_map(|&id| id_map.path(id).map(|p| p.to_string()))
            .collect()
    }

    /// Store `issues` and `reference_locations` for `file_path` with the given
    /// `content_hash` and `surface_hash`. `reference_locations` is a list of
    /// `(symbol_key, line, col_start, col_end)` recorded during body analysis.
    pub fn put(
        &self,
        file_path: &str,
        content_hash: String,
        surface_hash: String,
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
                surface_hash,
            },
        );
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// The stored cross-file surface digest for `file_path`, if an entry exists.
    /// `None` means no entry; an entry written before surface tracking returns
    /// an empty string (treated as "unknown" by callers).
    pub fn surface_hash(&self, file_path: &str) -> Option<String> {
        let id = self.file_id_map.lock().get(file_path)?;
        let entries = self.entries.lock();
        entries.get(&id).map(|e| e.surface_hash.clone())
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
            version: self.epoch,
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

    fn load(cache_dir: &Path, epoch: u64) -> CacheFile {
        // A file whose epoch doesn't match the running analyzer/stub set/PHP
        // version may hold stale results (content hashes still match, but the
        // analysis that produced them changed). Discard it rather than serve
        // stale entries.
        let fresh = |file: CacheFile| {
            if file.version == epoch {
                file
            } else {
                CacheFile::default()
            }
        };
        // Primary: bincode format
        if let Ok(bytes) = std::fs::read(cache_dir.join("cache.bin")) {
            // Bound the read to the file's own size: an incompatible or
            // bit-flipped cache.bin can desync bincode's length-prefixed
            // decoding into reading a bogus multi-gigabyte collection length
            // from garbage bytes, which otherwise tries to allocate that much
            // before deserialize() gets a chance to return an error. `config()`
            // (not the newer `options()`) is required here: it's the fixint
            // encoding this file is written with, while `options()`'s
            // `DefaultOptions` defaults to varint and silently misreads it.
            #[allow(deprecated)]
            let cfg = bincode::config().limit(bytes.len() as u64).clone();
            if let Ok(file) = cfg.deserialize::<CacheFile>(&bytes) {
                return fresh(file);
            }
        }
        // Fallback: legacy JSON format (migrate on next flush)
        if let Ok(bytes) = std::fs::read(cache_dir.join("cache.json")) {
            if let Ok(file) = serde_json::from_slice::<CacheFile>(&bytes) {
                return fresh(file);
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

    /// Fixed PHP version for cache tests — any value works as long as the
    /// open/write/reopen calls within one test agree on it.
    const TEST_PHP_V: u8 = 0;

    fn make_cache(dir: &TempDir) -> AnalysisCache {
        AnalysisCache::open(dir.path(), TEST_PHP_V, 0)
    }

    fn seed(cache: &AnalysisCache, file: &str) {
        cache.put(file, "hash".to_string(), String::new(), vec![], vec![]);
    }

    fn surface(src: &str) -> String {
        let parsed = php_rs_parser::parse(src);
        surface_fingerprint(src, &parsed.program)
    }

    #[test]
    fn surface_stable_across_declared_return_method_body_edit() {
        let a = "<?php\nclass C { public function f(): int { return 1; } }\n";
        let b = "<?php\nclass C { public function f(): int { $x = 2; return $x; } }\n";
        assert_eq!(
            surface(a),
            surface(b),
            "declared-return body edits must not change the surface"
        );
    }

    #[test]
    fn surface_stable_across_declared_return_body_line_shift() {
        // The common edit: adding lines inside a body shifts every later
        // declaration's position. The surface must ignore that.
        let a = "<?php\nfunction f(): int { return 1; }\nfunction g(): int { return 2; }\n";
        let b = "<?php\nfunction f(): int {\n    $a = 0;\n    $b = 1;\n    return $a + $b;\n}\nfunction g(): int { return 2; }\n";
        assert_eq!(surface(a), surface(b));
    }

    #[test]
    fn surface_changes_on_return_type_edit() {
        let a = "<?php\nclass C { public function f(): int { return 1; } }\n";
        let b = "<?php\nclass C { public function f(): string { return 1; } }\n";
        assert_ne!(surface(a), surface(b));
    }

    #[test]
    fn surface_changes_on_param_type_edit() {
        let a = "<?php\nclass C { public function f(int $x): int { return $x; } }\n";
        let b = "<?php\nclass C { public function f(string $x): int { return $x; } }\n";
        assert_ne!(surface(a), surface(b));
    }

    #[test]
    fn surface_changes_on_untyped_method_body_edit() {
        // No declared return type: the body feeds the inferred return type,
        // which is observable across files, so its bytes are kept.
        let a = "<?php\nclass C { public function f() { return 1; } }\n";
        let b = "<?php\nclass C { public function f() { return 'x'; } }\n";
        assert_ne!(surface(a), surface(b));
    }

    #[test]
    fn surface_changes_on_constructor_body_edit() {
        // Constructors have no declared return type — their body is kept.
        let a = "<?php\nclass C { public function __construct() { $this->x = 1; } }\n";
        let b = "<?php\nclass C { public function __construct() { $this->x = 2; } }\n";
        assert_ne!(surface(a), surface(b));
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
    fn cache_entry_without_reference_locations_deserializes_to_empty() {
        // A cache entry that predates the reference_locations field must still
        // be readable as long as its epoch matches (a same-epoch file simply
        // missing the newer field). The #[serde(default)] attribute covers this,
        // but we verify it explicitly so a future refactor can't silently break it.
        let dir = TempDir::new().unwrap();
        let cache_file = dir.path().join("cache.json");

        // Old entry shape (no reference_locations), but with the *current* epoch
        // so the file isn't discarded as stale.
        let json = format!(
            r#"{{"version":{},"entries":{{"a.php":{{"content_hash":"abc","issues":[]}}}},"reverse_deps":{{}}}}"#,
            cache_epoch(TEST_PHP_V, 0)
        );
        std::fs::write(&cache_file, json).unwrap();

        let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
        let hit = cache
            .get("a.php", "abc")
            .expect("same-epoch cache entry should deserialize successfully");

        assert!(hit.0.is_empty(), "no issues");
        assert!(
            hit.1.is_empty(),
            "reference_locations should default to empty vec, not fail"
        );
    }

    #[test]
    fn entries_survive_reopen_with_matching_epoch() {
        // Sanity: a flush + reopen within the same build (same epoch) keeps
        // entries. Guards against the epoch check being over-eager.
        let dir = TempDir::new().unwrap();
        {
            let cache = make_cache(&dir);
            cache.put("a.php", "h1".to_string(), String::new(), vec![], vec![]);
            cache.flush();
        }
        let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
        assert!(
            cache.get("a.php", "h1").is_some(),
            "entry written by the same build/stub set must survive a reopen"
        );
    }

    #[test]
    fn stale_epoch_discards_entire_cache() {
        // A cache.bin written under a different analyzer/stub epoch (e.g. before
        // an extension stub was vendored) must be discarded on load, even though
        // the per-file content hash still matches. This is the regression guard
        // for stale `UndefinedClass` diagnostics surviving a stub-set change.
        let dir = TempDir::new().unwrap();

        let mut entries: HashMap<String, CacheEntry> = HashMap::default();
        entries.insert(
            "a.php".to_string(),
            CacheEntry {
                content_hash: "h1".to_string(),
                issues: vec![],
                reference_locations: vec![],
                surface_hash: String::new(),
            },
        );
        let reverse_deps: HashMap<String, HashSet<String>> = HashMap::default();
        let view = CacheFileView {
            version: cache_epoch(TEST_PHP_V, 0).wrapping_add(1), // deliberately wrong epoch
            entries: &entries,
            reverse_deps: &reverse_deps,
        };
        std::fs::write(
            dir.path().join("cache.bin"),
            bincode::serialize(&view).unwrap(),
        )
        .unwrap();

        let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
        assert!(
            cache.get("a.php", "h1").is_none(),
            "entry from a mismatched epoch must not be served despite a matching content hash"
        );
    }

    #[test]
    fn switching_php_version_discards_cache() {
        // Results written targeting one PHP version must not be served when the
        // next run targets a different one: version-gated symbols resolve
        // differently though the file's content is identical. Regression guard
        // for the `--php-version` arm of the stale-cache family.
        let dir = TempDir::new().unwrap();
        {
            let cache = AnalysisCache::open(dir.path(), 74, 0); // analyzed as PHP 7.4
            cache.put("a.php", "h1".to_string(), String::new(), vec![], vec![]);
            cache.flush();
        }
        let same = AnalysisCache::open(dir.path(), 74, 0);
        assert!(
            same.get("a.php", "h1").is_some(),
            "same PHP version must reuse the cache"
        );
        let other = AnalysisCache::open(dir.path(), 80, 0); // now PHP 8.0
        assert!(
            other.get("a.php", "h1").is_none(),
            "a different PHP version must discard the cache, not serve stale results"
        );
    }

    #[test]
    fn changing_user_stub_fingerprint_discards_cache() {
        // User stubs are resolvable like bundled ones, so editing/adding/removing
        // them changes analysis output for dependent files. The user-stub
        // fingerprint is folded into the epoch; a different one must discard.
        let dir = TempDir::new().unwrap();
        {
            let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0xAAAA);
            cache.put("a.php", "h1".to_string(), String::new(), vec![], vec![]);
            cache.flush();
        }
        let same = AnalysisCache::open(dir.path(), TEST_PHP_V, 0xAAAA);
        assert!(
            same.get("a.php", "h1").is_some(),
            "identical user-stub fingerprint must reuse the cache"
        );
        let changed = AnalysisCache::open(dir.path(), TEST_PHP_V, 0xBBBB);
        assert!(
            changed.get("a.php", "h1").is_none(),
            "a changed user-stub fingerprint must discard the cache"
        );
    }

    #[test]
    fn legacy_versionless_cache_bin_is_discarded_not_paniced() {
        // A cache.bin written by a pre-`version` binary has the old 2-field
        // bincode layout (entries, reverse_deps). Reading it into the new
        // struct must NOT panic or OOM on the misaligned bytes — it must fall
        // through to an empty cache. We reproduce the old layout by serializing
        // a (entries, reverse_deps) tuple, which bincode encodes identically to
        // the old struct.
        let dir = TempDir::new().unwrap();
        let mut entries: HashMap<String, CacheEntry> = HashMap::default();
        entries.insert(
            "a.php".to_string(),
            CacheEntry {
                content_hash: "h1".to_string(),
                issues: vec![],
                reference_locations: vec![],
                surface_hash: String::new(),
            },
        );
        let reverse_deps: HashMap<String, HashSet<String>> = HashMap::default();
        let legacy_bytes = bincode::serialize(&(&entries, &reverse_deps)).unwrap();
        std::fs::write(dir.path().join("cache.bin"), legacy_bytes).unwrap();

        // Must not panic.
        let cache = AnalysisCache::open(dir.path(), TEST_PHP_V, 0);
        assert!(
            cache.get("a.php", "h1").is_none(),
            "legacy versionless entries must be discarded, not served"
        );
    }

    #[test]
    fn build_fingerprint_tracks_binary_identity() {
        // A different mir build (different executable bytes) must yield a
        // different fingerprint, so its cache epoch differs and an old build's
        // cache is never reused. Two distinct byte images stand in for two
        // builds; identical bytes must reproduce the same fingerprint.
        let build_a = compute_build_fingerprint(Some(b"mir-binary-image-A"));
        let build_a_again = compute_build_fingerprint(Some(b"mir-binary-image-A"));
        let build_b = compute_build_fingerprint(Some(b"mir-binary-image-B"));
        assert_eq!(
            build_a, build_a_again,
            "same binary bytes → same fingerprint"
        );
        assert_ne!(
            build_a, build_b,
            "a new mir build (different bytes) must change the fingerprint"
        );
        // The fallback path (executable unreadable) is also deterministic and
        // distinct from a real binary hash.
        assert_eq!(
            compute_build_fingerprint(None),
            compute_build_fingerprint(None)
        );
    }
}
