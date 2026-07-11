//! Bounded-memo guard for the per-scope / per-function inference queries.
//!
//! `infer_scope` and `infer_function` are keyed on resolved FQNs, so every
//! rename mints a brand-new memo key; without an eviction policy a long
//! editing session grows the memo tables monotonically. Both queries carry
//! `lru = 4096` — these tests prove salsa actually drops the evicted values
//! (observed through `Weak` handles on the memoized `Arc` results) once a
//! write opens a new revision.

use std::sync::{Arc, Weak};

use mir_analyzer::db::{
    file_scopes, infer_function, infer_scope, FunctionInferenceResult, MirDatabase,
    ScopeInferenceResult,
};
use mir_analyzer::{AnalysisSession, PhpVersion};

const LRU_CAP: usize = 4096;
/// Enough distinct keys past the cap that eviction is unambiguous.
const STORM: usize = LRU_CAP + 300;

fn storm_source(n: usize) -> String {
    let mut s = String::from("<?php\n");
    for i in 0..n {
        s.push_str(&format!("function f{i}(int $x): int {{ return $x + 1; }}\n"));
    }
    s
}

fn storm_session() -> (AnalysisSession, Arc<str>) {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let path: Arc<str> = Arc::from("/memo_bounds/storm.php");
    session.ingest_file(path.clone(), Arc::from(storm_source(STORM)));
    (session, path)
}

fn alive_count<T>(weaks: &[Weak<T>]) -> usize {
    weaks.iter().filter(|w| w.upgrade().is_some()).count()
}

#[test]
fn infer_function_memos_bounded_under_rename_storm() {
    let (session, path) = storm_session();

    let weaks: Vec<Weak<FunctionInferenceResult>> = {
        let db = session.snapshot_db();
        let file = db.lookup_source_file(path.as_ref()).unwrap();
        (0..STORM)
            .map(|i| {
                let result = infer_function(&db, file, Arc::from(format!("f{i}")))
                    .unwrap_or_else(|| panic!("f{i} not found"));
                Arc::downgrade(&result)
            })
            .collect()
    };
    assert_eq!(alive_count(&weaks), STORM, "memos live before eviction");

    // LRU eviction runs when the next revision opens; any write triggers it.
    session.set_file_text(Arc::from("/memo_bounds/other.php"), Arc::from("<?php\n"));

    let alive = alive_count(&weaks);
    assert!(
        alive <= LRU_CAP,
        "infer_function memo table not bounded: {alive} values alive after \
         {STORM} distinct keys (lru = {LRU_CAP})"
    );
    assert!(alive > 0, "eviction dropped everything — lru misconfigured");
}

#[test]
fn infer_scope_memos_bounded_under_rename_storm() {
    let (session, path) = storm_session();

    let weaks: Vec<Weak<ScopeInferenceResult>> = {
        let db = session.snapshot_db();
        let file = db.lookup_source_file(path.as_ref()).unwrap();
        let scopes = file_scopes(&db, file);
        assert!(scopes.len() > LRU_CAP, "fixture must overflow the lru cap");
        scopes
            .iter()
            .map(|key| Arc::downgrade(&infer_scope(&db, file, key.clone())))
            .collect()
    };
    assert_eq!(alive_count(&weaks), weaks.len(), "memos live before eviction");

    session.set_file_text(Arc::from("/memo_bounds/other.php"), Arc::from("<?php\n"));

    let alive = alive_count(&weaks);
    assert!(
        alive <= LRU_CAP,
        "infer_scope memo table not bounded: {alive} values alive after \
         {} distinct keys (lru = {LRU_CAP})",
        weaks.len()
    );
    assert!(alive > 0, "eviction dropped everything — lru misconfigured");
}
