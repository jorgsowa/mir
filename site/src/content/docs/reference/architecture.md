---
title: Architecture
description: Crate layout and analysis pipeline internals.
---

## Analysis pipeline

A CLI run goes through these phases. Steps marked *parallel* run on every core
with rayon.

```text
[1] File discovery
    composer.json autoload sections, or the paths given on the command line,
    minus <ignoreFiles>
        │
[2] Built-in stubs
    PHP functions, classes and constants for the target version, plus the
    vendor files Composer always loads (autoload.files)
        │
[3] Read and hash files                                       (parallel)
    BLAKE3 content hash per file; files deleted since the last run are evicted
        │
[4] Definition collection                                     (parallel)
    parse, then record classes, functions, constants and imports;
    unchanged files reuse cached definitions without parsing
        │
[5] Invalidation
    a file whose declarations changed evicts the cached results of the files
    that depend on it; a body-only edit doesn't
        │
[6] On-demand vendor loading
    classes not found yet are resolved through PSR-4 and loaded
        │
[7] Class-level checks
    inheritance, signatures, abstract and interface compliance, readonly, enums
        │
[8] Body analysis                                             (parallel)
    type inference, narrowing, call checking, taint tracking;
    largest files first, cached results replayed for unchanged files
        │
[9] Dead code and reporting
    unused symbols, plugin hooks, suppressions, baseline, error levels,
    output format
        │
[10] Cache write
    per-file results and the dependency graph in .mir/cache
```

The incremental engine behind these phases is
[salsa](https://github.com/salsa-rs/salsa). In a long-lived session (an editor,
the playground) the same queries re-run only for the file that changed and the
files that depend on its declarations.

## Stubs

PHP built-in definitions come from
[JetBrains phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs), vendored
in `crates/mir-analyzer/stubs/` with one directory per extension: 77
extensions, including `standard`, `Core`, `SPL`, `date`, `intl`, `mbstring`,
`PDO`, `curl`, `dom`, `redis`, `mongodb` and `xdebug`. A few files carry local
type fixes, and extensions missing upstream are written for mir.

`build.rs` embeds every stub file in the binary and builds a name index from
`PhpStormStubsMap.php`. Stubs are parsed with the same parser as user code, and
symbols are filtered by their `@since` and `@removed` tags for the target PHP
version.

User stubs from `<stubs>` in `mir.xml` take precedence over built-in and vendor
definitions.

## Column encoding convention

Source positions flow through several layers, each with a different encoding responsibility:

| Layer | Crate | Encoding | Notes |
|-------|-------|----------|-------|
| Parser | `php-rs-parser` | UTF-8 byte offset | `offset_to_line_col` returns the raw byte distance from the line start. Correct for a parser; consumers must convert. |
| Core data model | `mir-issues`, `mir-codebase` | **Unicode char count** | `IssueLocation.col_start`/`col_end` and `Location.col` are 0-based counts of Unicode code points (one slot per character as seen on screen). |
| CLI output | `mir-cli` | Unicode char count (direct) | Column numbers in terminal, GitHub Actions annotations, and JSON output match what editors display in their status bar. |
| LSP server | _(outside mir)_ | UTF-16 code units | The [LSP spec](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#position) requires UTF-16. Convert at the protocol boundary: `src[line_start..byte_offset].chars().map(\|c\| c.len_utf16()).sum()`. LSP 3.17 also supports `positionEncoding` negotiation for UTF-8 and UTF-32. |

For pure-ASCII PHP files all three encodings are identical. They diverge only for multi-byte identifiers:

- **Accented Latin / Cyrillic / CJK** (e.g. `$café`) — UTF-8 bytes > char count = UTF-16 units.
- **Emoji / supplementary-plane characters** (e.g. `$🎉`) — UTF-8 bytes > UTF-16 units > char count.

PHP 8 allows any Unicode letter in identifiers, so the distinction matters for correctness even if uncommon in practice.

## Crate layout

| Crate | Purpose |
|-------|---------|
| `mir-types` | `Union`, `Atomic`, type operations (merge, narrow, subtype, template substitution) |
| `mir-issues` | `IssueKind`, `Severity`, `Issue`, `IssueBuffer`, display/serialization |
| `mir-codebase` | Definition types for classes, functions, constants and stub slices |
| `mir-analyzer` | Parsing adapters, definition collection, salsa database, body analysis, narrowing, taint, dead code, batch runs and incremental sessions, embedded stubs |
| `mir-cli` | `clap` binary — flags, progress bar, output formatters, config/baseline loading |
| `mir-plugin` | Plugin API (`MirPlugin`, `PluginRegistry`), shared-library loading and the PHP plugin bridge |
| `mir-plugin-example` | Example Rust plugin built as a `cdylib` |
| `mir-wasm` | WebAssembly bindings used by the playground |
