//! Measures the cost of the open-file diagnostics path
//! (`FileAnalyzer::analyze` → `preload_psr4_classes_for_ast`), which is where
//! the declared-type-closure lazy-load fix lives. `perf_analysis.rs` does not
//! call `FileAnalyzer::analyze`, so it doesn't capture this path.
//!
//! Run with the Laravel fixture present:
//!   cargo test -p mir-analyzer --test perf_open_file_closure -- --ignored --nocapture

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mir_analyzer::{composer::Psr4Map, AnalysisSession, FileAnalyzer, PhpVersion};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures/laravel")
}

#[test]
#[ignore]
fn open_file_analyze_closure_cost() {
    let root = fixture_root();
    if !root.join("composer.json").exists() {
        eprintln!("skipping: Laravel fixture not present");
        return;
    }

    // A few framework files that reference several other classes through
    // imports and signatures — representative of an open buffer in an editor.
    let candidates = [
        "src/Illuminate/Database/Eloquent/Builder.php",
        "src/Illuminate/Routing/Router.php",
        "src/Illuminate/Http/Request.php",
    ];

    for rel in candidates {
        let path = root.join(rel);
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue;
        };
        let parsed = php_rs_parser::parse(&src);
        if !parsed.errors.is_empty() {
            continue;
        }

        let session = match Psr4Map::from_composer(&root) {
            Ok(map) => AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(map)),
            Err(_) => AnalysisSession::new(PhpVersion::LATEST),
        };

        let file: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
        session.ingest_file(file.clone(), Arc::from(src.as_str()));

        let before = session.all_classes().len();

        // Cold analyze: triggers preload of the file's declared-type closure.
        let t0 = Instant::now();
        let analysis = FileAnalyzer::new(&session).analyze(
            file.clone(),
            &src,
            &parsed.program,
            &parsed.source_map,
        );
        let cold = t0.elapsed();

        let after_cold = session.all_classes().len();

        // Warm analyze: closure already loaded; measures steady-state per-edit cost.
        let t0 = Instant::now();
        let _ = FileAnalyzer::new(&session).analyze(
            file.clone(),
            &src,
            &parsed.program,
            &parsed.source_map,
        );
        let warm = t0.elapsed();

        println!(
            "{rel}\n  cold analyze: {:>8.2?}   warm analyze: {:>8.2?}   classes loaded by closure: {}   issues: {}",
            cold,
            warm,
            after_cold.saturating_sub(before),
            analysis.issues.len(),
        );
    }
}
