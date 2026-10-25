//! Mention index: file → the needles its raw text mentions. The universe
//! holds declared class-like short names (seeded by indexing), literal
//! needles admitted at query time (member/function names), and raw call
//! tokens (`->__construct`), so it is the single implementation of the
//! textual gates.
//!
//! The reference-query and subtype-BFS gates admit a never-committed file
//! only when its text mentions a queried needle. This index memoizes that
//! purely textual predicate: a file scanned once answers all later gates
//! with an O(log n) set lookup instead of an O(text) scan.
//!
//! A scan records two things per file:
//! - `words`: hashes of every maximal `[A-Za-z0-9_]` run, lowercased. A
//!   bounded needle made only of those bytes ("word needle") is mentioned
//!   exactly when it equals one of those runs, so this answers any word
//!   needle — including one admitted after the scan — without rescanning.
//!   A hash collision only admits an extra file (the conservative side).
//! - `names`: hits of the Aho-Corasick automaton built from the remaining
//!   "pattern" needles: raw ones, and bounded ones holding a non-word byte
//!   (non-ASCII identifiers, `\`).
//!
//! Correctness model — the index can only ever say what a raw scan would:
//! - An entry is keyed to its source text by `Arc` identity; an edit
//!   self-invalidates it (the reader falls back to a raw scan).
//! - The name universe is append-only within a session. Each pattern needle
//!   records the epoch it entered; an entry scanned at epoch E answers only
//!   pattern needles added at or before E. Word needles never move the epoch.
//! - Bounded-needle semantics match `mentions_identifier` (the test oracle
//!   in `session/queries.rs`): whole identifier, ASCII-case-insensitive.
//!   Raw needles skip the boundary check.
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

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Whether a bounded `needle` is answered by a file's word set.
fn is_word(needle: &str) -> bool {
    !needle.is_empty() && needle.bytes().all(is_word_byte)
}

/// Whether `name` needs the automaton rather than the word set.
fn is_pattern(name: &Name, raw: bool) -> bool {
    raw || !is_word(name.as_str())
}

const FNV_OFFSET: u32 = 0x811c_9dc5;
const FNV_PRIME: u32 = 0x0100_0193;

fn word_key(word: &[u8]) -> u32 {
    word.iter().fold(FNV_OFFSET, |h, &b| {
        (h ^ u32::from(b.to_ascii_lowercase())).wrapping_mul(FNV_PRIME)
    })
}

/// Sorted, deduplicated keys of every maximal `[A-Za-z0-9_]` run in `hay`.
fn words_of(hay: &[u8]) -> Box<[u32]> {
    let mut keys = Vec::new();
    let mut h = FNV_OFFSET;
    let mut in_word = false;
    for &b in hay {
        if is_word_byte(b) {
            h = (h ^ u32::from(b.to_ascii_lowercase())).wrapping_mul(FNV_PRIME);
            in_word = true;
        } else if in_word {
            keys.push(h);
            h = FNV_OFFSET;
            in_word = false;
        }
    }
    if in_word {
        keys.push(h);
    }
    keys.sort_unstable();
    keys.dedup();
    keys.into_boxed_slice()
}

/// One file's complete mention set for one text.
#[derive(Debug, Clone, Default)]
pub struct FileScan {
    /// Pattern needles the text mentions, sorted.
    names: Box<[Name]>,
    /// Sorted [`word_key`]s of the text's identifier runs.
    words: Box<[u32]>,
}

impl FileScan {
    pub fn contains(&self, q: &MentionQuery) -> bool {
        match q.word {
            Some(key) => self.words.binary_search(&key).is_ok(),
            None => self.names.binary_search(&q.name).is_ok(),
        }
    }

    /// [`Self::contains`] for a lowercased bounded needle the scanner was built with.
    pub fn contains_name(&self, name: &Name) -> bool {
        if is_word(name.as_str()) {
            self.words
                .binary_search(&word_key(name.as_str().as_bytes()))
                .is_ok()
        } else {
            self.names.binary_search(name).is_ok()
        }
    }
}

