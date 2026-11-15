//! A cancelled snapshot reference query unwinds mid-file instead of finishing the analysis.

use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{AnalysisSession, IndexCancel, Name, PhpVersion, ReferenceIncludes};

/// Thousands of loop-heavy functions, so a cancel shortly after the query starts lands mid-file.
fn slow_file(functions: usize) -> String {
    let mut src = String::from("<?php\nnamespace App;\nfunction target(): void {}\n");
    for n in 0..functions {
        src.push_str(&format!(
            "function f{n}(array $xs): void {{ target(); $a = []; foreach ($xs as $x) {{ \
             foreach ($xs as $y) {{ for ($i = 0; $i < 3; $i++) {{ $a[] = [$x, $y, $i]; }} }} }} }}\n"
        ));
    }
    src
}

#[test]
fn cancelled_reference_query_stops_mid_file_without_memoizing() {
    let path: Arc<str> = Arc::from("slow.php");
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(path.clone(), Arc::from(slow_file(3000).as_str()));
    session.prepare_for_query(None);
    let symbol = Name::function("App\\target");
    let files = vec![path];

    let snap = session.snapshot();
    let cancel = IndexCancel::new();
    let canceller = {
        let cancel = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(200));
            cancel.cancel();
        })
    };
    let started = Instant::now();
    let result =
        snap.indexed_references_to(&symbol, &files, false, ReferenceIncludes::Plain, &cancel);
    let elapsed = started.elapsed();
    canceller.join().unwrap();
    assert!(
        matches!(result, Err(salsa::Cancelled::Local)),
        "{result:?} after {elapsed:?}"
    );

    // A memoized answer would be returned before `cancel` is consulted.
    let again =
        snap.indexed_references_to(&symbol, &files, false, ReferenceIncludes::Plain, &cancel);
    assert!(matches!(again, Err(salsa::Cancelled::Local)), "{again:?}");
}
