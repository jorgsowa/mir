//! Re-indexing benchmark for the eager background-indexing model: running
//! `index_batch` again over an unchanged tree must hit the unchanged-text fast
//! path instead of re-parsing.
//!
//! ```text
//! cargo test -p mir-analyzer --test it indexing_responsiveness:: -- --ignored --nocapture
//! ```

use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{AnalysisSession, IndexCancel, IndexParallelism, PhpVersion};

use crate::common::create_temp_dir;

fn n_files() -> usize {
    std::env::var("MIR_BENCH_FILES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4000)
}
const CHUNK: usize = 500;

/// Once a tree is indexed, running `index_batch` again over the *same,
/// unchanged* files (e.g. re-opening a workspace, or a background rescan
/// finding nothing new) must be instant.
///
/// `upsert_source_file_with_durability` only bumps the salsa revision when the
/// text actually differs, so an unchanged re-index should hit `SourceFile`'s
/// dependent memos (`collect_file_declarations` et al.) rather than
/// re-parsing. A regression here — e.g. something that unconditionally bumps
/// the revision or skips the equality check — would turn every "nothing
/// changed" rescan into a full cold index again.
#[test]
#[ignore = "perf benchmark; run explicitly with --ignored --nocapture"]
fn reindexing_unchanged_tree_is_instant() {
    let root = create_temp_dir("idx_cached_reindex");
    let vendor_src = root.path().join("vendor/Lib/src");
    std::fs::create_dir_all(&vendor_src).unwrap();
    std::fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"Lib\\":"vendor/Lib/src/"}}}"#,
    )
    .unwrap();

    let total = n_files();
    let mut files: Vec<(Arc<str>, Arc<str>)> = Vec::with_capacity(total);
    for i in 0..total {
        let name = format!("C{i}");
        let body = format!(
            "<?php\nnamespace Lib;\nclass {name} {{\n    \
             public function a(): int {{ return {i}; }}\n    \
             public function b(): string {{ return \"{i}\"; }}\n    \
             public function c(): bool {{ return true; }}\n}}\n"
        );
        let path = vendor_src.join(format!("{name}.php"));
        std::fs::write(&path, &body).unwrap();
        files.push((
            Arc::from(path.to_string_lossy().as_ref()),
            Arc::from(body.as_str()),
        ));
    }

    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root.path()).unwrap();
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));
    session.ensure_all_stubs();

    // Cold pass: not asserted.
    let cancel = IndexCancel::new();
    for chunk in files.chunks(CHUNK) {
        session.index_batch(chunk, IndexParallelism::Rayon, &cancel);
    }
    session.finalize_index();
    assert!(session.contains_class("Lib\\C0"));

    // Warm pass: identical (path, text) pairs — should short-circuit.
    let t = Instant::now();
    for chunk in files.chunks(CHUNK) {
        session.index_batch(chunk, IndexParallelism::Rayon, &cancel);
    }
    session.finalize_index();
    let warm_reindex = t.elapsed();

    eprintln!("reindexing {total} unchanged files: {warm_reindex:?}");

    assert!(
        warm_reindex < Duration::from_millis(200),
        "reindexing an unchanged {total}-file tree took {warm_reindex:?} — expected the \
         unchanged-text fast path (no revision bump, memoized declarations) to keep this instant"
    );
}
