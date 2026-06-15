use std::path::PathBuf;
use std::process;
use std::sync::Arc;
use std::time::{Duration, Instant};

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use mir_analyzer::{
    dead_code_issue_kinds, discover_files, AnalysisResult, AnalysisSession, BatchOptions,
    IndexCancel, IndexParallelism, PhpVersion,
};

use crate::config::Config;
use crate::{Cli, OutputFormat};

// ---------------------------------------------------------------------------
// Public entry points — one per project type
// ---------------------------------------------------------------------------

pub fn run_composer_flow(
    cli: &Cli,
    config: &Config,
    config_base: &std::path::Path,
    composer_root: &std::path::Path,
) -> (Vec<PathBuf>, AnalysisResult, Duration) {
    let map = match mir_analyzer::composer::Psr4Map::from_composer(composer_root) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("mir: composer error: {e}");
            process::exit(2);
        }
    };

    let version = resolve_php_version(config);
    let cache_dir = if cli.no_cache {
        None
    } else {
        Some(
            cli.cache_dir
                .clone()
                .unwrap_or_else(|| composer_root.join(".mir/cache")),
        )
    };
    let (stub_files, stub_dirs) = collect_stub_paths(config, config_base);
    let mut session = build_session(version, cache_dir, stub_files, stub_dirs);
    session = session.with_psr4(Arc::new(map.clone()));

    let opts = build_batch_opts(cli.find_dead_code);

    // Lazy vendor by default: only eagerly load `autoload.files` entries.
    // Set `MIR_EAGER_VENDOR=1` to parse every vendor file upfront.
    let eager_vendor = std::env::var("MIR_EAGER_VENDOR")
        .ok()
        .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "yes"));

    let ignore_dirs = resolve_ignore_dirs(config, config_base);

    let analyze_whole_composer_project = cli.paths.is_empty()
        || cli
            .paths
            .first()
            .and_then(|p| p.canonicalize().ok())
            .is_some_and(|p| p == composer_root);

    let discovered: Vec<PathBuf> = if analyze_whole_composer_project {
        map.project_files()
    } else {
        discover_files(&cli.paths[0])
    };

    let files = filter_ignore(discovered, &ignore_dirs, composer_root);

    if files.is_empty() {
        if !cli.quiet {
            eprintln!("No PHP files found via composer.json.");
        }
        process::exit(0);
    }

    if !cli.quiet {
        eprintln!(
            "{} Analyzing {} file{} (from composer.json)...",
            "mir".bold().green(),
            files.len(),
            if files.len() == 1 { "" } else { "s" },
        );
    }

    session.ensure_all_stubs();

    if eager_vendor {
        let vendor_files = map.vendor_files();
        if !vendor_files.is_empty() {
            if !cli.quiet {
                eprintln!(
                    "mir: scanning {} vendor files for types...",
                    vendor_files.len()
                );
            }
            index_vendor_chunked(&session, &vendor_files);
        }
    } else {
        let cancel = IndexCancel::new();
        let outcome = session.index_vendor_eager_files(IndexParallelism::Rayon, &cancel);
        if outcome.registered > 0 && !cli.quiet {
            eprintln!(
                "mir: eager-loading {} files-autoload entries ({} classmap entries available lazily)",
                outcome.registered,
                map.classmap_len()
            );
        }
    }

    let show_progress = !cli.no_progress && !cli.quiet && matches!(cli.format, OutputFormat::Text);
    let start = Instant::now();
    let result = run_with_progress(session, &files, opts, show_progress);
    (files, result, start.elapsed())
}

