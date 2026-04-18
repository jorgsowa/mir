use std::sync::{Arc, RwLock};

use dashmap::DashMap;

/// Thread-safe string interner — maps `Arc<str>` ↔ `u32` IDs.
///
/// Interning replaces repeated `Arc<str>` pointers (16 bytes each) with 4-byte
/// `u32` IDs. The same string always maps to the same ID for the lifetime of
/// this interner.
///
/// # Concurrency
///
/// - Fast path (already interned): lock-free read from the `DashMap`.
/// - Slow path (new string): acquires a `RwLock` write guard, re-checks under
///   the lock to handle races, then assigns an ID atomically.
/// - `get(id)` acquires a read guard; multiple concurrent readers are allowed.
#[derive(Debug, Default)]
pub struct Interner {
    /// Fast read path: string → ID.  Written only while holding `to_str` write lock.
    to_id: DashMap<Arc<str>, u32>,
    /// ID → string table.  The write lock also serialises ID assignment.
    to_str: RwLock<Vec<Arc<str>>>,
}

impl Interner {
    /// Intern `s` and return its ID. Idempotent: the same string always returns
    /// the same ID.
    pub fn intern(&self, s: Arc<str>) -> u32 {
        // Fast path — already interned, no allocation needed.
        if let Some(id) = self.to_id.get(s.as_ref()) {
            return *id;
        }
        // Slow path — serialise ID assignment under the write lock.
        let mut vec = self.to_str.write().expect("interner lock poisoned");
        // Re-check: another thread may have raced between the fast-path read
        // and our acquisition of the write lock.
        if let Some(id) = self.to_id.get(s.as_ref()) {
            return *id;
        }
        let id = vec.len() as u32;
        vec.push(s.clone());
        // Insert into DashMap while still holding the write lock so that any
        // thread doing `get(id)` after seeing this entry in `to_id` is
        // guaranteed to find the string already in `vec`.
        self.to_id.insert(s, id);
        id
    }

    /// Intern from a `&str` without allocating an `Arc` when the string is
    /// already interned.
    pub fn intern_str(&self, s: &str) -> u32 {
        // `Arc<str>: Borrow<str>`, so DashMap lets us look up with `&str` directly.
        if let Some(id) = self.to_id.get(s) {
            return *id;
        }
        self.intern(Arc::from(s))
    }

    /// Resolve an ID back to its string. Panics if `id` is out of range.
    pub fn get(&self, id: u32) -> Arc<str> {
        self.to_str.read().expect("interner lock poisoned")[id as usize].clone()
    }

    /// Return the ID for `s` if it has already been interned, or `None`.
    pub fn get_id(&self, s: &str) -> Option<u32> {
        self.to_id.get(s).map(|id| *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_string_gives_same_id() {
        let interner = Interner::default();
        let a = interner.intern_str("Foo::bar");
        let b = interner.intern_str("Foo::bar");
        assert_eq!(a, b);
    }

    #[test]
    fn different_strings_give_different_ids() {
        let interner = Interner::default();
        let a = interner.intern_str("Foo::bar");
        let b = interner.intern_str("Foo::baz");
        assert_ne!(a, b);
    }

    #[test]
    fn get_roundtrips_id_to_string() {
        let interner = Interner::default();
        let id = interner.intern_str("App\\Service");
        assert_eq!(interner.get(id).as_ref(), "App\\Service");
    }

    #[test]
    fn get_id_returns_none_for_unknown_string() {
        let interner = Interner::default();
        assert!(interner.get_id("unknown").is_none());
    }

    #[test]
    fn intern_and_intern_str_agree() {
        let interner = Interner::default();
        let id_arc = interner.intern(Arc::from("hello"));
        let id_str = interner.intern_str("hello");
        assert_eq!(id_arc, id_str);
    }

    #[test]
    fn concurrent_intern_is_consistent() {
        use std::sync::Arc as StdArc;
        let interner = StdArc::new(Interner::default());
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let i = interner.clone();
                std::thread::spawn(move || i.intern_str("shared"))
            })
            .collect();
        let ids: Vec<u32> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(
            ids.iter().all(|&id| id == ids[0]),
            "all threads must see the same ID"
        );
    }
}
