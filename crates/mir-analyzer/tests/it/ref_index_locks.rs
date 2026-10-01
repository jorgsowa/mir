//! The reference index's posting lock is taken per changed file, never per
//! candidate file: resolving a path to its id is a shared read outside it.
//! Hosts assert on `ref_index_lock_count` to catch per-request rebuilds, so
//! the count must not grow with the workspace.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, IndexCancel, Name, PhpVersion, ReferenceIncludes};

const TARGET: &str = "<?php\nclass Target { public function process(): void {} }\n";

fn target_with(edit: usize) -> Arc<str> {
    Arc::from(format!("{TARGET}// edit {edit}\n").as_str())
}

/// A declaring class plus `noise` classes that all mention `process`, so the
/// reference gate admits every file as a candidate.
fn workspace(noise: usize) -> (AnalysisSession, Vec<Arc<str>>) {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    let mut files: Vec<Arc<str>> = vec![Arc::from("Target.php")];
    session.ingest_file(files[0].clone(), target_with(0));
    for i in 0..noise {
        let path: Arc<str> = Arc::from(format!("Noise{i}.php").as_str());
        let text = format!(
            "<?php\nclass Noise{i} {{ public function process(): int {{ return {i}; }} }}\n"
        );
        session.ingest_file(path.clone(), Arc::from(text.as_str()));
        files.push(path);
    }
    session.prepare_for_query(None);
    (session, files)
}

fn references(session: &mut AnalysisSession, files: &[Arc<str>]) -> usize {
    session
        .indexed_references_to(
            &Name::method("Target", "process"),
            files,
            true,
            ReferenceIncludes::Plain,
            &|| false,
        )
        .unwrap()
        .len()
}

fn sweep(session: &mut AnalysisSession, files: &[Arc<str>]) {
    session.reanalyze_files_cancellable(files, &IndexCancel::new());
}

fn locks_taken(session: &mut AnalysisSession, work: impl FnOnce(&mut AnalysisSession)) -> u64 {
    let before = session.ref_index_lock_count();
    work(session);
    session.ref_index_lock_count() - before
}

#[test]
fn unchanged_sweeps_and_stale_scans_take_no_index_locks() {
    for noise in [8, 64] {
        let (mut session, files) = workspace(noise);
        references(&mut session, &files);
        sweep(&mut session, &files);

        let symbol = Name::method("Target", "process");
        let scan = locks_taken(&mut session, |s| {
            s.snapshot()
                .stale_reference_candidates(&symbol, &files)
                .unwrap();
        });
        let resweep = locks_taken(&mut session, |s| sweep(s, &files));
        assert_eq!((scan, resweep), (0, 0), "{noise} noise files");
    }
}

/// An edit followed by the dependents sweep (twice, as a host that re-publishes
/// on every event does) and a references read.
#[test]
fn edit_sweep_and_read_locks_do_not_scale_with_candidate_count() {
    let cycle = |noise: usize| {
        let (mut session, files) = workspace(noise);
        references(&mut session, &files);
        sweep(&mut session, &files);
        locks_taken(&mut session, |s| {
            s.ingest_file(files[0].clone(), target_with(1));
            sweep(s, &files);
            sweep(s, &files);
            assert_eq!(references(s, &files), 1, "the declaration");
        })
    };
    let (small, large) = (cycle(8), cycle(64));
    assert_eq!(small, large, "locks grew with the candidate count");
    assert!(small <= 16, "{small} locks for one edit/read cycle");
}
