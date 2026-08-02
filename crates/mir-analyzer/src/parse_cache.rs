//! Bounded in-process parse cache.
//!
//! Maps a source content hash → the parsed [`StubSlice`], so a file parsed once
//! in a session is not re-parsed on a later cold demand of
//! `collect_file_definitions`. Shared across db clones (an `Arc`), so parallel
//! indexing workers populate one cache.
//!
//! **Bounded.** In the eager-static-input model there is no per-file vendor
//! eviction to prune this cache, so it is capped at [`DEFAULT_CAPACITY`] entries
//! with FIFO eviction — the aggregate memory ceiling for parsed slices,
//! complementing the `lru = 4096` on `collect_file_definitions`. The cache is
//! content-addressed, so an evicted entry is recomputed cheaply from the on-disk
//! stub cache (no re-parse on a disk hit) the next time it is demanded.

use std::collections::VecDeque;
use std::sync::Arc;

use dashmap::DashMap;
use mir_codebase::StubSlice;
use mir_issues::Issue;
use parking_lot::Mutex;

/// Default entry cap. Set a touch above the `collect_file_definitions` LRU
/// (4096) so the parse cache never evicts a slice the memo still wants, while
/// still bounding total resident parsed slices.
pub const DEFAULT_CAPACITY: usize = 6144;

/// `(content_hash, php_version_cache_byte)` — a version-specific stub
/// collection (`@since`/`@removed`-filtered, `#[LanguageLevelTypeAware]`
/// resolved) is not interchangeable with another version's for the same
/// bytes, so the PHP version must be part of the key, same as the on-disk
/// [`crate::stub_cache::StubSliceCache`] one entry down.
type ParseCacheKey = ([u8; 32], u8);

/// A cached parse result: the definitions slice plus the issues found while
/// collecting it (parse errors, `BackedEnumCaseTypeMismatch`, docblock
/// warnings, etc.).
///
/// Both fields are content-hash-derived and path-agnostic in origin — the
/// `file` field on the slice, and each issue's `location.file`, get patched
/// to the caller's actual path by whoever consumes a cache entry (see
/// [`ParseCache::get`]'s callers), the same way [`crate::stub_cache::StubSliceCache`]
/// patches `StubSlice::file` on its own hits.
#[derive(Clone)]
pub struct CachedParse {
    pub slice: Arc<StubSlice>,
    pub issues: Arc<Vec<Issue>>,
}

/// Re-point every issue's `location.file` at `path` — needed when a cache hit
/// reuses another file's identically-hashed parse result. A no-op clone when
/// every issue already points at `path` (the common case: re-collecting the
/// *same* file after its salsa memo was invalidated, not a cross-file reuse).
pub fn patch_issue_locations(issues: &[Issue], path: &Arc<str>) -> Vec<Issue> {
    issues
        .iter()
        .map(|issue| {
            if issue.location.file == *path {
                issue.clone()
            } else {
                let mut patched = issue.clone();
                patched.location.file = path.clone();
                patched
            }
        })
        .collect()
}

/// Content-hash-keyed, capacity-bounded cache of parsed [`StubSlice`]s (and
/// the issues found alongside them).
pub struct ParseCache {
    map: DashMap<ParseCacheKey, CachedParse>,
    /// Insertion order of keys, for FIFO eviction. Holds keys that may already
    /// have been removed; eviction tolerates stale entries.
    order: Mutex<VecDeque<ParseCacheKey>>,
    capacity: usize,
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }
}

impl ParseCache {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: DashMap::new(),
            order: Mutex::new(VecDeque::new()),
            capacity: capacity.max(1),
        }
    }

    /// Look up a parsed slice (and its issues) by content hash and target PHP
    /// version (`PhpVersion::cache_byte()`).
    pub fn get(&self, hash: &[u8; 32], php_v: u8) -> Option<CachedParse> {
        self.map.get(&(*hash, php_v)).map(|r| r.clone())
    }

    /// Insert a parsed slice and its issues. On a genuinely new key, evicts
    /// oldest entries (FIFO) until the cache is within capacity.
    pub fn insert(&self, hash: [u8; 32], php_v: u8, slice: Arc<StubSlice>, issues: Arc<Vec<Issue>>) {
        let key = (hash, php_v);
        let is_new = self
            .map
            .insert(key, CachedParse { slice, issues })
            .is_none();
        if !is_new {
            return;
        }
        let mut order = self.order.lock();
        order.push_back(key);
        while self.map.len() > self.capacity {
            match order.pop_front() {
                Some(old) => {
                    self.map.remove(&old);
                }
                None => break,
            }
        }
    }

    /// Remove an entry (used when a file's content is known to have changed).
    pub fn remove(&self, hash: &[u8; 32], php_v: u8) {
        self.map.remove(&(*hash, php_v));
    }

    /// Current number of cached slices.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_issues() -> Arc<Vec<Issue>> {
        Arc::new(Vec::new())
    }

    #[test]
    fn get_is_isolated_per_php_version() {
        let cache = ParseCache::with_capacity(8);
        let hash = [1u8; 32];
        let slice_80 = Arc::new(StubSlice::default());
        cache.insert(hash, 80, slice_80, empty_issues());

        assert!(
            cache.get(&hash, 80).is_some(),
            "same content hash + same PHP version must hit"
        );
        assert!(
            cache.get(&hash, 81).is_none(),
            "same content hash but a DIFFERENT PHP version must miss — a \
             version-specific collected StubSlice is not interchangeable"
        );
    }

    #[test]
    fn two_versions_of_the_same_content_coexist() {
        let cache = ParseCache::with_capacity(8);
        let hash = [2u8; 32];
        cache.insert(hash, 80, Arc::new(StubSlice::default()), empty_issues());
        cache.insert(hash, 81, Arc::new(StubSlice::default()), empty_issues());

        assert!(cache.get(&hash, 80).is_some());
        assert!(cache.get(&hash, 81).is_some());
        assert_eq!(
            cache.len(),
            2,
            "both version-specific entries must be retained"
        );
    }

    #[test]
    fn get_returns_the_issues_stored_alongside_the_slice() {
        let cache = ParseCache::with_capacity(8);
        let hash = [3u8; 32];
        let issue = Issue::new(
            mir_issues::IssueKind::UndefinedVariable {
                name: "x".to_string(),
            },
            mir_types::Location {
                file: Arc::from("a.php"),
                line: 1,
                line_end: 1,
                col_start: 0,
                col_end: 1,
            },
        );
        cache.insert(
            hash,
            80,
            Arc::new(StubSlice::default()),
            Arc::new(vec![issue.clone()]),
        );

        let cached = cache.get(&hash, 80).expect("hit");
        assert_eq!(cached.issues.len(), 1);
        assert_eq!(cached.issues[0], issue);
    }
}
