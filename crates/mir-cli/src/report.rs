use std::path::PathBuf;
use std::process;

use owo_colors::OwoColorize;

use mir_issues::{Issue, Severity};

use crate::config::{Baseline, Config, ErrorLevel};
use crate::format::{format_issue, format_junit, format_sarif};
use crate::{Cli, OutputFormat};

/// Load baseline from `--baseline` flag or auto-discover `psalm-baseline.xml`.
///
/// Returns `None` when `--ignore-baseline` or `--set-baseline` is active.
pub fn load_baseline(cli: &Cli, _config: &Config) -> Option<(PathBuf, Baseline)> {
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

pub fn run_output(
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

    // Suppress issues matched by the baseline. For --update-baseline, accumulate
    // the consumed entries into a new baseline.
    let mut new_baseline = Baseline::default();
    let suppressed_by_baseline: std::collections::HashSet<usize> =
        if let Some(bl) = &mut baseline_data {
            result
                .issues
                .iter()
                .enumerate()
                .filter_map(|(idx, issue)| {
                    let file = issue.location.file.as_ref();
                    let kind = issue.kind.display_name();
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

    let show_info = cli.show_info || config.error_level >= 7;

    let visible_issues: Vec<(&Issue, Severity)> = result
        .issues
        .iter()
        .enumerate()
        .filter_map(|(idx, i)| {
            if suppressed_by_baseline.contains(&idx) {
                return None;
            }
            let sev = effective_severity(i, config)?;
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

    let display_issues: Vec<&Issue> = visible_issues.iter().map(|(i, _)| *i).collect();

    match cli.format {
        OutputFormat::Text => {
            if !cli.quiet {
                // Single locked, block-buffered writer: one flush at drop
                // instead of a lock + line-flush per issue.
                use std::io::Write;
                let mut out = std::io::BufWriter::new(std::io::stdout().lock());
                for issue in &display_issues {
                    let _ = writeln!(out, "{}", format_issue(issue));
                }
            }
        }

        OutputFormat::Json => match serde_json::to_string_pretty(&display_issues) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("JSON serialization error: {e}"),
        },

        OutputFormat::GithubActions => {
            use std::io::Write;
            let mut out = std::io::BufWriter::new(std::io::stdout().lock());
            for issue in &display_issues {
                let level = match issue.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "notice",
                };
                let _ = writeln!(
                    out,
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

    let has_errors = display_issues.iter().any(|i| i.severity == Severity::Error);
    if has_errors {
        process::exit(1);
    }
}

fn effective_severity(issue: &Issue, config: &Config) -> Option<Severity> {
    if issue.suppressed {
        return None;
    }
    let sev = if let Some(level) = config.issue_handlers.get(issue.kind.display_name()) {
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
}

fn baseline_from_issues(issues: &[Issue]) -> Baseline {
    let mut bl = Baseline::default();
    for issue in issues {
        bl.entries
            .entry(issue.location.file.to_string())
            .or_default()
            .entry(issue.kind.display_name().to_string())
            .or_default()
            .push(issue.snippet.clone().unwrap_or_default());
    }
    bl
}