/// Compiled pattern-needle scanner frozen at one epoch. Cheap to share
/// across rayon workers; rebuilt lazily when a pattern needle is added.
pub struct MentionScanner {
    epoch: u64,
    /// `None` when the universe holds no pattern needles.
    ac: Option<aho_corasick::AhoCorasick>,
    /// Pattern id → the (lowercased) name it matches.
    patterns: Vec<Name>,
    /// Pattern id → whether that pattern skips the boundary check (a raw
    /// substring needle, e.g. a call token like `->__construct` that isn't
    /// itself a whole identifier — see [`ClassMentionIndex::add_raw_names`]).
    raw: Vec<bool>,
}

impl MentionScanner {
    /// Word needles in `entries` are dropped: the word set answers them.
    pub(crate) fn build(epoch: u64, entries: Vec<(Name, bool)>) -> Option<Self> {
        let (patterns, raw): (Vec<Name>, Vec<bool>) = entries
            .into_iter()
            .filter(|(n, raw)| is_pattern(n, *raw))
            .unzip();
        let ac = if patterns.is_empty() {
            None
        } else {
            Some(
                aho_corasick::AhoCorasick::builder()
                    .ascii_case_insensitive(true)
                    .build(patterns.iter().map(|n| n.as_str()))
                    .ok()?,
            )
        };
        Some(Self {
            epoch,
            ac,
            patterns,
            raw,
        })
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Everything `hay` mentions. A bounded pattern must appear as a whole
    /// identifier (boundary rule matches the `mentions_identifier` test
    /// oracle); a raw pattern (see [`ClassMentionIndex::add_raw_names`])
    /// matches as a plain substring, no boundary check at all.
    pub fn scan(&self, hay: &str) -> FileScan {
        let bytes = hay.as_bytes();
        let mut hits: Vec<Name> = Vec::new();
        if let Some(ac) = &self.ac {
            let mut seen: FxHashSet<usize> = FxHashSet::default();
            for m in ac.find_overlapping_iter(hay) {
                let pid = m.pattern().as_usize();
                if seen.contains(&pid) {
                    continue;
                }
                let bounded_ok = self.raw[pid]
                    || ((m.start() == 0 || !is_word_byte(bytes[m.start() - 1]))
                        && (m.end() == bytes.len() || !is_word_byte(bytes[m.end()])));
                if bounded_ok {
                    seen.insert(pid);
                    hits.push(self.patterns[pid]);
                }
            }
            hits.sort_unstable();
        }
        FileScan {
            names: hits.into_boxed_slice(),
            words: words_of(bytes),
        }
    }
}

/// One file's recorded mention set.
struct FileMentions {
    /// Text the scan ran against (`Arc` identity; an edit self-invalidates).
    text: Arc<str>,
    /// Scanner epoch of the scan: definitive for pattern needles added at or before it.
    epoch: u64,
    scan: FileScan,
}

/// A prepared coverage key: the needle plus the epoch it entered the
/// universe. Built once per query, checked per file.
#[derive(Clone, Copy)]
pub struct MentionQuery {
    pub name: Name,
    added_epoch: u64,
    /// The needle's [`word_key`] when the word set answers it.
    word: Option<u32>,
}

/// Size/coverage counters for memory validation and host metrics.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClassMentionStats {
    pub universe_names: usize,
    pub files_covered: usize,
    /// Pattern-needle hits recorded across files.
    pub total_mentions: usize,
    /// Word keys recorded across files.
    pub total_words: usize,
    /// Heap bytes of the cached scanner's automaton, if built.
    pub scanner_bytes: usize,
    /// Cumulative scan results recorded (each is one raw-text pass a later
    /// query gets to skip). Flat across a query ⇒ the gate ran lookup-only.
    pub scans_recorded: u64,
}

/// One admitted name's coverage epoch and matching mode.
#[derive(Clone, Copy)]
struct NameEntry {
    /// Epoch this name (or its most recent upgrade to `raw`, see
    /// [`ClassMentionIndex::add_raw_names`]) entered the universe.
    epoch: u64,
    /// Whether this needle matches as a raw substring (no boundary check)
    /// rather than a whole identifier.
    raw: bool,
}

