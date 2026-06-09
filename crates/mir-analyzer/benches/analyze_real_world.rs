use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use mir_analyzer::{discover_files, AnalysisSession, BatchOptions, PhpVersion};
use std::alloc::{GlobalAlloc, Layout};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Counting allocator — wraps mimalloc (per-thread arenas, low lock contention)
// and tracks live/peak/total bytes across all threads.
// ---------------------------------------------------------------------------

struct CountingAllocator;

static INNER: mimalloc::MiMalloc = mimalloc::MiMalloc;

// G_LIVE  — current net live bytes (delta from last reset)
// G_PEAK  — max G_LIVE since last reset
// G_TOTAL — cumulative bytes allocated since last reset
// Each counter is padded to a full cache line (64 bytes) so the three
// hot-path atomics don't share a cache line and cause false sharing under
// multi-threaded bench runs.
#[repr(align(64))]
struct CacheAligned<T>(T);
static G_LIVE: CacheAligned<AtomicI64> = CacheAligned(AtomicI64::new(0));
static G_PEAK: CacheAligned<AtomicI64> = CacheAligned(AtomicI64::new(0));
static G_TOTAL: CacheAligned<AtomicUsize> = CacheAligned(AtomicUsize::new(0));

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = INNER.alloc(layout);
        if !ptr.is_null() {
            let sz = layout.size();
            G_TOTAL.0.fetch_add(sz, Relaxed);
            let new_live = G_LIVE.0.fetch_add(sz as i64, Relaxed) + sz as i64;
            G_PEAK.0.fetch_max(new_live, Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        INNER.dealloc(ptr, layout);
        G_LIVE.0.fetch_sub(layout.size() as i64, Relaxed);
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn reset_alloc_counters() {
    G_LIVE.0.store(0, Relaxed);
    G_PEAK.0.store(0, Relaxed);
    G_TOTAL.0.store(0, Relaxed);
}

fn print_alloc_stats(label: &str) {
    let peak = G_PEAK.0.load(Relaxed) as f64 / 1_048_576.0;
    let total = G_TOTAL.0.load(Relaxed) as f64 / 1_048_576.0;
    eprintln!(
        "  [memory] {label}: peak live {peak:.1} MiB, total allocated {total:.1} MiB  (one cold run)"
    );
}

fn checkpoint_alloc(label: &str) {
    let peak = G_PEAK.0.load(Relaxed) as f64 / 1_048_576.0;
    let total = G_TOTAL.0.load(Relaxed) as f64 / 1_048_576.0;
    eprintln!("    [checkpoint] {label:40} peak {peak:7.1} MiB, total {total:7.1} MiB");
}

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures/laravel")
}

/// Returns true (and prints a message) when the fixture is absent or
/// `composer install` has not been run yet.
fn skip_if_missing(root: &Path) -> bool {
    let src = root.join("src");
    let vendor = root.join("vendor");
    if !src.exists() || !vendor.exists() {
        eprintln!(
            "\nSkipping benchmark: fixture not found or incomplete at {}\n\
             Run once to download and install it:\n\
             \n    bash crates/mir-analyzer/benches/download-fixtures.sh\n",
            root.display()
        );
        true
    } else {
        false
    }
}

/// Split files into (vendor, project) using the real composer-installed
/// `vendor/` directory and the framework's own `src/` directory — the same
/// split the CLI performs for a composer-managed project.
fn split_vendor_project(root: &Path) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let vendor_files = discover_files(&root.join("vendor"));
    let project_files = discover_files(&root.join("src"));
    (vendor_files, project_files)
}

/// Run the full pipeline once into `cache_dir` so subsequent analyses can use
/// cached results.
fn warm_cache(cache_dir: &TempDir, vendor_files: &[PathBuf], project_files: &[PathBuf]) {
    let analyzer = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    analyzer.ensure_all_stubs();
    analyzer.collect_definitions(vendor_files);
    let _ = analyzer.analyze_paths(project_files, &BatchOptions::new());
}

// ---------------------------------------------------------------------------
// Benchmarks
//
// NOTE: Results are only meaningful under the `bench` profile (release-
// equivalent). Running under debug (`cargo test --bench`) produces numbers
// that are 5–10× slower and should be ignored.
// ---------------------------------------------------------------------------

