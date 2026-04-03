# mir

A fast, incremental PHP static analyzer written in Rust, inspired by [Psalm](https://psalm.dev).

## Features

- **Sound type system** — scalars, objects, generics (`@template`), unions, intersections, literal types, `never`, `void`
- **Full type inference** — return types inferred from function bodies; types narrowed through `if`/`match`/`instanceof`/`is_string()` etc.
- **Call checking** — argument count, argument types, return types for user-defined and built-in functions/methods
- **Class analysis** — inheritance, interface compliance, abstract method enforcement, visibility, `readonly`, `final`
- **Generics** — `@template T`, `@template T of UpperBound`, template substitution in call sites
- **Dead code detection** — unused private methods, properties, functions
- **Taint analysis** — tracks data from `$_GET`/`$_POST` superglobals to HTML/SQL/shell sinks
- **Incremental cache** — SHA-256 content hashing; unchanged files are skipped on re-runs
- **Parallel analysis** — rayon-powered Pass 2; scales to available CPUs
- **PHP 8.x syntax** — `match`, enums, readonly, named arguments, first-class callables, fibers (parse only)
- **~180 built-in stubs** — string, array, math, JSON, file I/O, date/time, PDO, curl and more; Exception hierarchy and core interfaces (`Throwable`, `Iterator`, `Countable`, `ArrayAccess`, `Stringable`, …)

## Installation

```bash
cargo install --path crates/mir-cli
```

Or build locally:

```bash
cargo build --release
# binary at target/release/mir
```

## Usage

```
mir [OPTIONS] [PATHS]...
```

Analyze the current directory:

```bash
mir
```

Analyze specific paths:

```bash
mir src/ lib/
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--format <FORMAT>` | `text` | Output format: `text`, `json`, `github`, `junit`, `sarif` |
| `--show-info` | off | Include info-level issues (redundancies, style) |
| `-j, --threads <N>` | CPU count | Parallelism |
| `--cache-dir <DIR>` | off | Enable incremental cache in `DIR` |
| `--stats` | off | Print file count, error/warning totals, elapsed time |
| `-v, --verbose` | off | Print per-file issue counts |
| `-q, --quiet` | off | Suppress all output; use exit code only |
| `--no-progress` | off | Disable the progress bar |
| `--php-version <X.Y>` | — | Target PHP version (stored; influences future checks) |
| `--version` | — | Print version |

### Output formats

| Format | Use case |
|--------|----------|
| `text` | Default terminal output with colors |
| `json` | Machine-readable array of issue objects |
| `github` | GitHub Actions annotations (`::error file=…`) |
| `junit` | JUnit XML — compatible with most CI systems |
| `sarif` | SARIF 2.1.0 — GitHub Code Scanning / VS Code |

### Exit codes

| Code | Meaning |
|------|---------|
| `0` | No errors found |
| `1` | One or more errors found |

## Issue kinds

<details>
<summary>Expand full list</summary>

**Undefined**
`UndefinedVariable`, `UndefinedFunction`, `UndefinedMethod`, `UndefinedClass`,
`UndefinedProperty`, `UndefinedConstant`, `PossiblyUndefinedVariable`

**Nullability**
`NullArgument`, `NullPropertyFetch`, `NullMethodCall`, `NullArrayAccess`,
`PossiblyNull*`, `NullableReturnStatement`

**Type mismatches**
`InvalidReturnType`, `InvalidArgument`, `InvalidPropertyAssignment`,
`InvalidCast`, `InvalidOperand`, `MismatchingDocblockReturnType`

**Array**
`InvalidArrayOffset`, `NonExistentArrayOffset`, `PossiblyInvalidArrayOffset`

**Redundancy**
`RedundantCondition`, `RedundantCast`, `UnnecessaryVarAnnotation`, `TypeDoesNotContainType`

**Dead code**
`UnusedVariable`, `UnusedParam`, `UnusedMethod`, `UnusedProperty`, `UnusedFunction`, `UnreachableCode`

**Inheritance**
`UnimplementedAbstractMethod`, `UnimplementedInterfaceMethod`,
`MethodSignatureMismatch`, `OverriddenMethodAccess`, `FinalClassExtended`, `FinalMethodOverridden`

**Security (taint)**
`TaintedHtml`, `TaintedSql`, `TaintedShell`

**Generics**
`InvalidTemplateParam`

**Other**
`DeprecatedMethod`, `DeprecatedClass`, `InternalMethod`, `InvalidThrow`,
`MissingThrowsDocblock`, `ReadonlyPropertyAssignment`, `ParseError`, `InvalidDocblock`

</details>

## Docblock annotations

mir understands the following docblock annotations:

| Annotation | Effect |
|-----------|--------|
| `@param Type $name` | Parameter type override |
| `@return Type` | Return type override |
| `@var Type` | Variable type annotation |
| `@throws ClassName` | Declares thrown exception |
| `@template T` | Declares a type parameter |
| `@template T of U` | Bounded type parameter |
| `@psalm-suppress IssueName` | Suppress a specific issue at this site |
| `@psalm-pure` | Marks function as side-effect-free |
| `@psalm-immutable` | Marks class as immutable |
| `@deprecated` | Marks class/method as deprecated |
| `@internal` | Marks as internal (emits `InternalMethod` if called externally) |

## Architecture

```
Source Files
    │
    ▼
[1] File Discovery  (recursive .php glob, skips vendor/.git/node_modules)
    │
    ▼
[2] Stubs Load  (mir-stubs: ~180 builtins + exception hierarchy + core interfaces)
    │
    ▼
[3] Parsing  (php-parser-rs → normalized AST)
    │
    ▼
[4] Pass 1 — Definition Collection  (sequential)
    │  Classes, interfaces, traits, enums, functions, constants, use/namespace maps
    ▼
[5] Codebase Finalization
    │  Resolve inheritance chains, build method dispatch tables, validate abstract impls
    ▼
[6] Class-level Checks  (M11)
    │  Signature compatibility, final enforcement, readonly
    ▼
[7] Pass 2 — Body Analysis  (parallel per file via rayon)
    │  Type inference, narrowing, call checking, branch merging, taint tracking
    ▼
[8] Dead Code Detection  (M18)
    │  Private unreferenced methods/properties/functions
    ▼
[9] Issue Collection & Reporting
    │  Deduplicate, filter by severity, format output
    ▼
[10] Cache Write  (optional)
     Persist per-file issue list keyed by SHA-256 content hash
```

## Crate layout

| Crate | Purpose |
|-------|---------|
| `mir-types` | `Union`, `Atomic`, type operations (merge, narrow, subtype, template substitution) |
| `mir-issues` | `IssueKind`, `Severity`, `Issue`, `IssueBuffer`, display/serialization |
| `mir-codebase` | `Codebase` — thread-safe DashMap registry of all symbols |
| `mir-parser` | Wraps `php-parser-rs`; span tracking, docblock extraction, type hint parsing |
| `mir-stubs` | Rust-native PHP built-in stubs loaded at analysis startup |
| `mir-analyzer` | `ExpressionAnalyzer`, `StatementsAnalyzer`, `CallAnalyzer`, `ClassAnalyzer`, `DeadCodeAnalyzer`, `ProjectAnalyzer` |
| `mir-cache` | SHA-256 content hashing, JSON-backed incremental cache |
| `mir-cli` | `clap` binary — flags, progress bar, output formatters |

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full milestone list and implementation status.

**What's next:**

- **M15 — Configuration** (`mir.xml` parsing, per-issue error level overrides, baseline files)
- **M16 remaining** — `--set-baseline`/`--update-baseline`, `--error-level`, `--no-cache`
- **M10** — literal equality narrowing (`$x === 'foo'` → `TLiteralString`)
- **M18** — `UnusedVariable`, `UnusedParam` detection
- **M20 — Plugin System**

## License

MIT
