//! Real-workload performance analysis of mir-analyzer.
//!
//! Run with:
//!   cargo test --release --test perf_analysis -- --nocapture --ignored
//!
//! Exercises Laravel fixture (~1.4k src files, ~10k vendor files) across the
//! scenarios that matter for an LSP consumer:
//!   1. Eager warm-up (legacy path — ingest entire workspace)
//!   2. Lazy warm-up (essentials-only stubs + open one file)
//!   3. Per-edit latency (keystroke-style ingest with no concurrent snapshot)
//!   4. Per-edit latency with snapshot held (LSP serving queries during edit)
//!   5. Lazy-load on first navigation
//!   6. Parallel dependent re-analysis on save

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{AnalysisSession, BatchOptions, Name, PhpVersion, Psr4Map};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures/laravel")
}

fn fixture_available() -> bool {
    let root = fixture_root();
    let src = root.join("src");
    let vendor = root.join("vendor");
    if !src.exists() || !vendor.exists() {
        eprintln!(
            "\nSkipping perf analysis: fixture not at {}\n\
             Run: bash crates/mir-analyzer/benches/download-fixtures.sh\n",
            root.display()
        );
        false
    } else {
        true
    }
}

fn discover_php(root: &std::path::Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "php") {
                out.push(path);
            }
        }
    }
    out
}

fn fmt_ms(d: Duration) -> String {
    let ms = d.as_secs_f64() * 1000.0;
    if ms >= 1000.0 {
        format!("{:.2}s", ms / 1000.0)
    } else if ms >= 1.0 {
        format!("{:.1}ms", ms)
    } else {
        format!("{:.1}µs", ms * 1000.0)
    }
}

fn print_header(title: &str) {
    println!("\n{:━<78}", "");
    println!("  {title}");
    println!("{:━<78}", "");
}

fn print_row(label: &str, time: Duration, note: &str) {
    println!("  {:32} {:>12}   {}", label, fmt_ms(time), note);
}

