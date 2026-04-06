# PSR-4 Autoload Support in ProjectAnalyzer

**Issue:** #39 (use case 1 only — use case 2 deferred to #50)
**Date:** 2026-04-06

## Goal

Add a `Psr4Map` struct and `ProjectAnalyzer::from_composer()` constructor so that:
- Callers (CLI, LSP) can discover project and vendor PHP files from `composer.json` without implementing their own file discovery.
- The CLI auto-detects `composer.json` in cwd when no explicit paths are given.
- The `Psr4Map` is stored on `ProjectAnalyzer` for future lazy FQCN resolution (issue #50).

## Out of scope

- On-demand lazy FQCN → file resolution during Pass 2 (issue #50).
- Changes to `DefinitionCollector`, `ExpressionAnalyzer`, or any analysis logic.
- New CLI flags.

---

## Components

### 1. `Psr4Map` (`crates/mir-analyzer/src/composer.rs`)

Parses two JSON files:
- `{root}/composer.json` — `autoload.psr-4` and `autoload-dev.psr-4` → **project entries**
- `{root}/vendor/composer/installed.json` — `packages[].autoload.psr-4` (Composer v2) or top-level array (Composer v1) → **vendor entries**

Missing `installed.json` is silently ignored.

```rust
pub struct Psr4Map {
    project_entries: Vec<(String, PathBuf)>,  // (namespace_prefix, dir), longest-first
    vendor_entries:  Vec<(String, PathBuf)>,
    root: PathBuf,
}
```

**Public API:**

```rust
impl Psr4Map {
    pub fn from_composer(root: &Path) -> Result<Self, ComposerError>;
    pub fn project_files(&self) -> Vec<PathBuf>;
    pub fn vendor_files(&self) -> Vec<PathBuf>;
    pub fn resolve(&self, fqcn: &str) -> Option<PathBuf>;
}
```

**Prefix matching rules:**
- Entries are sorted longest-prefix-first so `App\Models\` matches before `App\`.
- Before matching, append `\` to the prefix: `App\` must not match `Application\Foo`.
- `resolve("App\\Models\\User")` → strip prefix, convert `\` to `/`, append `.php`, check disk.

**Error type:**

```rust
pub enum ComposerError {
    Io(std::io::Error),
    Json(serde_json::Error),
    MissingAutoload,
}
```

---

### 2. `ProjectAnalyzer` changes (`crates/mir-analyzer/src/project.rs`)

Add `psr4` field:

```rust
pub struct ProjectAnalyzer {
    pub codebase: Arc<Codebase>,
    pub cache: Option<AnalysisCache>,
    pub on_file_done: Option<Arc<dyn Fn() + Send + Sync>>,
    pub psr4: Option<Arc<Psr4Map>>,   // new — used by issue #50
    stubs_loaded: AtomicBool,
}
```

Add constructor:

```rust
pub fn from_composer(root: &Path) -> Result<(Self, Psr4Map), ComposerError>
```

Returns both the analyzer (with `psr4` set) and the map so callers can call `map.project_files()` / `map.vendor_files()` directly.

Existing constructors (`new()`, `with_cache()`) are unchanged. They leave `psr4: None`.

---

### 3. CLI auto-detection (`crates/mir-cli/src/main.rs`)

When `cli.paths` is empty and `{cwd}/composer.json` exists, use `from_composer()` instead of the existing default-to-cwd path:

```
if no paths given AND composer.json exists in cwd:
    (analyzer, map) = ProjectAnalyzer::from_composer(cwd)
    vendor_files    = map.vendor_files()
    project_files   = map.project_files()
    collect_types_only(vendor_files)
    analyze(project_files)
else:
    existing behavior unchanged
```

No new flags. Explicit paths continue to work as before.

---

## Data flow

```
composer.json
vendor/composer/installed.json
        │
        ▼
    Psr4Map::from_composer()
        │
        ├─ project_entries → project_files() ──► analyze()
        └─ vendor_entries  → vendor_files()  ──► collect_types_only()
```

---

## Testing

All tests in `crates/mir-analyzer/tests/composer.rs` (or inline unit tests in `composer.rs`):

| Test | What it checks |
|------|---------------|
| `parse_project_entries` | `autoload.psr-4` and `autoload-dev.psr-4` produce correct prefix/dir entries |
| `longest_prefix_first` | Entries are sorted so longer prefixes match first |
| `boundary_check` | `App\` prefix does not match `Application\Foo` |
| `composer_v1_installed` | Bare array in `installed.json` parses correctly |
| `composer_v2_installed` | `{"packages": [...]}` in `installed.json` parses correctly |
| `resolve_existing_file` | `resolve("App\\Models\\User")` returns the correct path when file exists |
| `resolve_missing_file` | Returns `None` when the mapped path does not exist on disk |
| `missing_installed_json` | No error when `vendor/composer/installed.json` is absent |
