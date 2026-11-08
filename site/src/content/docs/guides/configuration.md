---
title: Configuration
description: Configure mir through CLI flags and an optional XML config file.
---

mir can be configured through CLI flags and an optional XML config file (`mir.xml`).
There is no required config file — mir works out of the box with sensible defaults.

## Config file (`mir.xml`)

When run, mir searches upward from the current directory for `mir.xml`.
You can also point to a config file explicitly with `-c`. An existing
`psalm.xml` is read when there's no `mir.xml`; see
[Migrating to mir](/mir/guides/migrating/).

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
| `errorLevel` attribute | Optional strictness: `1` (strictest) to `8` (most lenient). Default: unset, every issue at its own severity. See [Error levels](#error-levels). |
| `<projectFiles>` | Source directories to analyze. With a `composer.json`, narrows its autoload directories. |
| `<ignoreFiles>` | Directories or files to exclude (e.g. `vendor/`). |
| `<fileExtensions>` | `<extension name=".inc"/>` entries treated as PHP source. `.php` must be listed explicitly once any are given. |
| `<issueHandlers>` | Per-issue-kind severity overrides (see below). |
| `<phpVersion>` | Target PHP version string, e.g. `8.2`. Supported range: `7.4`–`8.6`. |
| `<findUnusedCode>` | Enable dead-code detection (`true`/`false`). Default: `false`. |
| `<findUnusedVariables>` | Enable unused-variable checking (`true`/`false`). Default: `false`. |
| `memoizeMethodCallResults` attribute | Assume repeated zero-argument method calls on the same object return the same value, so narrowing `$a->b()` holds for later calls. Default: `false`. |
| `<stubs>` | Extra stub files (`<file name="..."/>`) or directories (`<directory name="..."/>`) to load. Stubs take precedence over vendor definitions. |
| `<forbiddenFunctions>` | `<function name="..."/>` entries reported as `ForbiddenCode` when called. |
| `<plugins>` | Plugins to load. See [Plugins](#plugins). |

### Error levels

Without `errorLevel`, every issue is reported at its default severity. Set it
to a number from `1` (strictest) to `8` (most lenient) to hide less important
issues: each issue kind has a level, and kinds below the configured level are
downgraded to info, so they're hidden unless `--show-info` is passed. The scale
and the per-kind levels follow
[Psalm's](https://psalm.dev/docs/running_psalm/error_levels/); kinds without a
level keep their severity. Issue handlers take precedence over the level.

At level `1`, unused-variable checks are on unless `<findUnusedVariables>` is
set explicitly.

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

Nested `<errorLevel type="...">` blocks set the level for files under the listed `<directory>` and `<file>` entries, relative to the config file. `<referencedClass>`, `<referencedFunction>`, `<referencedMethod>` (`Foo::bar`), `<referencedProperty>` (`Foo::$bar`), `<referencedConstant>` and `<referencedVariable>` (`$x`) entries match the symbol an issue names, case-insensitively and with an optional trailing `*`. The first matching block wins; other issues use the element's `errorLevel` attribute, if any.

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
| `-c, --config <FILE>` | auto | Config file path. Auto-discovers `mir.xml` if omitted. |
| `--php-version <X.Y>` | from config | Target PHP version (e.g. `8.2`). |
| `--error-level <1-8>` | from config | Strictness, 1 (strictest) to 8 (most lenient). |
| `--find-dead-code` | from config | Report unused code, like `<findUnusedCode>true</findUnusedCode>`. |
| `--no-taint` | off | Skip taint analysis. |
| `--baseline <FILE>` | auto | Suppress known issues from a baseline XML. Auto-discovers `mir-baseline.xml`. |
| `--set-baseline [FILE]` | `mir-baseline.xml` | Write all current issues to a baseline file and exit. |
| `--update-baseline` | off | Remove resolved issues from the baseline. |
| `--ignore-baseline` | off | Report all issues, ignoring the baseline. |
| `--format <FORMAT>` | `text` | Output format: `text`, `json`, `github-actions`, `junit`, `sarif`. |
| `--show-info` | off | Include info-level issues (redundancies, style). |

See the [CLI reference](/mir/reference/cli/) for the full list.

## Plugins

Plugins add checks and teach mir about code it can't infer on its own, such as
return types computed from arguments. They're listed in `<plugins>`:

```xml
<plugins>
  <rustPlugin path="plugins/libmy_plugin.so" />
</plugins>
```

Plugins implement `MirPlugin` from the
[`mir-plugin`](https://crates.io/crates/mir-plugin) crate and export it with
`mir_plugin::export_plugin!`. Build them as a `cdylib` with the same Rust
toolchain and `mir-plugin` version as the `mir` binary. The
[example plugin](https://github.com/jorgsowa/mir/tree/main/crates/mir-plugin-example)
reports calls to `var_dump()` and provides a return type for a helper function.

Existing Psalm plugins can be loaded too; see
[Migrating to mir](/mir/guides/migrating/#plugins).

A configured plugin that fails to load stops the run with exit code `2`.

## Baselines

A baseline file records known issues so that they are suppressed on future runs.
This lets you adopt mir incrementally — silence pre-existing issues and focus on
new ones as they are introduced.

Entries are grouped by file and issue kind and matched by the code snippet, not
the line number, so unrelated edits don't make them stale. Use
`--report-stale-baseline` in CI to fail when entries no longer match anything.

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
