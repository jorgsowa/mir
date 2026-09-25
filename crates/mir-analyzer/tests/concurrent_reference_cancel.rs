//! Concurrency guards for the single-owner session: one thread owns the
//! [`AnalysisSession`] and writes, readers query [`AnalysisSnapshot`]s it hands
//! out. An owner write cancels in-flight snapshot queries; they must unwind
//! as `Err(Cancelled)` rather than abort (salsa's `ZalsaLocal` reentrancy
//! check) or deadlock the owner waiting for their handles.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{
    AnalysisSession, AnalysisSnapshot, IndexCancel, IndexParallelism, Name, PhpVersion,
    ReferenceIncludes,
};

const CALLERS: usize = 250;
const READERS: usize = 10;
const READ_ITERS: usize = 60;
/// Distinct paths the owner's mirror writes cycle through — kept small since past 32 files reconciliation switches to a full rebuild.
const OPENED_FILES: usize = 8;
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

/// Reconciling the symbol index must not hold a database handle across its own merge write, or it deadlocks itself once the symbol-index singleton exists.
#[test]
fn settle_workspace_index_does_not_deadlock_on_its_own_snapshot() {
    let _watchdog = Watchdog::new(
        "settle_workspace_index_does_not_deadlock_on_its_own_snapshot",
        SETTLE_BUDGET,
    );
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
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
    // `analyze_file` depends on the shared `Base` class query and re-runs when
    // the owner invalidates `Base` — keeping reader queries long enough for
    // owner writes to land mid-query. `marker` varies the body so a re-index
    // actually bumps the revision.
    let mut body = format!("<?php\nnamespace Lib;\nclass C{i} {{\n    const M = {marker};\n");
    for m in 0..24 {
        body.push_str(&format!(
            "    public function go{m}(Base $b): int {{ return $b->run() + $b->run() + {m}; }}\n"
        ));
    }
    body.push_str("}\n");
    Arc::from(body.as_str())
}

/// A reader asking the owner for a fresh snapshot, reporting whether its
/// previous query completed.
struct SnapshotRequest {
    completed_last: bool,
    reply: mpsc::Sender<AnalysisSnapshot>,
}

fn request_snapshot(
    owner: &mpsc::Sender<SnapshotRequest>,
    completed_last: bool,
) -> AnalysisSnapshot {
    let (reply, snapshot) = mpsc::channel();
    owner
        .send(SnapshotRequest {
            completed_last,
            reply,
        })
        .expect("owner alive while readers run");
    snapshot.recv().expect("owner answers every request")
}

/// Answers `request` with a snapshot of the current revision; returns
/// whether the reader's previous query completed.
fn serve(session: &AnalysisSession, request: SnapshotRequest) -> bool {
    let _ = request.reply.send(session.snapshot());
    request.completed_last
}

