use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

mod config;

use config::{Baseline, Config, ErrorLevel};
use mir_analyzer::{
    dead_code_issue_kinds, discover_files, AnalysisSession, BatchOptions, PhpVersion,
};
use mir_issues::{Issue, Severity};

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

/// mir — fast PHP static analyzer
#[derive(Parser, Debug)]
#[command(name = "mir", version, about, long_about = None)]
struct Cli {
    /// Files or directories to analyze (defaults to current directory)
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Show info-level issues (redundancies, style)
    #[arg(long)]
    show_info: bool,

    /// Suppress all output except errors (exit code only)
    #[arg(short, long)]
    quiet: bool,

    /// Extra diagnostic output (file-by-file counts)
    #[arg(short, long)]
    verbose: bool,

    /// Disable the progress bar
    #[arg(long)]
    no_progress: bool,

    /// Number of threads (defaults to logical CPU count)
    #[arg(short = 'j', long)]
    threads: Option<usize>,

    /// Print analysis statistics after the run
    #[arg(long)]
    stats: bool,

    /// PHP version to target (e.g. 8.2) — overrides config
    #[arg(long, value_name = "X.Y")]
    php_version: Option<String>,

    /// Override the cache directory (default: platform cache dir / mir)
    #[arg(long, value_name = "DIR")]
    cache_dir: Option<PathBuf>,

    /// Config file to use [default: mir.xml auto-discovered from current directory]
    #[arg(short = 'c', long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Baseline XML to suppress known issues (baseline.xml or psalm-baseline.xml)
    #[arg(long, value_name = "FILE")]
    baseline: Option<PathBuf>,

    /// Override global error level (1 = errors only, 2 = +warnings, 3+ = +info)
    #[arg(long, value_name = "1-8")]
    error_level: Option<u8>,

    /// Save all current issues to a baseline file and exit (default: psalm-baseline.xml)
    #[arg(long, value_name = "FILE", num_args = 0..=1, default_missing_value = "psalm-baseline.xml")]
    set_baseline: Option<PathBuf>,

    /// Update the baseline by removing issues that are no longer present
    #[arg(long)]
    update_baseline: bool,

    /// Ignore the baseline and report all issues
    #[arg(long)]
    ignore_baseline: bool,

    /// Skip reading from and writing to the cache for this run
    #[arg(long)]
    no_cache: bool,

    /// Delete all cached results and exit
    #[arg(long)]
    clear_cache: bool,

    /// Run dead code detection (UnusedMethod, UnusedProperty, UnusedFunction)
    #[arg(long)]
    find_dead_code: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    GithubActions,
    Junit,
    Sarif,
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();

    // --clear-cache: delete the cache file and exit before doing anything else
    if cli.clear_cache {
        let cache_dir = cli.cache_dir.clone().or_else(default_cache_dir);
        if let Some(cache_dir) = cache_dir {
            let cache_file = cache_dir.join("cache.json");
            if cache_file.exists() {
                if let Err(e) = std::fs::remove_file(&cache_file) {
                    eprintln!("mir: failed to remove cache file: {}", e);
                    std::process::exit(1);
                }
            }
            if !cli.quiet {
                eprintln!("mir: cache cleared ({})", cache_dir.display());
            }
        } else {
            eprintln!("mir: --clear-cache requires --cache-dir (no platform cache dir found)");
            std::process::exit(2);
        }
        std::process::exit(0);
    }

