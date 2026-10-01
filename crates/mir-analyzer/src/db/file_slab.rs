//! Per-file state indexed by the file's salsa input handle.
//!
//! Freshness scans read one entry per candidate file from many threads at
//! once. A path-keyed map makes each read hash and compare the path and, in a
//! sharded map, still writes one of a few shared lock words; here a read
//! indexes straight to the file's own slot, and every slot sits on its own
//! cache line so threads reading different files never contend.
//! The handle's index is dense and never reused, so it plays the part of
//! rust-analyzer's `FileId`.

use parking_lot::RwLock;
use salsa::plumbing::AsId;

use super::SourceFile;

/// The slab index of `file`.
fn slot_of(file: SourceFile) -> u32 {
    file.as_id().index()
}

#[repr(align(64))]
struct Slot<T>(RwLock<Option<T>>);

pub(crate) struct FileSlab<T> {
    slots: boxcar::Vec<Slot<T>>,
}

impl<T> Default for FileSlab<T> {
    fn default() -> Self {
        Self {
            slots: boxcar::Vec::new(),
        }
    }
}

impl<T> FileSlab<T> {
    /// Run `read` on `file`'s entry, if it has one.
    pub(crate) fn read<R>(&self, file: SourceFile, read: impl FnOnce(&T) -> R) -> Option<R> {
        let slot = self.slots.get(slot_of(file) as usize)?;
        slot.0.read().as_ref().map(read)
    }

    pub(crate) fn set(&self, file: SourceFile, value: T) {
        *self.slot_or_grow(slot_of(file)).0.write() = Some(value);
    }

    pub(crate) fn remove(&self, file: SourceFile) {
        if let Some(slot) = self.slots.get(slot_of(file) as usize) {
            *slot.0.write() = None;
        }
    }

    fn slot_or_grow(&self, index: u32) -> &Slot<T> {
        let index = index as usize;
        loop {
            if let Some(slot) = self.slots.get(index) {
                return slot;
            }
            if self.slots.count() <= index {
                self.slots.push(Slot(RwLock::new(None)));
            } else {
                // Another thread's push of this slot is in flight.
                std::hint::spin_loop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::db::MirDbStorage;

    fn files(db: &MirDbStorage, n: usize) -> Vec<SourceFile> {
        (0..n)
            .map(|i| {
                SourceFile::new(
                    db,
                    Arc::from(format!("f{i}.php").as_str()),
                    Arc::from("<?php"),
                )
            })
            .collect()
    }

    #[test]
    fn entries_are_per_file_and_removable() {
        let db = MirDbStorage::default();
        let files = files(&db, 3);
        let slab: FileSlab<u32> = FileSlab::default();
        assert_eq!(slab.read(files[2], |v| *v), None);

        slab.set(files[2], 7);
        assert_eq!(slab.read(files[2], |v| *v), Some(7));
        assert_eq!(slab.read(files[0], |v| *v), None);
        assert_eq!(slab.read(files[1], |v| *v), None);

        slab.set(files[2], 8);
        assert_eq!(slab.read(files[2], |v| *v), Some(8));
        slab.remove(files[2]);
        assert_eq!(slab.read(files[2], |v| *v), None);
        slab.remove(files[0]);
    }

    /// Writers growing the slab from many threads at once lose no entry.
    #[test]
    fn concurrent_growth_keeps_every_entry() {
        const THREADS: usize = 8;
        const PER_THREAD: usize = 500;
        let db = MirDbStorage::default();
        let files = Arc::new(files(&db, THREADS * PER_THREAD));
        let slab = Arc::new(FileSlab::<usize>::default());
        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                let (files, slab) = (files.clone(), slab.clone());
                std::thread::spawn(move || {
                    // Interleaved, high indices first, so growth races.
                    for i in (0..PER_THREAD).rev() {
                        let n = i * THREADS + t;
                        slab.set(files[n], n);
                        assert_eq!(slab.read(files[n], |v| *v), Some(n));
                    }
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
        for (n, file) in files.iter().enumerate() {
            assert_eq!(slab.read(*file, |v| *v), Some(n));
        }
    }
}
