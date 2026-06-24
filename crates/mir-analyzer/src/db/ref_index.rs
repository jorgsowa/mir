//! Unified reference index: the single derived aggregate behind the
//! `MirDatabase` reference-location API.
//!
//! Replaces three independently-locked maps (`reference_locations`,
//! `file_references`, `symbol_referencers`) that had to be maintained in
//! lockstep — every writer needed all three locks in the right order, and a
//! missed update in any one of them produced the reverse-deps drift bug
//! class. `RefIndex` owns all views behind one lock with two mutation
//! entry points ([`RefIndex::append_batch`] and [`RefIndex::clear_file`]),
//! so the views cannot disagree.
//!
//! File paths are interned to `u32` ids internally: per-location tuples
//! store 4 bytes instead of an `Arc<str>` (8 bytes + refcount traffic), and
//! file-keyed lookups hash an integer instead of a path string. The public
//! API still speaks `Arc<str>` paths; resolving an id back to its path is an
//! O(1) `Arc` clone.

use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use super::reference_locations::RefLoc;

/// Interned file id, valid within one `RefIndex` instance.
type FileNo = u32;

/// `(file, line, col_start, col_end)` with the file as an interned id.
type LocTuple = (FileNo, u32, u16, u16);

#[derive(Default, Debug)]
pub struct RefIndex {
    /// Path → interned id.
    path_ids: FxHashMap<Arc<str>, FileNo>,
    /// Id → path (`Arc` shared with `path_ids` keys).
    paths: Vec<Arc<str>>,
    /// Symbol key → `(file, line, col_start, col_end)` in insertion order,
    /// deduplicated. The id-resolved equivalent of the old
    /// `reference_locations` map.
    by_symbol: FxHashMap<Arc<str>, Vec<LocTuple>>,
    /// Forward view: file → set of symbol keys it references.
    file_symbols: FxHashMap<FileNo, FxHashSet<Arc<str>>>,
    /// Reverse view: symbol key → set of files referencing it.
    referencers: FxHashMap<Arc<str>, FxHashSet<FileNo>>,
}

impl RefIndex {
    fn intern(&mut self, path: &Arc<str>) -> FileNo {
        if let Some(&id) = self.path_ids.get(path.as_ref()) {
            return id;
        }
        let id = self.paths.len() as FileNo;
        self.paths.push(path.clone());
        self.path_ids.insert(path.clone(), id);
        id
    }

    fn lookup(&self, path: &str) -> Option<FileNo> {
        self.path_ids.get(path).copied()
    }

    fn path_of(&self, id: FileNo) -> Arc<str> {
        self.paths[id as usize].clone()
    }

    /// Append a batch of reference locations. Per-entry deduplicated against
    /// the existing locations of the same symbol, preserving insertion order
    /// (mirrors the legacy `commit_reference_locations_batch` semantics).
    pub fn append_batch(&mut self, locs: Vec<RefLoc>) {
        for loc in locs {
            let file_id = self.intern(&loc.file);
            self.file_symbols
                .entry(file_id)
                .or_default()
                .insert(loc.symbol_key.clone());
            self.referencers
                .entry(loc.symbol_key.clone())
                .or_default()
                .insert(file_id);
            let entry = self.by_symbol.entry(loc.symbol_key).or_default();
            let tuple = (file_id, loc.line, loc.col_start, loc.col_end);
            if !entry.contains(&tuple) {
                entry.push(tuple);
            }
        }
    }

    /// Remove every reference recorded as appearing in `file`. O(degree):
    /// the forward view names exactly the symbols that need fixing.
    pub fn clear_file(&mut self, file: &str) {
        let Some(file_id) = self.lookup(file) else {
            return;
        };
        let Some(symbol_keys) = self.file_symbols.remove(&file_id) else {
            return;
        };
        for key in &symbol_keys {
            if let Some(locs) = self.by_symbol.get_mut(key) {
                locs.retain(|&(f, _, _, _)| f != file_id);
                if locs.is_empty() {
                    self.by_symbol.remove(key);
                }
            }
            if let Some(refs) = self.referencers.get_mut(key) {
                refs.remove(&file_id);
                if refs.is_empty() {
                    self.referencers.remove(key);
                }
            }
        }
    }