    // Load configuration (explicit --config, or auto-discover mir.xml / psalm.xml as fallback)
    let mut config_base: PathBuf = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut config = if let Some(path) = &cli.config {
        config_base = path
            .parent()
            .map_or_else(|| config_base.clone(), |p| p.to_path_buf());
        match Config::from_file(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("mir: config error: {e}");
                std::process::exit(2);
            }
        }
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Some(found) = Config::find(&cwd) {
            config_base = found
                .parent()
                .map_or_else(|| cwd.clone(), |p| p.to_path_buf());
            match Config::from_file(&found) {
                Ok(c) => {
                    if !cli.quiet {
                        eprintln!("mir: using config {}", found.display());
                    }
                    c
                }
                Err(e) => {
                    eprintln!("mir: config error in {}: {}", found.display(), e);
                    std::process::exit(2);
                }
            }
        } else {
            Config::default()
        }
    };

    // CLI flags override config values
    if let Some(level) = cli.error_level {
        config.error_level = level.clamp(1, 8);
    }
    if let Some(ver) = &cli.php_version {
        config.php_version = Some(ver.clone());
    }

    // Configure rayon thread pool
    if let Some(n) = cli.threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
            .ok();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // --- Composer auto-detection -------------------------------------------
    // Trigger when: no paths given and cwd has composer.json, OR a single
    // explicit path lives inside a Composer project.
    let composer_root: Option<PathBuf> = if cli.paths.is_empty() {
        if cwd.join("composer.json").exists() {
            Some(cwd.clone())
        } else {
            None
        }
    } else if cli.paths.len() == 1 {
        find_composer_root_for_path(&cli.paths[0])
    } else {
        None
    };

    if let Some(ref composer_root) = composer_root {
        let map = match mir_analyzer::composer::Psr4Map::from_composer(composer_root) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("mir: composer error: {e}");
                std::process::exit(2);
            }
        };

        // Resolve PHP version, cache dir, stubs FIRST — before any
        // file ingest so `with_cache_dir` is safe to call.
        let version = config
            .php_version
            .as_deref()
            .and_then(|raw| match raw.parse::<PhpVersion>() {
                Ok(v) => Some(v),
                Err(e) => {
                    eprintln!("mir: {}; using default PHP {}", e, PhpVersion::LATEST);
                    None
                }
            })
            .unwrap_or(PhpVersion::LATEST);

        let mut session = AnalysisSession::new(version);
        if !cli.no_cache {
            let cache_dir = cli
                .cache_dir
                .clone()
                .unwrap_or_else(|| composer_root.join(".mir/cache"));
            session = session.with_cache_dir(&cache_dir);
        }
        let (stub_files, stub_dirs) = collect_stub_paths(&config, &config_base);
        if !stub_files.is_empty() || !stub_dirs.is_empty() {
            session = session.with_user_stubs(stub_files, stub_dirs);
        }
        session = session.with_psr4(Arc::new(map.clone()));

        let mut opts = BatchOptions::new();
        if !cli.find_dead_code {
            opts.suppressed_issue_kinds
                .extend(dead_code_issue_kinds().iter().map(|s| (*s).to_string()));
        }

        // Lazy vendor: by default only eagerly load `autoload.files` entries
        // (globals — functions/constants/polyfills not FQCN-resolvable). Everything
        // else (PSR-4 + classmap) is loaded on demand via `psr4.resolve(fqcn)`
        // when project code actually references it.
        //
        // Set `MIR_EAGER_VENDOR=1` to fall back to the old behavior (parse every
        // vendor file upfront) — useful for diagnosing lazy-load gaps.
        let eager_vendor = std::env::var("MIR_EAGER_VENDOR")
            .ok()
            .is_some_and(|v| matches!(v.as_str(), "1" | "true" | "yes"));
        let vendor_files: Vec<PathBuf> = if eager_vendor {
            map.vendor_files()
        } else {
            map.vendor_eager_files()
        };

        // Resolve ignore dirs to absolute paths (relative to config file location)
        let ignore_dirs: Vec<PathBuf> = config
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
            .collect();

        let analyze_whole_composer_project = cli.paths.is_empty()
            || cli
                .paths
                .first()
                .and_then(|p| p.canonicalize().ok())
                .is_some_and(|p| p == *composer_root);

        let discovered_files: Vec<PathBuf> = if analyze_whole_composer_project {
            map.project_files()
        } else {
            discover_files(&cli.paths[0])
        };

        // Filter out ignored directories from project files.
        let cwd_abs = composer_root.clone();
        let files: Vec<PathBuf> = discovered_files
            .into_iter()
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

        session.ensure_all_stubs_loaded();

        if !vendor_files.is_empty() {
            if !cli.quiet {
                if eager_vendor {
                    eprintln!(
                        "mir: scanning {} vendor files for types...",
                        vendor_files.len()
                    );
                } else {
                    eprintln!(
                        "mir: eager-loading {} files-autoload entries ({} classmap entries available lazily)",
                        vendor_files.len(),
                        map.classmap_len()
                    );
                }
            }
            session.collect_types_only(&vendor_files);
        }

        let show_progress =
            !cli.no_progress && !cli.quiet && matches!(cli.format, OutputFormat::Text);
        let start = std::time::Instant::now();
        let result = if show_progress {
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
            let r = session.analyze_paths(&files, &opts);
            pb.finish_and_clear();
            r
        } else {
            session.analyze_paths(&files, &opts)
        };
        let elapsed = start.elapsed();
        let baseline = load_baseline(&cli, &config);
        run_output(&cli, &config, &files, result, baseline, elapsed);
        return;
    }
    // --- End composer auto-detection ----------------------------------------

    // Resolve paths
    let paths: Vec<PathBuf> = if cli.paths.is_empty() {
        vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
    } else {
        cli.paths.clone()
    };

    // Resolve ignore dirs to absolute paths (relative to config file location)
    let ignore_dirs: Vec<PathBuf> = config
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
        .collect();

    // Discover files — when config specifies project dirs, use those; otherwise use CLI paths
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
        paths.clone()
    };

    // cwd is used to absolutize relative discovered paths for ignore_dirs comparison only
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

    // Resolve target PHP version: CLI overrides config; malformed values warn
    // and fall back to the default rather than aborting analysis.
    let version = config
        .php_version
        .as_deref()
        .and_then(|raw| match raw.parse::<PhpVersion>() {
            Ok(v) => Some(v),
            Err(e) => {
                eprintln!("mir: {}; using default PHP {}", e, PhpVersion::LATEST);
                None
            }
        })
        .unwrap_or(PhpVersion::LATEST);

    // Build the session (skip cache when --no-cache is set). Configure
    // cache, version, and user stubs before any ingest.
    let mut session = AnalysisSession::new(version);
    if !cli.no_cache {
        if let Some(cache_dir) = cli.cache_dir.clone().or_else(default_cache_dir) {
            session = session.with_cache_dir(&cache_dir);
        }
    }
    let (stub_files, stub_dirs) = collect_stub_paths(&config, &config_base);
    if !stub_files.is_empty() || !stub_dirs.is_empty() {
        session = session.with_user_stubs(stub_files, stub_dirs);
    }

    let mut opts = BatchOptions::new();
    if !cli.find_dead_code {
        opts.suppressed_issue_kinds
            .extend(dead_code_issue_kinds().iter().map(|s| (*s).to_string()));
    }

    // Load type stubs first (needed before collect_types_only)
    session.ensure_all_stubs_loaded();

    // Collect types from ignore_dirs (vendor) for Pass 1 — no error reporting there.
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
            session.collect_types_only(&vendor_files);
        }
    }

    let show_progress = !cli.no_progress && !cli.quiet && matches!(cli.format, OutputFormat::Text);
    let start = std::time::Instant::now();
    let result = if show_progress {
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
        let r = session.analyze_paths(&files, &opts);
        pb.finish_and_clear();
        r
    } else {
        session.analyze_paths(&files, &opts)
    };
    let elapsed = start.elapsed();
    let baseline = load_baseline(&cli, &config);
    run_output(&cli, &config, &files, result, baseline, elapsed);
}

