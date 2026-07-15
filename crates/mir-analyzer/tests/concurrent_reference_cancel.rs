//! Regression guard for the salsa query-stack reentrancy abort.
//!
//! Root cause: `index_generation()` fetched the workspace epoch by running a
//! salsa input read (`WorkspaceRevision::revision`) through the shared,
//! non-snapshot db handle. That read borrows the handle's single `ZalsaLocal`
//! query stack — a `RefCell` — so when a background indexer calls it while
//! another thread runs any salsa read on the same handle, the two race on that
//! `RefCell` and salsa's `try_borrow_mut().unwrap_unchecked()` hits a
//! non-unwinding `unreachable_unchecked`, aborting the whole process (SIGABRT,
//! gated on debug assertions so it fires under `cargo test`). The fix reads the
//! epoch from an off-salsa atomic mirror instead, so it never touches salsa.
//!
//! This test runs several reader threads (parallel `indexed_references_to` /
//! `reanalyze_dependents`) plus a background indexer looping `index_batch`
//! (which calls `index_generation`) and a writer toggling a base class, all on
//! the shared rayon pool. Before the fix it aborts deterministically within a
//! few seconds; after it, it completes. `catch_unwind` in the readers absorbs
//! the ordinary `salsa::Cancelled` unwinds (the abort is non-unwinding, so a
//! regression still kills the test binary).

use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{AnalysisSession, IndexCancel, IndexParallelism, Name, PhpVersion};

const CALLERS: usize = 250;
const READERS: usize = 10;
const WRITERS: usize = 4;
const READ_ITERS: usize = 60;
const CALL_BUDGET: Duration = Duration::from_millis(80);

fn caller_path(i: usize) -> Arc<str> {
    Arc::from(format!("callers/C{i}.php").as_str())
}

fn base_source(marker: usize) -> Arc<str> {
    Arc::from(format!("<?php\nnamespace Lib;\nclass Base {{\n    public function run(): int {{ return {marker}; }}\n}}\n").as_str())
}

fn caller_source(i: usize, marker: usize) -> Arc<str> {
    // Wide body: every method takes `Base` and calls `run()`, so each caller's
    // `analyze_file` depends on the shared `Base` class query and re-runs when a
    // writer invalidates `Base` — keeping the rayon pool saturated with salsa
    // reads so the indexer's shared-handle read reliably overlaps one. `marker`
    // varies the body so a re-index actually bumps the revision.
    let mut body = format!("<?php\nnamespace Lib;\nclass C{i} {{\n    const M = {marker};\n");
    for m in 0..24 {
        body.push_str(&format!(
            "    public function go{m}(Base $b): int {{ return $b->run() + $b->run() + {m}; }}\n"
        ));
    }
    body.push_str("}\n");
    Arc::from(body.as_str())
}

