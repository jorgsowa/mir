//! Performance benchmark for the CLI cache surface firewall + warm-run skip.
//!
//! Run with:
//!   cargo test --release -p mir-analyzer --test firewall_bench -- --ignored --nocapture
//!
//! It builds a project of one widely-depended-on Base class plus N dependents,
//! then times four incremental CLI-style runs against a persistent cache. The
//! signature-edit run is a faithful proxy for the *pre-firewall* behavior of a
//! body edit (both invalidate every dependent), so the body-edit vs signature-
//! edit delta is the firewall's win.

mod common;

use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use self::common::{create_temp_dir, write_file};

/// One CLI-style invocation against a persistent cache dir. Returns wall-clock
/// time and the number of files actually (re)analyzed (`on_file_done` fires
/// once per analyzed file; cache replays return before it).
fn run(cache_dir: &Path, paths: &[PathBuf]) -> (Duration, usize) {
    let n = Arc::new(AtomicUsize::new(0));
    let counter = n.clone();
    let opts = BatchOptions::new().with_progress_callback(Arc::new(move || {
        counter.fetch_add(1, Ordering::Relaxed);
    }));
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir);
    let t = Instant::now();
    session.analyze_paths(paths, &opts);
    (t.elapsed(), n.load(Ordering::Relaxed))
}

#[test]
#[ignore = "timing probe; run with --ignored --nocapture"]
fn probe_pass_breakdown() {
    std::env::set_var("MIR_TIMING", "1");
    const N: usize = 400;
    let src = create_temp_dir("probe src");
    let cache = create_temp_dir("probe cache");
    let mut paths = Vec::with_capacity(N + 1);
    paths.push(write_file(
        &src,
        "Base.php",
        "<?php\nclass Base { public function foo(): int { return 1; } }\n",
    ));
    for i in 0..N {
        paths.push(write_file(&src, &format!("Dep{i}.php"), &dependent_src(i)));
    }
    eprintln!("\n--- COLD (all files analyzed) ---");
    run(cache.path(), &paths);
    for k in 0..5 {
        eprintln!("--- WARM #{k} (all cache hits = the floor) ---");
        run(cache.path(), &paths);
    }
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

/// A non-trivial dependent body: many typed locals + repeated calls to the
/// inherited declared-return method, so body analysis (the work the firewall
/// skips for dependents) is a real cost, not noise.
fn dependent_src(i: usize) -> String {
    let mut s =
        format!("<?php\nclass Dep{i} extends Base {{\n    public function use{i}(): int {{\n");
    s.push_str("        $acc = $this->foo();\n");
    for k in 0..40 {
        s.push_str(&format!(
            "        $v{k} = $this->foo();\n        $acc = $acc + $v{k} * {k} - $this->foo();\n"
        ));
    }
    s.push_str("        return $acc;\n    }\n}\n");
    s
}

#[test]
#[ignore = "perf benchmark; run explicitly with --ignored --nocapture"]
fn bench_firewall_incremental_runs() {
    const N: usize = 400;
    const ITERS: usize = 7;
    let src = create_temp_dir("bench src");
    let cache = create_temp_dir("bench cache");

    let base = |ret: &str, body: &str| {
        format!("<?php\nclass Base {{ public function foo(): {ret} {{ {body} }} }}\n")
    };

    let mut paths = Vec::with_capacity(N + 1);
    paths.push(write_file(&src, "Base.php", &base("int", "return 1;")));
    for i in 0..N {
        paths.push(write_file(&src, &format!("Dep{i}.php"), &dependent_src(i)));
    }

    let (cold, cold_n) = run(cache.path(), &paths);

    // Warm (no change): all cache hits. Median over ITERS.
    let mut warm = Vec::new();
    let mut warm_n = 0;
    for _ in 0..ITERS {
        let (d, n) = run(cache.path(), &paths);
        warm.push(d);
        warm_n = n;
    }

    // Body-only edits: a distinct declared-`int` body each iteration forces Base
    // (only) to re-analyze. Firewall must spare all dependents.
    let mut body = Vec::new();
    let mut body_n = 0;
    for k in 0..ITERS {
        write_file(
            &src,
            "Base.php",
            &base("int", &format!("$x = {k}; return $x + 1;")),
        );
        let (d, n) = run(cache.path(), &paths);
        body.push(d);
        body_n = n;
    }

    // Signature edits: a distinct return type each iteration cascades to every
    // dependent — the cost a body edit incurred *before* the firewall. Types are
    // numeric so dependent bodies stay valid.
    let sig_types = ["float", "int", "float", "int", "float", "int", "float"];
    debug_assert_eq!(sig_types.len(), ITERS);
    let mut sig = Vec::new();
    let mut sig_n = 0;
    for ret in sig_types {
        write_file(&src, "Base.php", &base(ret, "return 1;"));
        let (d, n) = run(cache.path(), &paths);
        sig.push(d);
        sig_n = n;
    }

    let warm_m = median(warm);
    let body_m = median(body);
    let sig_m = median(sig);
    let speedup = sig_m.as_secs_f64() / body_m.as_secs_f64().max(f64::MIN_POSITIVE);

    eprintln!(
        "\n=== firewall benchmark: Base + {N} non-trivial dependents (median of {ITERS}) ==="
    );
    eprintln!(
        "cold run (populate cache) : {:>9.1?}   reanalyzed = {cold_n}",
        cold
    );
    eprintln!(
        "warm run (no changes)     : {:>9.1?}   reanalyzed = {warm_n}",
        warm_m
    );
    eprintln!(
        "body-only edit [firewall] : {:>9.1?}   reanalyzed = {body_n}",
        body_m
    );
    eprintln!(
        "signature edit [pre-fw]   : {:>9.1?}   reanalyzed = {sig_n}",
        sig_m
    );
    eprintln!("---------------------------------------------------------------");
    eprintln!(
        "firewall avoids re-analyzing {} dependents on a body-only edit",
        sig_n.saturating_sub(body_n)
    );
    eprintln!("body-edit speedup vs full cascade: {speedup:.2}x\n");

    assert_eq!(warm_n, 0, "warm run must re-analyze nothing");
    assert_eq!(body_n, 1, "body-only edit must re-analyze only Base");
    assert_eq!(
        sig_n,
        N + 1,
        "signature edit must re-analyze Base + all dependents"
    );
}
