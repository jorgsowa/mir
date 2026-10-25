//! Times `AnalysisSession::warm_start_files` against a warm on-disk cache.
//!
//!     cargo run --release --example warm_start_bench -- <php dir> <cache dir> seed|warm|blake

use std::sync::Arc;
use std::time::Instant;

use mir_analyzer::{discover_files, AnalysisSession, IndexCancel, IndexParallelism, PhpVersion};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root = std::path::PathBuf::from(&args[1]);
    let cache_dir = std::path::PathBuf::from(&args[2]);
    let mode = args[3].as_str();

    let files: Vec<(Arc<str>, Arc<str>)> = discover_files(&root)
        .iter()
        .filter_map(|p| {
            let src = std::fs::read_to_string(p).ok()?;
            Some((Arc::from(p.to_string_lossy().as_ref()), Arc::from(src)))
        })
        .collect();
    let bytes: usize = files.iter().map(|(_, t)| t.len()).sum();
    eprintln!("{} files, {} bytes", files.len(), bytes);

    if mode == "blake" {
        let t = Instant::now();
        for (_, text) in &files {
            std::hint::black_box(blake3::hash(text.as_bytes()));
        }
        eprintln!("blake3 serial, one pass over corpus: {:?}", t.elapsed());
        let t = Instant::now();
        for (_, text) in &files {
            std::hint::black_box(blake3::hash(text.as_bytes()).to_hex().to_string());
        }
        eprintln!("blake3+hex serial, one pass over corpus: {:?}", t.elapsed());
        return;
    }

    let t = Instant::now();
    let mut session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(&cache_dir);
    session.ensure_all_stubs();
    eprintln!("open+stubs: {:?}", t.elapsed());

    if mode == "seed" {
        let t = Instant::now();
        for chunk in files.chunks(512) {
            session.index_batch(chunk, IndexParallelism::Rayon, &IndexCancel::new());
        }
        session.finalize_index();
        eprintln!("index_batch: {:?}", t.elapsed());
        let t = Instant::now();
        drop(session);
        eprintln!("drop (flush writer): {:?}", t.elapsed());
        return;
    }

    let t = Instant::now();
    let unresolved = session.warm_start_files(&files);
    eprintln!(
        "warm_start_files: {:?} (unresolved {})",
        t.elapsed(),
        unresolved.len()
    );
}
