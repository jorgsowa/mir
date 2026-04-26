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
| `--cache-dir <DIR>` | off | Enable incremental cache in `DIR` |
| `--stats` | off | Print file count, error/warning totals, elapsed time |
| `-v, --verbose` | off | Print per-file issue counts |
| `-q, --quiet` | off | Suppress all output; use exit code only |
| `--no-progress` | off | Disable the progress bar |
| `--php-version <X.Y>` | — | Target PHP version (e.g. `8.2`) |
| `-c, --config <FILE>` | auto | Config file (`mir.xml` / `psalm.xml` auto-discovered) |
| `--baseline <FILE>` | off | Suppress issues listed in a baseline XML |
| `--error-level <1-8>` | — | Override global error level (1 = errors only) |
| `--set-baseline [FILE]` | — | Write all current issues to a baseline file and exit |
| `--update-baseline` | off | Remove resolved issues from the baseline |
| `--ignore-baseline` | off | Report all issues, ignoring the baseline |
| `--version` | — | Print version |

## Output formats

| Format | Use case |
|--------|----------|
| `text` | Default terminal output with colors |
| `json` | Machine-readable array of issue objects |
| `github` | GitHub Actions annotations (`::error file=…`) |
| `junit` | JUnit XML — compatible with most CI systems |
| `sarif` | SARIF 2.1.0 — GitHub Code Scanning / VS Code |

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | No errors found |
| `1` | One or more errors found |

## Examples

```bash
# Basic analysis
mir src/

# CI with GitHub Actions annotations
mir --format github --no-progress src/

# JUnit XML output for CI systems
mir --format junit --no-progress src/ > results.xml

# Target a specific PHP version
mir --php-version 8.2 src/
```