/// Cold-start full pipeline: stubs + vendor type collection + Pass 1 +
/// codebase finalization + Pass 2. No cache.
///
/// Benchmarked at both 1 thread and the default thread count so that
/// parallelism scaling (or contention regressions) are visible.
///
/// Memory stats for one cold run are printed to stderr before the timed loop.
fn bench_full_analysis(c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let (vendor_files, project_files) = split_vendor_project(&root);
    assert!(
        !project_files.is_empty(),
        "No project PHP files found under {}",
        root.display()
    );

    // Print memory stats once before the Criterion loop.
    reset_alloc_counters();
    {
        let analyzer = AnalysisSession::new(PhpVersion::LATEST);
        checkpoint_alloc("after new analyzer");
        analyzer.ensure_all_stubs();
        checkpoint_alloc("after load_stubs");
        analyzer.collect_definitions(&vendor_files);
        checkpoint_alloc("after collect_definitions (vendor)");
        let _ = analyzer.analyze_paths(&project_files, &BatchOptions::new());
        checkpoint_alloc("after analyze (project)");
    }
    print_alloc_stats("full_analysis/laravel");

    let num_threads = rayon::current_num_threads();
    let mut thread_counts = vec![1usize, num_threads];
    thread_counts.dedup(); // avoid duplicate variant on single-core machines

    let mut group = c.benchmark_group("full_analysis");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(60));
    group.throughput(Throughput::Elements(project_files.len() as u64));

    for &threads in &thread_counts {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .stack_size(16 * 1024 * 1024)
            .build()
            .unwrap();

        group.bench_function(BenchmarkId::new("laravel", format!("{threads}t")), |b| {
            b.iter(|| {
                pool.install(|| {
                    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
                    analyzer.ensure_all_stubs();
                    analyzer.collect_definitions(&vendor_files);
                    analyzer.analyze_paths(&project_files, &BatchOptions::new())
                })
            })
        });
    }

    group.finish();
}

