//! Benchmark for the "distinctive-static first-touch" scenario: every file
//! in a real workspace holds one unresolved reference (so its reference
//! commit is `resolved: false`), a generation bump then makes every one of
//! them stale, and a query for an unrelated, highly selective static-method
//! symbol has to decide whether each is worth re-analyzing.
//!
//! Run with:
//!     cargo run --release --example unresolved_commit_gate_bench
//!
//! Setup mirrors `cold_references_bench.rs` (Laravel fixture, pre-registered
//! text, no upfront parse/analysis).

use std::sync::Arc;
use std::time::Instant;

use mir_analyzer::{discover_files, perf_fixture::PerfFixture, AnalysisSession, Name, PhpVersion};

fn main() {
    let Some(fixture) = PerfFixture::discover() else {
        eprintln!("No supported perf fixture found.");
        std::process::exit(2);
    };
    let fixture_root = fixture.root();
    if !fixture.has_full_corpus() {
        eprintln!(
            "Perf fixture not found or incomplete at {}; provide MIR_PERF_FIXTURE/MIR_LARAVEL_FIXTURE/MIR_SYMFONY_FIXTURE or run `bash {}/benches/download-fixtures.sh`",
            fixture_root.display(),
            env!("CARGO_MANIFEST_DIR")
        );
        std::process::exit(2);
    }

    let psr4 = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(fixture_root)
            .expect("failed to load composer.json"),
    );
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4);
    session.ensure_all_stubs();

    let project_files = discover_files(&fixture.src_root());
    eprintln!("loaded {} project files", project_files.len());

    // Every file gets one guaranteed-unresolved reference appended (inside
    // `if (false)` so it never changes runtime behavior, just forces an
    // UndefinedClass issue) — every commit below lands `resolved: false`,
    // matching the reported "replayed postings have unresolved names"
    // scenario for the whole workspace, not just a sample of it.
    let workspace: Vec<(Arc<str>, Arc<str>)> = project_files
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let src = std::fs::read_to_string(p).ok()?;
            let doctored = format!(
                "{src}\nif (false) {{ \\App\\ForcedUnresolvedByBench_{i}::neverCalled(); }}\n"
            );
            Some((
                Arc::<str>::from(p.to_string_lossy().as_ref()),
                Arc::<str>::from(doctored),
            ))
        })
        .collect();
    session.set_workspace_files(workspace);

    let project_paths: Vec<Arc<str>> = project_files
        .iter()
        .map(|p| Arc::<str>::from(p.to_string_lossy().as_ref()))
        .collect();

    // Force every file to be analyzed + committed once: an empty-name
    // member query disables the needle gate entirely (`reference_gate_needles`
    // drops empty needles, leaving nothing to filter never-committed
    // candidates on), so this pass's cost is irrelevant to what's timed below.
    let t0 = Instant::now();
    session
        .indexed_references_to(
            &Name::method("", ""),
            &project_paths,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    eprintln!(
        "initial full-commit pass: {:.3}s",
        t0.elapsed().as_secs_f64()
    );

    // Bump the workspace generation without touching any project file —
    // mirrors a background indexer registering one more file mid-session,
    // which is exactly what makes every `resolved: false` commit above
    // stale again.
    session.ingest_file(
        Arc::from("src/BenchLater.php"),
        Arc::from("<?php\nnamespace Illuminate\\Support;\nclass BenchLater {}\n"),
    );

    // A distinctive static method name that appears in none of the real
    // Laravel source (nor in the injected unresolved-reference lines).
    let target = Name::method("Illuminate\\Support\\Distinctive", "staticThing");
    let t1 = Instant::now();
    let refs = session
        .indexed_references_to(
            &target,
            &project_paths,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    eprintln!(
        "post-growth distinctive-static query: {:.3}s, {} references found",
        t1.elapsed().as_secs_f64(),
        refs.len()
    );
}