pub fn run_plain_flow(
    cli: &Cli,
    config: &Config,
    config_base: &std::path::Path,
) -> (Vec<PathBuf>, AnalysisResult, Duration) {
    let paths: Vec<PathBuf> = if cli.paths.is_empty() {
        vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
    } else {
        cli.paths.clone()
    };

    let ignore_dirs = resolve_ignore_dirs(config, config_base);

    let scan_roots: Vec<PathBuf> = if !config.project_dirs.is_empty() && cli.paths.is_empty() {
        config
            .project_dirs
            .iter()
            .map(|d| {
                let p = PathBuf::from(d);
                if p.is_absolute() {
                    p
                } else {
                    config_base.join(d)
                }
            })
            .collect()
    } else {
        paths
    };

    let cwd_abs = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let files: Vec<PathBuf> = scan_roots
        .iter()
        .flat_map(|p| discover_files(p))
        .filter(|p| {
            if ignore_dirs.is_empty() {
                return true;
            }
            let abs = if p.is_absolute() {
                p.clone()
            } else {
                cwd_abs.join(p)
            };
            !ignore_dirs.iter().any(|ig| abs.starts_with(ig))
        })
        .collect();

    if files.is_empty() {
        if !cli.quiet {
            eprintln!("No PHP files found.");
        }
        process::exit(0);
    }

    if !cli.quiet {
        eprintln!(
            "{} Analyzing {} file{}{}...",
            "mir".bold().green(),
            files.len(),
            if files.len() == 1 { "" } else { "s" },
            cli.php_version
                .as_deref()
                .map(|v| format!(" (PHP {v})"))
                .unwrap_or_default(),
        );
    }

    let version = resolve_php_version(config);
    let cache_dir = if cli.no_cache {
        None
    } else {
        cli.cache_dir.clone().or_else(default_cache_dir)
    };
    let (stub_files, stub_dirs) = collect_stub_paths(config, config_base);
    let session = build_session(version, cache_dir, stub_files, stub_dirs);
    let opts = build_batch_opts(cli.find_dead_code);

    session.ensure_all_stubs();

    // Collect type definitions from ignore_dirs (vendor) — no error reporting there.
    if !ignore_dirs.is_empty() {
        let vendor_files: Vec<PathBuf> =
            ignore_dirs.iter().flat_map(|p| discover_files(p)).collect();
        if !vendor_files.is_empty() {
            if !cli.quiet {
                eprintln!(
                    "mir: scanning {} vendor files for types...",
                    vendor_files.len()
                );
            }
            index_vendor_chunked(&session, &vendor_files);
        }
    }

    let show_progress = !cli.no_progress && !cli.quiet && matches!(cli.format, OutputFormat::Text);
    let start = Instant::now();
    let result = run_with_progress(session, &files, opts, show_progress);
    (files, result, start.elapsed())
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Default number of vendor files indexed per `index_batch` chunk. Sized for a
/// short write-lock window; override with `MIR_INDEX_CHUNK` for tuning.
const VENDOR_INDEX_CHUNK: usize = 512;

/// Build the workspace symbol index from vendor files via the chunked
/// background-indexing engine (`index_batch` + `finalize_index`) instead of the
/// one-shot `collect_definitions`.
///
/// The CLI is a batch run with no concurrent interactive reads and no
/// cancellation, so the per-chunk short-write-window and `IndexCancel` are inert
/// here — the win over `collect_definitions` is **transient peak RSS**: this
/// reads and collects one chunk at a time, so the all-files read buffer and the
/// all-files `StubSlice` clones never coexist. The registered file *texts* are
/// retained by salsa either way.
///
/// No `finalize_index` (full rebuild) is issued: each `index_batch` merges its
/// chunk into the singleton incrementally, and `incremental_index_matches_-
/// full_rebuild` proves the incrementally-merged index equals a full rebuild for
/// arbitrary chunk order — so a trailing rebuild here would be pure duplicate
/// work. (A long-lived consumer that *edits* after warm-up still calls
/// `finalize_index` once; a one-shot batch run that only reads does not.)
fn index_vendor_chunked(session: &AnalysisSession, vendor_files: &[PathBuf]) {
    use rayon::prelude::*;

    let chunk_size = std::env::var("MIR_INDEX_CHUNK")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(VENDOR_INDEX_CHUNK);

    let cancel = IndexCancel::new();
    for paths in vendor_files.chunks(chunk_size) {
        // Read this chunk's texts off-thread, in parallel, then index and drop —
        // keeping only one chunk's worth of transient buffers alive at a time.
        let pairs: Vec<(Arc<str>, Arc<str>)> = paths
            .par_iter()
            .filter_map(|p| {
                let src = std::fs::read_to_string(p).ok()?;
                Some((
                    Arc::from(p.to_string_lossy().as_ref()),
                    Arc::from(src.as_str()),
                ))
            })
            .collect();
        session.index_batch(&pairs, IndexParallelism::Rayon, &cancel);
    }
}

pub fn default_cache_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Caches/mir"))
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("mir"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        std::env::var_os("XDG_CACHE_HOME")
            .map(|d| PathBuf::from(d).join("mir"))
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache/mir")))
    }
}