/// Incremental re-analysis with a warm cache, as the CLI experiences it: every
/// iteration rebuilds the codebase from scratch (stubs + vendor collection +
/// project analysis), but project files that have not changed are served from
/// the cache. Two variants show worst-case and best-case invalidation:
///
/// * `high_fanout` — touches `Model.php`, a large base class with many
///   subclasses. `evict_with_dependents` cascades widely; worst-case path.
///
/// * `leaf_file` — touches `Auth/Events/Login.php`, an event class with no
///   dependents. Only one file is re-analysed; best-case path.
///
/// Vendor-reload cost is intentionally included because the CLI always re-runs
/// it. See `bench_reanalysis_project_only` to isolate the cache benefit alone.
///
/// Memory stats for one warm re-analysis run are printed to stderr.
fn bench_reanalysis(c: &mut Criterion) {
    let root = fixtures_root();
    // Bug fix: check both src/ and vendor/, not just root existence.
    if skip_if_missing(&root) {
        return;
    }

    let (vendor_files, project_files) = split_vendor_project(&root);

    let model_path = root.join("src/Illuminate/Database/Eloquent/Model.php");
    let leaf_path = root.join("src/Illuminate/Auth/Events/Login.php");

    let model_original = model_path
        .exists()
        .then(|| std::fs::read_to_string(&model_path).unwrap());
    let leaf_original = leaf_path
        .exists()
        .then(|| std::fs::read_to_string(&leaf_path).unwrap());

    if model_original.is_none() && leaf_original.is_none() {
        eprintln!("\nSkipping reanalysis: neither target file found");
        return;
    }

    // Memory snapshot: one warm re-analysis run before the Criterion loop.
    if let Some(original) = &model_original {
        let cache_mem: TempDir = tempfile::tempdir().unwrap();
        warm_cache(&cache_mem, &vendor_files, &project_files);
        std::fs::write(&model_path, format!("{original}\n// memory-check")).unwrap();
        reset_alloc_counters();
        {
            let analyzer =
                AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_mem.path());
            analyzer.ensure_all_stubs();
            analyzer.collect_definitions(&vendor_files);
            let _ = analyzer.analyze_paths(&project_files, &BatchOptions::new());
        }
        print_alloc_stats("reanalysis/laravel_high_fanout");
        std::fs::write(&model_path, original).unwrap();
    }
    if let Some(original) = &leaf_original {
        let cache_mem: TempDir = tempfile::tempdir().unwrap();
        warm_cache(&cache_mem, &vendor_files, &project_files);
        std::fs::write(&leaf_path, format!("{original}\n// memory-check")).unwrap();
        reset_alloc_counters();
        {
            let analyzer =
                AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_mem.path());
            analyzer.ensure_all_stubs();
            analyzer.collect_definitions(&vendor_files);
            let _ = analyzer.analyze_paths(&project_files, &BatchOptions::new());
        }
        print_alloc_stats("reanalysis/laravel_leaf_file");
        std::fs::write(&leaf_path, original).unwrap();
    }

    // Separate cache dirs so the two variants don't interfere with each other.
    let cache_model: TempDir = tempfile::tempdir().unwrap();
    let cache_leaf: TempDir = tempfile::tempdir().unwrap();
    if model_original.is_some() {
        warm_cache(&cache_model, &vendor_files, &project_files);
    }
    if leaf_original.is_some() {
        warm_cache(&cache_leaf, &vendor_files, &project_files);
    }

    let mut counter_model = 0u32;
    let mut counter_leaf = 0u32;

    let mut group = c.benchmark_group("reanalysis");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.throughput(Throughput::Elements(project_files.len() as u64));

    if let Some(original) = &model_original {
        group.bench_function("laravel_high_fanout", |b| {
            b.iter_batched(
                || {
                    // Not timed: touch the file so its content hash changes.
                    counter_model += 1;
                    std::fs::write(&model_path, format!("{original}\n// bench {counter_model}"))
                        .unwrap();
                },
                |_| {
                    let analyzer =
                        AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_model.path());
                    analyzer.ensure_all_stubs();
                    analyzer.collect_definitions(&vendor_files);
                    analyzer.analyze_paths(&project_files, &BatchOptions::new())
                },
                BatchSize::LargeInput,
            );
        });
        std::fs::write(&model_path, original).unwrap();
    }

    if let Some(original) = &leaf_original {
        group.bench_function("laravel_leaf_file", |b| {
            b.iter_batched(
                || {
                    counter_leaf += 1;
                    std::fs::write(&leaf_path, format!("{original}\n// bench {counter_leaf}"))
                        .unwrap();
                },
                |_| {
                    let analyzer =
                        AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_leaf.path());
                    analyzer.ensure_all_stubs();
                    analyzer.collect_definitions(&vendor_files);
                    analyzer.analyze_paths(&project_files, &BatchOptions::new())
                },
                BatchSize::LargeInput,
            );
        });
        std::fs::write(&leaf_path, original).unwrap();
    }

    group.finish();
}

