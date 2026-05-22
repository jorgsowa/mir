//! Incremental-workload benchmarks: compare per-edit single-file analysis
//! latency between [`ProjectAnalyzer::re_analyze_file`] and the new
//! [`AnalysisSession`] + [`FileAnalyzer`] APIs.
//!
//! The fixture is the same Laravel checkout used by `analyze_real_world.rs`;
//! we use the leaf file `Auth/Events/Login.php` (no dependents — best-case
//! for both APIs) and `Database/Eloquent/Model.php` (high fanout — exercises
//! cross-file invalidation).
//!
//! NOTE: `FileAnalyzer::analyze` resolves cross-file inferred return types on
//! demand via salsa; no separate inference sweep is required.  The diagnostic
//! outputs should be equivalent to `ProjectAnalyzer::re_analyze_file`.
//! Run `analyze_real_world` for full-fidelity diagnostic benchmarks.

use std::alloc::{GlobalAlloc, Layout, System};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;
use std::time::Duration;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use mir_analyzer::cache::AnalysisCache;
use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion, ProjectAnalyzer, Symbol};
use mir_types::Symbol as MirSymbol;
use tempfile::TempDir;

// Counting allocator — global atomics updated on every alloc/dealloc.
struct CountingAllocator;
static G_LIVE: AtomicI64 = AtomicI64::new(0);
static G_PEAK: AtomicI64 = AtomicI64::new(0);
static G_TOTAL: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let sz = layout.size();
            G_TOTAL.fetch_add(sz, Relaxed);
            let new_live = G_LIVE.fetch_add(sz as i64, Relaxed) + sz as i64;
            G_PEAK.fetch_max(new_live, Relaxed);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        G_LIVE.fetch_sub(layout.size() as i64, Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn reset_alloc_counters() {
    G_LIVE.store(0, Relaxed);
    G_PEAK.store(0, Relaxed);
    G_TOTAL.store(0, Relaxed);
}

fn snapshot_alloc() -> (f64, f64, f64) {
    let live = G_LIVE.load(Relaxed) as f64 / 1_048_576.0;
    let peak = G_PEAK.load(Relaxed) as f64 / 1_048_576.0;
    let total = G_TOTAL.load(Relaxed) as f64 / 1_048_576.0;
    (live, peak, total)
}

// ---------------------------------------------------------------------------
// Fixture helpers (mirrored from analyze_real_world.rs)
// ---------------------------------------------------------------------------

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures/laravel")
}

fn skip_if_missing(root: &Path) -> bool {
    let src = root.join("src");
    let vendor = root.join("vendor");
    if !src.exists() || !vendor.exists() {
        eprintln!(
            "\nSkipping incremental workload benchmark: fixture not found at {}\n\
             Run: bash crates/mir-analyzer/benches/download-fixtures.sh\n",
            root.display()
        );
        true
    } else {
        false
    }
}

fn split_vendor_project(root: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let vendor_files = ProjectAnalyzer::discover_files(&root.join("vendor"));
    let project_files = ProjectAnalyzer::discover_files(&root.join("src"));
    (vendor_files, project_files)
}

// ---------------------------------------------------------------------------
// ProjectAnalyzer warmup
// ---------------------------------------------------------------------------

fn warm_project_analyzer(
    cache_dir: &TempDir,
    vendor_files: &[PathBuf],
    project_files: &[PathBuf],
) -> ProjectAnalyzer {
    let analyzer = ProjectAnalyzer::with_cache(cache_dir.path());
    analyzer.load_stubs();
    analyzer.collect_types_only(vendor_files);
    let _ = analyzer.analyze(project_files);
    analyzer
}

// ---------------------------------------------------------------------------
// AnalysisSession warmup — mirrors the workspace-open flow
// ---------------------------------------------------------------------------

