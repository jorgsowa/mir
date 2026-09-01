//! Persistent definition cache: serialized [`StubSlice`] per source file, keyed
//! by file path with the file's content hash as the validity field.
//!
//! On a cache hit, `collect_definitions` and the definition collection inside
//! `analyze()` deserialize the stored slice and skip the much more expensive
//! parse + definition-collection work. Vendor analysis on Laravel
//! (~10 k files) is dominated by parse+collect (≈800 ms) vs. ingest (≈45 ms),
//! so the cache addresses the dominant cost.
//!
//! Format choice (bincode 1.x): postcard was the original pick but it pulls
//! `heapless` -> the unmaintained `atomic-polyfill` (RUSTSEC-2023-0089),
//! which `cargo-deny` rejects. bincode v2 replaced it but was itself flagged
//! as unmaintained (RUSTSEC-2025-0141). bincode 1.3.3 is explicitly called
//! "complete" by the bincode team, carries no advisory, and uses the same
//! transparent serde compatibility.
//!
//! Layout: `<cache_dir>/stubs/<hh>/<full_hash>.bin` where `<hh>` is the first
//! two hex chars of the path hash. Sharding keeps any single directory below
//! ~40 entries even for large monorepos.
//!
//! Format: a fixed-size [`Header`] (magic + version fields + content hash)
//! followed by a bincode 1.x-encoded [`StubSlice`], followed by a bincode
//! 1.x-encoded `Vec<Issue>` (the collector-phase issues found alongside the
//! slice — parse errors, `BackedEnumCaseTypeMismatch`, docblock warnings,
//! etc.). Any header mismatch is treated as a miss so cache files survive
//! across mir upgrades without risking type-layout corruption.
//!
//! Writes are atomic: each shard is written to a sibling tempfile in the
//! same directory and then renamed into place. A SIGINT mid-write therefore
//! never produces a partially-written entry that the next session would
//! deserialize as garbage.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use mir_codebase::definitions::{deduplicate_params_in_slice, StubSlice};
use mir_issues::Issue;
use serde::{Deserialize, Serialize};

/// Magic bytes at the start of every cache entry. "MIR\x01" little-endian.
const MAGIC: u32 = 0x0152_494D;
/// Bumped when the on-disk header layout changes OR a serialized `StubSlice`
/// struct changes shape (e.g. `inferred_return_type: Option<Type>` →
/// `Option<Arc<Type>>`), so stale entries are rejected. Also bumped whenever
/// the trailing payload shape changes (e.g. adding the `Vec<Issue>` section).
const FORMAT_VERSION: u8 = 12;

/// Cache header. Any mismatch (magic, version, content_hash, php_version)
/// forces the consumer to treat the entry as a miss and recompute.
#[derive(Serialize, Deserialize)]
struct Header {
    magic: u32,
    /// Stable hash of `CARGO_PKG_VERSION`. Bumps with every mir release so
    /// cached `StubSlice` data produced by an older version is rejected.
    mir_version: u64,
    format_version: u8,
    php_version: u8,
    /// `blake3` digest of the source file content; the entry is valid iff this
    /// matches the caller's hash.
    content_hash: [u8; 32],
}

/// Precomputed at process start: hash of `CARGO_PKG_VERSION` so two mir
/// builds with different versions never share cache entries.
fn mir_version_hash() -> u64 {
    use std::sync::OnceLock;
    static HASH: OnceLock<u64> = OnceLock::new();
    *HASH.get_or_init(|| {
        let digest = blake3::hash(env!("CARGO_PKG_VERSION").as_bytes());
        let bytes = digest.as_bytes();
        u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ])
    })
}

/// One queued disk write for the background writer thread.
struct WriteJob {
    entry_path: PathBuf,
    header: Header,
    slice: std::sync::Arc<StubSlice>,
    issues: std::sync::Arc<Vec<Issue>>,
    seq: u64,
}

/// Persistent definition cache. Thread-safe. Reads go straight to the
/// filesystem; writes are handed to a lazily-spawned background thread so a
/// whole-workspace definition walk never serializes 15K slice files on its
/// own (query-latency) critical path. Reads-after-write in the same session
/// are served by Salsa's memos, not this cache, so the write delay is
/// unobservable except across sessions — and [`Self::flush`]/`Drop` join the
/// writer, so a clean shutdown loses nothing.
pub struct StubSliceCache {
    root: PathBuf,
    hits: AtomicU64,
    misses: AtomicU64,
    writes: AtomicU64,
    enabled: bool,
    writer: std::sync::Mutex<
        Option<(
            std::sync::mpsc::Sender<WriteJob>,
            std::thread::JoinHandle<()>,
        )>,
    >,
}