/// Isolates the pure cache benefit: vendor types are pre-loaded in the untimed
/// setup, so each timed iteration measures only `analyze()`. Directly
/// comparable to the `analyze()` portion of `full_analysis` without the
/// constant vendor-reload cost that would otherwise compress the signal.
fn bench_reanalysis_project_only(c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let (vendor_files, project_files) = split_vendor_project(&root);

    let model_path = root.join("src/Illuminate/Database/Eloquent/Model.php");
    let leaf_path = root.join("src/Illuminate/Auth/Events/Login.php");

    let model_original = model_path
        .exists()
        .then(|| std::fs::read_to_string(&model_path).unwrap());
    let leaf_original = leaf_path
        .exists()
        .then(|| std::fs::read_to_string(&leaf_path).unwrap());

    if model_original.is_none() && leaf_original.is_none() {
        return;
    }

    // Memory stats: vendor pre-loaded outside the timed section, measure only analyze().
    if let Some(original) = &model_original {
        let cache_mem: TempDir = tempfile::tempdir().unwrap();
        warm_cache(&cache_mem, &vendor_files, &project_files);
        std::fs::write(&model_path, format!("{original}\n// memory-check")).unwrap();
        let mem_analyzer =
            AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_mem.path());
        mem_analyzer.ensure_all_stubs();
        mem_analyzer.collect_definitions(&vendor_files);
        reset_alloc_counters();
        let _ = mem_analyzer.analyze_paths(&project_files, &BatchOptions::new());
        print_alloc_stats("reanalysis_project_only/laravel_high_fanout");
        std::fs::write(&model_path, original).unwrap();
    }
    if let Some(original) = &leaf_original {
        let cache_mem: TempDir = tempfile::tempdir().unwrap();
        warm_cache(&cache_mem, &vendor_files, &project_files);
        std::fs::write(&leaf_path, format!("{original}\n// memory-check")).unwrap();
        let mem_analyzer =
            AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_mem.path());
        mem_analyzer.ensure_all_stubs();
        mem_analyzer.collect_definitions(&vendor_files);
        reset_alloc_counters();
        let _ = mem_analyzer.analyze_paths(&project_files, &BatchOptions::new());
        print_alloc_stats("reanalysis_project_only/laravel_leaf_file");
        std::fs::write(&leaf_path, original).unwrap();
    }

    let cache_model: TempDir = tempfile::tempdir().unwrap();
    let cache_leaf: TempDir = tempfile::tempdir().unwrap();
    if model_original.is_some() {
        warm_cache(&cache_model, &vendor_files, &project_files);
    }
    if leaf_original.is_some() {
        warm_cache(&cache_leaf, &vendor_files, &project_files);
    }

    let mut counter_model = 0u32;
    let mut counter_leaf = 0u32;

    let mut group = c.benchmark_group("reanalysis_project_only");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.throughput(Throughput::Elements(project_files.len() as u64));

    if let Some(original) = &model_original {
        group.bench_function("laravel_high_fanout", |b| {
            b.iter_batched(
                || {
                    // Not timed: touch file and pre-load stubs + vendor types into
                    // a fresh analyzer so only `analyze()` is measured.
                    counter_model += 1;
                    std::fs::write(&model_path, format!("{original}\n// bench {counter_model}"))
                        .unwrap();
                    let analyzer =
                        AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_model.path());
                    analyzer.ensure_all_stubs();
                    analyzer.collect_definitions(&vendor_files);
                    analyzer
                },
                |analyzer| analyzer.analyze_paths(&project_files, &BatchOptions::new()),
                BatchSize::LargeInput,
            );
        });
        std::fs::write(&model_path, original).unwrap();
    }

    if let Some(original) = &leaf_original {
        group.bench_function("laravel_leaf_file", |b| {
            b.iter_batched(
                || {
                    counter_leaf += 1;
                    std::fs::write(&leaf_path, format!("{original}\n// bench {counter_leaf}"))
                        .unwrap();
                    let analyzer =
                        AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_leaf.path());
                    analyzer.ensure_all_stubs();
                    analyzer.collect_definitions(&vendor_files);
                    analyzer
                },
                |analyzer| analyzer.analyze_paths(&project_files, &BatchOptions::new()),
                BatchSize::LargeInput,
            );
        });
        std::fs::write(&leaf_path, original).unwrap();
    }

    group.finish();
}

/// Vendor type collection: stubs + `collect_definitions` across the real
/// composer-installed `vendor/` directory, no body analysis. Isolates the cost
/// of loading third-party type definitions before project analysis.
///
/// Note: unlike the project Pass 1 inside `full_analysis`, this skips the FQCN
/// pre-index step, so it measures a slightly narrower slice of the pipeline.
fn bench_vendor_collection(c: &mut Criterion) {
    let root = fixtures_root();
    // Bug fix: check vendor/ specifically, not just root existence.
    if skip_if_missing(&root) {
        return;
    }

    // Bug fix: collect from vendor/ only, not the entire repo root.
    let vendor_files = discover_files(&root.join("vendor"));

    reset_alloc_counters();
    {
        let analyzer = AnalysisSession::new(PhpVersion::LATEST);
        analyzer.ensure_all_stubs();
        analyzer.collect_definitions(&vendor_files);
    }
    print_alloc_stats("vendor_collection/laravel");

    let mut group = c.benchmark_group("vendor_collection");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.throughput(Throughput::Elements(vendor_files.len() as u64));

    group.bench_function("laravel", |b| {
        b.iter(|| {
            let analyzer = AnalysisSession::new(PhpVersion::LATEST);
            analyzer.ensure_all_stubs();
            analyzer.collect_definitions(&vendor_files)
        });
    });

    group.finish();
}

