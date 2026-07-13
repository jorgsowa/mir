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

/// Content-hash-keyed, capacity-bounded cache of parsed [`StubSlice`]s.
pub struct ParseCache {
    map: DashMap<ParseCacheKey, Arc<StubSlice>>,
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

    /// Look up a parsed slice by content hash and target PHP version
    /// (`PhpVersion::cache_byte()`).
    pub fn get(&self, hash: &[u8; 32], php_v: u8) -> Option<Arc<StubSlice>> {
        self.map.get(&(*hash, php_v)).map(|r| Arc::clone(&*r))
    }

    /// Insert a parsed slice. On a genuinely new key, evicts oldest entries
    /// (FIFO) until the cache is within capacity.
    pub fn insert(&self, hash: [u8; 32], php_v: u8, slice: Arc<StubSlice>) {
        let key = (hash, php_v);
        let is_new = self.map.insert(key, slice).is_none();
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

    #[test]
    fn get_is_isolated_per_php_version() {
        let cache = ParseCache::with_capacity(8);
        let hash = [1u8; 32];
        let slice_80 = Arc::new(StubSlice::default());
        cache.insert(hash, 80, slice_80);

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
        cache.insert(hash, 80, Arc::new(StubSlice::default()));
        cache.insert(hash, 81, Arc::new(StubSlice::default()));

        assert!(cache.get(&hash, 80).is_some());
        assert!(cache.get(&hash, 81).is_some());
        assert_eq!(cache.len(), 2, "both version-specific entries must be retained");
    }
}