/// Ingest every project + vendor file into a session so subsequent analyses
/// see the full codebase. Equivalent in coverage to ProjectAnalyzer's
/// load_stubs + collect_types_only + analyze.
///
/// Vendor files are registered with HIGH durability (they won't be edited
/// during the session) so salsa can skip O(N) dep-verification for them on
/// each subsequent project-file edit.
fn warm_session(
    cache_dir: &TempDir,
    vendor_files: &[PathBuf],
    project_files: &[PathBuf],
) -> AnalysisSession {
    let cache = Arc::new(AnalysisCache::open(cache_dir.path()));
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache);
    session.ensure_stubs_loaded();
    // Vendor files: HIGH durability (stable within session).
    let vendor_pairs: Vec<(Arc<str>, Arc<str>)> = vendor_files
        .iter()
        .filter_map(|path| {
            let src = std::fs::read_to_string(path).ok()?;
            Some((
                Arc::from(path.to_string_lossy().as_ref()),
                Arc::from(src.as_str()),
            ))
        })
        .collect();
    session.set_stable_workspace_files(vendor_pairs);
    // Project files: LOW durability (may be edited).
    for path in project_files.iter() {
        if let Ok(src) = std::fs::read_to_string(path) {
            let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
            session.ingest_file(file, Arc::from(src));
        }
    }
    // Build the workspace symbol index singleton once so subsequent
    // ingest_file + analyze iterations can read symbols in O(1).
    session.rebuild_workspace_symbol_index();
    session
}

// ---------------------------------------------------------------------------
// Core comparison: single-file edit latency
// ---------------------------------------------------------------------------

