use std::fmt;

use clap::ValueEnum;
use owo_colors::{OwoColorize, Stream, Style};

/// `--color` values, matching the convention used by git/ripgrep/cargo.
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ColorChoice {
    /// Colorize per-stream: honors `NO_COLOR`/`CLICOLOR(_FORCE)`/`FORCE_COLOR`,
    /// falling back to TTY detection on stdout/stderr independently.
    Auto,
    Always,
    Never,
}

/// Apply `choice` as a process-wide owo-colors override.
///
/// `Auto` clears any override so every call below defers to owo-colors' own
/// env-var/TTY detection instead of a single global answer — this matters
/// because stdout (issue text) and stderr (progress/stats) are piped
/// independently of each other.
pub fn apply(choice: ColorChoice) {
    match choice {
        ColorChoice::Auto => owo_colors::unset_override(),
        ColorChoice::Always => owo_colors::set_override(true),
        ColorChoice::Never => owo_colors::set_override(false),
    }
}

// This module is the only place that touches `owo_colors` — callers ask for
// a labeled piece of text (banner, dim, error count, ...) and never see
// `Stream`/`Style` themselves.

const DIM: Style = Style::new().bright_black();
const BOLD: Style = Style::new().bold();
const ERROR: Style = Style::new().red();
const ERROR_COUNT: Style = Style::new().red().bold();
const WARNING: Style = Style::new().yellow();
const WARNING_COUNT: Style = Style::new().yellow().bold();
const INFO: Style = Style::new().blue();
const BANNER: Style = Style::new().bold().green();

fn paint<T: fmt::Display + ?Sized>(stream: Stream, value: &T, style: Style) -> String {
    value
        .if_supports_color(stream, |t| t.style(style))
        .to_string()
}

// -- stdout: per-issue text (format.rs) --

pub fn dim<T: fmt::Display + ?Sized>(value: &T) -> String {
    paint(Stream::Stdout, value, DIM)
}

pub fn bold<T: fmt::Display + ?Sized>(value: &T) -> String {
    paint(Stream::Stdout, value, BOLD)
}

pub fn error_label(text: &str) -> String {
    paint(Stream::Stdout, text, ERROR)
}

pub fn warning_label(text: &str) -> String {
    paint(Stream::Stdout, text, WARNING)
}

pub fn info_label(text: &str) -> String {
    paint(Stream::Stdout, text, INFO)
}

// -- stderr: progress banner and run summary (analyze.rs, report.rs) --

pub fn banner() -> String {
    paint(Stream::Stderr, "mir", BANNER)
}

pub fn error_count(n: usize) -> String {
    paint(Stream::Stderr, &n.to_string(), ERROR_COUNT)
}

pub fn error_word() -> String {
    paint(Stream::Stderr, "errors", ERROR)
}

pub fn warning_count(n: usize) -> String {
    paint(Stream::Stderr, &n.to_string(), WARNING_COUNT)
}

pub fn warning_word() -> String {
    paint(Stream::Stderr, "warnings", WARNING)
}
