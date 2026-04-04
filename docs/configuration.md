# Configuration

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
    <PossiblyNullReference errorLevel="warning" />
  </issueHandlers>

  <phpVersion>8.2</phpVersion>
  <findUnusedCode>true</findUnusedCode>
  <findUnusedVariables>true</findUnusedVariables>
</mir>
```

### Config file fields

| Field | Description |
|-------|-------------|
| `errorLevel` attribute | Global strictness: `1` (errors only) to `8` (lenient). Default: `2`. |
| `<projectFiles>` | Source directories to analyze. |
| `<ignoreFiles>` | Directories or files to exclude (e.g. `vendor/`). |
| `<issueHandlers>` | Per-issue-kind severity overrides (see below). |
| `<phpVersion>` | Target PHP version string, e.g. `8.2`. |
| `<findUnusedCode>` | Enable dead-code detection (`true`/`false`). Default: `false`. |
| `<findUnusedVariables>` | Enable unused-variable checking (`true`/`false`). Default: `false`. |

### Issue handlers

Each child element of `<issueHandlers>` is an issue kind name with an `errorLevel` attribute:

| Level | Effect |
|-------|--------|
| `error` | Treat as error (exit code 1) |
| `warning` | Treat as warning |
| `info` | Treat as info (only shown with `--show-info` or `errorLevel >= 7`) |
| `suppress` | Silence the issue entirely |

```xml
<issueHandlers>
  <UndefinedMethod errorLevel="suppress" />
  <InvalidArgument errorLevel="warning" />
</issueHandlers>
```

## CLI flags

CLI flags always override values from the config file.

| Flag | Default | Description |
|------|---------|-------------|
| `-c, --config <FILE>` | auto | Config file path. Auto-discovers `mir.xml` / `psalm.xml` if omitted. |
| `--baseline <FILE>` | auto | Suppress known issues from a baseline XML. Auto-discovers `psalm-baseline.xml`. |
| `--set-baseline [FILE]` | `psalm-baseline.xml` | Write all current issues to a baseline file and exit. |
| `--update-baseline` | off | Remove resolved issues from the baseline. |
| `--ignore-baseline` | off | Report all issues, ignoring the baseline. |
| `--error-level <1-8>` | from config | Override global error level. |
| `--php-version <X.Y>` | from config | Target PHP version (e.g. `8.2`). |
| `--format <FORMAT>` | `text` | Output format: `text`, `json`, `github`, `junit`, `sarif`. |
| `--show-info` | off | Include info-level issues (redundancies, style). |
| `-j, --threads <N>` | CPU count | Parallelism. |
| `--cache-dir <DIR>` | off | Enable incremental cache in `DIR`. |
| `--stats` | off | Print file count, error/warning totals, elapsed time. |
| `-v, --verbose` | off | Print per-file issue counts. |
| `-q, --quiet` | off | Suppress all output; use exit code only. |
| `--no-progress` | off | Disable the progress bar. |

## Baselines

A baseline file records known issues so that they are suppressed on future runs.
This lets you adopt mir incrementally — silence pre-existing issues and focus on
new ones as they are introduced.

Baseline files use Psalm's XML format, so an existing `psalm-baseline.xml` works
directly.

### Baseline workflow

```bash
# 1. Generate a baseline from the current issues (first adoption)
mir --set-baseline psalm-baseline.xml src/

# 2. Subsequent runs suppress baselined issues automatically
#    (psalm-baseline.xml in the cwd is picked up automatically)
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
mir --format github --no-progress --baseline psalm-baseline.xml src/
```

### Strict analysis (errors only, no warnings)

```bash
mir --error-level 1 src/
```

### Target a specific PHP version

```bash
mir --php-version 8.1 src/
```

### Suppress a noisy issue kind project-wide

```xml
<issueHandlers>
  <UndefinedMethod errorLevel="suppress" />
</issueHandlers>
```

### Incremental analysis with caching

```bash
mir --cache-dir .mir-cache src/
```
