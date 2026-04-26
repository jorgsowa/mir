---
title: Architecture
description: Crate layout and analysis pipeline internals.
---

## Analysis pipeline

```
Source Files
    │
    ▼
[1] File Discovery  (recursive .php glob, skips vendor/.git/node_modules)
    │
    ▼
[2] Stubs Load  (two layers — see below)
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

## Stubs

PHP built-in definitions (functions, classes, interfaces, constants) are loaded in two layers before any user code is analysed.

### Layer 1 — phpstorm-stubs (authoritative)

The [JetBrains phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs) repository is included as a git submodule at `crates/mir-analyzer/phpstorm-stubs`. At compile time, `build.rs` walks 33 selected PHP extension directories and embeds every `.php` stub file as a string literal via `include_str!()`. At startup, each embedded file is parsed through the normal PHP parser + definition collector, populating the codebase with PHP built-ins.

Extensions covered: `Core`, `standard`, `SPL`, `bcmath`, `ctype`, `curl`, `date`, `dom`, `fileinfo`, `filter`, `gmp`, `hash`, `iconv`, `intl`, `json`, `libxml`, `mbstring`, `mysqli`, `openssl`, `pcntl`, `pcre`, `PDO`, `posix`, `random`, `Reflection`, `session`, `SimpleXML`, `sodium`, `sockets`, `tokenizer`, `xml`, `zip`, `zlib`.

This provides 500+ functions, 100+ classes, 20+ interfaces, and 200+ constants. Updating is a single command:

```bash
git submodule update --remote crates/mir-analyzer/phpstorm-stubs
```

### Layer 2 — hand-written supplements

A smaller set of Rust-coded stubs in `crates/mir-analyzer/src/stubs.rs` runs after phpstorm-stubs and overrides or extends where precise parameter shapes matter (e.g. by-reference variadic params on `sscanf`, PHPUnit assertion helpers).

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
| `mir-codebase` | `Codebase` — thread-safe DashMap registry of all symbols |
| `mir-analyzer` | `ExpressionAnalyzer`, `StatementsAnalyzer`, `CallAnalyzer`, `ClassAnalyzer`, `DeadCodeAnalyzer`, `ProjectAnalyzer`, stubs loader, `build.rs` code-gen |
| `mir-cli` | `clap` binary — flags, progress bar, output formatters, config/baseline loading |
