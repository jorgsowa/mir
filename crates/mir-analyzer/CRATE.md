# mir-analyzer — crate guide

Static analyzer for PHP source: parses files (via `mir-parser`/`php-ast`),
collects definitions into salsa-tracked stubs, then runs incremental
type inference, call-site checking, and narrowing over function bodies.
Downstream consumers are `mir-plugin` (LSP), `mir-cli`, and `mir-wasm`;
the type system lives in `mir-types`, diagnostics in `mir-issues`, and
this crate is deliberately workspace-internal: most modules are
`pub(crate)`, with only the entry points exposed.

## 1. Public API surface (`lib.rs`)

Only a few modules are `pub` (several `#[doc(hidden)]`):

- `batch` — multi-file project analysis (`analyze_project` and friends).
- `session` — `AnalysisSession`: the long-lived, salsa-backed,
  incremental per-file analysis context; `file_analyzer::FileAnalyzer`
  is the per-file entry point built on it.
- `db` — the salsa database (`MirDbStorage`) and query modules.
- `cache` / `parse_cache` / `stub_cache` — shared caches.

Everything else (`collector`, `body_analysis`, `call`, `narrowing`,
`parser`, `stmt`, `expr`, `diagnostics`, …) is crate-private.

## 2. Parser (`parser/`)

`parser/mod.rs` adapts `mir-parser` output: converts parse diagnostics
to `mir_issues::Issue` with precise source-map locations and re-exports
the docblock parser and hint helpers.

- `parser/docblock/mod.rs` — `DocblockParser`: delegates tag extraction
  to the `phpdoc_parser` crate, then converts tags into
  `ParsedDocblock` (params, returns, templates, assertions) with
  resolved `Type` values.
- `parser/type_from_hint.rs` — converts PHP type hints (function/method
  parameter and return position) into `Type`.
