//! Persistent definition cache: serialized [`StubSlice`] per source file, keyed
//! by file path with the file's content hash as the validity field.
//!
//! On a cache hit, `collect_definitions` and the definition collection inside
//! `analyze()` deserialize the stored slice and skip the much more expensive
//! parse + definition-collection work.
//!
//! Format choice (bincode 1.x): postcard was the original pick but it pulls
//! `heapless` -> the unmaintained `atomic-polyfill` (RUSTSEC-2023-0089),
//! which `cargo-deny` rejects. bincode v2 replaced it but was itself flagged
//! as unmaintained (RUSTSEC-2025-0141). bincode 1.3.3 is explicitly called
//! "complete" by the bincode team, carries no advisory, and uses the same
//! transparent serde compatibility.
//!
//! Layout: `<cache_dir>/stubs/<seq>-<writer>.pack`: a fixed header, the entry
//! payloads, an index and a footer. Entries are appended to a temp file that
//! is renamed into place once finished, so a crash never leaves a partial
//! pack, and a cold run creates a few files rather than one per source file.
//! Opening the cache reads only the indexes; a hit is one positioned read
//! from an already open pack. A payload is a bincode
//! [`StubSlice`] followed by a bincode `Vec<Issue>` (the collector-phase
//! issues found alongside the slice). Later packs supersede earlier entries
//! for the same path, and [`StubSliceCache::flush`] merges packs once there are
//! too many or they are mostly superseded entries.
//!
//! A pack whose header doesn't match this build is ignored and deleted on the
//! next flush, and every entry is still checked against the caller's content
//! hash and PHP version, so a stale entry is only ever a miss.

use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use mir_codebase::definitions::{deduplicate_params_in_slice, StubSlice};
use mir_issues::Issue;
use parking_lot::{Mutex, RwLock};
use rustc_hash::{FxHashMap, FxHashSet};

/// Magic bytes at the start and end of every pack. "MIRP" little-endian.
const MAGIC: u32 = 0x5052_494D;
/// Bumped when the pack layout changes OR a serialized `StubSlice`
/// struct or definition-collection behavior changes (e.g.
/// `inferred_return_type: Option<Type>` → `Option<Arc<Type>>`), so stale
/// entries are rejected.
const FORMAT_VERSION: u32 = 52;

/// magic: u32, format version: u32, mir version hash: u64.
const HEADER_LEN: u64 = 16;
/// index offset: u64, entry count: u32, magic: u32.
const FOOTER_LEN: u64 = 16;
/// path key, content hash, PHP version, payload offset: u64, payload length: u32.
const INDEX_ENTRY_LEN: usize = 32 + 32 + 1 + 8 + 4;

/// Pack size at which a session's pack is finished and a new one started, so
/// a long session's entries reach disk before it exits.
const PACK_BYTES: u64 = 8 << 20;
/// More packs than this are merged on flush.
const MAX_PACKS: usize = 8;
/// Superseded bytes are reclaimed once they exceed both the live bytes and this.
const MIN_RECLAIM_BYTES: u64 = 4 << 20;

fn mir_version_hash() -> u64 {
    use std::sync::OnceLock;
    static HASH: OnceLock<u64> = OnceLock::new();
    *HASH.get_or_init(|| {
        let mut hasher = blake3::Hasher::new();
        hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
        hasher.update(&[0]);
        hasher.update(env!("MIR_STUB_LOGIC_HASH").as_bytes());
        let digest = hasher.finalize();
        let bytes = digest.as_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    })
}

fn path_key(path: &str) -> [u8; 32] {
    *blake3::hash(path.as_bytes()).as_bytes()
}

/// An open pack file. Readers keep it alive through their [`Slot`]s, so a
/// pack deleted by a merge stays readable until its last reader is done.
struct Pack {
    file: File,
    path: PathBuf,
    payload_bytes: u64,
}

#[derive(Clone)]
struct Slot {
    pack: Arc<Pack>,
    offset: u64,
    len: u32,
    content_hash: [u8; 32],
    php_version: u8,
}

struct IndexEntry {
    path_key: [u8; 32],
    content_hash: [u8; 32],
    php_version: u8,
    offset: u64,
    len: u32,
}

