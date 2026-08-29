//! Quick regression timing — ensures the prototype additions (Issue PartialEq
//! derive, new dormant tracked query, new pure BodyAnalyzer method) don't slow
//! down the existing analyze_paths hot path.
//!
//! Run with:
//!   cargo test -p mir-analyzer --test regression_timing --release -- --nocapture --ignored

use std::time::Instant;

use mir_analyzer::{
    discover_files, perf_fixture::PerfFixture, AnalysisSession, BatchOptions, PhpVersion,
};

#[test]
#[ignore = "regression timing; run with --release --ignored"]
fn time_analyze_paths_repeats() {
    let Some(fixture) = PerfFixture::discover() else {
        eprintln!("Skipping: no supported perf fixture present");
        return;
    };
    let root = fixture.root();
    if !fixture.has_full_corpus() {
        eprintln!(
            "Skipping: fixture not present or incomplete at {}",
            root.display()
        );
        return;
    }

    // Use the smaller `src/` slice (project code only) to keep iteration time
    // bounded. About 1410 files; ~60 s per iter in release on the dev box.
    let project_files = discover_files(&fixture.src_root());
    eprintln!(
        "Timing analyze_paths over {} project files",
        project_files.len()
    );

    // Three cold runs (fresh session each time) — measure mean.
    let mut times = Vec::new();
    for i in 1..=3 {
        let session = AnalysisSession::new(PhpVersion::LATEST);
        session.ensure_all_stubs();
        let t = Instant::now();
        let _ = session.analyze_paths(&project_files, &BatchOptions::new().without_symbols());
        let elapsed = t.elapsed();
        eprintln!("run {i}: {:?}", elapsed);
        times.push(elapsed);
    }
    let total_nanos: u128 = times.iter().map(|d| d.as_nanos()).sum();
    let mean = std::time::Duration::from_nanos((total_nanos / times.len() as u128) as u64);
    eprintln!("mean: {:?}", mean);
}
