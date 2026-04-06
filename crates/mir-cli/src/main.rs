use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use clap::{Parser, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

mod config;

use config::{Baseline, Config, ErrorLevel};
use mir_analyzer::ProjectAnalyzer;
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

    /// Enable disk-backed result cache; specify the cache directory
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
        if let Some(cache_dir) = &cli.cache_dir {
            let cache_file = cache_dir.join("cache.json");
            if cache_file.exists() {
                std::fs::remove_file(&cache_file).expect("Failed to remove cache file");
            }
            if !cli.quiet {
                eprintln!("mir: cache cleared ({})", cache_dir.display());
            }
        } else {
            eprintln!("mir: --clear-cache requires --cache-dir");
            std::process::exit(2);
        }
        std::process::exit(0);
    }

    // Load configuration (explicit --config, or auto-discover mir.xml / psalm.xml as fallback)
    let mut config_base: PathBuf = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut config = if let Some(path) = &cli.config {
        config_base = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| config_base.clone());
        match Config::from_file(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("mir: config error: {}", e);
                std::process::exit(2);
            }
        }
    } else {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        if let Some(found) = Config::find(&cwd) {
            config_base = found
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| cwd.clone());
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
    if cli.paths.is_empty() && cwd.join("composer.json").exists() {
        let (mut analyzer, map) = match ProjectAnalyzer::from_composer(&cwd) {
            Ok(pair) => pair,
            Err(e) => {
                eprintln!("mir: composer error: {}", e);
                std::process::exit(2);
            }
        };

        // Apply --cache-dir if specified (skip when --no-cache is set)
        if let Some(cache_dir) = &cli.cache_dir {
            if !cli.no_cache {
                analyzer.cache = Some(mir_analyzer::cache::AnalysisCache::open(cache_dir));
            }
        }

        let vendor_files = map.vendor_files();

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

        // Filter out ignored directories from project files
        let cwd_abs = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let files: Vec<PathBuf> = map
            .project_files()
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

        analyzer.load_stubs();

        if !vendor_files.is_empty() {
            if !cli.quiet {
                eprintln!(
                    "mir: scanning {} vendor files for types...",
                    vendor_files.len()
                );
            }
            analyzer.collect_types_only(&vendor_files);
        }

        let show_progress =
            !cli.no_progress && !cli.quiet && matches!(cli.format, OutputFormat::Text);
        let start = std::time::Instant::now();
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
            analyzer.on_file_done = Some(Arc::new(move || {
                pb2.inc(1);
            }));
            let result = analyzer.analyze(&files);
            let elapsed = start.elapsed();
            pb.finish_and_clear();
            let baseline = load_baseline(&cli, &config);
            run_output(&cli, &config, &files, result, baseline, elapsed);
        } else {
            let result = analyzer.analyze(&files);
            let elapsed = start.elapsed();
            let baseline = load_baseline(&cli, &config);
            run_output(&cli, &config, &files, result, baseline, elapsed);
        }
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
        .flat_map(|p| ProjectAnalyzer::discover_files(p))
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
                .map(|v| format!(" (PHP {})", v))
                .unwrap_or_default(),
        );
    }

    // Build analyzer (skip cache when --no-cache is set)
    let mut analyzer = if let Some(cache_dir) = &cli.cache_dir {
        if !cli.no_cache {
            ProjectAnalyzer::with_cache(cache_dir)
        } else {
            ProjectAnalyzer::new()
        }
    } else {
        ProjectAnalyzer::new()
    };

    // Load type stubs first (needed before collect_types_only)
    analyzer.load_stubs();

    // Collect types from ignore_dirs (vendor) for Pass 1 — no error reporting there
    if !ignore_dirs.is_empty() {
        let vendor_files: Vec<PathBuf> = ignore_dirs
            .iter()
            .flat_map(|p| ProjectAnalyzer::discover_files(p))
            .collect();
        if !vendor_files.is_empty() {
            if !cli.quiet {
                eprintln!(
                    "mir: scanning {} vendor files for types...",
                    vendor_files.len()
                );
            }
            analyzer.collect_types_only(&vendor_files);
        }
    }

    // Progress bar (Pass 2)
    let show_progress = !cli.no_progress && !cli.quiet && matches!(cli.format, OutputFormat::Text);
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
        analyzer.on_file_done = Some(Arc::new(move || {
            pb2.inc(1);
        }));
        // Store the pb so we can finish it after analysis.
        // We use a thread-local trick: drop happens after `result` is obtained.
        let start = std::time::Instant::now();
        let result = analyzer.analyze(&files);
        let elapsed = start.elapsed();
        pb.finish_and_clear();
        let baseline = load_baseline(&cli, &config);
        run_output(&cli, &config, &files, result, baseline, elapsed);
    } else {
        let start = std::time::Instant::now();
        let result = analyzer.analyze(&files);
        let elapsed = start.elapsed();
        let baseline = load_baseline(&cli, &config);
        run_output(&cli, &config, &files, result, baseline, elapsed);
    }
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
    result: mir_analyzer::project::AnalysisResult,
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
            Err(e) => eprintln!("mir: failed to write baseline: {}", e),
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
        let path = baseline_path
            .as_deref()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| {
                std::env::current_dir()
                    .unwrap_or_else(|_| PathBuf::from("."))
                    .join("psalm-baseline.xml")
            });
        match new_baseline.write(&path) {
            Ok(()) => {
                if !cli.quiet {
                    eprintln!("mir: baseline updated at {}", path.display());
                }
            }
            Err(e) => eprintln!("mir: failed to update baseline: {}", e),
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
                    println!("{}", issue);
                }
            }
        }

        OutputFormat::Json => match serde_json::to_string_pretty(&display_issues) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("JSON serialization error: {}", e),
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

fn format_sarif(issues: &[&Issue]) -> String {
    // Build the set of unique rules (issue kinds)
    let mut rule_ids: Vec<String> = issues
        .iter()
        .map(|i| i.kind.name().to_string())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    rule_ids.sort_unstable();

    let rules_json: Vec<serde_json::Value> = rule_ids
        .iter()
        .map(|id| {
            serde_json::json!({
                "id": id,
                "name": id,
                "shortDescription": { "text": id },
                "helpUri": "https://github.com/adamspychala/mir",
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
            serde_json::json!({
                "ruleId": issue.kind.name(),
                "level": level,
                "message": { "text": issue.kind.message() },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": issue.location.file.as_ref(),
                            "uriBaseId": "%SRCROOT%",
                        },
                        "region": {
                            "startLine": issue.location.line,
                            "startColumn": issue.location.col_start,
                            "endColumn": issue.location.col_end,
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
                    "informationUri": "https://github.com/adamspychala/mir",
                    "rules": rules_json,
                }
            },
            "results": results_json,
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
}
