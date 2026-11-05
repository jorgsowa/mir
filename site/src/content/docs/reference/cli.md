---
title: CLI Reference
description: All flags, output formats, and exit codes for the mir command.
---

## Synopsis

```
mir [OPTIONS] [PATHS]...
```

Paths default to the current directory when omitted.

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--format <FORMAT>` | `text` | Output format (see below) |
| `--show-info` | off | Include info-level issues (redundancies, style) |
| `-j, --threads <N>` | CPU count | Parallelism |
| `--php-version <X.Y>` | `8.5` | Target PHP version (e.g. `7.4`, `8.2`). Supported range: `7.4`–`8.6`. Overrides `<phpVersion>`. |
| `-c, --config <FILE>` | auto | Config file (`mir.xml` / `psalm.xml` auto-discovered) |
| `--error-level <1-8>` | — | Psalm-compatible strictness, 1 (strictest) to 8 (most lenient) |
| `--find-dead-code` | off | Report unused code (`UnusedClass`, `UnusedMethod`, `UnusedVariable`, …) |
| `--no-taint` | off | Skip taint analysis (no `Tainted*` issues) |
| `--baseline <FILE>` | auto | Suppress issues listed in a baseline XML. Auto-discovers `mir-baseline.xml`, then `psalm-baseline.xml`. |
| `--set-baseline [FILE]` | `mir-baseline.xml` | Write all current issues to a baseline file and exit; without `FILE`, also deletes `psalm-baseline.xml` |
| `--update-baseline` | off | Remove resolved issues from the baseline |
| `--report-stale-baseline` | off | Report baseline entries that no longer match an issue, and exit with `1` if there are any |
| `--ignore-baseline` | off | Report all issues, ignoring the baseline |
| `--cache-dir <DIR>` | see below | Cache directory |
| `--no-cache` | off | Don't read or write the cache for this run |
| `--clear-cache` | — | Delete the cache directory and exit |
| `--stats` | off | Print file count, error/warning totals, elapsed time |
| `-v, --verbose` | off | Print per-file issue counts |
| `-q, --quiet` | off | Suppress all output; use exit code only |
| `--no-progress` | off | Disable the progress bar |
| `--color <WHEN>` | `auto` | `auto`, `always` or `never`. `auto` honors `NO_COLOR`, `CLICOLOR`, `FORCE_COLOR` and whether output is a terminal. |
| `--version` | — | Print version |

## Cache

Results are cached by file content hash, so unchanged files are skipped on the
next run. The cache lives in `.mir/cache` under the composer project root, or in
the platform cache directory (`~/Library/Caches/mir`, `$XDG_CACHE_HOME/mir` or `~/.cache/mir`,
`%LOCALAPPDATA%\mir`) for projects without `composer.json`. Use `--cache-dir` to
put it elsewhere and `--no-cache` to bypass it.

## Output formats

| Format | Use case |
|--------|----------|
| `text` | Default terminal output with colors |
| `json` | Machine-readable array of issue objects |
| `github-actions` | GitHub Actions annotations (`::error file=…`) |
| `junit` | JUnit XML — compatible with most CI systems |
| `sarif` | SARIF 2.1.0 — GitHub Code Scanning / VS Code |

Taint issues also show how the data reached the sink, source first: a `= taint flow:` line in `text`, a `taint_trace` array in `json`, and `codeFlows` in `sarif`.

```
src/search.php:7:4 error[MIR0801] TaintedHtml: Tainted HTML output — possible XSS
  = taint flow: $_GET['q'] (line 4) → $q (line 4) → trim($q) (line 5) → $label (line 5)
```

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | No errors found (warnings and info don't fail the run) |
| `1` | One or more errors found, or stale baseline entries with `--report-stale-baseline` |
| `2` | Configuration, composer or plugin loading failed, or the baseline can't be written or updated |

## Examples

```bash
# Basic analysis
mir src/

# CI with GitHub Actions annotations
mir --format github-actions --no-progress src/

# JUnit XML output for CI systems
mir --format junit --no-progress src/ > results.xml

# Target a specific PHP version
mir --php-version 8.2 src/
```