/// A pack being written to its temp file; payloads go straight to disk and
/// only the index is kept in memory.
struct PackWriter {
    file: File,
    tmp: PathBuf,
    path: PathBuf,
    end: u64,
    entries: Vec<IndexEntry>,
}

impl PackWriter {
    fn append(&mut self, entry: IndexEntry, payload: &[u8]) -> io::Result<()> {
        self.file.write_all(payload)?;
        self.entries.push(IndexEntry {
            offset: self.end,
            ..entry
        });
        self.end += payload.len() as u64;
        Ok(())
    }

    /// Write the index and footer and rename the pack into place.
    fn finish(mut self) -> Option<(Arc<Pack>, Vec<IndexEntry>)> {
        let mut tail = Vec::with_capacity(self.entries.len() * INDEX_ENTRY_LEN + 16);
        for e in &self.entries {
            tail.extend_from_slice(&e.path_key);
            tail.extend_from_slice(&e.content_hash);
            tail.push(e.php_version);
            tail.extend_from_slice(&e.offset.to_le_bytes());
            tail.extend_from_slice(&e.len.to_le_bytes());
        }
        tail.extend_from_slice(&self.end.to_le_bytes());
        tail.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        tail.extend_from_slice(&MAGIC.to_le_bytes());
        let renamed = self
            .file
            .write_all(&tail)
            .and_then(|()| std::fs::rename(&self.tmp, &self.path));
        if renamed.is_err() {
            self.abandon();
            return None;
        }
        let pack = Pack {
            file: File::open(&self.path).ok()?,
            path: self.path,
            payload_bytes: self.end - HEADER_LEN,
        };
        Some((Arc::new(pack), self.entries))
    }

    fn abandon(self) {
        drop(self.file);
        let _ = std::fs::remove_file(&self.tmp);
    }
}

/// Persistent definition cache. Thread-safe. Entries are appended to this
/// session's pack as they arrive; the pack becomes visible to later sessions
/// when it is finished — on reaching the pack size limit, or by [`Self::flush`] /
/// `Drop`, so a clean shutdown loses nothing. Reads-after-write in the same
/// session are served by Salsa's memos, not this cache.
pub struct StubSliceCache {
    root: PathBuf,
    hits: AtomicU64,
    misses: AtomicU64,
    enabled: bool,
    index: RwLock<FxHashMap<[u8; 32], Slot>>,
    /// Packs in `index` order, oldest first.
    packs: Mutex<Vec<Arc<Pack>>>,
    /// Packs from another build, corrupt packs and the old per-entry layout's
    /// shard directories, removed on the next flush.
    stale: Mutex<Vec<PathBuf>>,
    next_seq: AtomicU64,
    writer: Mutex<Option<PackWriter>>,
}

