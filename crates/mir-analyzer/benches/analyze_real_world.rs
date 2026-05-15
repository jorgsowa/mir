use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use mir_analyzer::ProjectAnalyzer;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering::Relaxed};
use std::time::Duration;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Counting allocator — per-thread accumulators, global flush on stats read
//
// Threads accumulate alloc/dealloc deltas in thread-local Cells (zero
// contention). Globals are only written when flushing for stats output, so
// the hot path (alloc/dealloc during benchmarks) has no shared-atomic
// cache-line bouncing regardless of thread count.
// ---------------------------------------------------------------------------

struct CountingAllocator;

// Thread-local accumulators — never shared, never contended.
thread_local! {
    static TL_LIVE:  Cell<i64>   = const { Cell::new(0) };
    static TL_TOTAL: Cell<usize> = const { Cell::new(0) };
    static TL_PEAK:  Cell<usize> = const { Cell::new(0) };
}

// Globals written only during flush (single-threaded stats path).
static G_LIVE: AtomicI64 = AtomicI64::new(0);
static G_PEAK: AtomicUsize = AtomicUsize::new(0);
static G_TOTAL: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            let sz = layout.size();
            TL_LIVE.with(|c| c.set(c.get() + sz as i64));
            TL_TOTAL.with(|c| c.set(c.get() + sz));
            TL_PEAK.with(|c| {
                let live = TL_LIVE.with(Cell::get) as usize;
                if live > c.get() {
                    c.set(live);
                }
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        TL_LIVE.with(|c| c.set(c.get() - layout.size() as i64));
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Flush thread-local counters into the global aggregates and zero them.
/// Call this on every thread that participated before reading globals.
fn flush_thread_counters() {
    let live = TL_LIVE.with(Cell::get);
    let total = TL_TOTAL.with(Cell::get);
    let peak = TL_PEAK.with(Cell::get);
    G_LIVE.fetch_add(live, Relaxed);
    G_TOTAL.fetch_add(total, Relaxed);
    G_PEAK.fetch_max(peak, Relaxed);
    TL_LIVE.with(|c| c.set(0));
    TL_TOTAL.with(|c| c.set(0));
    TL_PEAK.with(|c| c.set(0));
}

fn reset_alloc_counters() {
    G_LIVE.store(0, Relaxed);
    G_PEAK.store(0, Relaxed);
    G_TOTAL.store(0, Relaxed);
    TL_LIVE.with(|c| c.set(0));
    TL_TOTAL.with(|c| c.set(0));
    TL_PEAK.with(|c| c.set(0));
}

fn print_alloc_stats(label: &str) {
    flush_thread_counters();
    let peak = G_PEAK.load(Relaxed) as f64 / 1_048_576.0;
    let total = G_TOTAL.load(Relaxed) as f64 / 1_048_576.0;
    eprintln!(
        "  [memory] {label}: peak live {peak:.1} MiB, total allocated {total:.1} MiB  (one cold run)"
    );
}

fn checkpoint_alloc(label: &str) {
    flush_thread_counters();
    let peak = G_PEAK.load(Relaxed) as f64 / 1_048_576.0;
    let total = G_TOTAL.load(Relaxed) as f64 / 1_048_576.0;
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
    let vendor_files = ProjectAnalyzer::discover_files(&root.join("vendor"));
    let project_files = ProjectAnalyzer::discover_files(&root.join("src"));
    (vendor_files, project_files)
}

/// Run the full pipeline once into `cache_dir` so subsequent analyses can use
/// cached results.
fn warm_cache(cache_dir: &TempDir, vendor_files: &[PathBuf], project_files: &[PathBuf]) {
    let analyzer = ProjectAnalyzer::with_cache(cache_dir.path());
    analyzer.load_stubs();
    analyzer.collect_types_only(vendor_files);
    let _ = analyzer.analyze(project_files);
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
        let analyzer = ProjectAnalyzer::new();
        checkpoint_alloc("after new analyzer");
        analyzer.load_stubs();
        checkpoint_alloc("after load_stubs");
        analyzer.collect_types_only(&vendor_files);
        checkpoint_alloc("after collect_types_only (vendor)");
        let _ = analyzer.analyze(&project_files);
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
                    let analyzer = ProjectAnalyzer::new();
                    analyzer.load_stubs();
                    analyzer.collect_types_only(&vendor_files);
                    analyzer.analyze(&project_files)
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
            let analyzer = ProjectAnalyzer::with_cache(cache_mem.path());
            analyzer.load_stubs();
            analyzer.collect_types_only(&vendor_files);
            let _ = analyzer.analyze(&project_files);
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
            let analyzer = ProjectAnalyzer::with_cache(cache_mem.path());
            analyzer.load_stubs();
            analyzer.collect_types_only(&vendor_files);
            let _ = analyzer.analyze(&project_files);
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
                    let analyzer = ProjectAnalyzer::with_cache(cache_model.path());
                    analyzer.load_stubs();
                    analyzer.collect_types_only(&vendor_files);
                    analyzer.analyze(&project_files)
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
                    let analyzer = ProjectAnalyzer::with_cache(cache_leaf.path());
                    analyzer.load_stubs();
                    analyzer.collect_types_only(&vendor_files);
                    analyzer.analyze(&project_files)
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
        let mem_analyzer = ProjectAnalyzer::with_cache(cache_mem.path());
        mem_analyzer.load_stubs();
        mem_analyzer.collect_types_only(&vendor_files);
        reset_alloc_counters();
        let _ = mem_analyzer.analyze(&project_files);
        print_alloc_stats("reanalysis_project_only/laravel_high_fanout");
        std::fs::write(&model_path, original).unwrap();
    }
    if let Some(original) = &leaf_original {
        let cache_mem: TempDir = tempfile::tempdir().unwrap();
        warm_cache(&cache_mem, &vendor_files, &project_files);
        std::fs::write(&leaf_path, format!("{original}\n// memory-check")).unwrap();
        let mem_analyzer = ProjectAnalyzer::with_cache(cache_mem.path());
        mem_analyzer.load_stubs();
        mem_analyzer.collect_types_only(&vendor_files);
        reset_alloc_counters();
        let _ = mem_analyzer.analyze(&project_files);
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
                    let analyzer = ProjectAnalyzer::with_cache(cache_model.path());
                    analyzer.load_stubs();
                    analyzer.collect_types_only(&vendor_files);
                    analyzer
                },
                |analyzer| analyzer.analyze(&project_files),
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
                    let analyzer = ProjectAnalyzer::with_cache(cache_leaf.path());
                    analyzer.load_stubs();
                    analyzer.collect_types_only(&vendor_files);
                    analyzer
                },
                |analyzer| analyzer.analyze(&project_files),
                BatchSize::LargeInput,
            );
        });
        std::fs::write(&leaf_path, original).unwrap();
    }

    group.finish();
}