/// Detailed memory profiling: measure allocation at each phase of vendor collection.
fn bench_vendor_collection_detailed(_c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let vendor_files = discover_files(&root.join("vendor"));

    eprintln!("\n=== VENDOR COLLECTION DETAILED PROFILING ===\n");

    reset_alloc_counters();
    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    checkpoint_alloc("After analyzer::new()");

    analyzer.ensure_all_stubs();
    checkpoint_alloc("After load_stubs()");

    analyzer.collect_definitions(&vendor_files);
    checkpoint_alloc("After collect_definitions() - TOTAL VENDOR ALLOCATION");

    eprintln!();
}

/// Full analysis with detailed phase breakdown.
fn bench_full_analysis_detailed(_c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let (vendor_files, project_files) = split_vendor_project(&root);

    eprintln!("\n=== FULL ANALYSIS DETAILED PROFILING ===\n");

    // Print struct sizes for optimization profiling
    use mir_codebase::storage::FnParam as CodebaseFnParam;
    use mir_types::atomic::Atomic;
    use mir_types::union::Type;
    eprintln!("  [struct sizes]");
    eprintln!(
        "    size_of::<Type>()                    = {} bytes",
        std::mem::size_of::<Type>()
    );
    eprintln!(
        "    size_of::<Atomic>()                   = {} bytes",
        std::mem::size_of::<Atomic>()
    );
    eprintln!(
        "    size_of::<CodebaseFnParam>()          = {} bytes",
        std::mem::size_of::<CodebaseFnParam>()
    );
    eprintln!(
        "    size_of::<Option<Type>>()            = {} bytes",
        std::mem::size_of::<Option<Type>>()
    );
    eprintln!(
        "    size_of::<Option<Arc<Type>>>()       = {} bytes",
        std::mem::size_of::<Option<std::sync::Arc<Type>>>()
    );
    eprintln!();

    reset_alloc_counters();
    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    checkpoint_alloc("After analyzer::new()");

    analyzer.ensure_all_stubs();
    checkpoint_alloc("After load_stubs()");

    analyzer.collect_definitions(&vendor_files);
    checkpoint_alloc("After collect_definitions() - VENDOR COLLECTION");

    let _ = analyzer.analyze_paths(&project_files, &BatchOptions::new());
    checkpoint_alloc("After analyze() - FULL ANALYSIS COMPLETE");

    eprintln!();
}

/// Vendor collection with finer-grained breakdown (file parsing vs ingestion).
fn bench_vendor_collection_phase_breakdown(_c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let vendor_files = discover_files(&root.join("vendor"));
    eprintln!("\n=== VENDOR COLLECTION PHASE BREAKDOWN ===\n");
    eprintln!("  {} vendor files to collect\n", vendor_files.len());

    reset_alloc_counters();
    {
        let analyzer = AnalysisSession::new(PhpVersion::LATEST);
        analyzer.ensure_all_stubs();
        checkpoint_alloc("After load_stubs()");

        // Just loading and parsing files (no ingestion yet)
        analyzer.collect_definitions(&vendor_files);
        checkpoint_alloc("After collect_definitions() [parse + ingest complete]");
    }
    eprintln!();
}

