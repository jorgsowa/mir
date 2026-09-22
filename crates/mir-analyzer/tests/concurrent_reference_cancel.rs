//! Regression guards for the session's salsa locking hazards: the
//! `index_generation()` query-stack reentrancy abort (fixed by reading the
//! epoch from an off-salsa atomic mirror), plus stress coverage for a writer
//! toggling a base class and an opener mirroring buffer text on the shared
//! rayon pool. The two deadlock shapes those hazards can hide are pinned
//! deterministically instead, since this stress run is too narrow a race to
//! rely on: see `settle_workspace_index_does_not_deadlock_on_its_own_snapshot`
//! below and `session::incremental`'s
//! `sweep_commit_does_not_deadlock_a_concurrent_file_write` unit test.

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
/// Distinct paths the opener thread cycles through — kept small since past 32 files reconciliation switches to a full rebuild.
const OPENED_FILES: usize = 8;
/// Gap between the opener's mirror writes, far above an editor's real rate.
const OPEN_INTERVAL: Duration = Duration::from_millis(2);
/// Budget for the single-threaded settle guard, which is sub-second when it is not deadlocked.
const SETTLE_BUDGET: Duration = Duration::from_secs(30);
/// Generous upper bound for the stress test — turns a deadlock into a reported failure, not a performance gate.
const STRESS_BUDGET: Duration = Duration::from_secs(300);

/// Kills the test binary if the guarded section outlives its budget, so a deadlock reports which test stalled instead of hanging until CI's own timeout.
struct Watchdog {
    finished: Arc<AtomicBool>,
}

impl Watchdog {
    fn new(name: &'static str, budget: Duration) -> Self {
        let finished = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&finished);
        std::thread::spawn(move || {
            let deadline = Instant::now() + budget;
            while Instant::now() < deadline {
                if flag.load(Ordering::Relaxed) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            eprintln!("{name}: no progress within {budget:?} — deadlocked");
            std::process::exit(101);
        });
        Self { finished }
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        self.finished.store(true, Ordering::Relaxed);
    }
}

#[test]
fn cancelled_reanalysis_does_not_wait_behind_stub_writer_blocked_by_snapshot() {
    let _watchdog = Watchdog::new(
        "cancelled_reanalysis_does_not_wait_behind_stub_writer_blocked_by_snapshot",
        STRESS_BUDGET,
    );
    // Keep a Salsa snapshot alive, then start stub ingestion. The writer takes
    // the session write lock before Salsa waits for this snapshot to unwind.
    // A reanalysis started afterwards is therefore queued at the RwLock. Its
    // cancellation must release it even though the writer cannot proceed yet.
    let session = Arc::new(AnalysisSession::new(PhpVersion::LATEST));
    let file: Arc<str> = Arc::from("queued.php");
    session.upsert_source_file(
        file.clone(),
        Arc::from("<?php class Queued {}\n"),
        salsa::Durability::LOW,
    );
    let held_snapshot = session.snapshot_db();

    let writer_session = Arc::clone(&session);
    let writer = std::thread::spawn(move || writer_session.ensure_all_stubs());
    // Give the writer time to acquire the outer lock and block in Salsa on
    // `held_snapshot`. The snapshot is deliberately retained until after the
    // reanalysis has proved it can observe cancellation.
    std::thread::sleep(Duration::from_millis(30));

    let cancel = IndexCancel::new();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let reader_session = Arc::clone(&session);
    let reader_file = file.clone();
    let reader_cancel = cancel.clone();
    let reader = std::thread::spawn(move || {
        let result = reader_session.reanalyze_files_cancellable(&[reader_file], &reader_cancel);
        done_tx.send(result).unwrap();
    });

    std::thread::sleep(Duration::from_millis(30));
    assert!(
        done_rx.try_recv().is_err(),
        "reanalysis did not block behind the stub writer"
    );
    cancel.cancel();
    let result = done_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("cancelled reanalysis remained queued behind the stub writer");
    assert!(result.is_empty());

    drop(held_snapshot);
    reader.join().unwrap();
    writer.join().unwrap();
}

/// Reconciling the symbol index must not hold a database handle across its own merge write, or it deadlocks itself once the symbol-index singleton exists.
#[test]
fn settle_workspace_index_does_not_deadlock_on_its_own_snapshot() {
    let _watchdog = Watchdog::new(
        "settle_workspace_index_does_not_deadlock_on_its_own_snapshot",
        SETTLE_BUDGET,
    );
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(
        Arc::from("Owner.php"),
        Arc::from("<?php\nnamespace Lib;\nclass Owner {}\n"),
    );
    // Only a live singleton makes a mirror write pending.
    session.rebuild_workspace_symbol_index();
    session.upsert_source_file(
        Arc::from("Mirrored.php"),
        Arc::from("<?php\nnamespace Lib;\nclass Mirrored {}\n"),
        salsa::Durability::LOW,
    );

    session.settle_workspace_index();

    assert!(session.contains_class("Lib\\Mirrored"));
}

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
    let _watchdog = Watchdog::new(
        "concurrent_writes_do_not_abort_parallel_reference_reads",
        STRESS_BUDGET,
    );
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
                        // Every 50th write mints a brand-new class name so the
                        // mention-universe epoch churns while readers fetch /
                        // rebuild the gate scanner and commit mention sets —
                        // the class-mention index's own concurrency hazard.
                        let fresh = if n.is_multiple_of(50) {
                            format!("\nclass W{w}Gen{n} {{}}\n")
                        } else {
                            String::new()
                        };
                        let src: Arc<str> = Arc::from(
                            format!("<?php\nnamespace Lib;\nclass W{w} {{\n    public function f(): int {{ return {n}; }}\n}}\n{fresh}")
                                .as_str(),
                        );
                        session.ingest_file(path, src);
                    }
                    std::thread::sleep(Duration::from_micros(80));
                }
            })
        })
        .collect();

    // Opener: the host's `did_open` write path, kept running as background pressure for the sweep/settle deadlocks pinned by the deterministic guards.
    let opener = {
        let session = Arc::clone(&session);
        let writer_stop = Arc::clone(&writer_stop);
        std::thread::spawn(move || {
            let mut n: usize = 0;
            while !writer_stop.load(Ordering::Relaxed) {
                n += 1;
                let slot = n % OPENED_FILES;
                let path: Arc<str> = Arc::from(format!("opened/O{slot}.php").as_str());
                let src: Arc<str> = Arc::from(
                    format!("<?php\nnamespace Lib;\nclass O{slot} {{ const M = {n}; }}\n").as_str(),
                );
                session.upsert_source_file(path, src, salsa::Durability::LOW);
                std::thread::sleep(OPEN_INTERVAL);
            }
        })
    };

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
                        session.indexed_references_to(
                            &symbol,
                            &callers,
                            false,
                            mir_analyzer::ReferenceIncludes::Plain,
                            &|| Instant::now() > deadline,
                        )
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
    opener.join().expect("opener thread panicked");

    // Surviving to here without a process abort is the assertion. Confirm the
    // session is still usable after the concurrent churn.
    assert!(session.contains_class("Lib\\Base"));
    assert!(session.contains_class("Lib\\C0"));
}
