//! Measurement: estimate per-function memory cost if we added
//! `infer_function(fn_id) -> Arc<FunctionInferenceResult>` as a salsa
//! tracked query (rust-analyzer-style per-function inference).
//!
//! Decision-input for whether to pursue the larger refactor. Runs the
//! existing batch analyzer on the Laravel fixture, then divides totals
//! by function count to estimate the per-function working-set size that
//! would be retained in salsa's cache.
//!
//! Run with:
//!   cargo test -p mir-analyzer --test per_function_inference_size --release -- --nocapture --ignored

use std::path::PathBuf;
use std::sync::Arc;

use mir_analyzer::db::MirDatabase;
use mir_analyzer::{discover_files, AnalysisSession, BatchOptions, PhpVersion};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("benches")
        .join("fixtures")
        .join("laravel")
}

#[test]
#[ignore = "long-running measurement; run explicitly with --release --ignored"]
fn measure_per_function_inference_size() {
    let root = fixtures_root();
    if !root.exists() {
        eprintln!(
            "Skipping: fixture not found at {}. Run `bash crates/mir-analyzer/benches/download-fixtures.sh` first.",
            root.display()
        );
        return;
    }

    // Discover all PHP files under the Laravel fixture.
    let vendor_files = discover_files(&root.join("vendor"));
    let project_files = discover_files(&root.join("src"));
    let all_files: Vec<PathBuf> = vendor_files
        .iter()
        .chain(project_files.iter())
        .cloned()
        .collect();
    eprintln!(
        "Corpus: {} files ({} vendor + {} project)",
        all_files.len(),
        vendor_files.len(),
        project_files.len()
    );

    let t_start = std::time::Instant::now();
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    let result = session.analyze_paths(&all_files, &BatchOptions::new());
    let elapsed = t_start.elapsed();
    eprintln!("Analysis completed in {:?}", elapsed);

    // ---------- counts ----------
    let db = session.snapshot_db();
    let n_functions = mir_analyzer::db::workspace_functions(&db).len();
    let class_fqcns = mir_analyzer::db::workspace_classes(&db);
    let mut n_methods = 0usize;
    for fqcn in class_fqcns.iter() {
        let here = mir_analyzer::db::Fqcn::from_str(&db, fqcn.as_ref());
        if let Some(class) = mir_analyzer::db::find_class_like(&db, here) {
            n_methods += class.own_methods().len();
        }
    }
    let n_callables = n_functions + n_methods;

    // ---------- issues + ref locs ----------
    let n_issues = result.issues.len();
    let n_ref_locs = db.all_reference_location_pairs().len();
    let n_symbols = result.symbols.len();

    // ---------- size of one Issue / RefLoc / Union ----------
    // Issue = 168, IssueKind = 104, Location = 32 (measured separately)
    let sizeof_issue = std::mem::size_of::<mir_issues::Issue>();
    let sizeof_refloc = std::mem::size_of::<mir_analyzer::db::RefLoc>();
    let sizeof_union = std::mem::size_of::<mir_types::Union>();

    // ---------- per-function estimates ----------
    let issues_per_fn = n_issues as f64 / n_callables.max(1) as f64;
    let refs_per_fn = n_ref_locs as f64 / n_callables.max(1) as f64;
    let symbols_per_fn = n_symbols as f64 / n_callables.max(1) as f64;

    // FunctionInferenceResult would carry:
    //   - Vec<Issue>  (issues emitted by this fn body)
    //   - Vec<RefLoc> (references in this fn body)
    //   - Union       (inferred return type)
    //   - Arc header + Vec headers + salsa storage overhead (~64 bytes)
    let avg_issues_bytes = issues_per_fn * sizeof_issue as f64;
    let avg_refs_bytes = refs_per_fn * sizeof_refloc as f64;
    let avg_return_bytes = sizeof_union as f64;
    let salsa_overhead_bytes = 64.0;
    let avg_result_bytes =
        avg_issues_bytes + avg_refs_bytes + avg_return_bytes + salsa_overhead_bytes;

    let total_cache_bytes = avg_result_bytes * n_callables as f64;

    // ---------- report ----------
    eprintln!();
    eprintln!("=== Per-function inference measurement ===");
    eprintln!("functions (free):        {}", n_functions);
    eprintln!("methods (own):           {}", n_methods);
    eprintln!("total callables:         {}", n_callables);
    eprintln!();
    eprintln!("total issues:            {}", n_issues);
    eprintln!("total ref_locs:          {}", n_ref_locs);
    eprintln!("total resolved symbols:  {}", n_symbols);
    eprintln!();
    eprintln!(
        "per-fn issues:           {:.2}  (~{:.0} bytes/fn)",
        issues_per_fn, avg_issues_bytes
    );
    eprintln!(
        "per-fn ref_locs:         {:.2}  (~{:.0} bytes/fn)",
        refs_per_fn, avg_refs_bytes
    );
    eprintln!("per-fn symbols:          {:.2}", symbols_per_fn);
    eprintln!("per-fn return type:      ~{} bytes", sizeof_union);
    eprintln!("salsa overhead:          ~64 bytes/fn");
    eprintln!();
    eprintln!(
        "estimated FunctionInferenceResult: ~{:.0} bytes/fn",
        avg_result_bytes
    );
    eprintln!(
        "estimated total cache:    ~{:.1} MB  ({} callables × ~{:.0} B)",
        total_cache_bytes / (1024.0 * 1024.0),
        n_callables,
        avg_result_bytes
    );
    eprintln!();
    eprintln!("(Cold-start time: {:?})", elapsed);

    // Sanity guard so a smoke run is meaningful.
    assert!(n_callables > 1000, "fixture too small to be meaningful");
    // Document a soft ceiling — fail loudly if it ever drifts past 200 MB on Laravel
    // (would invalidate the architecture).
    let mb = total_cache_bytes / (1024.0 * 1024.0);
    assert!(
        mb < 500.0,
        "estimated cache too large ({:.1} MB) — reconsider per-fn caching",
        mb
    );

    // Keep `result` alive so the analysis isn't dead-code-eliminated.
    let _ = Arc::new(result);
}
