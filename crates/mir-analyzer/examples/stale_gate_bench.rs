//! Times `stale_reference_candidates` on a registered, never-analyzed workspace: each novel member needle vs a repeat.
//!
//!     cargo run --release --example stale_gate_bench -- <dir>

use std::sync::Arc;
use std::time::Instant;

use mir_analyzer::{discover_files, AnalysisSession, Name, PhpVersion};

fn main() {
    let root = std::env::args()
        .nth(1)
        .expect("usage: stale_gate_bench <dir>");
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    let workspace: Vec<(Arc<str>, Arc<str>)> = discover_files(std::path::Path::new(&root))
        .iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(p).ok()?;
            Some((Arc::from(p.to_string_lossy().as_ref()), Arc::from(src)))
        })
        .collect();
    let bytes: usize = workspace.iter().map(|(_, s)| s.len()).sum();
    let paths: Vec<Arc<str>> = workspace.iter().map(|(p, _)| p.clone()).collect();
    session.set_workspace_files(workspace);
    let t = Instant::now();
    session.rebuild_workspace_symbol_index();
    eprintln!(
        "{} files, {:.1} MB; symbol index {:.3}s",
        paths.len(),
        bytes as f64 / 1e6,
        t.elapsed().as_secs_f64()
    );
    session.prepare_for_query(None);
    let names = [
        "getcontainer",
        "handle",
        "dispatch",
        "normalize",
        "gettokenstring",
        "zzznomatch",
        "setlogger",
        "process",
    ];
    let snap = session.snapshot();
    for round in 0..2 {
        for n in names {
            let sym = Name::method("", n);
            let s0 = session.class_mention_stats().scans_recorded;
            let t = Instant::now();
            let stale = snap.stale_reference_candidates(&sym, &paths).unwrap();
            let dt = t.elapsed();
            let s1 = session.class_mention_stats().scans_recorded;
            eprintln!(
                "round {round} {n:>16}: {:7.1} ms, {:5} candidates, {:5} scans",
                dt.as_secs_f64() * 1e3,
                stale.len(),
                s1 - s0
            );
        }
    }
    let st = session.class_mention_stats();
    eprintln!(
        "universe {} names, {} files covered, {} mentions, {} words, scanner {:.1} MB",
        st.universe_names,
        st.files_covered,
        st.total_mentions,
        st.total_words,
        st.scanner_bytes as f64 / 1e6
    );
}