impl StubSliceCache {
    /// Open (or create) the cache under `<cache_dir>/stubs/`. If the
    /// directory can't be created, the cache silently disables itself so
    /// analysis still works on a read-only filesystem.
    pub fn open(cache_dir: &Path) -> Self {
        let root = cache_dir.join("stubs");
        let enabled = std::fs::create_dir_all(&root).is_ok();
        let mut found: Vec<(u64, String, Arc<Pack>, Vec<IndexEntry>)> = Vec::new();
        let mut stale = Vec::new();
        if let Some(dir) = enabled.then(|| std::fs::read_dir(&root).ok()).flatten() {
            for dir_entry in dir.flatten() {
                let path = dir_entry.path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned)
                else {
                    continue;
                };
                let is_dir = dir_entry.file_type().is_ok_and(|t| t.is_dir());
                if is_dir && name.len() == 2 && name.bytes().all(|b| b.is_ascii_hexdigit()) {
                    stale.push(path);
                } else if let Some(seq) = pack_seq(&name) {
                    match load_pack(&path) {
                        Some((pack, entries)) => found.push((seq, name, pack, entries)),
                        None => stale.push(path),
                    }
                }
            }
        }
        found.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
        let next_seq = found.last().map_or(0, |(seq, ..)| seq + 1);
        let mut index = FxHashMap::default();
        let mut packs = Vec::with_capacity(found.len());
        for (_, _, pack, entries) in found {
            register(&mut index, &pack, &entries);
            packs.push(pack);
        }
        Self {
            root,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            enabled,
            index: RwLock::new(index),
            packs: Mutex::new(packs),
            stale: Mutex::new(stale),
            next_seq: AtomicU64::new(next_seq),
            writer: Mutex::new(None),
        }
    }

    /// Return the cached [`StubSlice`] and its issues for `path` if its
    /// stored entry matches `(content_hash, php_version)` and the current
    /// mir version.
    ///
    /// On hit the deserialized slice has its `file` field restored from the
    /// caller-supplied `path` (we don't trust paths from disk), same for
    /// every issue's `location.file`, and the slice's `is_deduped` flag is
    /// preserved as `false`; callers running in parallel should re-run
    /// [`deduplicate_params_in_slice`] before ingest so the serial commit
    /// doesn't pay dedup costs.
    pub fn get(
        &self,
        path: &str,
        content_hash: &[u8; 32],
        php_version: u8,
    ) -> Option<(StubSlice, Vec<Issue>)> {
        if !self.enabled {
            return None;
        }
        let slot = self.index.read().get(&path_key(path)).cloned()?;
        if &slot.content_hash != content_hash || slot.php_version != php_version {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        let mut bytes = vec![0; slot.len as usize];
        let decoded = read_at(&slot.pack.file, &mut bytes, slot.offset)
            .ok()
            .and_then(|()| decode_payload(&bytes));
        let Some((mut slice, mut issues)) = decoded else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return None;
        };
        let path_arc: Arc<str> = Arc::from(path);
        slice.file = Some(path_arc.clone());
        for issue in &mut issues {
            issue.location.file = path_arc.clone();
        }
        self.hits.fetch_add(1, Ordering::Relaxed);
        Some((slice, issues))
    }

    /// Append `slice` and `issues` to this session's pack. Errors (disk full,
    /// permission denied) are swallowed — the cache is an optimization,
    /// never a correctness dependency.
    pub fn put(
        &self,
        path: &str,
        content_hash: &[u8; 32],
        php_version: u8,
        slice: &Arc<StubSlice>,
        issues: &Arc<Vec<Issue>>,
    ) {
        if !self.enabled {
            return;
        }
        let Some(payload) = encode_payload(slice, issues) else {
            return;
        };
        let Ok(len) = u32::try_from(payload.len()) else {
            return;
        };
        let entry = IndexEntry {
            path_key: path_key(path),
            content_hash: *content_hash,
            php_version,
            offset: 0,
            len,
        };
        let mut slot = self.writer.lock();
        if slot.is_none() {
            *slot = self.start_pack();
        }
        let Some(writer) = slot.as_mut() else {
            return;
        };
        if writer.append(entry, &payload).is_err() {
            if let Some(writer) = slot.take() {
                writer.abandon();
            }
            return;
        }
        if writer.end >= PACK_BYTES {
            let full = slot.take();
            drop(slot);
            if let Some(full) = full {
                self.finish_and_register(full);
            }
        }
    }

    /// Finish this session's pack, then merge packs if there are too many.
    /// Called by `Drop`, so a clean session shutdown never loses entries.
    pub fn flush(&self) {
        if !self.enabled {
            return;
        }
        let writer = self.writer.lock().take();
        if let Some(writer) = writer {
            self.finish_and_register(writer);
        }
        self.compact();
    }

    /// Cumulative hit count across this cache instance.
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Cumulative miss count across this cache instance.
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    fn start_pack(&self) -> Option<PackWriter> {
        let seq = self.next_seq.fetch_add(1, Ordering::Relaxed);
        let name = format!("{seq:016x}-{:08x}.pack", crate::tmp_suffix::next());
        let path = self.root.join(&name);
        let tmp = self.root.join(format!("{name}.tmp"));
        let mut file = File::create(&tmp).ok()?;
        if file.write_all(&header_bytes()).is_err() {
            drop(file);
            let _ = std::fs::remove_file(&tmp);
            return None;
        }
        Some(PackWriter {
            file,
            tmp,
            path,
            end: HEADER_LEN,
            entries: Vec::new(),
        })
    }

    fn finish_and_register(&self, writer: PackWriter) {
        let Some((pack, entries)) = writer.finish() else {
            return;
        };
        let mut packs = self.packs.lock();
        register(&mut self.index.write(), &pack, &entries);
        packs.push(pack);
    }

    /// Delete stale files, then merge every pack into one if there are too
    /// many or most of their bytes are superseded entries. Merging copies
    /// payload bytes without decoding them.
    fn compact(&self) {
        for path in std::mem::take(&mut *self.stale.lock()) {
            let _ = if path.is_dir() {
                std::fs::remove_dir_all(&path)
            } else {
                std::fs::remove_file(&path)
            };
        }
        let mut packs = self.packs.lock();
        let total: u64 = packs.iter().map(|p| p.payload_bytes).sum();
        let index = self.index.read();
        let live_bytes: u64 = index.values().map(|s| u64::from(s.len)).sum();
        let superseded = total.saturating_sub(live_bytes);
        let too_many = packs.len() > MAX_PACKS;
        if !too_many && (superseded <= live_bytes || superseded < MIN_RECLAIM_BYTES) {
            return;
        }
        let mut live: Vec<([u8; 32], Slot)> = index
            .iter()
            .map(|(key, slot)| (*key, slot.clone()))
            .collect();
        drop(index);
        // Group reads by pack and offset so the copy reads each pack in order.
        live.sort_by(|(_, a), (_, b)| {
            (Arc::as_ptr(&a.pack), a.offset).cmp(&(Arc::as_ptr(&b.pack), b.offset))
        });
        let Some(mut writer) = self.start_pack() else {
            return;
        };
        let mut buf = Vec::new();
        let mut copied = Vec::with_capacity(live.len());
        for (key, slot) in live {
            buf.resize(slot.len as usize, 0);
            if read_at(&slot.pack.file, &mut buf, slot.offset).is_err() {
                continue;
            }
            let entry = IndexEntry {
                path_key: key,
                content_hash: slot.content_hash,
                php_version: slot.php_version,
                offset: 0,
                len: slot.len,
            };
            if writer.append(entry, &buf).is_err() {
                writer.abandon();
                return;
            }
            copied.push(slot);
        }
        let Some((merged, entries)) = writer.finish() else {
            return;
        };
        {
            let mut index = self.index.write();
            for (entry, from) in entries.iter().zip(&copied) {
                // A slot replaced since the snapshot holds newer data.
                let unchanged = index.get(&entry.path_key).is_some_and(|cur| {
                    Arc::ptr_eq(&cur.pack, &from.pack) && cur.offset == from.offset
                });
                if unchanged {
                    index.insert(entry.path_key, slot_for(&merged, entry));
                }
            }
        }
        let still_referenced: FxHashSet<*const Pack> = self
            .index
            .read()
            .values()
            .map(|s| Arc::as_ptr(&s.pack))
            .collect();
        packs.retain(|pack| {
            let keep = still_referenced.contains(&Arc::as_ptr(pack));
            if !keep {
                let _ = std::fs::remove_file(&pack.path);
            }
            keep
        });
        packs.push(merged);
    }
}