- `parser/docblock/types.rs` — the docblock *type* parser; see §3.
- `parser/docblock/validate.rs` — post-parse validation: template
  instantiation, generic arity, `self`/`class-string` resolution.
  Note: its `normalize_fqcn` **preserves** a leading global `\`
  (unlike `type_from_hint`), so keep that asymmetry in mind when
  touching class-existence checks.

## 3. Docblock types and the keyword table (`parser/docblock/types.rs`)

`parse_type_string` is a top-down pipeline, in priority order:

1. nullable shorthand `?T`; 2. conditional `($x is T ? A : B)`;
3. balanced parentheses; 4. union `A|B`; 5. intersection `A&B`;
6. array shorthand `T[]` (keyed `array-key` → `T`, *not* `int`);
7. callable syntax `Closure(T): R`; 8. array/list/object shapes;
9. generics `name<...>`; 10. numeric and string literals;
11. **the keyword match**; 12. named-class fallthrough;
13. `mixed` last resort.

**The keyword table is the single source of truth.** `DOCBLOCK_TYPE_KEYWORDS`
lists every name that, in a docblock type position, means a type keyword
or pseudo-type and *never* a class (`int`, `boolean`, `self`, `interface-string`,
`int-mask`, `int-mask-of`, `class-string-map`, `empty`, `array-key`, …).
`is_docblock_type_keyword` (re-exported from `parser/docblock/mod.rs`) is the
shared predicate consulted by:

- `parse_type_string` / `parse_generic` — the keyword-vs-class decision
  (a leading `\` is stripped first; class names can't contain `-`),
- `validate.rs` — template / `self` / class-string handling,
- `collector::is_php_builtin_type` — the generic-parameter class gate,
- `diagnostics::is_docblock_keyword` — the wide diagnostic gate (§9).

The keyword *match arms* map each entry to its atom (or closest
approximation, documented per arm). Integrity tests in
`parser/docblock/tests.rs` keep the table and the arms in lockstep:
every bare entry must parse to a non-`TNamedObject`.

## 4. Collector (`collector/`)

Visits every top-level declaration in the AST and produces a `StubSlice`
(class, function, constant signatures) — **no type inference happens here**.
Per-kind files: `class.rs`, `interface.rs`, `trait.rs`, `enum.rs`,
`function.rs`, `annotation.rs`, `version_attrs.rs`, `literal_types.rs`;
`resolution.rs` handles name resolution into FQCNs, and
`preserve_native_nullability` merges native + doc nullability on signatures.

`is_php_builtin_type` (crate-private) gates which `TNamedObject`s may be
namespace-qualified; it **delegates** to the shared docblock keyword table
(§3) rather than keeping a duplicate list.

## 5. Batch analysis (`batch/`)

Multi-file orchestration on `AnalysisSession` (the retired `ProjectAnalyzer`
lives on here): parallel definition collection, lazy class loading
(`lazy.rs`), dead-code sweep, reverse-dependency index, and the
`AnalysisResult` return type. Batch-only configuration (issue
suppressions, progress callback, PHP version override) travels in
`BatchOptions` rather than being stored on the session; per-file LSP
entry points stay in `session/`.

## 6. Body analysis (`body_analysis/`)

Analyzes method/property bodies once a class's full definition is known:
`classes.rs` (methods, property hooks, constants), `functions.rs`,
`orchestration.rs` (work ordering across a file), `aggregates.rs`
(file-level rollups). Docblock-derived class references here are checked
through the wide `is_docblock_keyword` gate before any `class_exists`
question is asked.

## 7. Expression, statement, and call analysis

- `expr/` — infers the `Type` of any PHP expression; per-kind files for
  arrays, assignment, binary, casts, closures, conditionals, literals,
  objects, unary, variables, intrinsics, and shared helpers.
- `stmt/` — walks statements threading `FlowState` through control flow
  (control_flow, loops, declarations, expressions, `flow.rs`, and
  `return_type.rs` for declared-vs-inferred return checks).
- `call/` — call-site argument checking: `args/` (counts, types,
  nullability), `array_builtins.rs` (first-class array function
  semantics), `callable.rs` / `opaque_callback.rs`, `function.rs`,
  `method.rs`, `static_call.rs`.

## 8. Narrowing and flow state

`narrowing/` refines variable types from conditional expressions given a
branch direction, updating `flow_state::FlowState`: `core.rs` dispatches
to `arrays/` (count, `in_array`, `key_exists`, shapes), `assertions.rs`
(`@psalm-assert` and `!` negation), `class_const_compare.rs`,
`class_introspection.rs` (`is_subclass_of`, `get_class`, …),
`enum_class.rs`, `instanceof_core.rs` / `instanceof_disjuncts.rs`,
`literals.rs`, `strings.rs`, `type_fn.rs` (`gettype`/`is_*`).
`subtype.rs` and `contradiction.rs` back the type algebra used by
narrowing; `dead_code.rs` prunes provably-unreachable branches.

## 9. Diagnostics (`diagnostics.rs`)

Converts analyzer findings into `mir_issues::Issue`s with tight stored
spans (`storage_loc_to_location` for already-tight spans; class-level
clamping lives in `class.rs::issue_location`). Two name gates live here:

- `is_pseudo_type` — the narrow legacy check (a handful of Psalm
  pseudo-types).
- `is_docblock_keyword` — the **wide** gate:
  `is_pseudo_type || is_docblock_type_keyword`. Every docblock-type call
  site (class constants, return-type compatibility, signature filtering)
  uses the wide gate, so a new entry in the keyword table (§3) is
  automatically excluded from class-existence diagnostics.

`suppression.rs` applies `@psalm-suppress`-style annotations, and
`php_version.rs` / `attributes.rs` gate version- and attribute-dependent
behavior.

## 10. Guardrails, session plumbing, and tests

- `session/` — `AnalysisSession` owns the salsa DB and per-session
  caches; reads clone the DB under a brief lock (Arc-wrapped registries
  make the clone cheap) then run lock-free. Submodules: `loading`,
  `ingest`, `incremental`, `queries`, `stubs`.
- `db/` — the salsa layers: `mirdb.rs` (storage), `queries.rs`,
  `ancestors`, `scopes`, `resolver`, `inferred_types`, `per_function`,
  `ref_index`, `reference_locations`, `class_mention_index`,
  `subtype_index`, `deps`, `find_queries`, `workspace`, `nodes`.
- `indexing.rs` / `source_provider.rs` / `composer.rs` — project
  discovery: file lists, source loading, autoloader metadata.
- `metrics.rs`, `tmp_suffix.rs`, `util.rs`, `symbol.rs`, `reference_key.rs`
  — shared plumbing.
- **Tests**: parser tests in `parser/docblock/tests.rs` (including the
  keyword-table integrity tests), `db/tests.rs`, and `test_utils.rs`
  fixtures under `tests/` (organized `by-kind/`, e.g.
  `invalid_return_type/`); regression fixtures are kept per kind so a
  failing kind maps to one directory.
- **Guardrail**: the keyword table (§3) is the one place where docblock
  pseudo-types are named; add new pseudo-types there (plus their match
  arm) and the integrity tests force the arms to follow. Never add a
  second keyword list in `collector/`, `diagnostics.rs`, or
  `validate.rs` — delegate to the table instead.