/// Resolve stub file/directory paths from `Config`, treating relative paths
/// as anchored at `config_base` (the directory containing `mir.xml`).
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

fn default_cache_dir() -> Option<PathBuf> {
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

fn find_composer_root_for_path(path: &std::path::Path) -> Option<PathBuf> {
    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let start = if resolved.is_dir() {
        resolved.as_path()
    } else {
        resolved.parent()?
    };

    start
        .ancestors()
        .find(|dir| dir.join("composer.json").exists())
        .map(PathBuf::from)
}

/// Load baseline from `--baseline` flag or config (auto-discover `psalm-baseline.xml`).
/// Returns `None` when `--ignore-baseline` or `--set-baseline` is active (both bypass the baseline).
/// Otherwise returns `Some((path, baseline))`.
fn load_baseline(cli: &Cli, _config: &Config) -> Option<(PathBuf, Baseline)> {
    if cli.ignore_baseline || cli.set_baseline.is_some() {
        return None;
    }

    let path = if let Some(p) = &cli.baseline {
        p.clone()
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let candidate = cwd.join("psalm-baseline.xml");
        if candidate.exists() {
            candidate
        } else {
            return None;
        }
    };

    match Baseline::from_file(&path) {
        Ok(b) => {
            if !cli.quiet {
                eprintln!("mir: using baseline {}", path.display());
            }
            Some((path, b))
        }
        Err(e) => {
            eprintln!("mir: baseline error in {}: {}", path.display(), e);
            None
        }
    }
}

fn run_output(
    cli: &Cli,
    config: &Config,
    files: &[PathBuf],
    result: mir_analyzer::AnalysisResult,
    baseline: Option<(PathBuf, Baseline)>,
    elapsed: std::time::Duration,
) {
    // --set-baseline: write every issue to the baseline file and exit 0.
    if let Some(path) = &cli.set_baseline {
        let bl = baseline_from_issues(&result.issues);
        match bl.write(path) {
            Ok(()) => {
                if !cli.quiet {
                    eprintln!("mir: baseline written to {}", path.display());
                }
            }
            Err(e) => eprintln!("mir: failed to write baseline: {e}"),
        }
        return;
    }

    let (baseline_path, mut baseline_data) = match baseline {
        Some((p, b)) => (Some(p), Some(b)),
        None => (None, None),
    };

    // Suppress issues matched by the baseline.
    // For --update-baseline, also accumulate the consumed entries into a new baseline.
    let mut new_baseline = Baseline::default();
    let suppressed_by_baseline: std::collections::HashSet<usize> =
        if let Some(bl) = &mut baseline_data {
            result
                .issues
                .iter()
                .enumerate()
                .filter_map(|(idx, issue)| {
                    let file = issue.location.file.as_ref();
                    let kind = issue.kind.name();
                    let snippet = issue.snippet.as_deref().unwrap_or("");
                    let matched = bl.consume(file, kind, snippet);
                    if matched {
                        if cli.update_baseline {
                            new_baseline
                                .entries
                                .entry(file.to_string())
                                .or_default()
                                .entry(kind.to_string())
                                .or_default()
                                .push(snippet.to_string());
                        }
                        Some(idx)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            std::collections::HashSet::new()
        };

    // --update-baseline: write back only the issues still present in the baseline.
    if cli.update_baseline {
        let path = baseline_path.as_deref().map_or_else(
            || {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("psalm-baseline.xml")
            },
            |p| p.to_path_buf(),
        );
        match new_baseline.write(&path) {
            Ok(()) => {
                if !cli.quiet {
                    eprintln!("mir: baseline updated at {}", path.display());
                }
            }
            Err(e) => eprintln!("mir: failed to update baseline: {e}"),
        }
    }

    // Apply per-issue-kind overrides from config, then filter by effective severity.
    let effective_severity = |issue: &Issue| -> Option<Severity> {
        if issue.suppressed {
            return None;
        }
        // Per-issue-kind handler overrides default severity
        let sev = if let Some(level) = config.issue_handlers.get(issue.kind.name()) {
            match level {
                ErrorLevel::Error => Severity::Error,
                ErrorLevel::Warning => Severity::Warning,
                ErrorLevel::Info => Severity::Info,
                ErrorLevel::Suppress => return None,
            }
        } else {
            issue.severity
        };
        Some(sev)
    };

    // Minimum severity to show based on error_level.
    // Error level scale: 1 (strict) to 8 (lenient).
    // Info-level issues (PossiblyNull*, PossiblyUndefined*) are only shown
    // when the configured level is ≥ 7.
    let show_info = cli.show_info || config.error_level >= 7;

    let visible_issues: Vec<(&Issue, Severity)> = result
        .issues
        .iter()
        .enumerate()
        .filter_map(|(idx, i)| {
            if suppressed_by_baseline.contains(&idx) {
                return None;
            }
            let sev = effective_severity(i)?;
            match sev {
                Severity::Error | Severity::Warning => Some((i, sev)),
                Severity::Info => {
                    if show_info {
                        Some((i, sev))
                    } else {
                        None
                    }
                }
            }
        })
        .collect();

    // For display, wrap back into &Issue slices
    let display_issues: Vec<&Issue> = visible_issues.iter().map(|(i, _)| *i).collect();

    // Output
    match cli.format {
        OutputFormat::Text => {
            if !cli.quiet {
                for issue in &display_issues {
                    println!("{issue}");
                }
            }
        }

        OutputFormat::Json => match serde_json::to_string_pretty(&display_issues) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("JSON serialization error: {e}"),
        },

        OutputFormat::GithubActions => {
            for issue in &display_issues {
                let level = match issue.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "notice",
                };
                println!(
                    "::{} file={},line={},col={}::{}",
                    level,
                    issue.location.file,
                    issue.location.line,
                    issue.location.col_start,
                    issue.kind.message()
                );
            }
        }

        OutputFormat::Junit => {
            println!("{}", format_junit(&display_issues));
        }

        OutputFormat::Sarif => {
            println!("{}", format_sarif(&display_issues));
        }
    }

    // Verbose: per-file issue counts
    if cli.verbose && !cli.quiet && matches!(cli.format, OutputFormat::Text) {
        let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for issue in &display_issues {
            *counts.entry(issue.location.file.as_ref()).or_default() += 1;
        }
        let mut entries: Vec<_> = counts.iter().collect();
        entries.sort_by_key(|(f, _)| *f);
        eprintln!();
        for (file, count) in entries {
            eprintln!(
                "  {} — {} issue{}",
                file,
                count,
                if *count == 1 { "" } else { "s" }
            );
        }
    }

    // Stats
    if cli.stats && !cli.quiet {
        let errors = display_issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        let warnings = display_issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
            .count();
        eprintln!(
            "\n{} Analyzed {} files in {:.2}s  |  {} {}  {} {}",
            "mir".bold().green(),
            files.len(),
            elapsed.as_secs_f64(),
            errors.to_string().red().bold(),
            "errors".red(),
            warnings.to_string().yellow().bold(),
            "warnings".yellow(),
        );
    }

    // Exit code: 1 if any errors remain after config filtering
    let has_errors = display_issues.iter().any(|i| i.severity == Severity::Error);
    if has_errors {
        process::exit(1);
    }
}

/// Build a Baseline from a slice of issues (used by --set-baseline).
fn baseline_from_issues(issues: &[Issue]) -> Baseline {
    let mut bl = Baseline::default();
    for issue in issues {
        bl.entries
            .entry(issue.location.file.to_string())
            .or_default()
            .entry(issue.kind.name().to_string())
            .or_default()
            .push(issue.snippet.clone().unwrap_or_default());
    }
    bl
}

// ---------------------------------------------------------------------------
// JUnit XML output
// ---------------------------------------------------------------------------

fn format_junit(issues: &[&Issue]) -> String {
    use std::collections::HashMap;

    // Group by file
    let mut by_file: HashMap<&str, Vec<&Issue>> = HashMap::new();
    for issue in issues {
        by_file
            .entry(issue.location.file.as_ref())
            .or_default()
            .push(issue);
    }

    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let total_failures: usize = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    out.push_str(&format!(
        "<testsuites name=\"mir\" tests=\"{}\" failures=\"{}\">\n",
        issues.len(),
        total_failures,
    ));

    let mut files: Vec<&str> = by_file.keys().copied().collect();
    files.sort_unstable();

    for file in files {
        let file_issues = &by_file[file];
        let failures = file_issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        out.push_str(&format!(
            "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\">\n",
            xml_escape(file),
            file_issues.len(),
            failures,
        ));
        for issue in file_issues.iter() {
            let name = issue.kind.name();
            let msg = issue.kind.message();
            let severity = match issue.severity {
                Severity::Error => "failure",
                Severity::Warning => "warning",
                Severity::Info => "notice",
            };
            out.push_str(&format!(
                "    <testcase name=\"{}\" classname=\"{}\">\n",
                xml_escape(name),
                xml_escape(file),
            ));
            out.push_str(&format!(
                "      <{} message=\"{}\" type=\"{}\">{}</{}>\n",
                severity,
                xml_escape(&msg),
                xml_escape(name),
                xml_escape(&format!(
                    "{}:{}:{} {} {}: {}",
                    file, issue.location.line, issue.location.col_start, issue.severity, name, msg
                )),
                severity,
            ));
            out.push_str("    </testcase>\n");
        }
        out.push_str("  </testsuite>\n");
    }

    out.push_str("</testsuites>\n");
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ---------------------------------------------------------------------------
// SARIF output (GitHub Code Scanning compatible)
// ---------------------------------------------------------------------------

/// FNV-1a 64-bit hash for stable partial fingerprints without extra dependencies.
fn fnv1a(data: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in data.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x00000100000001b3);
    }
    hash
}

fn format_sarif(issues: &[&Issue]) -> String {
    // Build unique rules with their default severity for rule-level metadata.
    let mut rule_map: std::collections::HashMap<String, Severity> =
        std::collections::HashMap::new();
    for issue in issues {
        rule_map
            .entry(issue.kind.name().to_string())
            .or_insert_with(|| issue.kind.default_severity());
    }
    let mut rule_ids: Vec<String> = rule_map.keys().cloned().collect();
    rule_ids.sort_unstable();

    let rules_json: Vec<serde_json::Value> = rule_ids
        .iter()
        .map(|id| {
            let level = match rule_map[id] {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            };
            let tag = if id.starts_with("Tainted") {
                "security"
            } else {
                "maintainability"
            };
            serde_json::json!({
                "id": id,
                "name": id,
                "shortDescription": { "text": id },
                "helpUri": "https://github.com/jorgsowa/mir",
                "defaultConfiguration": { "level": level },
                "properties": { "tags": [tag] },
            })
        })
        .collect();

    let results_json: Vec<serde_json::Value> = issues
        .iter()
        .map(|issue| {
            let level = match issue.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            };

            // Fingerprint based on issue kind + snippet content (not location) so
            // GitHub Code Scanning can track findings across renames/reformats.
            let fingerprint_input = format!(
                "{}:{}",
                issue.kind.name(),
                issue.snippet.as_deref().unwrap_or("")
            );
            let fingerprint = format!("{:016x}", fnv1a(&fingerprint_input));

            // rank: Error → 90, Warning → 95, Info → 99 (matches Psalm's 90–99 range).
            let rank = match issue.severity {
                Severity::Error => 90.0_f64,
                Severity::Warning => 95.0,
                Severity::Info => 99.0,
            };

            // SARIF 2.1.0 §3.30.5: columns are 1-based; col_start/col_end are 0-based.
            serde_json::json!({
                "ruleId": issue.kind.name(),
                "level": level,
                "rank": rank,
                "message": { "text": issue.kind.message() },
                "partialFingerprints": {
                    "primaryLocationLineHash": fingerprint,
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": issue.location.file.as_ref(),
                            "uriBaseId": "%SRCROOT%",
                        },
                        "region": {
                            "startLine": issue.location.line,
                            "endLine": issue.location.line_end,
                            "startColumn": issue.location.col_start + 1,
                            "endColumn": issue.location.col_end + 1,
                        }
                    }
                }]
            })
        })
        .collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "mir",
                    "informationUri": "https://github.com/jorgsowa/mir",
                    "rules": rules_json,
                }
            },
            "results": results_json,
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::find_composer_root_for_path;
    use std::fs;

    fn temp_project(name: &str) -> std::path::PathBuf {
        let thread_name = std::thread::current()
            .name()
            .unwrap_or("test")
            .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
        let root = std::env::temp_dir().join(format!(
            "mir_cli_{name}_{}_{}",
            std::process::id(),
            thread_name
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn composer_root_is_found_for_explicit_root_config_file() {
        let root = temp_project("root_config");
        fs::write(root.join("composer.json"), "{}").unwrap();
        fs::write(root.join(".php-cs-fixer.php"), "<?php\n").unwrap();

        let found = find_composer_root_for_path(&root.join(".php-cs-fixer.php"));

        assert_eq!(found, Some(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn composer_root_is_found_for_nested_file() {
        let root = temp_project("nested_file");
        let nested = root.join("src/App");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("composer.json"), "{}").unwrap();
        fs::write(nested.join("Service.php"), "<?php\n").unwrap();

        let found = find_composer_root_for_path(&nested.join("Service.php"));

        assert_eq!(found, Some(root.canonicalize().unwrap()));
        let _ = fs::remove_dir_all(root);
    }
}
