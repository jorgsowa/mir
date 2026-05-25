//! Performance check for `infer_function` tracked query.
//!
//! Run with:
//!   cargo test -p mir-analyzer --test infer_function_perf --release -- --nocapture --ignored
//!
//! Measures three numbers:
//!   1. Cold call latency — first time `infer_function` is asked for a fn.
//!   2. Warm call latency — repeat call (should be memoized, ~free).
//!   3. Full-file equivalent — running the existing per-file Pass-2 walk for
//!      reference, so we can compare "per-fn × N" vs "whole file once".

use std::sync::Arc;
use std::time::Instant;

use mir_analyzer::db::{
    collect_file_definitions, infer_function, parse_file, AnalyzeFileInput, MirDatabase,
};
use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

/// 40 short free functions — enough to amortize fixed setup, small enough that
/// the file-scale walk isn't dominated by class-body work.
fn gen_source(n: usize) -> String {
    let mut s = String::from("<?php\n");
    for i in 0..n {
        s.push_str(&format!(
            "function f{i}(int $x, string $y): string {{\n    $z = $x + 1;\n    return $y . (string)$z;\n}}\n",
        ));
    }
    s
}

fn fn_names(n: usize) -> Vec<Arc<str>> {
    (0..n).map(|i| Arc::from(format!("f{i}"))).collect()
}

#[test]
#[ignore = "perf measurement; run with --release --ignored"]
fn measure_infer_function_timings() {
    const N: usize = 40;
    let src = gen_source(N);
    let names = fn_names(N);

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("perf.php");
    std::fs::write(&path, &src).unwrap();

    // Set up a session, run the existing batch path once so workspace state
    // is populated for both paths.
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs_loaded();
    let t_full_walk = {
        let t = Instant::now();
        let _ = session.analyze_paths(std::slice::from_ref(&path), &BatchOptions::new());
        t.elapsed()
    };

    // Use the same db snapshot for the per-fn measurement so the workspace
    // index, stubs, parse cache, etc. are warm — apples to apples.
    let db = session.snapshot_db();
    let path_str: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
    let file = db.lookup_source_file(path_str.as_ref()).unwrap();
    let _ = parse_file(&db, file);
    let _ = collect_file_definitions(&db, file);
    let input = AnalyzeFileInput::new(&db, Arc::from("8.4"));

    // ---- Cold: each fn queried for the first time on this db ----
    let t_cold = Instant::now();
    for name in &names {
        let _ = infer_function(&db, file, name.clone(), input);
    }
    let elapsed_cold = t_cold.elapsed();

    // ---- Warm: repeat the same calls; should be memoized ----
    let t_warm = Instant::now();
    for name in &names {
        let _ = infer_function(&db, file, name.clone(), input);
    }
    let elapsed_warm = t_warm.elapsed();

    // ---- Report ----
    eprintln!();
    eprintln!("=== infer_function performance ===");
    eprintln!("functions in fixture:    {N}");
    eprintln!();
    eprintln!(
        "full-file walk (old):    {:>10?}   ({:.1} us / fn)",
        t_full_walk,
        t_full_walk.as_micros() as f64 / N as f64
    );
    eprintln!(
        "infer_function cold:     {:>10?}   ({:.1} us / fn)",
        elapsed_cold,
        elapsed_cold.as_micros() as f64 / N as f64
    );
    eprintln!(
        "infer_function warm:     {:>10?}   ({:.1} us / fn)",
        elapsed_warm,
        elapsed_warm.as_micros() as f64 / N as f64
    );
    eprintln!();
    let speedup = elapsed_cold.as_nanos() as f64 / elapsed_warm.as_nanos().max(1) as f64;
    eprintln!("warm/cold speedup:       {speedup:.1}x");

    // Sanity: warm must be meaningfully faster than cold.
    assert!(
        elapsed_warm * 4 < elapsed_cold,
        "memoization not effective: warm {:?} not < cold/4 {:?}",
        elapsed_warm,
        elapsed_cold / 4
    );
}
