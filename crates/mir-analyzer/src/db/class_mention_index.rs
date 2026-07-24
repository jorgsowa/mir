//! Class-name mention index: file → the declared class-like short names its
//! raw text mentions as whole identifiers.
//!
//! The reference-query gate in `indexed_references_to` admits a
//! never-committed file only when its text mentions the queried symbol's
//! name — previously a fresh Aho-Corasick pass over every candidate's raw
//! text on *every* query. This index memoizes that purely textual predicate:
//! a file scanned once (against the whole known-name universe) answers all
//! later single-needle gates with an O(log n) set lookup instead of an
//! O(text) scan.
//!
//! Correctness model — the index can only ever say what a raw scan would:
//! - An entry is keyed to its source text by `Arc` identity; an edit
//!   self-invalidates it (the reader falls back to a raw scan).
//! - The name universe is append-only within a session; each name records
//!   the epoch it entered. An entry scanned at epoch E answers only needles
//!   added at or before E — a class declared later falls back to the raw
//!   scan until the file is rescanned (which any later scan upgrades).
//! - Scan semantics are byte-identical to `IdentifierNeedles::matches`
//!   (whole identifier, ASCII-case-insensitive).
//!
//! Concurrency: per-file entries live in a sharded `DashMap` (the gate loop
//! reads one entry per candidate from many rayon workers at once — a single
//! mutex there serializes the whole pass, measured slower than the raw scan
//! it replaces). The universe/scanner state sits behind one mutex, touched
//! O(1) times per query.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};

use mir_types::Name;

/// Compiled whole-universe scanner frozen at one epoch. Cheap to share
/// across rayon workers; rebuilt lazily when the universe grows.
pub struct MentionScanner {
    epoch: u64,
    ac: aho_corasick::AhoCorasick,
    /// Pattern id → the (lowercased) name it matches.
    patterns: Vec<Name>,
}

impl MentionScanner {
    pub(crate) fn build(epoch: u64, patterns: Vec<Name>) -> Option<Self> {
        if patterns.is_empty() {
            return None;
        }
        let ac = aho_corasick::AhoCorasick::builder()
            .ascii_case_insensitive(true)
            .build(patterns.iter().map(|n| n.as_str()))
            .ok()?;
        Some(Self {
            epoch,
            ac,
            patterns,
        })
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Every universe name `hay` mentions as a whole identifier, sorted for
    /// binary search. Boundary rule matches `IdentifierNeedles::matches`.
    pub fn scan(&self, hay: &str) -> Box<[Name]> {
        let bytes = hay.as_bytes();
        let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let mut seen: FxHashSet<usize> = FxHashSet::default();
        let mut hits: Vec<Name> = Vec::new();
        for m in self.ac.find_overlapping_iter(hay) {
            let pid = m.pattern().as_usize();
            if seen.contains(&pid) {
                continue;
            }
            if (m.start() == 0 || !is_ident(bytes[m.start() - 1]))
                && (m.end() == bytes.len() || !is_ident(bytes[m.end()]))
            {
                seen.insert(pid);
                hits.push(self.patterns[pid]);
            }
        }
        hits.sort_unstable();
        hits.into_boxed_slice()
    }
}

/// One file's recorded mention set.
struct FileMentions {
    /// Text the scan ran against (`Arc` identity; an edit self-invalidates).
    text: Arc<str>,
    /// Scanner epoch of the scan: definitive for names added at or before it.
    epoch: u64,
    /// Sorted lowercased short names the text mentions.
    names: Box<[Name]>,
}

/// A prepared coverage key: the needle plus the epoch it entered the
/// universe. Built once per query, checked per file.
#[derive(Clone, Copy)]
pub struct MentionQuery {
    pub name: Name,
    added_epoch: u64,
}

/// Size/coverage counters for memory validation and host metrics.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClassMentionStats {
    pub universe_names: usize,
    pub files_covered: usize,
    pub total_mentions: usize,
    /// Heap bytes of the cached scanner's automaton, if built.
    pub scanner_bytes: usize,
    /// Cumulative scan results recorded (each is one raw-text pass a later
    /// query gets to skip). Flat across a query ⇒ the gate ran lookup-only.
    pub scans_recorded: u64,
}