#[test]
#[ignore]
fn perf_analysis_full_report() {
    if !fixture_available() {
        return;
    }
    let root = fixture_root();
    let src_files = discover_php(&root.join("src"));
    let vendor_files = discover_php(&root.join("vendor"));
    let total = src_files.len() + vendor_files.len();

    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                  mir-analyzer Performance Analysis                           ║");
    println!("║                       Fixture: Laravel                                       ║");
    println!("╚══════════════════════════════════════════════════════════════════════════════╝");
    println!("  src files:    {:>5}", src_files.len());
    println!("  vendor files: {:>5}", vendor_files.len());
    println!("  total:        {:>5}", total);

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 1: Eager warm-up (legacy)
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 1 — Eager warm-up (legacy / pre-optimization path)");
    println!("  Loads every PHP stub + ingests every src file at startup.");
    println!("  This is what the old LSP did. The 60-second pathology lives here.");
    println!();

    let t0 = Instant::now();
    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let init_time = t0.elapsed();
    print_row("AnalysisSession::new(PhpVersion::LATEST)", init_time, "");

    let t0 = Instant::now();
    analyzer.ensure_all_stubs();
    let stubs_time = t0.elapsed();
    print_row("ensure_all_stubs() (all 120)", stubs_time, "one-time cost");

    let t0 = Instant::now();
    let _result = analyzer.analyze_paths(&src_files, &BatchOptions::new());
    let analyze_src_time = t0.elapsed();
    print_row(
        "analyze(src) — 1410 files",
        analyze_src_time,
        "no vendor, no PSR-4",
    );

    let total_eager = init_time + stubs_time + analyze_src_time;
    print_row("─ TOTAL", total_eager, "before user can do anything");

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 2: Lazy warm-up (new LSP path)
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 2 — Lazy warm-up (new LSP-optimized path)");
    println!("  Essentials-only stubs + ingest a single open file. Vendor never");
    println!("  touched. PSR-4 resolver attached for lazy load on first miss.");
    println!();

    let composer_root = root.clone();
    let t0 = Instant::now();
    let session = match Psr4Map::from_composer(&composer_root) {
        Ok(map) => AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(map)),
        Err(_) => AnalysisSession::new(PhpVersion::LATEST),
    };
    let session_new = t0.elapsed();
    print_row("AnalysisSession::new + psr4", session_new, "");

    // Pick a representative file to "open"
    let open_path = root.join("src/Illuminate/Auth/Events/Login.php");
    let open_source = std::fs::read_to_string(&open_path).unwrap_or_else(|_| "<?php\n".to_string());
    let open_arc: Arc<str> = Arc::from(open_path.to_string_lossy().as_ref());

    let t0 = Instant::now();
    session.ingest_file(open_arc.clone(), Arc::from(open_source.as_str()));
    let ingest_one = t0.elapsed();
    print_row("ingest_file(open file)", ingest_one, "Login.php");

    let total_lazy = session_new + ingest_one;
    print_row("─ TOTAL", total_lazy, "user can interact NOW");

    let speedup = total_eager.as_secs_f64() / total_lazy.as_secs_f64();
    println!();
    println!(
        "  ┃ Warm-up speedup: {:.0}× faster ({} → {})",
        speedup,
        fmt_ms(total_eager),
        fmt_ms(total_lazy)
    );

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 3: Per-edit latency (no concurrent snapshot)
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 3 — Per-edit latency (keystroke-style)");
    println!("  Re-ingest the open file repeatedly. Measures the hot path the LSP");
    println!("  hits on every `didChange`.");
    println!();

    const EDIT_ITERS: u32 = 50;
    let mut samples_edit: Vec<Duration> = Vec::with_capacity(EDIT_ITERS as usize);
    for i in 0..EDIT_ITERS {
        let new_src = format!("{open_source}\n// edit {i}\n");
        let t0 = Instant::now();
        session.ingest_file(open_arc.clone(), Arc::from(new_src.as_str()));
        samples_edit.push(t0.elapsed());
    }
    samples_edit.sort();
    let p50 = samples_edit[samples_edit.len() / 2];
    let p95 = samples_edit[samples_edit.len() * 95 / 100];
    print_row("p50 ingest", p50, "median");
    print_row("p95 ingest", p95, "tail");
    print_row("min ingest", samples_edit[0], "best case");
    print_row("max ingest", *samples_edit.last().unwrap(), "worst case");

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 4: Per-edit latency WITH concurrent snapshot held
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 4 — Per-edit latency with cached snapshot held");
    println!("  LSP cached_mir_db is alive (queries on the way) while ingests run.");
    println!("  Stresses the Arc::make_mut copy-on-write path.");
    println!();

    let mut samples_held: Vec<Duration> = Vec::with_capacity(EDIT_ITERS as usize);
    for i in 0..EDIT_ITERS {
        let _snapshot_held = session
            .definition_of(&Name::class("Illuminate\\Auth\\Events\\Login"))
            .ok(); // snapshot lifetime-bound to this iteration
        let new_src = format!("{open_source}\n// held-edit {i}\n");
        let t0 = Instant::now();
        session.ingest_file(open_arc.clone(), Arc::from(new_src.as_str()));
        samples_held.push(t0.elapsed());
    }
    samples_held.sort();
    let p50_h = samples_held[samples_held.len() / 2];
    let p95_h = samples_held[samples_held.len() * 95 / 100];
    print_row("p50 ingest (snapshot held)", p50_h, "");
    print_row("p95 ingest (snapshot held)", p95_h, "");
    let overhead = p50_h.as_secs_f64() / p50.as_secs_f64();
    println!();
    println!(
        "  ┃ Snapshot-held overhead: {overhead:.2}× ({})",
        fmt_ms(p50_h)
    );

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 5: Lazy-load on first navigation
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 5 — Lazy-load on first navigation (Cmd+Click)");
    println!("  User clicks an imported vendor symbol that isn't loaded yet.");
    println!();

    let targets = [
        "Illuminate\\Foundation\\Application",
        "Illuminate\\Database\\Eloquent\\Model",
        "Illuminate\\Support\\Collection",
        "Illuminate\\Http\\Request",
    ];
    for target in &targets {
        if session.contains_class(target) {
            continue;
        }
        let t0 = Instant::now();
        let loaded = session.load_class(target).is_loaded();
        let took = t0.elapsed();
        print_row(
            &format!("lazy_load {target}"),
            took,
            if loaded {
                "✓ resolved"
            } else {
                "✗ not in PSR-4"
            },
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 6: Background prefetch
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 6 — Background prefetch of imports");
    println!("  After ingesting a file, prefetch its `use` imports so the first");
    println!("  cross-file navigation hits a warm cache.");
    println!();

    let pending = session.pending_lazy_loads(open_arc.as_ref());
    println!("  Pending imports for Login.php: {}", pending.len());
    let t0 = Instant::now();
    let loaded = session.prefetch_imports(open_arc.as_ref());
    let prefetch_time = t0.elapsed();
    print_row(
        "prefetch_imports",
        prefetch_time,
        &format!("{loaded} classes loaded"),
    );

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 7: Parallel dependent re-analysis on save
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 7 — Parallel dependent re-analysis on save");
    println!("  After ingesting a base class, re-analyze its dependents in parallel.");
    println!();

    // Pick a high-fanout file
    let base_path = root.join("src/Illuminate/Database/Eloquent/Model.php");
    let base_arc: Arc<str> = Arc::from(base_path.to_string_lossy().as_ref());
    if let Ok(src) = std::fs::read_to_string(&base_path) {
        let t0 = Instant::now();
        session.ingest_file(base_arc.clone(), Arc::from(src.as_str()));
        let ingest_base = t0.elapsed();
        print_row("ingest_file(Model.php)", ingest_base, "");

        let t0 = Instant::now();
        let results = session.reanalyze_dependents(base_arc.as_ref());
        let dep_time = t0.elapsed();
        print_row(
            "reanalyze_dependents",
            dep_time,
            &format!("{} dependents, parallel via rayon", results.len()),
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Summary
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Summary");
    println!(
        "  Warm-up:     {} → {}  ({:.0}× speedup)",
        fmt_ms(total_eager),
        fmt_ms(total_lazy),
        speedup
    );
    println!("  Per-edit p50: {}", fmt_ms(p50));
    println!(
        "  Per-edit p50 (snapshot held): {}  ({:.2}× overhead)",
        fmt_ms(p50_h),
        overhead
    );
    println!(
        "  Prefetch ({} imports): {}",
        pending.len(),
        fmt_ms(prefetch_time)
    );
    println!();
}