/// Vendor collection cold vs. warm: a persistent on-disk cache is populated
/// on the first run, and the second run reads back `StubSlice` data instead
/// of re-parsing 10 k vendor files. This is the metric users feel on every
/// repeated CLI invocation (`mir`, `mir --watch`, CI re-runs).
///
/// The cache directory persists across both runs in a single TempDir. Both
/// timings and memory checkpoints are printed.
fn bench_vendor_collection_cache_cold_vs_warm(_c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }
    let vendor_files = discover_files(&root.join("vendor"));
    let cache_dir = tempfile::tempdir().unwrap();

    eprintln!("\n=== VENDOR COLLECTION: COLD vs WARM (persistent StubSlice cache) ===\n");
    eprintln!(
        "  {} vendor files; cache dir = {}\n",
        vendor_files.len(),
        cache_dir.path().display()
    );

    // Cold run: cache is empty → every file parses + collects + writes back.
    reset_alloc_counters();
    let cold_start = std::time::Instant::now();
    {
        let analyzer = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
        analyzer.ensure_all_stubs();
        analyzer.collect_definitions(&vendor_files);
    }
    let cold = cold_start.elapsed();
    let cold_peak = G_PEAK.0.load(Relaxed) as f64 / 1_048_576.0;
    let cold_total = G_TOTAL.0.load(Relaxed) as f64 / 1_048_576.0;
    eprintln!(
        "  COLD  wall {:>7.0} ms  peak {:>6.1} MiB  churn {:>7.1} MiB",
        cold.as_secs_f64() * 1000.0,
        cold_peak,
        cold_total,
    );

    // Warm run: identical content → every cache lookup hits.
    reset_alloc_counters();
    let warm_start = std::time::Instant::now();
    let (hits, misses) = {
        let analyzer = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
        analyzer.ensure_all_stubs();
        analyzer.collect_definitions(&vendor_files);
        analyzer.stub_cache_stats()
    };
    let warm = warm_start.elapsed();
    let warm_peak = G_PEAK.0.load(Relaxed) as f64 / 1_048_576.0;
    let warm_total = G_TOTAL.0.load(Relaxed) as f64 / 1_048_576.0;
    eprintln!(
        "  WARM  wall {:>7.0} ms  peak {:>6.1} MiB  churn {:>7.1} MiB  (cache hits={hits} misses={misses})",
        warm.as_secs_f64() * 1000.0,
        warm_peak,
        warm_total,
    );
    assert!(
        hits > 0,
        "warm vendor collection must observe cache hits; saw {hits}/{misses}"
    );

    let saved_ms = cold.as_secs_f64() * 1000.0 - warm.as_secs_f64() * 1000.0;
    let saved_churn = cold_total - warm_total;
    let saved_pct = if cold.as_secs_f64() > 0.0 {
        (1.0 - warm.as_secs_f64() / cold.as_secs_f64()) * 100.0
    } else {
        0.0
    };
    eprintln!(
        "\n  Δ wall  {:>+7.0} ms  ({:>+5.1}%)\n  Δ churn {:>+7.1} MiB",
        -saved_ms, -saved_pct, -saved_churn,
    );
    eprintln!();
}

/// Memory probe for file-removal churn.
///
/// Ingests all project files into a fresh session, snapshots live bytes, then
/// removes every project file via `invalidate_file`. Measures how many bytes
/// are freed immediately vs. retained as orphaned salsa input slots.
///
/// With text-nulling: file content (Arc<str> text) is dropped on removal, so
/// retained bytes should be small (path strings + salsa slot overhead only).
/// Without text-nulling: full file text stays alive in the immortal input slot.
fn bench_file_removal_memory_probe(_c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }
    let project_files = discover_files(&root.join("src"));

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

    let total_text_bytes: usize = sources.iter().map(|(_, t)| t.len()).sum();

    eprintln!(
        "\n=== FILE-REMOVAL MEMORY PROBE ({} project files, {:.1} MiB text) ===\n",
        sources.len(),
        total_text_bytes as f64 / 1_048_576.0,
    );

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    for (path, text) in &sources {
        session.ingest_file(path.clone(), text.clone());
    }

    let live_before = G_LIVE.0.load(Relaxed);
    reset_alloc_counters();
    G_LIVE.0.store(live_before, Relaxed);
    G_PEAK.0.store(live_before, Relaxed);

    for (path, _) in &sources {
        session.invalidate_file(path.as_ref());
    }

    let live_after = G_LIVE.0.load(Relaxed);
    let freed = (live_before - live_after).max(0) as f64 / 1_048_576.0;
    let retained = live_after as f64 / 1_048_576.0;

    eprintln!(
        "  freed on removal: {:>7.1} MiB  ({:.0}% of text content)",
        freed,
        freed * 1_048_576.0 / total_text_bytes as f64 * 100.0,
    );
    eprintln!("  retained (slots): {:>7.1} MiB\n", retained);
}

criterion_group!(
    benches,
    bench_full_analysis,
    bench_reanalysis,
    bench_reanalysis_project_only,
    bench_vendor_collection,
    bench_vendor_collection_detailed,
    bench_full_analysis_detailed,
    bench_vendor_collection_phase_breakdown,
    bench_vendor_collection_cache_cold_vs_warm,
    bench_file_removal_memory_probe,
);
criterion_main!(benches);