impl Drop for StubSliceCache {
    fn drop(&mut self) {
        self.flush();
    }
}

fn header_bytes() -> [u8; HEADER_LEN as usize] {
    let mut header = [0; HEADER_LEN as usize];
    header[..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
    header[8..].copy_from_slice(&mir_version_hash().to_le_bytes());
    header
}

/// Sequence number from a pack file name, `None` for any other file.
fn pack_seq(name: &str) -> Option<u64> {
    let stem = name.strip_suffix(".pack")?;
    u64::from_str_radix(stem.get(..16)?, 16).ok()
}

/// Open a pack and read its index; `None` if it is from another build or
/// malformed.
fn load_pack(path: &Path) -> Option<(Arc<Pack>, Vec<IndexEntry>)> {
    let file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len < HEADER_LEN + FOOTER_LEN {
        return None;
    }
    let mut header = [0; HEADER_LEN as usize];
    read_at(&file, &mut header, 0).ok()?;
    if header != header_bytes() {
        return None;
    }
    let mut footer = [0; FOOTER_LEN as usize];
    read_at(&file, &mut footer, len - FOOTER_LEN).ok()?;
    let index_offset = u64::from_le_bytes(footer[..8].try_into().ok()?);
    let count = u32::from_le_bytes(footer[8..12].try_into().ok()?) as u64;
    let magic = u32::from_le_bytes(footer[12..].try_into().ok()?);
    let index_len = count.checked_mul(INDEX_ENTRY_LEN as u64)?;
    if magic != MAGIC
        || index_offset < HEADER_LEN
        || index_offset
            .checked_add(index_len)?
            .checked_add(FOOTER_LEN)?
            != len
    {
        return None;
    }
    let mut raw = vec![0; index_len as usize];
    read_at(&file, &mut raw, index_offset).ok()?;
    let mut entries = Vec::with_capacity(count as usize);
    for chunk in raw.as_chunks::<INDEX_ENTRY_LEN>().0 {
        let entry = IndexEntry {
            path_key: chunk[..32].try_into().ok()?,
            content_hash: chunk[32..64].try_into().ok()?,
            php_version: chunk[64],
            offset: u64::from_le_bytes(chunk[65..73].try_into().ok()?),
            len: u32::from_le_bytes(chunk[73..77].try_into().ok()?),
        };
        if entry.offset < HEADER_LEN || entry.offset + u64::from(entry.len) > index_offset {
            return None;
        }
        entries.push(entry);
    }
    let pack = Pack {
        file,
        path: path.to_path_buf(),
        payload_bytes: index_offset - HEADER_LEN,
    };
    Some((Arc::new(pack), entries))
}

fn slot_for(pack: &Arc<Pack>, entry: &IndexEntry) -> Slot {
    Slot {
        pack: Arc::clone(pack),
        offset: entry.offset,
        len: entry.len,
        content_hash: entry.content_hash,
        php_version: entry.php_version,
    }
}

fn register(index: &mut FxHashMap<[u8; 32], Slot>, pack: &Arc<Pack>, entries: &[IndexEntry]) {
    for entry in entries {
        index.insert(entry.path_key, slot_for(pack, entry));
    }
}

fn encode_payload(slice: &StubSlice, issues: &[Issue]) -> Option<Vec<u8>> {
    // Strip path-bearing field; the loader re-applies it.
    let mut slice_for_disk = slice.clone();
    slice_for_disk.file = None;
    let mut payload = Vec::new();
    bincode::serialize_into(&mut payload, &slice_for_disk).ok()?;
    // Issues keep their `location.file`; `get` overwrites it from the caller's
    // path, same "don't trust disk" policy as the slice.
    bincode::serialize_into(&mut payload, issues).ok()?;
    Some(payload)
}

fn decode_payload(bytes: &[u8]) -> Option<(StubSlice, Vec<Issue>)> {
    // Bounding every read to the payload's own size keeps a desynced
    // length prefix from allocating gigabytes before failing. `config()`
    // (not `options()`) matches the fixint encoding `serialize_into` writes;
    // `options()` defaults to varint and silently misreads every field.
    #[allow(deprecated)]
    let cfg = bincode::config().limit(bytes.len() as u64).clone();
    let mut cursor = io::Cursor::new(bytes);
    let slice: StubSlice = cfg.deserialize_from(&mut cursor).ok()?;
    let issues: Vec<Issue> = cfg.deserialize_from(&mut cursor).ok()?;
    Some((slice, issues))
}

fn read_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::FileExt::read_exact_at(file, buf, offset)
    }
    #[cfg(windows)]
    {
        let mut done = 0;
        while done < buf.len() {
            let n = std::os::windows::fs::FileExt::seek_read(
                file,
                &mut buf[done..],
                offset + done as u64,
            )?;
            if n == 0 {
                return Err(io::ErrorKind::UnexpectedEof.into());
            }
            done += n;
        }
        Ok(())
    }
    #[cfg(not(any(unix, windows)))]
    {
        use std::io::{Read, Seek, SeekFrom};
        let mut file = file;
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(buf)
    }
}