/// Universe state: what names exist, since when, and the cached automaton.
#[derive(Default)]
struct Universe {
    /// Bumped once per batch that adds at least one new name.
    epoch: u64,
    /// Lowercased short name → epoch it entered the universe. Append-only:
    /// a name outliving its declaration only widens mention sets (the
    /// predicate is textual, so answers stay exact).
    names: FxHashMap<Name, u64>,
    /// Cached scanner; valid while its epoch matches `epoch`.
    scanner: Option<Arc<MentionScanner>>,
}

#[derive(Default)]
pub struct ClassMentionIndex {
    universe: Mutex<Universe>,
    /// Sharded per-file entries: the gate loop's per-candidate lookups come
    /// from many rayon workers concurrently and must not serialize.
    by_file: dashmap::DashMap<Arc<str>, FileMentions, FxBuildHasher>,
    scans_recorded: AtomicU64,
}

impl ClassMentionIndex {
    /// Insert lowercased short names; bumps the epoch once if any is new.
    pub fn add_names(&self, names: impl IntoIterator<Item = Name>) {
        let mut u = self.universe.lock();
        let next = u.epoch + 1;
        let mut added = false;
        for n in names {
            if let std::collections::hash_map::Entry::Vacant(v) = u.names.entry(n) {
                v.insert(next);
                added = true;
            }
        }
        if added {
            u.epoch = next;
        }
    }

    /// The whole-universe scanner for the current epoch, building it if the
    /// universe grew. The automaton build runs outside the universe lock (a
    /// concurrent builder may waste one build; installs are idempotent).
    /// `None` while the universe is empty.
    pub fn scanner(&self) -> Option<Arc<MentionScanner>> {
        let (epoch, patterns) = {
            let u = self.universe.lock();
            if u.names.is_empty() {
                return None;
            }
            if let Some(s) = u.scanner.as_ref().filter(|s| s.epoch == u.epoch) {
                return Some(s.clone());
            }
            (u.epoch, u.names.keys().copied().collect::<Vec<Name>>())
        };
        let scanner = Arc::new(MentionScanner::build(epoch, patterns)?);
        let mut u = self.universe.lock();
        if scanner.epoch == u.epoch {
            u.scanner = Some(scanner.clone());
        }
        Some(scanner)
    }

    /// Resolve `needle` (any case, short name) against the universe.
    pub fn prepare_query(&self, needle: &str) -> Option<MentionQuery> {
        let name = Name::new(needle).ascii_lowercase();
        let added_epoch = *self.universe.lock().names.get(&name)?;
        Some(MentionQuery { name, added_epoch })
    }

    /// Whether `file`'s current text mentions the queried name. `None` when
    /// the entry can't answer (missing, text changed, or scanned before the
    /// name entered the universe) — the caller must fall back to a raw scan.
    pub fn answer(&self, file: &str, q: &MentionQuery, current_text: &Arc<str>) -> Option<bool> {
        let e = self.by_file.get(file)?;
        if !Arc::ptr_eq(&e.text, current_text) || e.epoch < q.added_epoch {
            return None;
        }
        Some(e.names.binary_search(&q.name).is_ok())
    }

    /// Whether `file` already holds a scan of exactly `text` at `epoch` or
    /// newer (used by analyze sweeps to skip redundant rescans).
    pub fn is_current(&self, file: &str, text: &Arc<str>, epoch: u64) -> bool {
        self.by_file
            .get(file)
            .is_some_and(|e| Arc::ptr_eq(&e.text, text) && e.epoch >= epoch)
    }

    /// Record a scan result. Never downgrades a same-text entry from a newer
    /// epoch.
    pub fn set_file(&self, file: Arc<str>, text: Arc<str>, epoch: u64, names: Box<[Name]>) {
        match self.by_file.entry(file) {
            dashmap::mapref::entry::Entry::Occupied(mut o) => {
                let prev = o.get();
                if Arc::ptr_eq(&prev.text, &text) && prev.epoch >= epoch {
                    return;
                }
                o.insert(FileMentions { text, epoch, names });
            }
            dashmap::mapref::entry::Entry::Vacant(v) => {
                v.insert(FileMentions { text, epoch, names });
            }
        }
        self.scans_recorded.fetch_add(1, Ordering::Relaxed);
    }

    pub fn clear_file(&self, file: &str) {
        self.by_file.remove(file);
    }