/// Best-case path: edit a leaf file with no dependents. Measures pure
/// per-file Pass 2 cost.
fn bench_single_file_edit(c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let (vendor_files, project_files) = split_vendor_project(&root);
    let target = root.join("src/Illuminate/Auth/Events/Login.php");
    if !target.exists() {
        eprintln!("Skipping: target Login.php not found");
        return;
    }
    let target_str = target.to_string_lossy().to_string();
    let original = std::fs::read_to_string(&target).unwrap();

    let mut group = c.benchmark_group("single_file_edit");
    group.sample_size(20);
    group.measurement_time(Duration::from_secs(15));

    // ----- A) ProjectAnalyzer::re_analyze_file -----
    {
        let cache: TempDir = tempfile::tempdir().unwrap();
        let analyzer = warm_project_analyzer(&cache, &vendor_files, &project_files);
        let mut counter = 0u32;

        group.bench_function("project_analyzer", |b| {
            b.iter_batched(
                || {
                    counter += 1;
                    format!("{original}\n// edit {counter}\n")
                },
                |new_content| analyzer.re_analyze_file(&target_str, &new_content),
                BatchSize::LargeInput,
            );
        });
    }

    // ----- B) AnalysisSession + FileAnalyzer (single-pass, no inference sweep) -----
    {
        let cache: TempDir = tempfile::tempdir().unwrap();
        let session = warm_session(&cache, &vendor_files, &project_files);
        let target_arc: Arc<str> = Arc::from(target_str.as_str());
        let mut counter = 0u32;

        group.bench_function("file_analyzer", |b| {
            b.iter_batched(
                || {
                    counter += 1;
                    Arc::<str>::from(format!("{original}\n// edit {counter}\n"))
                },
                |new_content| {
                    // Re-ingest Pass 1 + run single-pass Pass 2.
                    session.ingest_file(target_arc.clone(), new_content.clone());

                    let parsed = php_rs_parser::parse(new_content.as_ref());
                    let hard_errors: Vec<_> = parsed
                        .errors
                        .iter()
                        .filter(|e| {
                            matches!(e.severity(), php_rs_parser::diagnostics::Severity::Error)
                        })
                        .collect();
                    assert!(
                        hard_errors.is_empty(),
                        "bench source must parse (hard errors: {})",
                        hard_errors.len()
                    );
                    FileAnalyzer::new(&session).analyze(
                        target_arc.clone(),
                        new_content.as_ref(),
                        &parsed.program,
                        &parsed.source_map,
                    )
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();

    // Restore file content (paranoia — the format! adds a marker line that
    // would otherwise drift across runs).
    std::fs::write(&target, original).unwrap();
}

/// High-fanout path: edit a base class with many subclasses. Tests the
/// reverse-dep / cache-eviction interaction. ProjectAnalyzer triggers full
/// dependent re-analysis; FileAnalyzer measures only the edited file
/// (consumers typically publish diagnostics for the open buffer; dependents
/// are picked up on their own re-analysis).
fn bench_high_fanout_edit(c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let (vendor_files, project_files) = split_vendor_project(&root);
    let target = root.join("src/Illuminate/Database/Eloquent/Model.php");
    if !target.exists() {
        eprintln!("Skipping: target Model.php not found");
        return;
    }
    let target_str = target.to_string_lossy().to_string();
    let original = std::fs::read_to_string(&target).unwrap();

    let mut group = c.benchmark_group("high_fanout_edit");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    {
        let cache: TempDir = tempfile::tempdir().unwrap();
        let analyzer = warm_project_analyzer(&cache, &vendor_files, &project_files);
        let mut counter = 0u32;

        group.bench_function("project_analyzer", |b| {
            b.iter_batched(
                || {
                    counter += 1;
                    format!("{original}\n// edit {counter}\n")
                },
                |new_content| analyzer.re_analyze_file(&target_str, &new_content),
                BatchSize::LargeInput,
            );
        });
    }

    {
        let cache: TempDir = tempfile::tempdir().unwrap();
        let session = warm_session(&cache, &vendor_files, &project_files);
        let target_arc: Arc<str> = Arc::from(target_str.as_str());
        let mut counter = 0u32;

        group.bench_function("file_analyzer", |b| {
            b.iter_batched(
                || {
                    counter += 1;
                    Arc::<str>::from(format!("{original}\n// edit {counter}\n"))
                },
                |new_content| {
                    session.ingest_file(target_arc.clone(), new_content.clone());

                    let parsed = php_rs_parser::parse(new_content.as_ref());
                    let hard_errors: Vec<_> = parsed
                        .errors
                        .iter()
                        .filter(|e| {
                            matches!(e.severity(), php_rs_parser::diagnostics::Severity::Error)
                        })
                        .collect();
                    assert!(
                        hard_errors.is_empty(),
                        "bench source must parse (hard errors: {})",
                        hard_errors.len()
                    );
                    FileAnalyzer::new(&session).analyze(
                        target_arc.clone(),
                        new_content.as_ref(),
                        &parsed.program,
                        &parsed.source_map,
                    )
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();

    std::fs::write(&target, original).unwrap();
}

// ---------------------------------------------------------------------------
// Hover-style read latency: snapshot read vs lock-held read
// ---------------------------------------------------------------------------

/// Measure the cost of a single read-only query (symbol_location). With M1's
/// clone-then-release pattern, this is dominated by `MirDb::clone()` plus the
/// query itself; not by waiting for any concurrent edits.
fn bench_read_query_latency(c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let (vendor_files, project_files) = split_vendor_project(&root);
    let cache: TempDir = tempfile::tempdir().unwrap();
    let analyzer = warm_project_analyzer(&cache, &vendor_files, &project_files);

    let mut group = c.benchmark_group("read_query");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("project_analyzer_symbol_location", |b| {
        b.iter(|| analyzer.symbol_location("Illuminate\\Database\\Eloquent\\Model"));
    });

    let cache_b: TempDir = tempfile::tempdir().unwrap();
    let session = warm_session(&cache_b, &vendor_files, &project_files);

    group.bench_function("session_read_lookup", |b| {
        b.iter(|| {
            session.read(|db| {
                let fqcn = mir_analyzer::db::Fqcn::new(
                    db,
                    MirSymbol::new("Illuminate\\Database\\Eloquent\\Model"),
                );
                mir_analyzer::db::find_class_like(db, fqcn).is_some()
            })
        });
    });

    group.finish();
}

/// Cold-start stub-loading time: essentials-only vs every embedded stub.
/// Models the "session start → first analysis" path: no project files, no
/// codebase work — just the cost of priming the session's built-in symbols.
fn bench_stub_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("stub_loading");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(15));

    group.bench_function("essential_only", |b| {
        b.iter(|| {
            let session = AnalysisSession::new(PhpVersion::LATEST);
            session.ensure_essential_stubs_loaded();
            session.loaded_stub_count()
        });
    });

    group.bench_function("all_stubs", |b| {
        b.iter(|| {
            let session = AnalysisSession::new(PhpVersion::LATEST);
            session.ensure_all_stubs_loaded();
            session.loaded_stub_count()
        });
    });

    // Common incremental shape: load essentials, then a couple of extension
    // stubs as user code references them. Should still be much cheaper than
    // full load.
    group.bench_function("essential_plus_a_few_lazy", |b| {
        b.iter(|| {
            let session = AnalysisSession::new(PhpVersion::LATEST);
            session.ensure_essential_stubs_loaded();
            let _ = session.ensure_stub_for_function("imagecreate"); // gd
            let _ = session.ensure_stub_for_function("openssl_encrypt"); // openssl
            let _ = session.ensure_stub_for_function("json_encode"); // json
            let _ = session.ensure_stub_for_class("\\ReflectionClass"); // Reflection
            session.loaded_stub_count()
        });
    });

    group.finish();
}

/// Concurrent-read workload: N reader threads do `definition_of` lookups in
/// a tight loop while one writer thread re-ingests Login.php at editor-typing
/// cadence. Validates the central architectural claim that
/// `AnalysisSession::snapshot_db` lets readers proceed without blocking on
/// the writer's brief lock.
///
/// Reports per-iteration wall time for a fixed batch of reads across all
/// reader threads. Lower is better; flat scaling with reader count means
/// the lock discipline is working.
fn bench_concurrent_read_under_edits(c: &mut Criterion) {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let (vendor_files, project_files) = split_vendor_project(&root);
    let cache: TempDir = tempfile::tempdir().unwrap();
    let session = Arc::new(warm_session(&cache, &vendor_files, &project_files));

    // Pick a class that exists in the warmed session so reads are cache-hot.
    let target_class = "Illuminate\\Auth\\Events\\Login";

    // Pre-load the editing target's source so the writer doesn't pay disk I/O.
    let edit_path = root.join("src/Illuminate/Auth/Events/Login.php");
    let edit_path_str: Arc<str> = Arc::from(edit_path.to_string_lossy().as_ref());
    let original = std::fs::read_to_string(&edit_path).unwrap();

    let mut group = c.benchmark_group("concurrent_read_under_edits");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(20));

    // Constants kept modest so the bench finishes in reasonable time per
    // iteration. The reader work dwarfs the writer work, so adjusting reads
    // per iteration is what controls measurement granularity.
    const READS_PER_THREAD: u32 = 5_000;
    let thread_counts = [1usize, 4, 8];

    for &n_readers in &thread_counts {
        let id = format!("{n_readers}_readers");
        let session_outer = Arc::clone(&session);
        let edit_path_outer = edit_path_str.clone();
        let original_outer = original.clone();

        group.bench_function(&id, |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let stop = Arc::new(AtomicBool::new(false));

                    // Background writer: re-ingest the target file repeatedly.
                    let writer_session = Arc::clone(&session_outer);
                    let writer_path = edit_path_outer.clone();
                    let writer_orig = original_outer.clone();
                    let writer_stop = Arc::clone(&stop);
                    let writer = thread::spawn(move || {
                        let mut counter: u32 = 0;
                        while !writer_stop.load(Ordering::Relaxed) {
                            counter = counter.wrapping_add(1);
                            let new_src: Arc<str> =
                                Arc::from(format!("{writer_orig}\n// edit {counter}\n"));
                            writer_session.ingest_file(writer_path.clone(), new_src);
                        }
                    });

                    // Spawn readers and time their combined wall-clock work.
                    let start = std::time::Instant::now();
                    let mut handles = Vec::with_capacity(n_readers);
                    for _ in 0..n_readers {
                        let s = Arc::clone(&session_outer);
                        handles.push(thread::spawn(move || {
                            for _ in 0..READS_PER_THREAD {
                                let _ = std::hint::black_box(
                                    s.definition_of(&Symbol::class(target_class)),
                                );
                            }
                        }));
                    }
                    for h in handles {
                        h.join().unwrap();
                    }
                    total += start.elapsed();

                    stop.store(true, Ordering::Relaxed);
                    writer.join().unwrap();
                }
                total
            });
        });
    }

    group.finish();

    // Restore source content.
    std::fs::write(&edit_path, &original).unwrap();
}

/// LSP-shaped cold-start: how long does a fresh `AnalysisSession::with_cache_dir`
/// take to ingest all vendor files when the persistent Pass-1 cache is warm
/// from a previous session?
///
/// Each iteration:
/// 1. Builds a fresh `AnalysisSession::with_cache_dir(persisted_dir)`.
/// 2. Loads essential stubs.
/// 3. Calls `ingest_file` for every vendor file (the real path LSP servers
///    take when warming up).
///
/// The first iteration populates the cache; subsequent iterations measure
/// the LSP cold-start the user feels every time they restart their editor.
fn bench_lsp_cold_start_warm_cache(_c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }
    let (vendor_files, _project_files) = split_vendor_project(&root);
    let cache_dir = TempDir::new().unwrap();

    eprintln!("\n=== LSP COLD-START via AnalysisSession::with_cache_dir ===\n");
    eprintln!(
        "  {} vendor files; persistent cache dir = {}\n",
        vendor_files.len(),
        cache_dir.path().display()
    );

    // Read all vendor sources up front so the timed loop measures only
    // session work — not filesystem I/O variance.
    let sources: Vec<(Arc<str>, Arc<str>)> = vendor_files
        .iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(p).ok()?;
            Some((
                Arc::from(p.to_string_lossy().as_ref()),
                Arc::from(src.as_str()),
            ))
        })
        .collect();

    let measure = |label: &str| -> Duration {
        let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
        session.ensure_essential_stubs_loaded();
        let start = std::time::Instant::now();
        for (file, src) in &sources {
            session.ingest_file(file.clone(), src.clone());
        }
        let elapsed = start.elapsed();
        eprintln!(
            "  {label:<14} ingest_file × {} = {:>7.0} ms",
            sources.len(),
            elapsed.as_secs_f64() * 1000.0
        );
        elapsed
    };

    let cold = measure("COLD");
    let warm = measure("WARM");

    let saved_pct = if cold.as_secs_f64() > 0.0 {
        (1.0 - warm.as_secs_f64() / cold.as_secs_f64()) * 100.0
    } else {
        0.0
    };
    eprintln!(
        "\n  Δ wall {:>+7.0} ms  ({:>+5.1}%)\n",
        warm.as_secs_f64() * 1000.0 - cold.as_secs_f64() * 1000.0,
        -saved_pct,
    );
}

