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

    let gen_before = session.index_generation();
    let t0 = Instant::now();
    let refs = session
        .indexed_references_to(&target, &project_paths, false, &|| false)
        .expect("not cancelled");
    let cold_elapsed = t0.elapsed();
    eprintln!(
        "cold indexed_references_to({target:?}): {:.3}s, {} references found, {} revision bumps",
        cold_elapsed.as_secs_f64(),
        refs.len(),
        session.index_generation() - gen_before
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

    // php-lsp's protected-member reference scoping path (`subtype_files`,
    // via `method_reference_scope`): an O(workspace) completeness sweep per
    // BFS round with no per-class cache. The repeat calls are the number
    // that matters — every protected-member references query on the same
    // hierarchy pays it again.
    let t3 = Instant::now();
    let subtypes = session.subtype_files("Illuminate\\Support\\ServiceProvider");
    eprintln!(
        "subtype_files(ServiceProvider) cold: {:.3}s, {} files",
        t3.elapsed().as_secs_f64(),
        subtypes.len()
    );
    let t4 = Instant::now();
    let n = 5;
    for _ in 0..n {
        let repeat = session.subtype_files("Illuminate\\Support\\ServiceProvider");
        assert_eq!(repeat.len(), subtypes.len());
    }
    eprintln!(
        "subtype_files(ServiceProvider) repeat: {:.3}s avg over {n}",
        t4.elapsed().as_secs_f64() / n as f64
    );

    // Scenario: the LSP-host shape from php-lsp's references issue — only
    // project files are registered up front; vendor stays known solely
    // through the PSR-4 resolver and is read from disk + registered as a NEW
    // SourceFile input per lazily-loaded class *inside* the query. Every
    // new-file registration bumps the workspace revision, so this is the
    // path where per-load bumps (reader cancellations) actually accumulate.
    let psr4b = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(&fixture)
            .expect("failed to load composer.json"),
    );
    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4b);
    session2.ensure_all_stubs();
    let project_only: Vec<(Arc<str>, Arc<str>)> = project_files
        .iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(p).ok()?;
            Some((
                Arc::<str>::from(p.to_string_lossy().as_ref()),
                Arc::<str>::from(src),
            ))
        })
        .collect();
    session2.set_workspace_files(project_only);

    let gen_before = session2.index_generation();
    let t5 = Instant::now();
    let refs2 = session2
        .indexed_references_to(&target, &project_paths, false, &|| false)
        .expect("not cancelled");
    eprintln!(
        "cold references, vendor-unregistered (php-lsp shape): {:.3}s, {} references, {} revision bumps",
        t5.elapsed().as_secs_f64(),
        refs2.len(),
        session2.index_generation() - gen_before
    );

    // Scenario: cold subtype queries on a fully-registered, symbol-indexed,
    // never-analyzed workspace (an LSP session right after its scan) — the
    // `commit_defs_for_matching` gate must textually vet every
    // never-committed candidate per BFS round, so this times that gate and
    // counts the raw-text passes it records/skips via the mention index.
    let psr4c = Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(&fixture)
            .expect("failed to load composer.json"),
    );
    let session3 = AnalysisSession::new(PhpVersion::LATEST).with_psr4(psr4c);
    session3.ensure_all_stubs();
    let workspace3: Vec<(Arc<str>, Arc<str>)> = project_files
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
    let all_paths: Vec<Arc<str>> = workspace3.iter().map(|(p, _)| p.clone()).collect();
    session3.set_workspace_files(workspace3);
    session3.rebuild_workspace_symbol_index();

    let t6 = Instant::now();
    let subs1 =
        session3.indexed_subtype_classes("Illuminate\\Support\\ServiceProvider", &all_paths, false);
    let scans1 = session3.class_mention_stats().scans_recorded;
    eprintln!(
        "cold indexed_subtype_classes(ServiceProvider), unanalyzed workspace: {:.3}s, {} subtypes, {} mention scans recorded",
        t6.elapsed().as_secs_f64(),
        subs1.len(),
        scans1
    );
    let t7 = Instant::now();
    let subs2 = session3.indexed_subtype_classes(
        "Illuminate\\Contracts\\Support\\Arrayable",
        &all_paths,
        false,
    );
    let scans2 = session3.class_mention_stats().scans_recorded;
    eprintln!(
        "cold indexed_subtype_classes(Arrayable), same session: {:.3}s, {} subtypes, {} new mention scans",
        t7.elapsed().as_secs_f64(),
        subs2.len(),
        scans2 - scans1
    );

    // Scenario: member-symbol references (method name + owner short name
    // needles) on the same session. The method name is novel to the mention
    // universe, so the first query pays the once-per-needle recording pass;
    // the post-edit repeat (memo missed — revision moved) is the steady
    // state an editor lives in.
    let m_target = Name::method("Illuminate\\Support\\Str", "studly");
    let t8 = Instant::now();
    let mrefs = session3
        .indexed_references_to(&m_target, &all_paths, false, &|| false)
        .expect("not cancelled");
    let scans3 = session3.class_mention_stats().scans_recorded;
    eprintln!(
        "cold references(Str::studly), novel member needle: {:.3}s, {} references, {} new mention scans",
        t8.elapsed().as_secs_f64(),
        mrefs.len(),
        scans3 - scans2
    );

    let edited = all_paths[0].clone();
    let edited_text: Arc<str> = {
        let orig = std::fs::read_to_string(edited.as_ref()).expect("read edited file");
        Arc::from(format!("{orig}\n// bench edit\n").as_str())
    };
    session3.set_file_text(edited.clone(), edited_text);
    let t9 = Instant::now();
    let mrefs2 = session3
        .indexed_references_to(&m_target, &all_paths, false, &|| false)
        .expect("not cancelled");
    let scans4 = session3.class_mention_stats().scans_recorded;
    eprintln!(
        "references(Str::studly) after one file edit: {:.3}s, {} references, {} new mention scans",
        t9.elapsed().as_secs_f64(),
        mrefs2.len(),
        scans4 - scans3
    );

    // Scenario: constructor references — the gate carries the two raw call
    // tokens (`->__construct`/`::__construct`) alongside the class needle.
    // Project scope keeps the admitted-candidate analysis bounded.
    let c_target = Name::method("Illuminate\\Support\\Str", "__construct");
    let t10 = Instant::now();
    let crefs = session3
        .indexed_references_to(&c_target, &project_paths, false, &|| false)
        .expect("not cancelled");
    let scans5 = session3.class_mention_stats().scans_recorded;
    eprintln!(
        "cold references(Str::__construct), raw needles: {:.3}s, {} references, {} new mention scans",
        t10.elapsed().as_secs_f64(),
        crefs.len(),
        scans5 - scans4
    );
    let t11 = Instant::now();
    let crefs2 = session3
        .indexed_references_to(
            &c_target,
            &project_paths[..project_paths.len() - 1],
            false,
            &|| false,
        )
        .expect("not cancelled");
    let scans6 = session3.class_mention_stats().scans_recorded;
    eprintln!(
        "references(Str::__construct) repeat (memo missed): {:.3}s, {} references, {} new mention scans",
        t11.elapsed().as_secs_f64(),
        crefs2.len(),
        scans6 - scans5
    );

    let stats = session3.class_mention_stats();
    eprintln!(
        "mention index footprint: {} universe names | {} files covered | {} mentions (~{:.1} MB entries) | scanner {:.1} MB | {} scans recorded",
        stats.universe_names,
        stats.files_covered,
        stats.total_mentions,
        (stats.total_mentions * 8 + stats.files_covered * 64) as f64 / 1e6,
        stats.scanner_bytes as f64 / 1e6,
        stats.scans_recorded
    );
}