impl StubSliceCache {
    /// Open (or create) the cache under `<cache_dir>/stubs/`. The directory
    /// is created lazily; if creation fails, the cache silently disables
    /// itself so analysis still works on a read-only filesystem.
    pub fn open(cache_dir: &Path) -> Self {
        let root = cache_dir.join("stubs");
        let enabled = std::fs::create_dir_all(&root).is_ok();
        Self {
            root,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            enabled,
            writer: std::sync::Mutex::new(None),
        }
    }

    fn shard_path(&self, path: &str) -> PathBuf {
        let digest = blake3::hash(path.as_bytes());
        let hex = digest.to_hex();
        let s = hex.as_str();
        self.root.join(&s[..2]).join(format!("{}.bin", s))
    }

    /// Return the cached [`StubSlice`] and its issues for `path` if its
    /// stored entry matches `(content_hash, php_version)` and the current
    /// mir version.
    ///
    /// On hit the deserialized slice has its `file` field restored from the
    /// caller-supplied `path` (we don't trust paths from disk), same for
    /// every issue's `location.file`, and the slice's `is_deduped` flag is
    /// preserved as `false`; callers running in parallel should re-run
    /// [`deduplicate_params_in_slice`] before ingest so the serial
    /// write-lock section doesn't pay dedup costs (commit 3018a1d).
    pub fn get(
        &self,
        path: &str,
        content_hash: &[u8; 32],
        php_version: u8,
    ) -> Option<(StubSlice, Vec<Issue>)> {
        if !self.enabled {
            return None;
        }
        let entry_path = self.shard_path(path);
        let bytes = std::fs::read(&entry_path).ok()?;
        // A stale or bit-flipped entry can desync bincode's length-prefixed
        // decoding, making it read a bogus multi-gigabyte collection length
        // from garbage bytes and try to allocate it before returning an
        // error. Bounding every read to the entry's own size makes that
        // impossible: the real data can never need more bytes than the file
        // contains, so a desync now fails fast as a miss instead of paging
        // the machine to death. `config()` (not the newer `options()`) is
        // required here: it's the fixint encoding `put()` writes with, while
        // `options()`'s `DefaultOptions` defaults to varint and silently
        // misreads every field.
        #[allow(deprecated)]
        let cfg = bincode::config().limit(bytes.len() as u64).clone();
        let mut cursor = Cursor::new(&bytes);
        let header: Header = cfg.deserialize_from(&mut cursor).ok()?;
        if header.magic != MAGIC
            || header.format_version != FORMAT_VERSION
            || header.mir_version != mir_version_hash()
            || header.php_version != php_version
            || &header.content_hash != content_hash
        {
            self.misses.fetch_add(1, Ordering::Relaxed);
            return None;
        }
        let mut slice: StubSlice = match cfg.deserialize_from(&mut cursor) {
            Ok(slice) => slice,
            Err(_) => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        };
        let mut issues: Vec<Issue> = match cfg.deserialize_from(&mut cursor) {
            Ok(issues) => issues,
            Err(_) => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        };
        // Restore the caller's path; cached paths are not trusted.
        slice.file = Some(std::sync::Arc::from(path));
        let path_arc: std::sync::Arc<str> = std::sync::Arc::from(path);
        for issue in &mut issues {
            issue.location.file = path_arc.clone();
        }
        self.hits.fetch_add(1, Ordering::Relaxed);
        Some((slice, issues))
    }

    /// Write `slice` and `issues` to the cache. Atomic via
    /// tempfile-in-same-directory + rename. Errors (disk full, permission
    /// denied, race with another writer) are swallowed — the cache is an
    /// optimization, never a correctness dependency.
    pub fn put(
        &self,
        path: &str,
        content_hash: &[u8; 32],
        php_version: u8,
        slice: &std::sync::Arc<StubSlice>,
        issues: &std::sync::Arc<Vec<Issue>>,
    ) {
        if !self.enabled {
            return;
        }
        let job = WriteJob {
            entry_path: self.shard_path(path),
            header: Header {
                magic: MAGIC,
                mir_version: mir_version_hash(),
                format_version: FORMAT_VERSION,
                php_version,
                content_hash: *content_hash,
            },
            slice: std::sync::Arc::clone(slice),
            issues: std::sync::Arc::clone(issues),
            seq: self.writes.fetch_add(1, Ordering::Relaxed),
        };
        let mut guard = self.writer.lock().unwrap();
        if guard.is_none() {
            let (tx, rx) = std::sync::mpsc::channel::<WriteJob>();
            let handle = std::thread::Builder::new()
                .name("mir-stub-cache-writer".into())
                .spawn(move || {
                    for job in rx {
                        write_entry(job);
                    }
                })
                .ok();
            match handle {
                Some(h) => *guard = Some((tx, h)),
                // Can't spawn: degrade to a synchronous write.
                None => {
                    drop(guard);
                    write_entry(job);
                    return;
                }
            }
        }
        if let Some((tx, _)) = guard.as_ref() {
            // A send can only fail if the writer thread died; the entry is
            // then simply not cached — same contract as any other I/O error.
            let _ = tx.send(job);
        }
    }