/// Universe state: what names exist, since when, and the cached automaton.
#[derive(Default)]
struct Universe {
    /// Bumped once per batch that adds at least one new pattern needle or
    /// upgrades an existing name to `raw`.
    epoch: u64,
    /// Lowercased needle → its coverage entry. Append-only: a name outliving
    /// its declaration only widens mention sets (the predicate is textual,
    /// so answers stay exact), and a bounded name is only ever upgraded to
    /// raw, never downgraded.
    names: FxHashMap<Name, NameEntry>,
    /// How many of `names` are pattern needles.
    pattern_count: usize,
    /// Cached scanner; valid while its epoch matches `epoch`.
    scanner: Option<Arc<MentionScanner>>,
}

#[derive(Default)]
pub struct ClassMentionIndex {
    universe: Mutex<Universe>,
    /// Sharded per-file entries: the gate loop's per-candidate lookups come
    /// from many rayon workers concurrently and must not serialize.
    by_file: dashmap::DashMap<crate::db::FileNo, FileMentions, FxBuildHasher>,
    scans_recorded: AtomicU64,
}

impl ClassMentionIndex {
    /// Insert lowercased short names, matched as whole identifiers.
    /// Idempotent; bumps the epoch once if any new one is a pattern needle.
    pub fn add_names(&self, names: impl IntoIterator<Item = Name>) {
        self.add_names_impl(names, false);
    }

    /// Insert lowercased needles matched as a raw substring — no word-
    /// boundary check at all — for a needle that isn't itself a whole
    /// identifier (e.g. a call token like `->__construct`, whose preceding
    /// byte in real usage, e.g. `$obj->__construct()`, is itself an
    /// identifier character and would otherwise fail the boundary check
    /// that protects [`Self::add_names`]). Shares the same universe/scanner
    /// and per-file cache. Idempotent; upgrading an existing bounded name to
    /// raw bumps the epoch (forces a rescan) since a same-text entry scanned
    /// before the upgrade may have rejected a match the boundary check would
    /// have caught.
    pub fn add_raw_names(&self, names: impl IntoIterator<Item = Name>) {
        self.add_names_impl(names, true);
    }

