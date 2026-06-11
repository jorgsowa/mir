//! Measurement harness: batch analysis over the full Laravel corpus under
//! different option combinations, for timing + peak-RSS comparison.
//!
//! Run under `/usr/bin/time -l` (macOS) to capture max RSS:
//!
//! ```sh
//! MIR_MEASURE=symbols   /usr/bin/time -l target/release/deps/measure_batch-* --ignored --nocapture
//! MIR_MEASURE=nosymbols /usr/bin/time -l target/release/deps/measure_batch-* --ignored --nocapture
//! ```
//!
//! Modes (`MIR_MEASURE`, default `symbols`):
//! - `symbols`   — default options (per-expression symbols collected)
//! - `nosymbols` — `BatchOptions::without_symbols()`
//!
//! Each mode also re-runs `analyze_paths` a second time on the same session
//! ("warm repeat") to expose memo/cache reuse on watch-mode-style workloads.

use std::path::PathBuf;

use mir_analyzer::{discover_files, AnalysisSession, BatchOptions, PhpVersion};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("fixtures")
        .join("laravel")
}

#[test]
#[ignore = "measurement harness; run explicitly with --release --ignored"]
fn measure_batch() {
    let root = fixtures_root();
    if !root.exists() {
        eprintln!("Skipping: fixture not present at {}", root.display());
        return;
    }

    let mode = std::env::var("MIR_MEASURE").unwrap_or_else(|_| "symbols".to_string());
    let opts = match mode.as_str() {
        "symbols" => BatchOptions::new(),
        "nosymbols" => BatchOptions::new().without_symbols(),
        other => panic!("unknown MIR_MEASURE mode: {other}"),
    };

    let vendor_files = discover_files(&root.join("vendor"));
    let project_files = discover_files(&root.join("src"));
    let all_files: Vec<PathBuf> = vendor_files
        .iter()
        .chain(project_files.iter())
        .cloned()
        .collect();
    eprintln!(
        "[measure mode={mode}] corpus: {} files ({} vendor + {} project)",
        all_files.len(),
        vendor_files.len(),
        project_files.len()
    );

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let t = std::time::Instant::now();
    let result = session.analyze_paths(&all_files, &opts);
    let cold = t.elapsed();

    let t = std::time::Instant::now();
    let result2 = session.analyze_paths(&all_files, &opts);
    let warm = t.elapsed();

    eprintln!(
        "[measure mode={mode}] cold: {:.3}s  warm-repeat: {:.3}s  issues: {} / {}  symbols: {}",
        cold.as_secs_f64(),
        warm.as_secs_f64(),
        result.issues.len(),
        result2.issues.len(),
        result.symbols.len(),
    );
}