    /// Join the background writer, guaranteeing every queued entry is on
    /// disk. The next [`Self::put`] respawns it. Called by `Drop`, so a
    /// clean session shutdown never loses queued writes.
    pub fn flush(&self) {
        let taken = self.writer.lock().unwrap().take();
        if let Some((tx, handle)) = taken {
            drop(tx);
            let _ = handle.join();
        }
    }

    /// Cumulative hit count across this cache instance.
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// Cumulative miss count across this cache instance.
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }
}

impl Drop for StubSliceCache {
    fn drop(&mut self) {
        self.flush();
    }
}

/// Serialize and atomically write one cache entry. Runs on the background
/// writer thread (or inline when spawning failed).
fn write_entry(job: WriteJob) {
    let WriteJob {
        entry_path,
        header,
        slice,
        issues,
        seq,
    } = job;
    let Some(shard_dir) = entry_path.parent() else {
        return;
    };
    if std::fs::create_dir_all(shard_dir).is_err() {
        return;
    }
    // Serialize header + body into a single buffer so we issue exactly
    // one write syscall.
    let mut buf = match bincode::serialize(&header) {
        Ok(b) => b,
        Err(_) => return,
    };
    // Strip path-bearing field; the loader re-applies it.
    let mut slice_for_disk = (*slice).clone();
    slice_for_disk.file = None;
    // `is_deduped` is #[serde(skip)] so it does not need stripping.
    match bincode::serialize(&slice_for_disk) {
        Ok(body) => buf.extend_from_slice(&body),
        Err(_) => return,
    }
    // Issues' `location.file` isn't stripped before serializing (unlike the
    // slice's `file` field) — it's simpler to just always overwrite it from
    // the caller-supplied path on read (same "don't trust disk" policy),
    // rather than giving `Location` an optional file field solely for this.
    match bincode::serialize(&*issues) {
        Ok(body) => buf.extend_from_slice(&body),
        Err(_) => return,
    }
    // Tempfile in the same directory so the rename is atomic on every
    // POSIX filesystem (cross-mount renames would degrade to copy).
    let tmp = entry_path.with_extension(format!("tmp.{}.{}", crate::tmp_suffix::next(), seq));
    if std::fs::write(&tmp, &buf).is_err() {
        return;
    }
    let _ = std::fs::rename(&tmp, &entry_path);
}

/// Convenience: hash a source string into the 32-byte digest the cache
/// expects. Centralises the hash choice (BLAKE3) so callers don't pick
/// inconsistent functions.
pub fn hash_source(source: &str) -> [u8; 32] {
    *blake3::hash(source.as_bytes()).as_bytes()
}