/// Convenience: hash a source string into the 32-byte digest the cache
/// expects. Centralises the hash choice (BLAKE3) so callers don't pick
/// inconsistent functions.
pub fn hash_source(source: &str) -> [u8; 32] {
    *blake3::hash(source.as_bytes()).as_bytes()
}

/// Convert a slice produced by [`StubSliceCache::get`] into one that's safe
/// to consume immediately without paying dedup cost inside the serial
/// commit. Call from a parallel worker.
pub fn prepare_for_ingest(slice: &mut StubSlice) {
    if !slice.is_deduped {
        deduplicate_params_in_slice(slice);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mir_codebase::definitions::StubSlice;
    use tempfile::TempDir;

    fn make_cache() -> (TempDir, StubSliceCache) {
        let dir = TempDir::new().unwrap();
        let cache = StubSliceCache::open(dir.path());
        (dir, cache)
    }

    fn no_issues() -> Arc<Vec<Issue>> {
        Arc::new(Vec::new())
    }

    fn put_empty(cache: &StubSliceCache, path: &str, hash: &[u8; 32]) {
        cache.put(path, hash, 8, &Arc::new(StubSlice::default()), &no_issues());
    }

    fn packs(dir: &TempDir) -> Vec<PathBuf> {
        let mut packs: Vec<PathBuf> = std::fs::read_dir(dir.path().join("stubs"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "pack"))
            .collect();
        packs.sort();
        packs
    }

    fn sample_issue(file: &str) -> Issue {
        Issue::new(
            mir_issues::IssueKind::UndefinedVariable {
                name: "x".to_string(),
            },
            mir_types::Location {
                file: Arc::from(file),
                line: 1,
                line_end: 1,
                col_start: 0,
                col_end: 1,
            },
        )
    }

    #[test]
    fn roundtrip_returns_equivalent_slice() {
        let (dir, cache) = make_cache();
        let hash = hash_source("<?php class A {}");
        put_empty(&cache, "/x/a.php", &hash);
        cache.flush();

        let (got, issues) = cache.get("/x/a.php", &hash, 8).expect("hit");
        assert_eq!(got.file.as_deref(), Some("/x/a.php"));
        assert!(issues.is_empty());
        assert_eq!(cache.hits(), 1);

        let reopened = StubSliceCache::open(dir.path());
        assert!(reopened.get("/x/a.php", &hash, 8).is_some());
    }

    #[test]
    fn roundtrip_keeps_namespace_blocks() {
        let (dir, cache) = make_cache();
        let src = "<?php\nnamespace A { use X\\Y; }\nnamespace { class C {} }\n";
        let hash = hash_source(src);
        let parsed = php_rs_parser::parse(src);
        let collector = crate::collector::DefinitionCollector::new_for_slice(
            Arc::from("/x/a.php"),
            src,
            &parsed.source_map,
        );
        let (slice, _) = collector.collect_slice(&parsed.program);
        assert_eq!(slice.namespace_blocks.len(), 2);
        let slice = Arc::new(slice);
        cache.put("/x/a.php", &hash, 8, &slice, &no_issues());
        drop(cache);

        let (got, _) = StubSliceCache::open(dir.path())
            .get("/x/a.php", &hash, 8)
            .expect("hit");
        assert_eq!(got.namespace_blocks, slice.namespace_blocks);
    }

    #[test]
    fn roundtrip_returns_equivalent_issues_with_path_patched() {
        // Entries are keyed by path, so even if a serialized issue's
        // `location.file` were something else, `get` must overwrite it with
        // the caller-supplied path, same as the slice's own `file` field.
        let (_dir, cache) = make_cache();
        let hash = hash_source("<?php $x;");
        let issues = Arc::new(vec![sample_issue("/different/path.php")]);
        cache.put(
            "/x/a.php",
            &hash,
            8,
            &Arc::new(StubSlice::default()),
            &issues,
        );
        cache.flush();

        let (_, got_issues) = cache.get("/x/a.php", &hash, 8).expect("hit");
        assert_eq!(got_issues.len(), 1);
        assert_eq!(got_issues[0].location.file.as_ref(), "/x/a.php");
    }

    #[test]
    fn miss_on_content_hash_mismatch() {
        let (_dir, cache) = make_cache();
        put_empty(&cache, "/x/a.php", &hash_source("a"));
        cache.flush();

        assert!(cache.get("/x/a.php", &hash_source("b"), 8).is_none());
    }

    #[test]
    fn miss_on_php_version_mismatch() {
        let (_dir, cache) = make_cache();
        let hash = hash_source("a");
        put_empty(&cache, "/x/a.php", &hash);
        cache.flush();

        assert!(cache.get("/x/a.php", &hash, 7).is_none());
    }

    #[test]
    fn miss_on_unknown_path_does_not_error() {
        let (_dir, cache) = make_cache();
        assert!(cache.get("/no/such/file.php", &[0u8; 32], 8).is_none());
    }

    #[test]
    fn restores_file_field_from_path_not_disk() {
        let (_dir, cache) = make_cache();
        let hash = hash_source("a");
        let slice = StubSlice {
            file: Some(Arc::from("/different/path.php")),
            ..Default::default()
        };
        cache.put("/x/a.php", &hash, 8, &Arc::new(slice), &no_issues());
        cache.flush();

        let (got, _) = cache.get("/x/a.php", &hash, 8).unwrap();
        assert_eq!(got.file.as_deref(), Some("/x/a.php"));
    }

    #[test]
    fn later_entry_for_a_path_supersedes_earlier_across_packs() {
        let (dir, cache) = make_cache();
        put_empty(&cache, "/x/a.php", &hash_source("v1"));
        cache.flush();
        put_empty(&cache, "/x/a.php", &hash_source("v2"));
        cache.flush();
        assert_eq!(packs(&dir).len(), 2);

        let reopened = StubSliceCache::open(dir.path());
        assert!(reopened.get("/x/a.php", &hash_source("v2"), 8).is_some());
        assert!(reopened.get("/x/a.php", &hash_source("v1"), 8).is_none());
    }

    #[test]
    fn too_many_packs_are_merged_keeping_the_newest_entries() {
        let (dir, cache) = make_cache();
        for i in 0..=MAX_PACKS {
            put_empty(&cache, &format!("/x/{i}.php"), &hash_source(&i.to_string()));
            put_empty(&cache, "/x/shared.php", &hash_source(&format!("v{i}")));
            cache.flush();
        }
        assert_eq!(packs(&dir).len(), 1, "flush must merge past MAX_PACKS");

        for c in [&cache, &StubSliceCache::open(dir.path())] {
            for i in 0..=MAX_PACKS {
                assert!(c
                    .get(&format!("/x/{i}.php"), &hash_source(&i.to_string()), 8)
                    .is_some());
            }
            let newest = hash_source(&format!("v{MAX_PACKS}"));
            assert!(c.get("/x/shared.php", &newest, 8).is_some());
        }
    }

    #[test]
    fn pack_from_another_build_is_ignored_and_removed() {
        let (dir, cache) = make_cache();
        let hash = hash_source("a");
        put_empty(&cache, "/x/a.php", &hash);
        drop(cache);
        let pack = packs(&dir).remove(0);
        let mut bytes = std::fs::read(&pack).unwrap();
        bytes[8] ^= 0xff; // mir version hash
        std::fs::write(&pack, &bytes).unwrap();

        let cache = StubSliceCache::open(dir.path());
        assert!(cache.get("/x/a.php", &hash, 8).is_none());
        cache.flush();
        assert!(packs(&dir).is_empty());
    }

    #[test]
    fn old_per_entry_shard_directories_are_removed() {
        let dir = TempDir::new().unwrap();
        let shard = dir.path().join("stubs").join("ab");
        std::fs::create_dir_all(&shard).unwrap();
        std::fs::write(shard.join("abcd.bin"), b"old").unwrap();

        StubSliceCache::open(dir.path()).flush();
        assert!(!shard.exists());
    }

    #[test]
    fn truncated_pack_is_treated_as_miss() {
        let (dir, cache) = make_cache();
        let hash = hash_source("a");
        put_empty(&cache, "/x/a.php", &hash);
        drop(cache);
        let pack = packs(&dir).remove(0);
        let bytes = std::fs::read(&pack).unwrap();
        std::fs::write(&pack, &bytes[..bytes.len() - 1]).unwrap();

        assert!(StubSliceCache::open(dir.path())
            .get("/x/a.php", &hash, 8)
            .is_none());
    }

    /// A desynced payload can make bincode read a garbage collection length
    /// instead of erroring immediately; without a size limit it would try to
    /// allocate that much before returning `Err`.
    #[test]
    fn oversized_length_prefix_is_treated_as_miss_not_a_huge_allocation() {
        let (dir, cache) = make_cache();
        let hash = hash_source("a");
        put_empty(&cache, "/x/a.php", &hash);
        drop(cache);
        let pack = packs(&dir).remove(0);
        let mut bytes = std::fs::read(&pack).unwrap();
        let start = HEADER_LEN as usize;
        bytes[start..start + 8].copy_from_slice(&u64::MAX.to_le_bytes());
        std::fs::write(&pack, &bytes).unwrap();

        let cache = StubSliceCache::open(dir.path());
        assert!(cache.get("/x/a.php", &hash, 8).is_none());
        assert_eq!(cache.misses(), 1);
    }
}