/// Cache-hit reanalysis: call `FileAnalyzer::analyze` repeatedly on the
/// *same* unchanged source text. This is the scenario the salsa-tracked
/// `analyze_file` query is supposed to make near-free: text input is
/// identical, so salsa must return cached accumulator output without
/// re-running Pass 2.
///
/// A regression here (vs prior versions where every call ran Pass 2) means
/// the caching wiring is broken; an improvement means S5-B paid off.
fn bench_file_analyzer_cache_hit(c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }
    let (vendor_files, project_files) = split_vendor_project(&root);
    let target = root.join("src/Illuminate/Auth/Events/Login.php");
    if !target.exists() {
        eprintln!("Skipping: target Login.php not found");
        return;
    }
    let target_str = target.to_string_lossy().to_string();
    let original = std::fs::read_to_string(&target).unwrap();

    let cache: TempDir = tempfile::tempdir().unwrap();
    let session = warm_session(&cache, &vendor_files, &project_files);
    let target_arc: Arc<str> = Arc::from(target_str.as_str());
    let source_arc: Arc<str> = Arc::from(original.as_str());

    // Prime the salsa cache + ingest once so the first iteration is also a
    // cache hit (matches steady-state LSP behaviour where the file has
    // already been analyzed at least once).
    session.ingest_file(target_arc.clone(), source_arc.clone());

    let parsed = php_rs_parser::parse(source_arc.as_ref());
    let _ = FileAnalyzer::new(&session).analyze(
        target_arc.clone(),
        source_arc.as_ref(),
        &parsed.program,
        &parsed.source_map,
    );

    let mut group = c.benchmark_group("file_analyzer_cache_hit");
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("login_php_unchanged", |b| {
        b.iter(|| {
            // Caller-side parse is part of the actual FileAnalyzer API
            // contract, so measure it as part of the iteration body.

            let parsed = php_rs_parser::parse(source_arc.as_ref());
            FileAnalyzer::new(&session).analyze(
                target_arc.clone(),
                source_arc.as_ref(),
                &parsed.program,
                &parsed.source_map,
            )
        });
    });

    group.finish();
}

