---
title: Configuration
description: Configure mir through CLI flags and an optional XML config file.
---

mir can be configured through CLI flags and an optional XML config file (`mir.xml`).
There is no required config file — mir works out of the box with sensible defaults.

## Config file (`mir.xml`)

When run, mir searches upward from the current directory for `mir.xml`.
If no `mir.xml` is found, it falls back to `psalm.xml` (Psalm compatibility).
You can also point to a config file explicitly with `-c`.

### Example `mir.xml`

```xml
<?xml version="1.0"?>
<mir xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" errorLevel="3">
  <projectFiles>
    <directory name="src" />
    <directory name="lib" />
  </projectFiles>

  <ignoreFiles>
    <directory name="vendor" />
  </ignoreFiles>

  <issueHandlers>
    <UndefinedVariable errorLevel="suppress" />
    <PossiblyNullMethodCall errorLevel="warning" />
  </issueHandlers>

  <phpVersion>8.2</phpVersion>
  <findUnusedCode>true</findUnusedCode>
  <findUnusedVariables>true</findUnusedVariables>
</mir>
```

### Config file fields

| Field | Description |
|-------|-------------|
| `errorLevel` attribute | Optional Psalm-compatible strictness: `1` (strictest) to `8` (most lenient). Default: unset, mir's own severities. See [Psalm error levels](#psalm-error-levels). |
| `<projectFiles>` | Source directories to analyze. |
| `<ignoreFiles>` | Directories or files to exclude (e.g. `vendor/`). |
| `<fileExtensions>` | `<extension name=".inc"/>` entries treated as PHP source. `.php` must be listed explicitly once any are given. |
| `<issueHandlers>` | Per-issue-kind severity overrides (see below). |
| `<phpVersion>` | Target PHP version string, e.g. `8.2`. Supported range: `7.4`–`8.6`. |
| `<findUnusedCode>` | Enable dead-code detection (`true`/`false`). Default: `false`. |
| `<findUnusedVariables>` | Enable unused-variable checking (`true`/`false`). Default: `false`. |
| `memoizeMethodCallResults` attribute | Assume repeated zero-argument method calls on the same object return the same value, so narrowing `$a->b()` holds for later calls, as in Psalm. Default: `false`. |
| `<stubs>` | Extra stub files (`<file name="..."/>`) or directories (`<directory name="..."/>`) to load. Stubs take precedence over vendor definitions. |
| `<forbiddenFunctions>` | `<function name="..."/>` entries reported as `ForbiddenCode` when called. |
| `<plugins>` | Rust and Psalm plugins to load. See [Plugins](#plugins). |

### Psalm error levels

Set `errorLevel` to apply [Psalm's per-issue levels](https://psalm.dev/docs/running_psalm/error_levels/). An issue whose Psalm level is below the configured level is downgraded to info, so it is hidden unless `--show-info` is passed. Issues Psalm always reports, and mir-only issues, keep their severity. Issue handlers take precedence over the level.

### Issue handlers

Each child element of `<issueHandlers>` is an issue kind name with an `errorLevel` attribute:

| Level | Effect |
|-------|--------|
| `error` | Treat as error (exit code 1) |
| `warning` | Treat as warning |
| `info` | Treat as info (only shown with `--show-info`) |
| `suppress` | Silence the issue entirely |

```xml
<issueHandlers>
  <UndefinedMethod errorLevel="suppress" />
  <InvalidArgument errorLevel="warning" />
</issueHandlers>
```

As in Psalm, nested `<errorLevel type="...">` blocks set the level for files under the listed `<directory>` and `<file>` entries, relative to the config file. The first matching block wins; other files use the element's `errorLevel` attribute, if any. `<referencedClass>`, `<referencedMethod>` and similar filters are not supported and never match.

```xml
<issueHandlers>
  <PossiblyUndefinedVariable errorLevel="error">
    <errorLevel type="suppress">
      <directory name="tests" />
      <file name="src/Legacy.php" />
    </errorLevel>
  </PossiblyUndefinedVariable>
</issueHandlers>
```

## CLI flags

CLI flags always override values from the config file.

| Flag | Default | Description |
|------|---------|-------------|
| `-c, --config <FILE>` | auto | Config file path. Auto-discovers `mir.xml` / `psalm.xml` if omitted. |
| `--php-version <X.Y>` | from config | Target PHP version (e.g. `8.2`). |
| `--error-level <1-8>` | from config | Psalm-compatible strictness, 1 (strictest) to 8 (most lenient). |
| `--find-dead-code` | from config | Report unused code, like `<findUnusedCode>true</findUnusedCode>`. |
| `--no-taint` | off | Skip taint analysis. |
| `--baseline <FILE>` | auto | Suppress known issues from a baseline XML. Auto-discovers `mir-baseline.xml`, then `psalm-baseline.xml`. |
| `--set-baseline [FILE]` | `mir-baseline.xml` | Write all current issues to a baseline file and exit. Without `FILE`, also deletes `psalm-baseline.xml`. |
| `--update-baseline` | off | Remove resolved issues from the baseline. |
| `--ignore-baseline` | off | Report all issues, ignoring the baseline. |
| `--format <FORMAT>` | `text` | Output format: `text`, `json`, `github-actions`, `junit`, `sarif`. |
| `--show-info` | off | Include info-level issues (redundancies, style). |

See the [CLI reference](/mir/reference/cli/) for the full list.

## Plugins

mir loads two kinds of plugins from `<plugins>`:

```xml
<plugins>
  <!-- Rust plugin built as a shared library -->
  <rustPlugin path="plugins/libmy_plugin.so" />
  <!-- Psalm plugin installed with composer -->
  <pluginClass class="Psalm\PhpUnitPlugin\Plugin" />
  <!-- Psalm plugin loaded from a file -->
  <plugin filename="tools/MyPsalmPlugin.php" />
</plugins>
```

**Rust plugins** implement `MirPlugin` from the
[`mir-plugin`](https://crates.io/crates/mir-plugin) crate and export it with
`mir_plugin::export_plugin!`. Build them as a `cdylib` with the same Rust
toolchain and `mir-plugin` version as the `mir` binary. The
[example plugin](https://github.com/jorgsowa/mir/tree/main/crates/mir-plugin-example)
reports calls to `var_dump()` and provides a return type for a helper function.

**Psalm plugins** run in a `php` subprocess that boots the project's
`vendor/autoload.php`. Stub files, function and method return type providers,
and the class-like and function-like analysis hooks are supported; other hooks
are skipped with a warning. Set `MIR_PHP` to choose the PHP binary and
`MIR_PSALM_AUTOLOAD` to point at a Psalm install outside the project's vendor
directory.

A configured plugin that fails to load stops the run with exit code `2`.

## Baselines

A baseline file records known issues so that they are suppressed on future runs.
This lets you adopt mir incrementally — silence pre-existing issues and focus on
new ones as they are introduced.

Baseline files use Psalm's XML format, so an existing `psalm-baseline.xml` works
directly. Running `mir --set-baseline` without a file name migrates it: mir writes
`mir-baseline.xml` and deletes `psalm-baseline.xml`.

### Baseline workflow

```bash
# 1. Generate a baseline from the current issues (first adoption)
mir src/ --set-baseline

# 2. Subsequent runs suppress baselined issues automatically
#    (mir-baseline.xml in the cwd is picked up automatically)
mir src/

# 3. Explicitly point to a baseline in a non-standard location
mir --baseline path/to/my-baseline.xml src/

# 4. After fixing issues, shrink the baseline (removes resolved entries)
mir --update-baseline src/

# 5. Temporarily ignore the baseline to see all issues
mir --ignore-baseline src/
```

## Common configurations

### Basic project with vendor exclusion

```xml
<?xml version="1.0"?>
<mir errorLevel="3">
  <projectFiles>
    <directory name="src" />
  </projectFiles>
  <ignoreFiles>
    <directory name="vendor" />
  </ignoreFiles>
</mir>
```

### CI pipeline (GitHub Actions)

```bash
# In .github/workflows/mir.yml:
mir --format github-actions --no-progress src/
```

### Strictest analysis

```bash
mir --error-level 1 src/
```

### Target a specific PHP version

Supported range is `7.4`–`8.6`. Stub symbols are filtered by `@since`/`@removed` tags automatically.

```bash
mir --php-version 7.4 src/
mir --php-version 8.1 src/
```

### Suppress a noisy issue kind project-wide

```xml
<issueHandlers>
  <UndefinedMethod errorLevel="suppress" />
</issueHandlers>
```

### Caching

The cache is on by default (`.mir/cache` under the composer root). Add `.mir/`
to `.gitignore`, or move the cache with `--cache-dir`:

```bash
mir --cache-dir /tmp/mir-cache src/
mir --no-cache src/      # one run without the cache
```