    pub fn stats(&self) -> ClassMentionStats {
        let u = self.universe.lock();
        ClassMentionStats {
            universe_names: u.names.len(),
            files_covered: self.by_file.len(),
            total_mentions: self.by_file.iter().map(|e| e.names.len()).sum(),
            scanner_bytes: u.scanner.as_ref().map(|s| s.ac.memory_usage()).unwrap_or(0),
            scans_recorded: self.scans_recorded.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lc(s: &str) -> Name {
        Name::new(s).ascii_lowercase()
    }

    fn scanner_for(idx: &ClassMentionIndex) -> Arc<MentionScanner> {
        idx.scanner().unwrap()
    }

    #[test]
    fn scan_is_whole_identifier_and_case_insensitive() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Color"), lc("Save")]);
        let s = scanner_for(&idx);
        assert_eq!(s.scan("new COLOR();").as_ref(), &[lc("Color")]);
        assert!(s.scan("$this->saveAll();").is_empty());
        let both = s.scan("Color::save()");
        assert_eq!(both.len(), 2);
    }

    #[test]
    fn epoch_gates_needles_added_after_the_scan() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Alpha")]);
        let s1 = scanner_for(&idx);
        let text: Arc<str> = Arc::from("uses Alpha and Beta");
        let file: Arc<str> = Arc::from("a.php");
        idx.set_file(file.clone(), text.clone(), s1.epoch(), s1.scan(&text));

        let q_alpha = idx.prepare_query("Alpha").unwrap();
        assert_eq!(idx.answer(&file, &q_alpha, &text), Some(true));

        // Beta enters the universe later: the old scan can't answer for it.
        idx.add_names([lc("Beta")]);
        let q_beta = idx.prepare_query("Beta").unwrap();
        assert_eq!(idx.answer(&file, &q_beta, &text), None);
        // Old needles still answer.
        assert_eq!(idx.answer(&file, &q_alpha, &text), Some(true));

        // A rescan at the new epoch covers Beta.
        let s2 = scanner_for(&idx);
        idx.set_file(file.clone(), text.clone(), s2.epoch(), s2.scan(&text));
        assert_eq!(idx.answer(&file, &q_beta, &text), Some(true));
    }

    #[test]
    fn text_identity_gates_answers() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Job")]);
        let s = scanner_for(&idx);
        let text: Arc<str> = Arc::from("new Job();");
        let file: Arc<str> = Arc::from("a.php");
        idx.set_file(file.clone(), text.clone(), s.epoch(), s.scan(&text));
        let q = idx.prepare_query("Job").unwrap();
        assert_eq!(idx.answer(&file, &q, &text), Some(true));
        // Content-equal but different Arc: not answerable (must rescan).
        let other: Arc<str> = Arc::from("new Job();");
        assert_eq!(idx.answer(&file, &q, &other), None);
    }

    #[test]
    fn re_adding_existing_names_does_not_bump_epoch() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("A"), lc("B")]);
        let e1 = scanner_for(&idx).epoch();
        idx.add_names([lc("A"), lc("B")]);
        let e2 = scanner_for(&idx).epoch();
        assert_eq!(e1, e2, "no new names → no epoch bump");
        idx.add_names([lc("C")]);
        let e3 = scanner_for(&idx).epoch();
        assert_eq!(e3, e2 + 1);
    }

    #[test]
    fn unknown_needle_prepares_no_query() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Known")]);
        assert!(idx.prepare_query("Unknown").is_none());
        assert!(idx.prepare_query("known").is_some(), "case-folded lookup");
    }

    #[test]
    fn set_file_keeps_newer_same_text_entry() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("A")]);
        let text: Arc<str> = Arc::from("A");
        let file: Arc<str> = Arc::from("f.php");
        idx.set_file(file.clone(), text.clone(), 5, Box::new([lc("A")]));
        // A stale (older-epoch) commit racing in must not downgrade coverage.
        idx.set_file(file.clone(), text.clone(), 3, Box::new([]));
        let q = idx.prepare_query("A").unwrap();
        assert_eq!(idx.answer(&file, &q, &text), Some(true));
    }

    #[test]
    fn scanner_is_cached_per_epoch() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("A")]);
        let s1 = idx.scanner().unwrap();
        let s2 = idx.scanner().unwrap();
        assert!(Arc::ptr_eq(&s1, &s2), "same epoch → cached automaton");
        idx.add_names([lc("B")]);
        let s3 = idx.scanner().unwrap();
        assert!(!Arc::ptr_eq(&s1, &s3), "universe grew → rebuilt automaton");
    }
}
