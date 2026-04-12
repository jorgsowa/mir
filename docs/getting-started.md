# Getting Started

This guide walks you through installing mir, running your first analysis, and understanding the results.

## Installation

### From crates.io (recommended)

```bash
cargo install mir-cli
```

### Build from source

```bash
git clone --recurse-submodules https://github.com/jorgsowa/mir.git
cd mir
cargo build --release
# Binary is at target/release/mir
```

> **Important:** The `--recurse-submodules` flag initializes the
> [phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs) submodule that
> provides PHP built-in definitions. Without it the build succeeds but mir will
> not recognise any PHP built-in functions, classes, or constants.
>
> If you already cloned without it:
> ```bash
> git submodule update --init
> ```

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

mir will recursively scan all `.php` files under the given paths and print any issues to stdout.

### Targeting a PHP version

If your project targets a specific PHP version, pass `--php-version`:

```bash
mir --php-version 8.2 src/
```

### Parallel analysis

By default mir uses all available CPU cores. To limit parallelism:

```bash
mir -j 4 src/
```

## Understanding the output

Each issue is printed on one line with the format:

```
path/to/file.php:LINE:COL  IssueKind  message
```

For example:

```
src/Controller/UserController.php:42:5  UndefinedMethod  Method User::getName() does not exist
src/Service/Mailer.php:17:12  InvalidArgument  Argument 1 of sendMessage expects string, int provided
```

A non-zero exit code (`1`) means at least one issue was found; exit code `0` means the analysis is clean.

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

See [Issue Kinds](issue-kinds.md) for the full list.

## Next steps

### Use a configuration file

mir auto-discovers `mir.xml` or `psalm.xml` in the project root. You can also specify a config file explicitly:

```bash
mir -c psalm.xml src/
```

### Suppress existing issues with a baseline

On large projects you may want to introduce mir without fixing every pre-existing issue at once. Generate a baseline file that records all current issues:

```bash
mir --set-baseline psalm-baseline.xml src/
```

On subsequent runs, pass the baseline to suppress those issues and only report new ones:

```bash
mir --baseline psalm-baseline.xml src/
```

When you fix issues, shrink the baseline so they are not re-introduced:

```bash
mir --update-baseline --baseline psalm-baseline.xml src/
```

### CI integration

mir supports several output formats suited to CI environments:

```bash
# GitHub Actions inline annotations
mir --format github --no-progress src/

# JUnit XML (compatible with most CI systems)
mir --format junit src/ > results.xml

# SARIF (GitHub Code Scanning / VS Code)
mir --format sarif src/ > results.sarif
```

### Incremental analysis

For large codebases, enable the incremental cache to speed up repeated runs:

```bash
mir --cache-dir .mir-cache src/
```

See the [CLI Reference](cli.md) for the full list of options and the [Docblock Annotations](docblock.md) page for how to annotate your PHP code with type information.
