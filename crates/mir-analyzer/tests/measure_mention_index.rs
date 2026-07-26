//! Class-mention gate index measurement harness (Laravel corpus).
//!
//! Reports, for the reference-query gate at real-workspace scale:
//! - wall time of a raw-scan gate pass (cold) vs an index-answered pass (warm)
//! - allocation churn (count + bytes) of each pass
//! - the mention index's memory footprint (entries + automaton)
//!
//! Run explicitly (`#[ignore]`d like the other measurement harnesses):
//!
//! ```sh
//! cargo test --release -p mir-analyzer --test measure_mention_index -- --ignored --nocapture
//! ```

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use mir_analyzer::{discover_files, AnalysisSession, IndexCancel, IndexParallelism, PhpVersion};

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);

struct CountingAlloc;

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static A: CountingAlloc = CountingAlloc;

fn alloc_snapshot() -> (u64, u64) {
    (
        ALLOCS.load(Ordering::Relaxed),
        ALLOC_BYTES.load(Ordering::Relaxed),
    )
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("fixtures")
        .join("laravel")
}

#[test]
#[ignore = "measurement harness; run explicitly with --release --ignored"]
fn measure_mention_index() {
    let root = fixtures_root();
    if !root.exists() {
        eprintln!("Skipping: fixture not present at {}", root.display());
        return;
    }

    // Register the whole corpus the LSP way (bulk text + index batches).
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    let mut files: Vec<(Arc<str>, Arc<str>)> = Vec::new();
    let mut total_bytes = 0usize;
    for dir in ["src", "vendor", "types", "tests", "config"] {
        for path in discover_files(&root.join(dir)) {
            if let Ok(text) = std::fs::read_to_string(&path) {
                total_bytes += text.len();
                files.push((
                    Arc::from(path.to_string_lossy().as_ref()),
                    Arc::from(text.as_str()),
                ));
            }
        }
    }
    // Two probe classes that nothing references: their reference queries are
    // pure gate passes over the full candidate set, isolating gate cost from
    // candidate analysis cost.
    let probe_a: Arc<str> = Arc::from("probe_a.php");
    let probe_b: Arc<str> = Arc::from("probe_b.php");
    files.push((
        probe_a.clone(),
        Arc::from("<?php\nnamespace Probe;\nclass ZzzProbeAlpha {}\n"),
    ));
    files.push((
        probe_b.clone(),
        Arc::from("<?php\nnamespace Probe;\nclass ZzzProbeBeta {}\n"),
    ));

    let t = Instant::now();
    for chunk in files.chunks(256) {
        session.index_batch(chunk, IndexParallelism::Rayon, &IndexCancel::new());
    }
    session.finalize_index();
    eprintln!(
        "corpus: {} files, {:.1} MB text; indexed in {:.2?}",
        files.len(),
        total_bytes as f64 / 1e6,
        t.elapsed()
    );

    let paths: Vec<Arc<str>> = files.iter().map(|(p, _)| p.clone()).collect();
    let never = || false;

    // Pass 1 — cold gate: no mention entries exist; every candidate is
    // raw-scanned against the whole universe and recorded.
    let (a0, b0) = alloc_snapshot();
    let t = Instant::now();
    let r1 = session
        .indexed_references_to(
            &mir_analyzer::Name::class("Probe\\ZzzProbeAlpha"),
            &paths,
            false,
            &never,
        )
        .unwrap();
    let cold = t.elapsed();
    let (a1, b1) = alloc_snapshot();

    // Pass 2 — warm gate, same needle: answered entirely from the index.
    let t = Instant::now();
    let r2 = session
        .indexed_references_to(
            &mir_analyzer::Name::class("Probe\\ZzzProbeAlpha"),
            &paths,
            false,
            &never,
        )
        .unwrap();
    let warm_same = t.elapsed();
    let (a2, b2) = alloc_snapshot();

    // Pass 3 — warm gate, different needle known at scan time: still
    // lookup-only (this is the cross-query reuse the index exists for).
    let t = Instant::now();
    let r3 = session
        .indexed_references_to(
            &mir_analyzer::Name::class("Probe\\ZzzProbeBeta"),
            &paths,
            false,
            &never,
        )
        .unwrap();
    let warm_other = t.elapsed();
    let (a3, b3) = alloc_snapshot();

    // Pass 4 — baseline: a needle outside the universe takes the pre-index
    // path verbatim (per-file `IdentifierNeedles` raw scan, no recording).
    // This is what EVERY query paid before the index existed.
    let (a3b, b3b) = alloc_snapshot();
    let t = Instant::now();
    let r4 = session
        .indexed_references_to(
            &mir_analyzer::Name::class("Probe\\ZzzNotDeclaredAnywhere"),
            &paths,
            false,
            &never,
        )
        .unwrap();
    let baseline = t.elapsed();
    let (a4, b4) = alloc_snapshot();
    eprintln!(
        "raw-scan baseline (needle outside universe): {baseline:>10.2?}   allocs {:>10}  bytes {:>12}",
        a4 - a3b,
        b4 - b3b
    );

    assert!(r1.is_empty() && r2.is_empty() && r3.is_empty() && r4.is_empty());

    let stats = session.class_mention_stats();
    // Entry cost model: Name is 8 bytes; FileMentions ≈ text Arc ptr (8) +
    // epoch (8) + boxed slice (16) + map slot ≈ 64 B fixed per file.
    let entry_bytes = stats.total_mentions * 8 + stats.files_covered * 64;
    eprintln!("--- gate pass timings (zero-reference probe classes) ---");
    eprintln!(
        "cold  (scan+record) : {cold:>10.2?}   allocs {:>10}  bytes {:>12}",
        a1 - a0,
        b1 - b0
    );
    eprintln!(
        "warm  (same needle) : {warm_same:>10.2?}   allocs {:>10}  bytes {:>12}",
        a2 - a1,
        b2 - b1
    );
    eprintln!(
        "warm  (other needle): {warm_other:>10.2?}   allocs {:>10}  bytes {:>12}",
        a3 - a2,
        b3 - b2
    );
    eprintln!("--- mention index footprint ---");
    eprintln!(
        "universe {} names | files covered {} | mentions {} (~{:.1} MB entries) | scanner {:.1} MB | scans recorded {}",
        stats.universe_names,
        stats.files_covered,
        stats.total_mentions,
        entry_bytes as f64 / 1e6,
        stats.scanner_bytes as f64 / 1e6,
        stats.scans_recorded
    );

    // Loose guards so a regression that silently disables the index (or
    // makes warm passes rescan) fails the harness rather than just printing.
    assert!(
        stats.files_covered > files.len() / 2,
        "cold pass should cover most of the corpus: {stats:?}"
    );
    assert!(
        warm_other < cold,
        "index-answered gate must beat the raw-scan gate: cold {cold:?} vs warm {warm_other:?}"
    );
    let scans_after = session.class_mention_stats().scans_recorded;
    assert_eq!(
        scans_after, stats.scans_recorded,
        "stats read must not record scans"
    );
}