/// Memory probe (not a Criterion bench — uses `eprintln!` for output).
///
/// Warms a session over the full project + vendor, snapshots allocator
/// state, then runs `FileAnalyzer::analyze` once on every project file so
/// the salsa cache fills with accumulator entries (IssueAccumulator,
/// RefLocAccumulator, SymbolAccumulator). Reports the live-bytes delta
/// retained by the cache + the total bytes allocated during the loop.
///
/// Comparing this number with-and-without S5-B is the only signal for
/// "did the accumulator-based cache balloon memory?"
fn bench_file_analyzer_memory_probe(_c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }
    let (vendor_files, project_files) = split_vendor_project(&root);
    let cache: TempDir = tempfile::tempdir().unwrap();
    let session = warm_session(&cache, &vendor_files, &project_files);

    // Pre-load all project sources to remove I/O variance from the timed loop.
    let sources: Vec<(Arc<str>, Arc<str>)> = project_files
        .iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(p).ok()?;
            Some((
                Arc::from(p.to_string_lossy().as_ref()),
                Arc::from(src.as_str()),
            ))
        })
        .collect();

    eprintln!(
        "\n=== FileAnalyzer MEMORY PROBE ({} project files) ===",
        sources.len()
    );

    // Snapshot live bytes after warmup, before the FileAnalyzer loop.
    // (Total allocated is reset so we measure only the loop's churn.)
    let (live_before, _, _) = snapshot_alloc();
    reset_alloc_counters();
    G_LIVE.store((live_before * 1_048_576.0) as i64, Relaxed);
    G_PEAK.store((live_before * 1_048_576.0) as i64, Relaxed);

    let start = std::time::Instant::now();
    let mut analyzed = 0usize;
    for (file, source) in &sources {
        let parsed = php_rs_parser::parse(source.as_ref());
        if !parsed.errors.is_empty() {
            continue;
        }
        let _ = FileAnalyzer::new(&session).analyze(
            file.clone(),
            source.as_ref(),
            &parsed.program,
            &parsed.source_map,
        );
        analyzed += 1;
    }
    let elapsed = start.elapsed();
    let (live_after, peak_after, total_after) = snapshot_alloc();

    let retained_delta = live_after - live_before;
    eprintln!(
        "  analyzed {} files in {:.0} ms",
        analyzed,
        elapsed.as_secs_f64() * 1000.0
    );
    eprintln!(
        "  live bytes:    before {:>7.1} MiB → after {:>7.1} MiB    (retained Δ: {:>+7.1} MiB)",
        live_before, live_after, retained_delta
    );
    eprintln!(
        "  peak live:     {:>7.1} MiB    total allocated during loop: {:>7.1} MiB\n",
        peak_after, total_after
    );
}

criterion_group!(
    benches,
    bench_single_file_edit,
    bench_high_fanout_edit,
    bench_file_analyzer_cache_hit,
    bench_file_analyzer_memory_probe,
    bench_read_query_latency,
    bench_stub_loading,
    bench_concurrent_read_under_edits,
    bench_lsp_cold_start_warm_cache,
);
criterion_main!(benches);
