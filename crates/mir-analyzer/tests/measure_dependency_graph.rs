//! Measurement harness for AnalysisSession::dependency_graph.
//!
//! Run with:
//!
//! ```sh
//! cargo test --release -p mir-analyzer --test measure_dependency_graph -- --ignored --nocapture
//! ```

use std::hint::black_box;
use std::time::Instant;

use mir_analyzer::{
    discover_files, perf_fixture::PerfFixture, AnalysisSession, BatchOptions, PhpVersion,
};

fn skip_if_missing(fixture: &PerfFixture) -> bool {
    if !fixture.has_full_corpus() {
        eprintln!(
            "Skipping measurement: fixture not found or incomplete at {}",
            fixture.root().display()
        );
        true
    } else {
        false
    }
}

#[test]
#[ignore = "measurement harness; run explicitly with --release --ignored"]
fn measure_dependency_graph() {
    let Some(fixture) = PerfFixture::discover() else {
        eprintln!("Skipping measurement: no supported perf fixture found");
        return;
    };
    if skip_if_missing(&fixture) {
        return;
    }

    let vendor_files = discover_files(&fixture.vendor_root());
    let project_files = discover_files(&fixture.src_root());
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.collect_definitions(&vendor_files);
    let _ = session.analyze_paths(&project_files, &BatchOptions::new().without_symbols());
    session.rebuild_workspace_symbol_index();

    let db = session.snapshot_db();
    let mut files: Vec<String> = db
        .source_file_paths()
        .iter()
        .map(|file| file.as_ref().to_string())
        .collect();
    files.sort();
    drop(db);

    eprintln!(
        "[measure_dependency_graph] files={} (vendor={} project={})",
        files.len(),
        vendor_files.len(),
        project_files.len()
    );

    let mut build_times = Vec::new();
    for iteration in 0..8 {
        let start = Instant::now();
        black_box(session.dependency_graph());
        let elapsed = start.elapsed();
        let label = if iteration == 0 { "cold" } else { "warm" };
        eprintln!(
            "[measure_dependency_graph] build_{label}[{iteration}]={:.3}s",
            elapsed.as_secs_f64(),
        );
        if iteration > 0 {
            build_times.push(elapsed.as_secs_f64());
        }
    }
    build_times.sort_by(|a, b| a.total_cmp(b));
    eprintln!(
        "[measure_dependency_graph] build_warm_median={:.3}s build_warm_min={:.3}s build_warm_max={:.3}s",
        build_times[build_times.len() / 2],
        build_times[0],
        build_times[build_times.len() - 1],
    );

    let graph = session.dependency_graph();

    let seeds: Vec<&String> = files.iter().take(256).collect();
    let transitive_start = Instant::now();
    let transitive_total = seeds
        .iter()
        .map(|file| graph.transitive_dependents(file).len())
        .sum::<usize>();
    let transitive_elapsed = transitive_start.elapsed();
    eprintln!(
        "[measure_dependency_graph] transitive_dependents_256={:.3}s total={}",
        transitive_elapsed.as_secs_f64(),
        black_box(transitive_total),
    );

    let direct_arc_start = Instant::now();
    let dependency_edges = files
        .iter()
        .map(|file| graph.dependency_paths_of(file).len())
        .sum::<usize>();
    let dependent_edges = files
        .iter()
        .map(|file| graph.dependent_paths_of(file).len())
        .sum::<usize>();
    let direct_arc_elapsed = direct_arc_start.elapsed();
    eprintln!(
        "[measure_dependency_graph] direct_arc_access={:.3}s dependencies={} dependents={}",
        direct_arc_elapsed.as_secs_f64(),
        black_box(dependency_edges),
        black_box(dependent_edges),
    );

    let legacy_start = Instant::now();
    let dependency_edges = files
        .iter()
        .map(|file| graph.dependencies_of(file).len())
        .sum::<usize>();
    let dependent_edges = files
        .iter()
        .map(|file| graph.dependents_of(file).len())
        .sum::<usize>();
    let legacy_elapsed = legacy_start.elapsed();
    eprintln!(
        "[measure_dependency_graph] legacy_direct_access={:.3}s dependencies={} dependents={}",
        legacy_elapsed.as_secs_f64(),
        black_box(dependency_edges),
        black_box(dependent_edges),
    );
}