/// Vendor type collection: stubs + `collect_types_only` across the real
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
    let vendor_files = ProjectAnalyzer::discover_files(&root.join("vendor"));

    reset_alloc_counters();
    {
        let analyzer = ProjectAnalyzer::new();
        analyzer.load_stubs();
        analyzer.collect_types_only(&vendor_files);
    }
    print_alloc_stats("vendor_collection/laravel");

    let mut group = c.benchmark_group("vendor_collection");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.throughput(Throughput::Elements(vendor_files.len() as u64));

    group.bench_function("laravel", |b| {
        b.iter(|| {
            let analyzer = ProjectAnalyzer::new();
            analyzer.load_stubs();
            analyzer.collect_types_only(&vendor_files)
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

    let vendor_files = ProjectAnalyzer::discover_files(&root.join("vendor"));

    eprintln!("\n=== VENDOR COLLECTION DETAILED PROFILING ===\n");

    reset_alloc_counters();
    let analyzer = ProjectAnalyzer::new();
    checkpoint_alloc("After analyzer::new()");

    analyzer.load_stubs();
    checkpoint_alloc("After load_stubs()");

    analyzer.collect_types_only(&vendor_files);
    checkpoint_alloc("After collect_types_only() - TOTAL VENDOR ALLOCATION");

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
    use mir_types::union::Union;
    eprintln!("  [struct sizes]");
    eprintln!(
        "    size_of::<Union>()                    = {} bytes",
        std::mem::size_of::<Union>()
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
        "    size_of::<Option<Union>>()            = {} bytes",
        std::mem::size_of::<Option<Union>>()
    );
    eprintln!(
        "    size_of::<Option<Arc<Union>>>()       = {} bytes",
        std::mem::size_of::<Option<std::sync::Arc<Union>>>()
    );
    eprintln!();

    reset_alloc_counters();
    let analyzer = ProjectAnalyzer::new();
    checkpoint_alloc("After analyzer::new()");

    analyzer.load_stubs();
    checkpoint_alloc("After load_stubs()");

    analyzer.collect_types_only(&vendor_files);
    checkpoint_alloc("After collect_types_only() - VENDOR COLLECTION");

    let _ = analyzer.analyze(&project_files);
    checkpoint_alloc("After analyze() - FULL ANALYSIS COMPLETE");

    eprintln!();
}

/// Vendor collection with finer-grained breakdown (file parsing vs ingestion).
fn bench_vendor_collection_phase_breakdown(_c: &mut Criterion) {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let vendor_files = ProjectAnalyzer::discover_files(&root.join("vendor"));
    eprintln!("\n=== VENDOR COLLECTION PHASE BREAKDOWN ===\n");
    eprintln!("  {} vendor files to collect\n", vendor_files.len());

    reset_alloc_counters();
    {
        let analyzer = ProjectAnalyzer::new();
        analyzer.load_stubs();
        checkpoint_alloc("After load_stubs()");

        // Just loading and parsing files (no ingestion yet)
        analyzer.collect_types_only(&vendor_files);
        checkpoint_alloc("After collect_types_only() [parse + ingest complete]");
    }
    eprintln!();
}

criterion_group!(
    benches,
    bench_full_analysis,
    bench_reanalysis,
    bench_reanalysis_project_only,
    bench_vendor_collection,
    bench_vendor_collection_detailed,
    bench_full_analysis_detailed,
    bench_vendor_collection_phase_breakdown
);
criterion_main!(benches);