/// Convert a slice produced by [`StubSliceCache::get`] into one that's safe
/// to consume immediately without paying dedup cost inside the serial
/// write-lock section. Call from a parallel worker.
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

    fn no_issues() -> std::sync::Arc<Vec<Issue>> {
        std::sync::Arc::new(Vec::new())
    }

    fn sample_issue(file: &str) -> Issue {
        Issue::new(
            mir_issues::IssueKind::UndefinedVariable {
                name: "x".to_string(),
            },
            mir_types::Location {
                file: std::sync::Arc::from(file),
                line: 1,
                line_end: 1,
                col_start: 0,
                col_end: 1,
            },
        )
    }

    #[test]
    fn roundtrip_returns_equivalent_slice() {
        let (_dir, cache) = make_cache();
        let hash = hash_source("<?php class A {}");
        let slice = StubSlice::default();
        cache.put(
            "/x/a.php",
            &hash,
            8,
            &std::sync::Arc::new(slice),
            &no_issues(),
        );
        cache.flush();

        let (got, issues) = cache.get("/x/a.php", &hash, 8).expect("hit");
        assert_eq!(
            got.file.as_deref().map(|s| s.to_string()),
            Some("/x/a.php".to_string())
        );
        assert!(issues.is_empty());
        assert_eq!(cache.hits(), 1);
    }

    #[test]
    fn roundtrip_returns_equivalent_issues_with_path_patched() {
        // Each on-disk shard is keyed by path (unlike the in-process
        // ParseCache, which shares one entry across files with identical
        // content) — content_hash is only a within-shard validity check. So
        // "don't trust disk" here means: even if a serialized issue's
        // `location.file` were something else, `get` must always overwrite
        // it to the caller-supplied path, same as it already does for the
        // slice's own `file` field.
        let (_dir, cache) = make_cache();
        let hash = hash_source("<?php $x;");
        let issues = std::sync::Arc::new(vec![sample_issue("/different/path.php")]);
        cache.put(
            "/x/a.php",
            &hash,
            8,
            &std::sync::Arc::new(StubSlice::default()),
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
        let hash_a = hash_source("a");
        let hash_b = hash_source("b");
        cache.put(
            "/x/a.php",
            &hash_a,
            8,
            &std::sync::Arc::new(StubSlice::default()),
            &no_issues(),
        );
        cache.flush();

        assert!(cache.get("/x/a.php", &hash_b, 8).is_none());
    }

    #[test]
    fn miss_on_php_version_mismatch() {
        let (_dir, cache) = make_cache();
        let hash = hash_source("a");
        cache.put(
            "/x/a.php",
            &hash,
            8,
            &std::sync::Arc::new(StubSlice::default()),
            &no_issues(),
        );
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
        // A slice may have been written with file=Some("a.php") but the
        // loader must always use the caller-supplied path so two distinct
        // paths reading the same shard never disagree about provenance.
        let (_dir, cache) = make_cache();
        let hash = hash_source("a");
        let slice = StubSlice {
            file: Some(std::sync::Arc::from("/different/path.php")),
            ..Default::default()
        };
        cache.put(
            "/x/a.php",
            &hash,
            8,
            &std::sync::Arc::new(slice),
            &no_issues(),
        );
        cache.flush();

        let (got, _) = cache.get("/x/a.php", &hash, 8).unwrap();
        assert_eq!(
            got.file.as_deref().map(|s| s.to_string()),
            Some("/x/a.php".to_string())
        );
    }

    #[test]
    fn corrupt_entry_is_treated_as_miss() {
        let (dir, cache) = make_cache();
        let hash = hash_source("a");
        cache.put(
            "/x/a.php",
            &hash,
            8,
            &std::sync::Arc::new(StubSlice::default()),
            &no_issues(),
        );
        cache.flush();

        // Overwrite the shard with garbage.
        let digest = blake3::hash("/x/a.php".as_bytes()).to_hex();
        let s = digest.as_str();
        let bad = dir
            .path()
            .join("stubs")
            .join(&s[..2])
            .join(format!("{}.bin", s));
        std::fs::write(&bad, b"not a header").unwrap();

        assert!(cache.get("/x/a.php", &hash, 8).is_none());
    }

    /// A desynced body (valid header, incompatible payload) can make bincode
    /// read a garbage multi-exabyte collection length instead of erroring
    /// immediately. Without a size limit this tries to allocate that much
    /// before ever returning `Err`; regression test for that failure mode.
    #[test]
    fn oversized_length_prefix_is_treated_as_miss_not_a_huge_allocation() {
        let (dir, cache) = make_cache();
        let hash = hash_source("a");
        cache.put(
            "/x/a.php",
            &hash,
            8,
            &std::sync::Arc::new(StubSlice::default()),
            &no_issues(),
        );
        cache.flush();

        let digest = blake3::hash("/x/a.php".as_bytes()).to_hex();
        let s = digest.as_str();
        let entry_path = dir
            .path()
            .join("stubs")
            .join(&s[..2])
            .join(format!("{}.bin", s));
        let mut bytes = std::fs::read(&entry_path).unwrap();

        // Find where the header ends, same as `get` does.
        let mut cursor = Cursor::new(&bytes);
        #[allow(deprecated)]
        let cfg = bincode::config().limit(bytes.len() as u64).clone();
        let _: Header = cfg.deserialize_from(&mut cursor).unwrap();
        let header_len = cursor.position() as usize;

        // Replace the body with a claimed length far larger than the file
        // could possibly contain.
        bytes.truncate(header_len);
        bytes.extend_from_slice(&bincode::serialize(&u64::MAX).unwrap());
        std::fs::write(&entry_path, &bytes).unwrap();

        assert!(cache.get("/x/a.php", &hash, 8).is_none());
    }
}