    /// Replace `file`'s reference set wholesale with `locs`, which must be
    /// `file`'s own complete reference set (every `loc.file == file` — the
    /// batch + incremental pipelines that call this all satisfy that).
    ///
    /// Dedups *within the batch only*, which is sound here and avoids the
    /// O(total-refs-to-symbol) scan [`append_batch`] performs per entry:
    /// `clear_file` just removed `file`'s prior tuples, and every other file's
    /// tuples carry a different interned id, so no incoming tuple can collide
    /// with one already in `by_symbol`. Scanning the full per-symbol vector
    /// each time made committing a hot symbol (one referenced by many files)
    /// quadratic in the number of referencing files.
    pub fn set_file_refs(&mut self, file: &str, locs: Vec<RefLoc>) {
        self.clear_file(file);
        let mut seen: FxHashSet<(Arc<str>, LocTuple)> = FxHashSet::default();
        for loc in locs {
            let file_id = self.intern(&loc.file);
            let tuple = (file_id, loc.line, loc.col_start, loc.col_end);
            if !seen.insert((loc.symbol_key.clone(), tuple)) {
                continue;
            }
            self.file_symbols
                .entry(file_id)
                .or_default()
                .insert(loc.symbol_key.clone());
            self.referencers
                .entry(loc.symbol_key.clone())
                .or_default()
                .insert(file_id);
            self.by_symbol
                .entry(loc.symbol_key)
                .or_default()
                .push(tuple);
        }
    }

