//! Times a language-server style workspace sweep: register the project's
//! files, replay the disk cache, then warm every file in chunks (owner-side
//! prepare, then a snapshot `warm_files` pass).
//!
//!     cargo run --release --example lsp_sweep_bench -- <composer root> <src dir> <cache dir> seed|sweep [--taint]

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use mir_analyzer::{
    discover_files, AnalysisSession, FsSourceProvider, IndexCancel, PhpVersion, Psr4Map,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = PathBuf::from(&args[1]);
    let src = PathBuf::from(&args[2]);
    let cache_dir = PathBuf::from(&args[3]);
    let mode = args[4].as_str();
    let taint = args.iter().any(|a| a == "--taint");
    // Hosts give analysis workers large stacks; rayon's default would grow one per parse.
    rayon::ThreadPoolBuilder::new()
        .stack_size(mir_analyzer::recursion::STACK_SIZE)
        .build_global()
        .expect("global thread pool");

    let files: Vec<(Arc<str>, Arc<str>)> = discover_files(&src)
        .iter()
        .filter_map(|p| {
            let text = std::fs::read_to_string(p).ok()?;
            Some((Arc::from(p.to_string_lossy().as_ref()), Arc::from(text)))
        })
        .collect();
    eprintln!("{} files", files.len());

    let resolver = Arc::new(Psr4Map::from_composer(&root).expect("composer.json"));
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(resolver)
        .with_source_provider(Arc::new(FsSourceProvider::new()))
        .with_taint_analysis(taint)
        .with_cache_dir(&cache_dir);

    let t = Instant::now();
    session.set_workspace_files(files.iter().cloned());
    eprintln!("register: {:?}", t.elapsed());

    if mode == "sweep" {
        let t = Instant::now();
        let untrusted = session.warm_start_files(&files);
        if !session.workspace_symbol_index_ready() {
            session.ensure_all_stubs();
            session.rebuild_workspace_symbol_index();
        }
        eprintln!(
            "warm_start: {:?} (untrusted {})",
            t.elapsed(),
            untrusted.len()
        );
    }

    let paths: Vec<Arc<str>> = files.iter().map(|(p, _)| p.clone()).collect();
    let chunk_size = 32 * std::thread::available_parallelism().map_or(1, |n| n.get().min(16));
    let cancel = IndexCancel::new();
    let (mut prepare, mut warm) = (Duration::ZERO, Duration::ZERO);
    for chunk in paths.chunks(chunk_size) {
        let t = Instant::now();
        session.prepare_for_query(None);
        for file in chunk {
            session.prepare_file_for_analysis(file);
        }
        prepare += t.elapsed();
        let t = Instant::now();
        let snapshot = session.snapshot();
        assert!(matches!(snapshot.warm_files(chunk, &cancel), Ok(true)));
        drop(snapshot);
        warm += t.elapsed();
    }
    eprintln!("posting writes: {}", session.ref_index_lock_count());
    eprintln!("prepare: {prepare:?}");
    eprintln!("warm_files: {warm:?}");

    let t = Instant::now();
    session.flush_analysis_cache();
    drop(session);
    eprintln!("flush + drop: {:?}", t.elapsed());
}