#[test]
fn concurrent_writes_do_not_abort_parallel_reference_reads() {
    let session = Arc::new(AnalysisSession::new(PhpVersion::LATEST));
    session.ensure_all_stubs();

    let base_path: Arc<str> = Arc::from("Base.php");
    session.ingest_file(base_path.clone(), base_source(0));

    let mut callers: Vec<Arc<str>> = Vec::with_capacity(CALLERS);
    for i in 0..CALLERS {
        let path = caller_path(i);
        session.ingest_file(path.clone(), caller_source(i, 0));
        callers.push(path);
    }

    // Two full caller batches with differing bodies; the background indexer
    // toggles between them so each re-index bumps the revision.
    let batch_a: Vec<(Arc<str>, Arc<str>)> = (0..CALLERS)
        .map(|i| (caller_path(i), caller_source(i, 1)))
        .collect();
    let batch_b: Vec<(Arc<str>, Arc<str>)> = (0..CALLERS)
        .map(|i| (caller_path(i), caller_source(i, 2)))
        .collect();

    let symbol = Name::class("Lib\\Base");
    let writer_stop = Arc::new(AtomicBool::new(false));

    // Writers: concurrent `ingest_file` calls, each toggling its own file.
    // `ingest_file` derives the file's defined-symbol set right after writing,
    // and if that derivation runs the `collect_file_definitions` tracked query
    // on the shared (non-snapshot) db handle, two writers doing it at once race
    // the shared `ZalsaLocal` query stack — the same abort as the indexer path.
    // Writer 0 bumps the shared `Base` (also driving revision churn for the
    // readers); the rest each own a distinct file.
    let writers: Vec<_> = (0..WRITERS)
        .map(|w| {
            let session = Arc::clone(&session);
            let writer_stop = Arc::clone(&writer_stop);
            let base_path = base_path.clone();
            std::thread::spawn(move || {
                let mut n: usize = 0;
                while !writer_stop.load(Ordering::Relaxed) {
                    n += 1;
                    if w == 0 {
                        session.ingest_file(base_path.clone(), base_source(n));
                    } else {
                        let path: Arc<str> = Arc::from(format!("writers/W{w}.php").as_str());
                        let src: Arc<str> = Arc::from(
                            format!("<?php\nnamespace Lib;\nclass W{w} {{\n    public function f(): int {{ return {n}; }}\n}}\n")
                                .as_str(),
                        );
                        session.ingest_file(path, src);
                    }
                    std::thread::sleep(Duration::from_micros(80));
                }
            })
        })
        .collect();

    // Background indexer: re-runs the parallel `index_batch` (rayon-side
    // `collect_file_declarations`) against the readers' parallel `analyze_file`
    // on the shared rayon pool — the frameworks-suite interleaving.
    let indexer = {
        let session = Arc::clone(&session);
        let writer_stop = Arc::clone(&writer_stop);
        std::thread::spawn(move || {
            let mut toggle = false;
            while !writer_stop.load(Ordering::Relaxed) {
                let batch = if toggle { &batch_a } else { &batch_b };
                toggle = !toggle;
                let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    session.index_batch(batch, IndexParallelism::Rayon, &IndexCancel::new());
                }));
            }
        })
    };

    let readers: Vec<_> = (0..READERS)
        .map(|r| {
            let session = Arc::clone(&session);
            let callers = callers.clone();
            let symbol = symbol.clone();
            let base_path = base_path.clone();
            std::thread::spawn(move || {
                for _ in 0..READ_ITERS {
                    // Each call is wrapped in `catch_unwind`: a `salsa::Cancelled`
                    // raised by the serial warm-up phase unwinds normally and is
                    // caught here (in production the host's panic guard does this).
                    // The reentrancy abort we guard against is a NON-unwinding
                    // `unreachable_unchecked` — `catch_unwind` cannot absorb it, so
                    // if it regresses the whole test binary aborts and CI fails.
                    // A bounded deadline keeps a sustained cancellation stream from
                    // livelocking the Phase-2 retry loop, and drives the exact
                    // cancellable path the LSP server uses.
                    let deadline = Instant::now() + CALL_BUDGET;
                    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                        session.indexed_references_to(&symbol, &callers, false, &|| {
                            Instant::now() > deadline
                        })
                    }));
                    // Every other reader also drives the incremental sweep, which
                    // shares the same rayon-join hazard.
                    if r % 2 == 0 {
                        let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
                            session.reanalyze_dependents(&base_path)
                        }));
                    }
                }
            })
        })
        .collect();

    for reader in readers {
        reader.join().expect("reader thread panicked");
    }
    writer_stop.store(true, Ordering::Relaxed);
    for writer in writers {
        writer.join().expect("writer thread panicked");
    }
    indexer.join().expect("indexer thread panicked");

    // Surviving to here without a process abort is the assertion. Confirm the
    // session is still usable after the concurrent churn.
    assert!(session.contains_class("Lib\\Base"));
    assert!(session.contains_class("Lib\\C0"));
}