#[test]
fn owner_writes_cancel_snapshot_readers_without_aborting() {
    let _watchdog = Watchdog::new(
        "owner_writes_cancel_snapshot_readers_without_aborting",
        STRESS_BUDGET,
    );
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let base_path: Arc<str> = Arc::from("Base.php");
    session.ingest_file(base_path.clone(), base_source(0));

    let callers: Arc<[Arc<str>]> = (0..CALLERS)
        .map(|i| {
            let path = caller_path(i);
            session.ingest_file(path.clone(), caller_source(i, 0));
            path
        })
        .collect();
    // Only a live singleton makes mirror writes pending, so the owner's
    // per-round settle has reconciliation work to do.
    session.rebuild_workspace_symbol_index();

    // Two full caller batches with differing bodies; the owner toggles
    // between them so each re-index bumps the revision.
    let batch_a: Vec<(Arc<str>, Arc<str>)> = (0..CALLERS)
        .map(|i| (caller_path(i), caller_source(i, 1)))
        .collect();
    let batch_b: Vec<(Arc<str>, Arc<str>)> = (0..CALLERS)
        .map(|i| (caller_path(i), caller_source(i, 2)))
        .collect();

    let (request_tx, requests) = mpsc::channel::<SnapshotRequest>();
    let readers: Vec<_> = (0..READERS)
        .map(|r| {
            let owner = request_tx.clone();
            let callers = Arc::clone(&callers);
            std::thread::spawn(move || {
                let symbol = Name::method("Lib\\Base", "run");
                let (mut completed, mut cancelled) = (0usize, 0usize);
                let mut completed_last = false;
                for i in 0..READ_ITERS {
                    let snap = request_snapshot(&owner, completed_last);
                    // Half the readers alternate with the sweep a host runs
                    // for open files, which commits analyses from the reader
                    // thread.
                    let result = if r % 2 == 0 && i % 2 == 1 {
                        snap.warm_files(&callers, &IndexCancel::new()).map(drop)
                    } else {
                        snap.indexed_references_to(
                            &symbol,
                            &callers,
                            false,
                            ReferenceIncludes::Plain,
                        )
                        .map(drop)
                    };
                    completed_last = result.is_ok();
                    if completed_last {
                        completed += 1;
                    } else {
                        cancelled += 1;
                    }
                }
                (completed, cancelled)
            })
        })
        .collect();
    drop(request_tx);

    // Owner: one host write path per round. Odd rounds answer the queued
    // requests and write again at once, cancelling readers mid-query; even
    // rounds stay quiet until readers report `READERS` completed queries.
    // Ends once every reader has hung up.
    let mut round: usize = 0;
    'owner: loop {
        round += 1;
        match round % 4 {
            0 => session.ingest_file(base_path.clone(), base_source(round)),
            1 => {
                // Every 50th write mints a brand-new class name so the
                // mention-universe epoch churns while readers fetch the gate
                // scanner and commit mention sets.
                let fresh = if round.is_multiple_of(50) {
                    format!("\nclass WGen{round} {{}}\n")
                } else {
                    String::new()
                };
                let src = format!(
                    "<?php\nnamespace Lib;\nclass W {{\n    public function f(): int {{ return {round}; }}\n}}\n{fresh}"
                );
                session.ingest_file(Arc::from("writers/W.php"), Arc::from(src.as_str()));
            }
            2 => {
                let slot = round % OPENED_FILES;
                let src =
                    format!("<?php\nnamespace Lib;\nclass O{slot} {{ const M = {round}; }}\n");
                session.upsert_source_file(
                    Arc::from(format!("opened/O{slot}.php").as_str()),
                    Arc::from(src.as_str()),
                    salsa::Durability::LOW,
                );
            }
            _ => {
                let batch = if round % 8 == 3 { &batch_a } else { &batch_b };
                session.index_batch(batch, IndexParallelism::Rayon, &IndexCancel::new());
            }
        }
        session.prepare_for_query(None);
        loop {
            match requests.try_recv() {
                Ok(request) => {
                    serve(&session, request);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break 'owner,
            }
        }
        if round.is_multiple_of(2) {
            let mut completions = 0;
            while completions < READERS {
                match requests.recv() {
                    Ok(request) => completions += usize::from(serve(&session, request)),
                    Err(mpsc::RecvError) => break 'owner,
                }
            }
        }
    }

    let (completed, cancelled) = readers
        .into_iter()
        .map(|reader| reader.join().expect("reader thread panicked"))
        .fold((0, 0), |(c, x), (rc, rx)| (c + rc, x + rx));
    assert!(cancelled > 0, "owner writes never cancelled a reader");
    assert!(completed > 0, "no reader query completed between writes");
    assert!(session.contains_class("Lib\\Base"));
    assert!(session.contains_class("Lib\\C0"));
    let refs = session
        .indexed_references_to(
            &Name::method("Lib\\Base", "run"),
            &callers,
            false,
            ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        !refs.is_empty(),
        "callers' run() sites must survive the churn"
    );
}
