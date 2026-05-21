//! Simulates the LSP-style per-file analysis path against the Laravel
//! fixture so we can measure how often `FileAnalyzer`'s post-Pass-2 retry
//! loop actually fires on a real workload.
//!
//! Run with:
//!     MIR_TIMING=1 cargo run --release --example fileanalyzer_retry_bench
//!
//! What it does:
//!   1. Loads Laravel's composer.json into a Psr4Map.
//!   2. Builds an AnalysisSession with that resolver attached.
//!   3. Bulk-registers all project + vendor source texts via
//!      `set_workspace_files` (no parsing yet — matches what a smart LSP
//!      would do on workspace open).
//!   4. Iterates over project files, parsing each and calling
//!      `FileAnalyzer::analyze`. This is the path equivalent to the LSP's
//!      "open file → request diagnostics" flow.
//!   5. At end, the MIR_TIMING dump prints to stderr with retry-loop counts.
//!
//! Output the user cares about (printed to stderr):
//!   file analyses        : N
//!   pass-2 runs          : M  (avg per analysis: M/N)
//!   retry iterations     : R
//!   lazy load attempts   : A  resolved: R'
//!
//! Avg per analysis = M/N is the key number. 1.000 → retry loop never
//! fires. >1.05 → retry loop is exercising the lazy-load fault-in. The
//! plan's decision gate (`docs/perf-baseline.md`) reads this number.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion, ProjectAnalyzer};

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

    // 1. Load composer autoload map.
    let psr4 = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(&fixture)
            .expect("failed to load composer.json"),
    );

    // 2. Build session.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4.clone());

    // 3. Bulk-register every file's source text.
    let project_files = ProjectAnalyzer::discover_files(&fixture.join("src"));
    let vendor_files = ProjectAnalyzer::discover_files(&fixture.join("vendor"));
    eprintln!(
        "loaded {} project files, {} vendor files",
        project_files.len(),
        vendor_files.len()
    );

    let t0 = Instant::now();
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
    eprintln!(
        "read workspace files in {:.2}s ({} files)",
        t0.elapsed().as_secs_f64(),
        workspace.len()
    );

    let t1 = Instant::now();
    session.set_workspace_files(workspace);
    eprintln!("set_workspace_files in {:.2}s", t1.elapsed().as_secs_f64());

    // 4. Walk project files, analyze each via FileAnalyzer.
    //
    // We only analyze project/src/ (matches what an editor would open) and
    // we cap to 200 files so the run finishes in a couple of minutes.
    let analyze_subset: Vec<&PathBuf> = project_files.iter().take(200).collect();
    eprintln!(
        "analyzing {} project files via FileAnalyzer::analyze...",
        analyze_subset.len()
    );

    let t2 = Instant::now();
    let mut total_issues: usize = 0;
    for path in &analyze_subset {
        let Ok(source) = std::fs::read_to_string(path) else {
            continue;
        };
        let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
        // FileAnalyzer::analyze's documented contract: the session must have
        // Pass-1 state for the file. Without this, `resolve_name_via_db`
        // can't see the file's `use` aliases, and Pass-2 emits
        // UndefinedClass with the unqualified shorthand the user wrote.
        // Mimics the LSP's didOpen / didChange flow.
        session.ingest_file(file.clone(), Arc::<str>::from(source.as_str()));

        let arena = bumpalo::Bump::new();
        let parsed = php_rs_parser::parse_arena(&arena, &source);
        let analyzer = FileAnalyzer::new(&session);
        let result = analyzer.analyze(file, &source, &parsed.program, &parsed.source_map);
        total_issues += result.issues.len();
    }
    let elapsed = t2.elapsed();
    eprintln!(
        "analyzed {} files in {:.2}s ({:.1} ms/file avg), {} total issues",
        analyze_subset.len(),
        elapsed.as_secs_f64(),
        elapsed.as_millis() as f64 / analyze_subset.len() as f64,
        total_issues,
    );

    // 5. Metrics dump (MIR_TIMING=1 prints automatically on Drop in
    // ProjectAnalyzer; FileAnalyzer doesn't have that hook, so we trigger
    // it manually here.)
    if let Some(s) = mir_analyzer::metrics::dump() {
        eprintln!("{s}");
    } else {
        eprintln!("(MIR_TIMING not set; re-run with MIR_TIMING=1 to capture metrics)");
    }
}