    fn add_names_impl(&self, names: impl IntoIterator<Item = Name>, raw: bool) {
        let mut u = self.universe.lock();
        let next = u.epoch + 1;
        let mut added_pattern = false;
        let mut new_patterns = 0;
        for n in names {
            let pattern = is_pattern(&n, raw);
            match u.names.entry(n) {
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(NameEntry { epoch: next, raw });
                    if pattern {
                        added_pattern = true;
                        new_patterns += 1;
                    }
                }
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    if raw && !o.get().raw {
                        let was_pattern = is_pattern(o.key(), false);
                        o.get_mut().raw = true;
                        o.get_mut().epoch = next;
                        added_pattern = true;
                        if !was_pattern {
                            new_patterns += 1;
                        }
                    }
                }
            }
        }
        u.pattern_count += new_patterns;
        if added_pattern {
            u.epoch = next;
        }
    }

    /// The pattern scanner for the current epoch, building it if a pattern
    /// needle was added. The automaton build runs outside the universe lock
    /// (a concurrent builder may waste one build; installs are idempotent).
    /// `None` while the universe is empty.
    pub fn scanner(&self) -> Option<Arc<MentionScanner>> {
        let (epoch, entries) = {
            let u = self.universe.lock();
            if u.names.is_empty() {
                return None;
            }
            if let Some(s) = u.scanner.as_ref().filter(|s| s.epoch == u.epoch) {
                return Some(s.clone());
            }
            (
                u.epoch,
                u.names
                    .iter()
                    .filter(|(n, e)| is_pattern(n, e.raw))
                    .map(|(n, e)| (*n, e.raw))
                    .collect::<Vec<(Name, bool)>>(),
            )
        };
        let scanner = Arc::new(MentionScanner::build(epoch, entries)?);
        let mut u = self.universe.lock();
        if scanner.epoch == u.epoch {
            u.scanner = Some(scanner.clone());
        }
        Some(scanner)
    }

    /// [`Self::scanner`], but keeps the cached automaton while the universe
    /// has grown only slightly since it was built; a rebuild is O(patterns).
    pub fn scanner_reusing_cached(&self) -> Option<Arc<MentionScanner>> {
        {
            let u = self.universe.lock();
            if let Some(s) = u.scanner.as_ref() {
                let grown = u.pattern_count.saturating_sub(s.patterns.len());
                if s.epoch == u.epoch || grown <= (s.patterns.len() / 4).max(512) {
                    return Some(s.clone());
                }
            }
        }
        self.scanner()
    }

    /// Resolve `needle` (any case, short name) against the universe.
    pub fn prepare_query(&self, needle: &str) -> Option<MentionQuery> {
        let name = Name::new(needle).ascii_lowercase();
        let entry = *self.universe.lock().names.get(&name)?;
        let word = (!is_pattern(&name, entry.raw)).then(|| word_key(name.as_str().as_bytes()));
        Some(MentionQuery {
            name,
            added_epoch: entry.epoch,
            word,
        })
    }

    /// Whether `file`'s current text mentions the queried name. `None` when
    /// the entry can't answer (missing, text changed, or a pattern needle
    /// scanned before it entered the universe) — the caller must fall back
    /// to a raw scan.
    pub fn answer(
        &self,
        file: crate::db::FileNo,
        q: &MentionQuery,
        current_text: &Arc<str>,
    ) -> Option<bool> {
        let e = self.by_file.get(&file)?;
        if !Arc::ptr_eq(&e.text, current_text) || (q.word.is_none() && e.epoch < q.added_epoch) {
            return None;
        }
        Some(e.scan.contains(q))
    }

    /// Whether `file` already holds a scan of exactly `text` at `epoch` or
    /// newer (used by analyze sweeps to skip redundant rescans).
    pub fn is_current(&self, file: crate::db::FileNo, text: &Arc<str>, epoch: u64) -> bool {
        self.by_file
            .get(&file)
            .is_some_and(|e| Arc::ptr_eq(&e.text, text) && e.epoch >= epoch)
    }

    /// Record a scan result. Never downgrades a same-text entry from a newer
    /// epoch.
    pub fn set_file(&self, file: crate::db::FileNo, text: Arc<str>, epoch: u64, scan: FileScan) {
        match self.by_file.entry(file) {
            dashmap::mapref::entry::Entry::Occupied(mut o) => {
                let prev = o.get();
                if Arc::ptr_eq(&prev.text, &text) && prev.epoch >= epoch {
                    return;
                }
                o.insert(FileMentions { text, epoch, scan });
            }
            dashmap::mapref::entry::Entry::Vacant(v) => {
                v.insert(FileMentions { text, epoch, scan });
            }
        }
        self.scans_recorded.fetch_add(1, Ordering::Relaxed);
    }

    pub fn clear_file(&self, file: crate::db::FileNo) {
        self.by_file.remove(&file);
    }

    pub fn stats(&self) -> ClassMentionStats {
        let u = self.universe.lock();
        ClassMentionStats {
            universe_names: u.names.len(),
            files_covered: self.by_file.len(),
            total_mentions: self.by_file.iter().map(|e| e.scan.names.len()).sum(),
            total_words: self.by_file.iter().map(|e| e.scan.words.len()).sum(),
            scanner_bytes: u
                .scanner
                .as_ref()
                .and_then(|s| s.ac.as_ref())
                .map(|ac| ac.memory_usage())
                .unwrap_or(0),
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
        let upper = s.scan("new COLOR();");
        assert!(upper.contains_name(&lc("Color")));
        assert!(!upper.contains_name(&lc("Save")));
        assert!(!s.scan("$this->saveAll();").contains_name(&lc("Save")));
        let both = s.scan("Color::save()");
        assert!(both.contains_name(&lc("Color")) && both.contains_name(&lc("Save")));
    }

    /// A raw needle (`add_raw_names`) matches as a plain substring even
    /// when the byte immediately preceding it is an identifier character —
    /// exactly the shape a call token like `->__construct` needs
    /// (`$obj->__construct()`), and exactly the case the boundary check
    /// would reject for a bounded needle.
    #[test]
    fn raw_needle_matches_without_a_boundary() {
        let idx = ClassMentionIndex::default();
        idx.add_raw_names([lc("->__construct")]);
        let s = scanner_for(&idx);
        let q = idx.prepare_query("->__construct").unwrap();
        assert!(s.scan("$obj->__construct();").contains(&q));
    }

    /// A bounded needle sharing a scanner with a raw one still enforces its
    /// own boundary check — raw-ness is per-pattern, not global once any
    /// raw needle exists in the universe.
    #[test]
    fn raw_and_bounded_needles_coexist_in_one_scanner() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Color"), lc("Éclair")]);
        idx.add_raw_names([lc("->__construct")]);
        let s = scanner_for(&idx);
        let raw = idx.prepare_query("->__construct").unwrap();
        let hits = s.scan("$obj->__construct(); // recolor() xÉclair");
        assert!(hits.contains(&raw));
        assert!(!hits.contains_name(&lc("Color")));
        assert!(!hits.contains_name(&lc("Éclair")));
        let plain = s.scan("new Color(); new Éclair();");
        assert!(plain.contains_name(&lc("Color")) && plain.contains_name(&lc("Éclair")));
        assert!(!plain.contains(&raw));
    }

    /// Upgrading an existing bounded name to raw bumps the epoch, forcing a
    /// rescan of any file cached before the upgrade — otherwise a stale
    /// entry could answer "no match" for a real raw-only occurrence.
    #[test]
    fn upgrading_a_name_to_raw_invalidates_prior_scans() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("token")]);
        let s1 = scanner_for(&idx);
        let text: Arc<str> = Arc::from("xtoken");
        let file: crate::db::FileNo = 0;
        idx.set_file(file, text.clone(), s1.epoch(), s1.scan(&text));
        let q = idx.prepare_query("token").unwrap();
        assert_eq!(idx.answer(file, &q, &text), Some(false));

        idx.add_raw_names([lc("token")]);
        let q_after = idx.prepare_query("token").unwrap();
        assert_eq!(
            idx.answer(file, &q_after, &text),
            None,
            "the pre-upgrade scan must no longer answer for this needle"
        );
        let s2 = scanner_for(&idx);
        assert!(
            s2.scan(&text).contains(&q_after),
            "once raw, the same text now matches with no boundary"
        );
    }

    #[test]
    fn word_needles_added_after_the_scan_answer_without_rescan() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Alpha")]);
        let s1 = scanner_for(&idx);
        let text: Arc<str> = Arc::from("uses Alpha and BETA, not gammaray");
        let file: crate::db::FileNo = 0;
        idx.set_file(file, text.clone(), s1.epoch(), s1.scan(&text));

        idx.add_names([lc("Beta"), lc("Gamma")]);
        assert_eq!(
            scanner_for(&idx).epoch(),
            s1.epoch(),
            "word needles keep the epoch"
        );
        let q_beta = idx.prepare_query("Beta").unwrap();
        let q_gamma = idx.prepare_query("Gamma").unwrap();
        assert_eq!(idx.answer(file, &q_beta, &text), Some(true));
        assert_eq!(idx.answer(file, &q_gamma, &text), Some(false));
    }

    #[test]
    fn epoch_gates_pattern_needles_added_after_the_scan() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Àlpha")]);
        let s1 = scanner_for(&idx);
        let text: Arc<str> = Arc::from("uses Àlpha and Ébeta");
        let file: crate::db::FileNo = 0;
        idx.set_file(file, text.clone(), s1.epoch(), s1.scan(&text));

        let q_alpha = idx.prepare_query("Àlpha").unwrap();
        assert_eq!(idx.answer(file, &q_alpha, &text), Some(true));

        idx.add_names([lc("Ébeta")]);
        let q_beta = idx.prepare_query("Ébeta").unwrap();
        assert_eq!(idx.answer(file, &q_beta, &text), None);
        assert_eq!(idx.answer(file, &q_alpha, &text), Some(true));

        let s2 = scanner_for(&idx);
        idx.set_file(file, text.clone(), s2.epoch(), s2.scan(&text));
        assert_eq!(idx.answer(file, &q_beta, &text), Some(true));
    }

    #[test]
    fn text_identity_gates_answers() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Job")]);
        let s = scanner_for(&idx);
        let text: Arc<str> = Arc::from("new Job();");
        let file: crate::db::FileNo = 0;
        idx.set_file(file, text.clone(), s.epoch(), s.scan(&text));
        let q = idx.prepare_query("Job").unwrap();
        assert_eq!(idx.answer(file, &q, &text), Some(true));
        // Content-equal but different Arc: not answerable (must rescan).
        let other: Arc<str> = Arc::from("new Job();");
        assert_eq!(idx.answer(file, &q, &other), None);
    }

    #[test]
    fn small_universe_growth_reuses_the_cached_scanner() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Àa")]);
        let first = idx.scanner().unwrap();
        idx.add_names([lc("Àb")]);
        let reused = idx.scanner_reusing_cached().unwrap();
        assert!(Arc::ptr_eq(&first, &reused));
        assert_ne!(idx.scanner().unwrap().epoch(), first.epoch());
    }

    #[test]
    fn large_universe_growth_rebuilds_the_scanner() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Àa")]);
        let first = idx.scanner().unwrap();
        idx.add_names((0..2000).map(|i| lc(&format!("À{i}"))));
        let rebuilt = idx.scanner_reusing_cached().unwrap();
        assert!(!Arc::ptr_eq(&first, &rebuilt));
        assert_eq!(rebuilt.epoch(), idx.scanner().unwrap().epoch());
    }

    #[test]
    fn name_added_after_a_reused_scan_falls_back_to_a_raw_scan() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Àa")]);
        idx.scanner().unwrap();
        idx.add_names([lc("Àb")]);
        let old = idx.scanner_reusing_cached().unwrap();
        let text: Arc<str> = Arc::from("new Àa(); new Àb();");
        let file: crate::db::FileNo = 0;
        idx.set_file(file, text.clone(), old.epoch(), old.scan(&text));
        let q = idx.prepare_query("Àb").unwrap();
        assert_eq!(idx.answer(file, &q, &text), None);
        let qa = idx.prepare_query("Àa").unwrap();
        assert_eq!(idx.answer(file, &qa, &text), Some(true));
    }

    #[test]
    fn only_new_pattern_needles_bump_epoch() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("A"), lc("Àb")]);
        let e1 = scanner_for(&idx).epoch();
        idx.add_names([lc("A"), lc("Àb")]);
        assert_eq!(
            scanner_for(&idx).epoch(),
            e1,
            "no new names → no epoch bump"
        );
        idx.add_names([lc("C")]);
        assert_eq!(
            scanner_for(&idx).epoch(),
            e1,
            "a word needle → no epoch bump"
        );
        idx.add_names([lc("Àd")]);
        assert_eq!(scanner_for(&idx).epoch(), e1 + 1);
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
        let file: crate::db::FileNo = 0;
        idx.set_file(file, text.clone(), 5, scanner_for(&idx).scan(&text));
        // A stale (older-epoch) commit racing in must not downgrade coverage.
        idx.set_file(file, text.clone(), 3, FileScan::default());
        let q = idx.prepare_query("A").unwrap();
        assert_eq!(idx.answer(file, &q, &text), Some(true));
    }

    #[test]
    fn scanner_is_cached_per_epoch() {
        let idx = ClassMentionIndex::default();
        idx.add_names([lc("Àa")]);
        let s1 = idx.scanner().unwrap();
        let s2 = idx.scanner().unwrap();
        assert!(Arc::ptr_eq(&s1, &s2), "same epoch → cached automaton");
        idx.add_names([lc("Àb")]);
        let s3 = idx.scanner().unwrap();
        assert!(
            !Arc::ptr_eq(&s1, &s3),
            "pattern universe grew → rebuilt automaton"
        );
    }

    #[test]
    fn words_split_on_every_non_identifier_byte() {
        let scan = MentionScanner::build(0, Vec::new())
            .unwrap()
            .scan("App\\Color::save('x'); /** @var Éclair */ $a_b1");
        for w in ["app", "color", "save", "x", "var", "clair", "a_b1"] {
            assert!(scan.contains_name(&lc(w)), "{w}");
        }
        for w in ["app\\color", "a", "b1", "colo"] {
            assert!(!scan.contains_name(&lc(w)), "{w}");
        }
    }
}
