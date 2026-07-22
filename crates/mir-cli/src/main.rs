use std::path::PathBuf;
use std::process;

// macOS's default `libmalloc` serializes heavily under the multi-threaded,
// allocation-dense `body_analysis` phase. mimalloc (per-thread arenas) is what
// the benchmarks already run under; matching it in the shipping binary removes
// that allocator-lock contention from real `mir` runs.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::{Parser, ValueEnum};

mod analyze;
mod color;
mod composer;
mod config;
mod format;
mod plugins;
mod report;

use color::ColorChoice;
use config::Config;

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

    /// Colorize output [default: auto-detect NO_COLOR/CLICOLOR/tty]
    #[arg(long, value_enum, default_value = "auto")]
    color: ColorChoice,
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
    color::apply(cli.color);

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if cli.clear_cache {
        // Resolve the composer root first so we clear the *project-local*
        // `.mir/cache` the analyze path actually uses — not just the platform
        // default. Done before config loading: clearing the cache needs neither.
        clear_cache(&cli, resolve_composer_root(&cli, &cwd).as_deref());
    }

    let (mut config, config_base) = load_config(&cli);

    if let Some(level) = cli.error_level {
        config.error_level = level.clamp(1, 8);
    }
    if let Some(ver) = &cli.php_version {
        config.php_version = Some(ver.clone());
    }

    if let Some(n) = cli.threads {
        if let Err(e) = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global()
        {
            eprintln!("mir: failed to set thread pool size: {e}");
        }
    }

    let composer_root = resolve_composer_root(&cli, &cwd);

    plugins::setup_plugins(&cli, &mut config, &config_base, composer_root.as_deref());

    let baseline = report::load_baseline(&cli, &config);

    let (files, result, elapsed) = if let Some(ref root) = composer_root {
        analyze::run_composer_flow(&cli, &config, &config_base, root)
    } else {
        analyze::run_plain_flow(&cli, &config, &config_base)
    };

    report::run_output(&cli, &config, &files, result, baseline, elapsed);
}

// ---------------------------------------------------------------------------
// Bootstrap helpers
// ---------------------------------------------------------------------------

fn clear_cache(cli: &Cli, composer_root: Option<&std::path::Path>) -> ! {
    // Mirror the cache-dir resolution in `analyze::run_composer_flow`: an
    // explicit `--cache-dir`, else the project-local `{composer_root}/.mir/cache`,
    // else the platform default. The previous version only ever looked at the
    // platform default and removed a `cache.json` that no longer exists (the
    // format is `cache.bin`), so `--clear-cache` silently did nothing for a
    // normal project run.
    let cache_dir = cli
        .cache_dir
        .clone()
        .or_else(|| composer_root.map(|r| r.join(".mir/cache")))
        .or_else(analyze::default_cache_dir);
    if let Some(cache_dir) = cache_dir {
        if cache_dir.exists() {
            // Remove the whole cache directory: it holds the result cache
            // (`cache.bin`), any legacy `cache.json`, and the stub-definition
            // cache under `stubs/`. A partial delete leaves a half-stale cache.
            if let Err(e) = std::fs::remove_dir_all(&cache_dir) {
                eprintln!("mir: failed to clear cache: {}", e);
                process::exit(1);
            }
        }
        if !cli.quiet {
            eprintln!("mir: cache cleared ({})", cache_dir.display());
        }
    } else {
        eprintln!(
            "mir: --clear-cache requires --cache-dir (no project or platform cache dir found)"
        );
        process::exit(2);
    }
    process::exit(0);
}

fn load_config(cli: &Cli) -> (Config, PathBuf) {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if let Some(path) = &cli.config {
        let config_base = path
            .parent()
            .map_or_else(|| cwd.clone(), |p| p.to_path_buf());
        let config = match Config::from_file(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("mir: config error: {e}");
                process::exit(2);
            }
        };
        return (config, config_base);
    }

    if let Some(found) = Config::find(&cwd) {
        let config_base = found
            .parent()
            .map_or_else(|| cwd.clone(), |p| p.to_path_buf());
        let config = match Config::from_file(&found) {
            Ok(c) => {
                if !cli.quiet {
                    eprintln!("mir: using config {}", found.display());
                }
                c
            }
            Err(e) => {
                eprintln!("mir: config error in {}: {}", found.display(), e);
                process::exit(2);
            }
        };
        return (config, config_base);
    }

    (Config::default(), cwd)
}

fn resolve_composer_root(cli: &Cli, cwd: &std::path::Path) -> Option<PathBuf> {
    if cli.paths.is_empty() {
        if cwd.join("composer.json").exists() {
            Some(cwd.to_path_buf())
        } else {
            None
        }
    } else if cli.paths.len() == 1 {
        composer::find_composer_root_for_path(&cli.paths[0])
    } else {
        None
    }
}
