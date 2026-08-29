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
//!   6. Cold vs warm diagnostics
//!   7. Hover after diagnostics
//!   8. Parallel dependent re-analysis on save
//!   9. Edit-locality and retained-memory metrics snapshot

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{
    perf_fixture::PerfFixture, AnalysisSession, BatchOptions, Name, PhpVersion, Psr4Map,
};

fn fixture_available(fixture: &PerfFixture) -> bool {
    if !fixture.has_full_corpus() {
        eprintln!(
            "\nSkipping perf analysis: fixture not at {}\n\
             Provide MIR_PERF_FIXTURE/MIR_LARAVEL_FIXTURE/MIR_SYMFONY_FIXTURE,\n\
             or run: bash crates/mir-analyzer/benches/download-fixtures.sh\n",
            fixture.root().display()
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

fn print_metrics_lines(title: &str, dump: &str, prefixes: &[&str]) {
    println!("  {title}");
    for line in dump.lines() {
        if prefixes.iter().any(|prefix| line.starts_with(prefix)) {
            println!("{line}");
        }
    }
}

#[test]
#[ignore]
fn perf_analysis_full_report() {
    let Some(fixture) = PerfFixture::discover() else {
        eprintln!(
            "\nSkipping perf analysis: no supported perf fixture found\n\
             Looked for MIR_PERF_FIXTURE, MIR_LARAVEL_FIXTURE, MIR_SYMFONY_FIXTURE,\n\
             and local benches/fixtures/{{laravel,symfony}}\n"
        );
        return;
    };
    if !fixture_available(&fixture) {
        return;
    }
    let root = fixture.root();
    let src_files = discover_php(&fixture.src_root());
    let vendor_files = discover_php(&fixture.vendor_root());
    let total = src_files.len() + vendor_files.len();

    println!("\n╔══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                  mir-analyzer Performance Analysis                           ║");
    println!("║                       Fixture: {:<45}║", fixture.label());
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
    let _result = analyzer.analyze_paths(&src_files, &BatchOptions::new().without_symbols());
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

    let composer_root = root.to_path_buf();
    let t0 = Instant::now();
    let session = match Psr4Map::from_composer(&composer_root) {
        Ok(map) => AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(map)),
        Err(_) => AnalysisSession::new(PhpVersion::LATEST),
    };
    let session_new = t0.elapsed();
    print_row("AnalysisSession::new + psr4", session_new, "");

    // Pick a representative file to "open"
    let open_path = fixture.open_file();
    let open_source = std::fs::read_to_string(&open_path).unwrap_or_else(|_| "<?php\n".to_string());
    let open_arc: Arc<str> = Arc::from(open_path.to_string_lossy().as_ref());

    let t0 = Instant::now();
    session.ingest_file(open_arc.clone(), Arc::from(open_source.as_str()));
    let ingest_one = t0.elapsed();
    print_row(
        "ingest_file(open file)",
        ingest_one,
        fixture.open_file_label(),
    );

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

    for target in fixture.lazy_load_targets() {
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
    println!(
        "  Pending imports for {}: {}",
        fixture.open_file_label(),
        pending.len()
    );
    let t0 = Instant::now();
    let loaded = session.prefetch_imports(open_arc.as_ref());
    let prefetch_time = t0.elapsed();
    print_row(
        "prefetch_imports",
        prefetch_time,
        &format!("{loaded} classes loaded"),
    );

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 7: Cold vs warm diagnostics
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 7 — Cold vs warm diagnostics");
    println!("  Run open-file diagnostics twice on the same ingested text to separate");
    println!("  first-run cost from the warm repeat after caches and indexes exist.");
    println!();

    let parsed = php_rs_parser::parse(&open_source);
    let t0 = Instant::now();
    let first_diagnostics = mir_analyzer::FileAnalyzer::new(&session).analyze_diagnostics_only(
        open_arc.clone(),
        &open_source,
        &parsed.program,
        &parsed.source_map,
    );
    let first_diagnostics_time = t0.elapsed();
    print_row(
        "first diagnostics_only",
        first_diagnostics_time,
        &format!("{} issues", first_diagnostics.issues.len()),
    );

    let t0 = Instant::now();
    let second_diagnostics = mir_analyzer::FileAnalyzer::new(&session).analyze_diagnostics_only(
        open_arc.clone(),
        &open_source,
        &parsed.program,
        &parsed.source_map,
    );
    let second_diagnostics_time = t0.elapsed();
    print_row(
        "repeat diagnostics_only",
        second_diagnostics_time,
        &format!("{} issues", second_diagnostics.issues.len()),
    );
    let diagnostics_speedup =
        first_diagnostics_time.as_secs_f64() / second_diagnostics_time.as_secs_f64();
    println!();
    println!(
        "  ┃ Warm diagnostics speedup: {:.2}× ({} → {})",
        diagnostics_speedup,
        fmt_ms(first_diagnostics_time),
        fmt_ms(second_diagnostics_time)
    );

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 8: Cursor-name resolution after diagnostics
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 8 — Name lookup after diagnostics");
    println!("  Reuse the post-diagnostics open file and resolve the codebase name");
    println!("  at a concrete cursor offset without retaining whole-file symbols.");
    println!();

    if let Some(offset) = open_source.find(fixture.open_symbol_probe()) {
        let name_offset = offset as u32;
        let t0 = Instant::now();
        let first_name = session.name_at(open_arc.as_ref(), name_offset);
        let first_name_time = t0.elapsed();
        print_row(
            "first name_at",
            first_name_time,
            if first_name.is_some() {
                "after diagnostics"
            } else {
                "no symbol at chosen offset"
            },
        );

        let t0 = Instant::now();
        let second_name = session.name_at(open_arc.as_ref(), name_offset);
        let repeat_name_time = t0.elapsed();
        print_row(
            "repeat name_at",
            repeat_name_time,
            if second_name.is_some() {
                "warm repeat"
            } else {
                "no symbol at chosen offset"
            },
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 9: Hover after diagnostics
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 9 — Hover after diagnostics");
    println!("  Reuse the same post-diagnostics cursor position to resolve");
    println!("  hover info through the targeted navigation path.");
    println!();

    if let Some(offset) = open_source.find(fixture.open_symbol_probe()) {
        let hover_offset = offset as u32;
        let t0 = Instant::now();
        let first_hover = session.hover_at(open_arc.as_ref(), hover_offset);
        let first_hover_time = t0.elapsed();
        print_row(
            "first hover_at",
            first_hover_time,
            if first_hover.is_ok() {
                "after diagnostics"
            } else {
                "no symbol at chosen offset"
            },
        );

        let t0 = Instant::now();
        let second_hover = session.hover_at(open_arc.as_ref(), hover_offset);
        let repeat_hover_time = t0.elapsed();
        print_row(
            "repeat hover_at",
            repeat_hover_time,
            if second_hover.is_ok() {
                "warm repeat"
            } else {
                "no symbol at chosen offset"
            },
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Scenario 10: Parallel dependent re-analysis on save
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 10 — Parallel dependent re-analysis on save");
    println!("  After ingesting a base class, re-analyze its dependents in parallel.");
    println!();

    // Pick a high-fanout file
    let base_path = fixture.high_fanout_file();
    let base_arc: Arc<str> = Arc::from(base_path.to_string_lossy().as_ref());
    if let Ok(src) = std::fs::read_to_string(&base_path) {
        let t0 = Instant::now();
        session.ingest_file(base_arc.clone(), Arc::from(src.as_str()));
        let ingest_base = t0.elapsed();
        print_row(
            &format!(
                "ingest_file({})",
                base_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("target.php")
            ),
            ingest_base,
            "",
        );

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
    // Scenario 11: Metrics snapshot for locality + retained memory
    // ─────────────────────────────────────────────────────────────────────────
    print_header("Scenario 11 — Edit locality and retained memory snapshot");
    println!("  Enable with MIR_TIMING=1 to capture how much work was avoided and");
    println!("  how much Salsa-retained state each canonical query shape keeps.");
    println!();

    match mir_analyzer::metrics::dump() {
        Some(dump) => {
            print_metrics_lines(
                "Locality counters:",
                &dump,
                &[
                    "  whole-file walks",
                    "  scopes analyzed",
                    "  symbols allocated",
                    "  top body walks/file:",
                    "  top scopes analyzed/file:",
                ],
            );
            print_metrics_lines(
                "Retained memory:",
                &dump,
                &[
                    "  retained/analyze_file:",
                    "  retained/infer_scope :",
                    "  retained/infer_fn    :",
                ],
            );
        }
        None => {
            println!("  metrics disabled; rerun with `MIR_TIMING=1` to print locality and retained-memory counters");
        }
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
    println!(
        "  Diagnostics: {} → {}  ({:.2}× speedup)",
        fmt_ms(first_diagnostics_time),
        fmt_ms(second_diagnostics_time),
        diagnostics_speedup
    );
    println!("  Hover:       first + repeat measured after diagnostics");
    println!("  Locality:    metrics snapshot printed when MIR_TIMING=1");
    println!();
}
