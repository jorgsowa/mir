//! Analyzing the whole Symfony tree, Tests fixtures included, on rayon's default 2 MiB
//! worker stacks must not overflow. Run via `-- --ignored` once the fixture is downloaded
//! (`benches/download-symfony-fixture.sh`) or `MIR_SYMFONY_FIXTURE` points at a checkout.

use std::path::PathBuf;

use mir_analyzer::{discover_files, AnalysisSession, BatchOptions, PhpVersion};

const DEFAULT_RAYON_STACK: usize = 2 * 1024 * 1024;

fn fixture_src() -> PathBuf {
    let root = std::env::var_os("MIR_SYMFONY_FIXTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures/symfony")
        });
    let src = root.join("src");
    assert!(
        src.is_dir(),
        "Symfony fixture not found at {}",
        root.display()
    );
    src
}

#[test]
#[ignore = "requires MIR_SYMFONY_FIXTURE or benches/fixtures/symfony"]
fn full_symfony_tree_does_not_overflow_default_worker_stacks() {
    let files = discover_files(&fixture_src());
    assert!(
        files.len() > 10_000,
        "only {} files discovered",
        files.len()
    );

    let pool = rayon::ThreadPoolBuilder::new()
        .stack_size(DEFAULT_RAYON_STACK)
        .build()
        .unwrap();
    let result = pool.install(|| {
        let mut session = AnalysisSession::new(PhpVersion::LATEST);
        session.ensure_all_stubs();
        session.analyze_paths(&files, &BatchOptions::new().without_symbols())
    });
    assert!(!result.issues.is_empty());
}
