---
title: Getting Started
description: Install mir and run your first analysis.
---

This guide walks you through installing mir, running your first analysis, and understanding the results.

## Installation

### From Composer (recommended for PHP projects)

```bash
composer require --dev miropen/mir-php
```

The package is a thin wrapper: a `post-install-cmd` hook downloads the
prebuilt `mir` binary that matches the installed version and host platform
from [GitHub Releases](https://github.com/jorgsowa/mir/releases), verifies
its SHA-256 checksum, and places it at `vendor/miropen/mir-php/bin/mir`.
Composer exposes it as `vendor/bin/mir`:

```bash
vendor/bin/mir src/
```

Prebuilt binaries are published for `x86_64-unknown-linux-gnu`,
`aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`,
and `x86_64-pc-windows-msvc`. On other platforms install with `cargo` or
build from source.

### From crates.io

```bash
cargo install mir-php
```

### Build from source

```bash
git clone https://github.com/jorgsowa/mir.git
cd mir
cargo build --release
# Binary is at target/release/mir
```

You can then copy the binary to a directory on your `$PATH`:

```bash
cp target/release/mir ~/.local/bin/
```

## Basic usage

Point mir at one or more directories containing PHP source files:

```bash
mir src/
```

You can pass multiple paths:

```bash
mir application/library application/module public/
```

mir recursively scans all `.php` files under the given paths and prints any issues to stdout.

When the project has a `composer.json`, mir reads its autoload sections to find
the project files and loads vendor classes on demand, so you can run plain `mir`
from the project root.

### Targeting a PHP version

mir supports PHP 7.4–8.6. If your project targets a specific version, pass `--php-version`:

```bash
mir --php-version 8.2 src/
mir --php-version 7.4 src/
```

Version-aware stub filtering is applied automatically — symbols introduced or removed in a given PHP release are included or excluded accordingly.

### Parallel analysis

By default mir uses all available CPU cores. To limit parallelism:

```bash
mir -j 4 src/
```

## Understanding the output

Each issue is printed on one line with the format:

```
path/to/file.php:LINE:COL severity[CODE] IssueKind: message
```

For example:

```
src/demo.php:16:6 error[MIR0201] InvalidArgument: Argument $s of greet() expects 'string', got '42'
src/demo.php:18:0 error[MIR0004] UndefinedMethod: Method User::getName() does not exist
src/demo.php:21:0 error[MIR0802] TaintedSql: Tainted SQL query — possible SQL injection
  = taint flow: $_GET['q'] (line 20) → $q (line 20)
```

Taint issues add a `taint flow` line showing how the data reached the sink.

Exit code `1` means at least one error was reported; warnings and info-level
issues (shown with `--show-info`) don't fail the run.

### Issue kinds

The most common issue kinds you will encounter:

| Kind | What it means |
|------|---------------|
| `UndefinedVariable` | A variable is used before it is assigned |
| `UndefinedFunction` | A function call that has no definition |
| `UndefinedMethod` | A method call on a type that does not have that method |
| `UndefinedClass` | A class / interface / trait that does not exist |
| `InvalidArgument` | A value passed to a function does not match the declared type |
| `InvalidReturnType` | A function returns a type that does not match its declaration |
| `PossiblyInvalidArrayAccess` | Array access on a value that may be `false` or `null` |
| `NullableReturnStatement` | A nullable value returned from a non-nullable return type |

See [Issues](/mir/reference/issues/) for the full list.

## Next steps

### Use a configuration file

mir works without a config file. To set the PHP version, the source directories
or per-issue severities, add a `mir.xml` to the project root; mir finds it in the
current directory or any parent. You can also pass one explicitly:

```bash
mir -c config/mir.xml src/
```

See the [Configuration guide](/mir/guides/configuration/). Coming from another
analyzer? [Migrating to mir](/mir/guides/migrating/) covers existing configs,
baselines and annotations.

### Find dead code

Unused code checks are off by default. Enable them with `--find-dead-code` or
`<findUnusedCode>true</findUnusedCode>`:

```bash
mir --find-dead-code src/
```

Mark classes and members that are used from outside the analyzed code with
`@api` to keep them out of the report.

### Suppress existing issues with a baseline

On large projects you may want to introduce mir without fixing every pre-existing issue at once. Generate a baseline file that records all current issues:

```bash
mir src/ --set-baseline
```

This writes `mir-baseline.xml`. On subsequent runs, mir picks the baseline up from the current directory and only reports new issues:

```bash
mir src/
```

When you fix issues, shrink the baseline so they are not re-introduced:

```bash
mir --update-baseline src/
```

### CI integration

#### GitHub Actions

The [mir GitHub Action](https://github.com/jorgsowa/mir-action) installs and
runs mir in a workflow. Create `.github/workflows/mir.yml`:

```yaml
name: MIR

on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7.0.1
      - uses: jorgsowa/mir-action@v1
        with:
          path: src
```

Set `version` to an exact mir package version and pin the action to a commit
SHA when you need reproducible builds. The action also accepts `php-version`
and `arguments` inputs; see its [README](https://github.com/jorgsowa/mir-action)
for the complete reference.

#### Other CI systems

mir supports several output formats suited to CI environments:

```bash
# GitHub Actions inline annotations
mir --format github-actions --no-progress src/

# JUnit XML (compatible with most CI systems)
mir --format junit src/ > results.xml

# SARIF (GitHub Code Scanning / VS Code)
mir --format sarif src/ > results.sarif
```

### Incremental analysis

Results are cached in `.mir/cache` under the composer root, so repeated runs
skip files that haven't changed. Add `.mir/` to `.gitignore`.
Use `--no-cache` for a run without it and `--clear-cache` to delete it.

See the [CLI Reference](/mir/reference/cli/) for the full list of options and the [Docblock Annotations](/mir/reference/docblock/) page for how to annotate your PHP code with type information.
