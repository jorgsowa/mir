# Architecture

## Analysis pipeline

```
Source Files
    │
    ▼
[1] File Discovery  (recursive .php glob, skips vendor/.git/node_modules)
    │
    ▼
[2] Stubs Load  (~580 builtins + exception hierarchy + core interfaces)
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
[6] Class-level Checks
    │  Signature compatibility, final enforcement, readonly
    ▼
[7] Pass 2 — Body Analysis  (parallel per file via rayon)
    │  Type inference, narrowing, call checking, branch merging, taint tracking
    ▼
[8] Dead Code Detection
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
| `mir-analyzer` | `ExpressionAnalyzer`, `StatementsAnalyzer`, `CallAnalyzer`, `ClassAnalyzer`, `DeadCodeAnalyzer`, `ProjectAnalyzer`, built-in stubs |
| `mir-cli` | `clap` binary — flags, progress bar, output formatters, config/baseline loading |
