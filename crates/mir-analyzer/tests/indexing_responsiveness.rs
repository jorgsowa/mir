//! Responsiveness benchmark for the eager background-indexing model.
//!
//! The headline requirement: indexing a large vendor tree must NOT freeze
//! interactive work. The freeze mechanism (if present) is the salsa write lock:
//! `index_batch` registers files and updates the workspace symbol index under
//! `db.salsa.write()`, and any interactive request must take the read lock
//! first. If the heavy work (parsing / declaration collection) happened *under*
//! the write lock, a foreground request would block for the whole chunk.
//!
//! This test drives `index_batch` over a synthetic ~N-file tree on a background
//! thread while a foreground thread takes the read lock in a tight loop
//! (`tracked_file_count`, which acquires the read lock but runs no cancellable
//! salsa query), and asserts the foreground read-lock acquisition is never
//! blocked for long — i.e. the write window stays short because parsing happens
//! off-lock.
//!
//! ```text
//! cargo test -p mir-analyzer --test indexing_responsiveness -- --ignored --nocapture
//! ```

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{AnalysisSession, IndexCancel, IndexParallelism, PhpVersion};

use self::common::create_temp_dir;

fn n_files() -> usize {
    std::env::var("MIR_BENCH_FILES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4000)
}
const CHUNK: usize = 500;

#[test]
#[ignore = "perf benchmark; run explicitly with --ignored --nocapture"]
fn background_indexing_does_not_block_interactive_reads() {
    let root = create_temp_dir("idx_responsiveness");
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
    let session = Arc::new(AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4)));
    session.ensure_all_stubs();

    let done = Arc::new(AtomicBool::new(false));
    let cancel = IndexCancel::new();

    let bg = {
        let session = Arc::clone(&session);
        let done = Arc::clone(&done);
        let cancel = cancel.clone();
        let files = files.clone();
        std::thread::spawn(move || {
            let mut max_chunk = Duration::ZERO;
            for chunk in files.chunks(CHUNK) {
                let t = Instant::now();
                session.index_batch(chunk, IndexParallelism::Rayon, &cancel);
                max_chunk = max_chunk.max(t.elapsed());
            }
            let t = Instant::now();
            session.finalize_index();
            let finalize = t.elapsed();
            done.store(true, Ordering::Relaxed);
            (max_chunk, finalize)
        })
    };

    // Foreground: time read-lock acquisition (no cancellable query) until done.
    let mut max_read = Duration::ZERO;
    let mut reads = 0u64;
    while !done.load(Ordering::Relaxed) {
        let t = Instant::now();
        let _ = session.tracked_file_count();
        max_read = max_read.max(t.elapsed());
        reads += 1;
        std::thread::sleep(Duration::from_micros(200));
    }

    let (max_chunk, finalize) = bg.join().unwrap();

    eprintln!(
        "indexing responsiveness ({total} files, chunk {CHUNK}):\n  \
         max per-chunk index_batch : {max_chunk:?}\n  \
         finalize_index            : {finalize:?}\n  \
         foreground read-lock acqs : {reads}\n  \
         max read-lock wait        : {max_read:?}"
    );

    assert!(session.contains_class("Lib\\C0"));
    assert!(session.contains_class(&format!("Lib\\C{}", total - 1)));

    // The headline guarantee: a foreground read-lock acquisition is never
    // blocked for long by the background indexer, because parsing happens
    // off-lock and only a cheap map merge runs under the write lock. A
    // regression to parsing-under-the-lock would push this into the hundreds of
    // ms (the old `set_workspace_files`-style whole-tree write window).
    assert!(
        max_read < Duration::from_millis(50),
        "foreground read-lock blocked for {max_read:?} during background indexing — \
         the write window is too long (parsing under the lock?)"
    );
}
