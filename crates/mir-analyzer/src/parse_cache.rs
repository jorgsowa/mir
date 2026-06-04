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

/// Content-hash-keyed, capacity-bounded cache of parsed [`StubSlice`]s.
pub struct ParseCache {
    map: DashMap<[u8; 32], Arc<StubSlice>>,
    /// Insertion order of keys, for FIFO eviction. Holds keys that may already
    /// have been removed; eviction tolerates stale entries.
    order: Mutex<VecDeque<[u8; 32]>>,
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

    /// Look up a parsed slice by content hash.
    pub fn get(&self, hash: &[u8; 32]) -> Option<Arc<StubSlice>> {
        self.map.get(hash).map(|r| Arc::clone(&*r))
    }

    /// Insert a parsed slice. On a genuinely new key, evicts oldest entries
    /// (FIFO) until the cache is within capacity.
    pub fn insert(&self, hash: [u8; 32], slice: Arc<StubSlice>) {
        let is_new = self.map.insert(hash, slice).is_none();
        if !is_new {
            return;
        }
        let mut order = self.order.lock();
        order.push_back(hash);
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
    pub fn remove(&self, hash: &[u8; 32]) {
        self.map.remove(hash);
    }

    /// Current number of cached slices.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
