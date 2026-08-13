//! Measurement harness for AnalysisSession::dependency_graph.
//!
//! Run with:
//!
//! ```sh
//! cargo test --release -p mir-analyzer --test measure_dependency_graph -- --ignored --nocapture
//! ```

use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use mir_analyzer::{discover_files, AnalysisSession, BatchOptions, PhpVersion};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("fixtures")
        .join("laravel")
}

fn skip_if_missing(root: &Path) -> bool {
    let src = root.join("src");
    let vendor = root.join("vendor");
    if !src.exists() || !vendor.exists() {
        eprintln!(
            "Skipping measurement: fixture not found or incomplete at {}",
            root.display()
        );
        true
    } else {
        false
    }
}

#[test]
#[ignore = "measurement harness; run explicitly with --release --ignored"]
fn measure_dependency_graph() {
    let root = fixtures_root();
    if skip_if_missing(&root) {
        return;
    }

    let vendor_files = discover_files(&root.join("vendor"));
    let project_files = discover_files(&root.join("src"));
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.collect_definitions(&vendor_files);
    let _ = session.analyze_paths(&project_files, &BatchOptions::new());
    session.rebuild_workspace_symbol_index();

    let db = session.snapshot_db();
    let files: Vec<String> = db
        .source_file_paths()
        .iter()
        .map(|file| file.as_ref().to_string())
        .collect();
    drop(db);

    eprintln!(
        "[measure_dependency_graph] files={} (vendor={} project={})",
        files.len(),
        vendor_files.len(),
        project_files.len()
    );

    let mut warm_times = Vec::new();
    for iteration in 0..8 {
        let start = Instant::now();
        let graph = black_box(session.dependency_graph());
        let elapsed = start.elapsed();
        let dependency_edges = files
            .iter()
            .map(|file| graph.dependencies_of(file).len())
            .sum::<usize>();
        let dependent_edges = files
            .iter()
            .map(|file| graph.dependents_of(file).len())
            .sum::<usize>();
        let label = if iteration == 0 { "cold" } else { "warm" };
        eprintln!(
            "[measure_dependency_graph] {label}[{iteration}]={:.3}s dependencies={} dependents={}",
            elapsed.as_secs_f64(),
            black_box(dependency_edges),
            black_box(dependent_edges),
        );
        if iteration > 0 {
            warm_times.push(elapsed.as_secs_f64());
        }
    }
    warm_times.sort_by(|a, b| a.total_cmp(b));
    eprintln!(
        "[measure_dependency_graph] warm_median={:.3}s warm_min={:.3}s warm_max={:.3}s",
        warm_times[warm_times.len() / 2],
        warm_times[0],
        warm_times[warm_times.len() - 1],
    );
}