fn resolve_php_version(config: &Config) -> PhpVersion {
    config
        .php_version
        .as_deref()
        .and_then(|raw| match raw.parse::<PhpVersion>() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("mir: {}; using default PHP {}", e, PhpVersion::LATEST);
                None
            }
        })
        .unwrap_or(PhpVersion::LATEST)
}

fn build_session(
    version: PhpVersion,
    cache_dir: Option<PathBuf>,
    stub_files: Vec<PathBuf>,
    stub_dirs: Vec<PathBuf>,
) -> AnalysisSession {
    let mut session = AnalysisSession::new(version);
    // User stubs must be configured BEFORE the cache is opened: the cache epoch
    // folds in the user-stub fingerprint, so it has to see them at open time.
    if !stub_files.is_empty() || !stub_dirs.is_empty() {
        session = session.with_user_stubs(stub_files, stub_dirs);
    }
    if let Some(dir) = cache_dir {
        session = session.with_cache_dir(&dir);
    }
    session
}

fn build_batch_opts(find_dead_code: bool) -> BatchOptions {
    // The CLI only reports diagnostics; per-expression symbols (hover /
    // go-to-definition data for LSP consumers) would be collected and never
    // read — a Laravel-scale run retains ~600k of them.
    let mut opts = BatchOptions::new().without_symbols();
    if !find_dead_code {
        opts.suppressed_issue_kinds
            .extend(dead_code_issue_kinds().iter().map(|s| (*s).to_string()));
    }
    opts
}

fn resolve_ignore_dirs(config: &Config, config_base: &std::path::Path) -> Vec<PathBuf> {
    config
        .ignore_dirs
        .iter()
        .map(|d| {
            let p = PathBuf::from(d);
            if p.is_absolute() {
                p
            } else {
                config_base.join(d)
            }
        })
        .collect()
}

fn collect_stub_paths(
    config: &Config,
    config_base: &std::path::Path,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let stub_files = config
        .stub_files
        .iter()
        .map(|f| {
            let p = PathBuf::from(f);
            if p.is_absolute() {
                p
            } else {
                config_base.join(f)
            }
        })
        .collect();
    let stub_dirs = config
        .stub_dirs
        .iter()
        .map(|d| {
            let p = PathBuf::from(d);
            if p.is_absolute() {
                p
            } else {
                config_base.join(d)
            }
        })
        .collect();
    (stub_files, stub_dirs)
}

fn filter_ignore(
    files: Vec<PathBuf>,
    ignore_dirs: &[PathBuf],
    base: &std::path::Path,
) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter(|p| {
            if ignore_dirs.is_empty() {
                return true;
            }
            let abs = if p.is_absolute() {
                p.clone()
            } else {
                base.join(p)
            };
            !ignore_dirs.iter().any(|ig| abs.starts_with(ig))
        })
        .collect()
}

fn run_with_progress(
    session: AnalysisSession,
    files: &[PathBuf],
    mut opts: BatchOptions,
    show_progress: bool,
) -> AnalysisResult {
    if show_progress {
        let pb = Arc::new(
            ProgressBar::new(files.len() as u64).with_style(
                ProgressStyle::with_template(
                    "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} files {elapsed_precise}",
                )
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("=> "),
            ),
        );
        let pb2 = pb.clone();
        opts.on_file_done = Some(Arc::new(move || {
            pb2.inc(1);
        }));
        let r = session.analyze_paths(files, &opts);
        pb.finish_and_clear();
        r
    } else {
        session.analyze_paths(files, &opts)
    }
}
