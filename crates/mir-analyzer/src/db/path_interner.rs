//! Path ↔ [`FileNo`] interning, shared by every index keyed on file identity.
//!
//! Apart from the posting lists, like rust-analyzer's VFS `FileId` table:
//! translating a path to its id never contends with posting reads or writes,
//! and ids are never reused. Reads scale across threads: the path → id map is
//! sharded and id → path is an append-only vector read without any lock. A
//! single `RwLock` here collapsed to a seventh of its one-thread throughput
//! at 18 threads, because every reader writes the lock's cache line.

use std::sync::Arc;

use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use rustc_hash::FxBuildHasher;

use super::ref_index::FileNo;

#[derive(Default)]
pub struct PathInterner {
    ids: DashMap<Arc<str>, FileNo, FxBuildHasher>,
    /// Id → path. An id is published only after its path is pushed.
    paths: boxcar::Vec<Arc<str>>,
}

impl PathInterner {
    /// The id for `path`, assigning the next one when it is new.
    pub fn intern(&self, path: &Arc<str>) -> FileNo {
        if let Some(id) = self.ids.get(path.as_ref()) {
            return *id;
        }
        match self.ids.entry(path.clone()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = self.paths.push(path.clone()) as FileNo;
                entry.insert(id);
                id
            }
        }
    }

    pub fn lookup(&self, path: &str) -> Option<FileNo> {
        self.ids.get(path).map(|id| *id)
    }

    /// Resolve an id back to its path. Panics if the id is invalid.
    pub fn path_of(&self, file: FileNo) -> Arc<str> {
        self.paths[file as usize].clone()
    }
}

impl std::fmt::Debug for PathInterner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathInterner")
            .field("paths", &self.paths.count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_dense_and_stable() {
        let interner = PathInterner::default();
        assert_eq!(interner.lookup("a.php"), None);
        assert_eq!(interner.intern(&Arc::from("a.php")), 0);
        assert_eq!(interner.intern(&Arc::from("b.php")), 1);
        assert_eq!(interner.intern(&Arc::from("a.php")), 0);
        assert_eq!(interner.lookup("b.php"), Some(1));
        assert_eq!(interner.path_of(1).as_ref(), "b.php");
    }

    /// Racing interners agree on one id per path, ids are dense, and every
    /// id resolves to the path it was assigned to.
    #[test]
    fn concurrent_interning_assigns_each_path_one_dense_id() {
        const THREADS: usize = 16;
        const PATHS: usize = 2_000;
        let interner = Arc::new(PathInterner::default());
        let paths: Arc<Vec<Arc<str>>> = Arc::new(
            (0..PATHS)
                .map(|i| Arc::from(format!("src/File{i}.php").as_str()))
                .collect(),
        );
        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                let (interner, paths) = (interner.clone(), paths.clone());
                std::thread::spawn(move || {
                    (0..PATHS)
                        .map(|i| {
                            // Each thread walks the paths from its own offset.
                            let path = &paths[(i + t * 97) % PATHS];
                            let id = interner.intern(path);
                            assert_eq!(interner.path_of(id).as_ref(), path.as_ref());
                            assert_eq!(interner.lookup(path), Some(id));
                            (path.clone(), id)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let mut agreed = std::collections::HashMap::new();
        for handle in handles {
            for (path, id) in handle.join().unwrap() {
                assert_eq!(*agreed.entry(path).or_insert(id), id);
            }
        }
        let mut ids: Vec<_> = agreed.values().copied().collect();
        ids.sort_unstable();
        assert_eq!(ids, (0..PATHS as FileNo).collect::<Vec<_>>());
    }
}
