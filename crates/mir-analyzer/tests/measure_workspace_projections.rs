//! Measurement harness for workspace-wide declaration projections.
//!
//! Run with:
//!
//! ```sh
//! cargo test --release -p mir-analyzer measure_workspace_projections -- --ignored --nocapture
//! ```

use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use mir_analyzer::db::{
    file_structural_deps, workspace_classes, workspace_functions, workspace_symbol_index,
    MirDatabase,
};
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
fn measure_workspace_projections() {
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
    let files = db.all_source_files();
    eprintln!(
        "[measure_workspace_projections] files={} (vendor={} project={})",
        files.len(),
        vendor_files.len(),
        project_files.len()
    );

    let classes_cold_start = Instant::now();
    let classes_cold = black_box(workspace_classes(&db).len());
    let classes_cold_elapsed = classes_cold_start.elapsed();
    let classes_warm_start = Instant::now();
    let classes_warm = black_box(workspace_classes(&db).len());
    let classes_warm_elapsed = classes_warm_start.elapsed();

    let functions_cold_start = Instant::now();
    let functions_cold = black_box(workspace_functions(&db).len());
    let functions_cold_elapsed = functions_cold_start.elapsed();
    let functions_warm_start = Instant::now();
    let functions_warm = black_box(workspace_functions(&db).len());
    let functions_warm_elapsed = functions_warm_start.elapsed();

    let index_cold_start = Instant::now();
    let index_cold = black_box(workspace_symbol_index(&db).class_like_len());
    let index_cold_elapsed = index_cold_start.elapsed();
    let index_warm_start = Instant::now();
    let index_warm = black_box(workspace_symbol_index(&db).class_like_len());
    let index_warm_elapsed = index_warm_start.elapsed();

    let deps_cold_start = Instant::now();
    let deps_cold = black_box(
        files
            .iter()
            .map(|file| file_structural_deps(&db, *file).len())
            .sum::<usize>(),
    );
    let deps_cold_elapsed = deps_cold_start.elapsed();
    let deps_warm_start = Instant::now();
    let deps_warm = black_box(
        files
            .iter()
            .map(|file| file_structural_deps(&db, *file).len())
            .sum::<usize>(),
    );
    let deps_warm_elapsed = deps_warm_start.elapsed();

    eprintln!(
        "[measure_workspace_projections] workspace_classes cold={:.3}s warm={:.3}s count={classes_cold}/{classes_warm}",
        classes_cold_elapsed.as_secs_f64(),
        classes_warm_elapsed.as_secs_f64(),
    );
    eprintln!(
        "[measure_workspace_projections] workspace_functions cold={:.3}s warm={:.3}s count={functions_cold}/{functions_warm}",
        functions_cold_elapsed.as_secs_f64(),
        functions_warm_elapsed.as_secs_f64(),
    );
    eprintln!(
        "[measure_workspace_projections] workspace_symbol_index cold={:.3}s warm={:.3}s class_like={index_cold}/{index_warm}",
        index_cold_elapsed.as_secs_f64(),
        index_warm_elapsed.as_secs_f64(),
    );
    eprintln!(
        "[measure_workspace_projections] file_structural_deps(all files) cold={:.3}s warm={:.3}s edges={deps_cold}/{deps_warm}",
        deps_cold_elapsed.as_secs_f64(),
        deps_warm_elapsed.as_secs_f64(),
    );
}