    /// All locations of one symbol: `(file, line, col_start, col_end)`.
    pub fn locations_of(&self, symbol: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        self.by_symbol
            .get(symbol)
            .map(|locs| {
                locs.iter()
                    .map(|&(f, line, cs, ce)| (self.path_of(f), line, cs, ce))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether the symbol has at least one recorded reference.
    pub fn has_reference(&self, symbol: &str) -> bool {
        self.by_symbol.get(symbol).is_some_and(|l| !l.is_empty())
    }

    /// All files referencing `symbol`.
    pub fn referencers_of(&self, symbol: &str) -> Vec<Arc<str>> {
        self.referencers
            .get(symbol)
            .map(|files| files.iter().map(|&f| self.path_of(f)).collect())
            .unwrap_or_default()
    }

    /// All symbol keys referenced by `file`.
    pub fn symbols_referenced_by(&self, file: &str) -> Vec<Arc<str>> {
        self.lookup(file)
            .and_then(|id| self.file_symbols.get(&id))
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// All of `file`'s reference locations in cache-storage shape:
    /// `(symbol_key, line, col_start, col_end)`. O(file degree × per-symbol
    /// locations) via the forward view, not O(total index size).
    pub fn file_locations(&self, file: &str) -> Vec<(Arc<str>, u32, u16, u16)> {
        let Some(file_id) = self.lookup(file) else {
            return Vec::new();
        };
        let Some(symbols) = self.file_symbols.get(&file_id) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for sym in symbols {
            if let Some(locs) = self.by_symbol.get(sym) {
                for &(f, line, cs, ce) in locs {
                    if f == file_id {
                        out.push((sym.clone(), line, cs, ce));
                    }
                }
            }
        }
        out
    }

    /// Every `(file, symbol_key)` reference pair across the index.
    pub fn all_pairs(&self) -> Vec<(Arc<str>, Arc<str>)> {
        let mut pairs = Vec::new();
        for (file_id, symbols) in &self.file_symbols {
            let path = self.path_of(*file_id);
            for sym in symbols {
                pairs.push((path.clone(), sym.clone()));
            }
        }
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc(sym: &str, file: &str, line: u32) -> RefLoc {
        RefLoc {
            symbol_key: Arc::from(sym),
            file: Arc::from(file),
            line,
            col_start: 1,
            col_end: 2,
        }
    }

    #[test]
    fn append_dedup_and_views_stay_consistent() {
        let mut idx = RefIndex::default();
        idx.append_batch(vec![
            loc("fn:foo", "a.php", 1),
            loc("fn:foo", "a.php", 1), // duplicate
            loc("fn:foo", "b.php", 2),
            loc("cls:Bar", "a.php", 3),
        ]);
        assert_eq!(idx.locations_of("fn:foo").len(), 2);
        assert!(idx.has_reference("cls:Bar"));
        let mut refs = idx.referencers_of("fn:foo");
        refs.sort();
        assert_eq!(refs, vec![Arc::<str>::from("a.php"), Arc::from("b.php")]);
        let mut syms = idx.symbols_referenced_by("a.php");
        syms.sort();
        assert_eq!(syms, vec![Arc::<str>::from("cls:Bar"), Arc::from("fn:foo")]);
    }

    #[test]
    fn clear_file_prunes_all_views() {
        let mut idx = RefIndex::default();
        idx.append_batch(vec![
            loc("fn:foo", "a.php", 1),
            loc("fn:foo", "b.php", 2),
            loc("cls:OnlyA", "a.php", 3),
        ]);
        idx.clear_file("a.php");
        assert_eq!(idx.locations_of("fn:foo").len(), 1);
        assert_eq!(
            idx.referencers_of("fn:foo"),
            vec![Arc::<str>::from("b.php")]
        );
        assert!(!idx.has_reference("cls:OnlyA"));
        assert!(idx.symbols_referenced_by("a.php").is_empty());
        assert!(idx.file_locations("a.php").is_empty());
    }

    #[test]
    fn set_file_refs_replaces_only_that_file() {
        let mut idx = RefIndex::default();
        idx.append_batch(vec![loc("fn:foo", "a.php", 1), loc("fn:foo", "b.php", 2)]);
        idx.set_file_refs("a.php", vec![loc("cls:New", "a.php", 9)]);
        assert!(!idx
            .referencers_of("fn:foo")
            .contains(&Arc::<str>::from("a.php")));
        assert_eq!(
            idx.referencers_of("fn:foo"),
            vec![Arc::<str>::from("b.php")]
        );
        assert_eq!(idx.file_locations("a.php").len(), 1);
    }

    #[test]
    fn set_file_refs_dedups_within_batch() {
        let mut idx = RefIndex::default();
        idx.set_file_refs(
            "a.php",
            vec![
                loc("fn:foo", "a.php", 1),
                loc("fn:foo", "a.php", 1), // same position, dropped
                loc("fn:foo", "a.php", 2),
            ],
        );
        assert_eq!(idx.locations_of("fn:foo").len(), 2);
    }

    #[test]
    fn set_file_refs_hot_symbol_across_many_files() {
        // One symbol referenced by many files — the case that used to be
        // quadratic. Views must stay exact and replace-on-recommit must work.
        let mut idx = RefIndex::default();
        for f in 0..50 {
            let file = format!("f{f}.php");
            idx.set_file_refs(
                &file,
                vec![loc("m:Base::foo", &file, 1), loc("m:Base::foo", &file, 2)],
            );
        }
        assert_eq!(idx.locations_of("m:Base::foo").len(), 100);
        assert_eq!(idx.referencers_of("m:Base::foo").len(), 50);
        // Re-commit one file with fewer refs: only its entries change.
        idx.set_file_refs("f0.php", vec![loc("m:Base::foo", "f0.php", 9)]);
        assert_eq!(idx.locations_of("m:Base::foo").len(), 99);
        assert_eq!(idx.referencers_of("m:Base::foo").len(), 50);
    }

    #[test]
    fn file_locations_matches_cache_shape() {
        let mut idx = RefIndex::default();
        idx.append_batch(vec![loc("fn:foo", "a.php", 1), loc("fn:bar", "a.php", 5)]);
        let mut locs = idx.file_locations("a.php");
        locs.sort();
        assert_eq!(locs.len(), 2);
        assert_eq!(locs[0].0.as_ref(), "fn:bar");
    }
}
