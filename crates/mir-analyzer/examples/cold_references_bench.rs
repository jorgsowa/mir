//! Times `indexed_references_to`'s cold-query warm-up path (Phase 1 in
//! `AnalysisSession::indexed_references_to`) against the Laravel fixture —
//! a real, repeatable number for changes touching the cold reference-query
//! path. Dominated by the one-time workspace symbol index seed that the
//! first `index_batch` call (via `ensure_vendor_eager_functions`) performs.
//!
//! Run with:
//!     cargo run --release --example cold_references_bench
//!
//! Setup mirrors `fileanalyzer_retry_bench.rs`: all project + vendor source
//! text is pre-registered via `set_workspace_files` (so the timed call pays
//! zero disk I/O), but nothing is parsed, analyzed, or background-indexed —
//! the symbol registry (`contains_class`) is empty until something resolves
//! and loads a class, same as an LSP session between "workspace opened" and
//! "background indexer has caught up". Querying for a symbol used widely
//! across the framework's own `src/` tree (`Illuminate\Support\Str`, ~180
//! call sites) forces the freshness-gate to admit a wide `stale` set, so
//! Phase 1's warm-up loop has real, comparable work to do.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mir_analyzer::{discover_files, AnalysisSession, IndexCancel, Name, PhpVersion};

fn main() {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures/laravel");
    if !fixture.join("vendor").exists() || !fixture.join("src").exists() {
        eprintln!(
            "Laravel fixture not found at {}; run `bash {}/benches/download-fixtures.sh`",
            fixture.display(),
            env!("CARGO_MANIFEST_DIR")
        );
        std::process::exit(2);
    }

    let psr4 = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(&fixture)
            .expect("failed to load composer.json"),
    );
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    session.ensure_all_stubs();

    let project_files = discover_files(&fixture.join("src"));
    let vendor_files = discover_files(&fixture.join("vendor"));
    eprintln!(
        "loaded {} project files, {} vendor files",
        project_files.len(),
        vendor_files.len()
    );

    let workspace: Vec<(Arc<str>, Arc<str>)> = project_files
        .iter()
        .chain(vendor_files.iter())
        .filter_map(|p| {
            let src = std::fs::read_to_string(p).ok()?;
            Some((
                Arc::<str>::from(p.to_string_lossy().as_ref()),
                Arc::<str>::from(src),
            ))
        })
        .collect();
    session.set_workspace_files(workspace);

    let project_paths: Vec<Arc<str>> = project_files
        .iter()
        .map(|p| Arc::<str>::from(p.to_string_lossy().as_ref()))
        .collect();

    // Warms nothing beyond the vendor `autoload.files` entries — deliberately
    // NOT calling `collect_definitions`/`analyze_paths`/`index_batch` here,
    // so `contains_class` starts empty and Phase 1's lazy `load_class` path
    // does real resolve-and-parse work, not just cache hits.
    let target = Name::class("Illuminate\\Support\\Str");

    let t0 = Instant::now();
    let refs = session
        .indexed_references_to(&target, &project_paths, false, &|| false)
        .expect("not cancelled");
    let cold_elapsed = t0.elapsed();
    eprintln!(
        "cold indexed_references_to({target:?}): {:.3}s, {} references found",
        cold_elapsed.as_secs_f64(),
        refs.len()
    );

    // Same query again: every candidate file is now prepared+committed, so
    // this should be a near-pure index lookup — confirms the cold run above
    // didn't leave anything half-warmed and gives a sanity floor for how
    // much of the cold cost is "real" resolve/parse work vs. one-time setup.
    let t1 = Instant::now();
    let refs_warm = session
        .indexed_references_to(&target, &project_paths, false, &|| false)
        .expect("not cancelled");
    eprintln!(
        "warm repeat: {:.3}s, {} references found",
        t1.elapsed().as_secs_f64(),
        refs_warm.len()
    );
    assert_eq!(
        refs.len(),
        refs_warm.len(),
        "warm repeat must match cold results"
    );

    // Second, independent scenario exercising the same batched warm-up via
    // `reanalyze_dependents_cancellable` (incremental.rs), which shares
    // `prepare_files_for_analysis_batch` with the query path above.
    let cancel = IndexCancel::new();
    let dependents_target = project_files
        .iter()
        .find(|p| p.to_string_lossy().ends_with("Arr.php"))
        .expect("Arr.php present in fixture");
    let t2 = Instant::now();
    let dependents =
        session.reanalyze_dependents_cancellable(&dependents_target.to_string_lossy(), &cancel);
    eprintln!(
        "reanalyze_dependents(Arr.php): {:.3}s, {} dependents",
        t2.elapsed().as_secs_f64(),
        dependents.len()
    );
}
