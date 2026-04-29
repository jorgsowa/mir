# mir Roadmap

Current version: **v0.16.0**

---

## Milestone Status

| Milestone | Status |
|-----------|--------|
| M0 — Workspace bootstrap | ✅ Complete |
| M1 — Type system | ✅ Complete |
| M2 — Parser wrapper | ✅ Complete |
| M3 — Stubs (phpstorm-stubs) | ✅ Complete |
| M4 — Codebase registry | ✅ Complete |
| M5 — Pass 1: definition collection | ✅ Complete |
| M6 — Issue system | ✅ Complete |
| M7 — Expression analyzer | ✅ Complete |
| M8 — Statement analyzer | ✅ Complete |
| M9 — Call analyzer | ✅ Complete |
| M10 — Type narrowing | ✅ Complete |
| M11 — Class analyzer | ✅ Complete |
| M12 — Loop analysis | ✅ Complete |
| M13 — Generic types | ✅ Complete |
| M14 — Pass 2: body analysis | ✅ Complete |
| M15 — Configuration (`mir.xml`) | ✅ Complete |
| M16 — CLI | ✅ Complete |
| M17 — Cache layer (Pass 2, content-hash) | ✅ Complete |
| M18 — Dead code detection | ✅ Complete |
| M19 — Taint analysis | ✅ Complete |
| M20 — Plugin system | ❌ Not started |
| M21 — LSP API surface | ✅ Complete |
| M22 — WASM playground | ✅ Complete |
| M23 — Psalm docblock parity | ✅ Complete |

### M15 — Configuration (`mir.xml`)

Completed: `<projectFiles>`, `<ignoreFiles>`, `<issueHandlers>`, `<stubs>` (file and
directory entries), `phpVersion` (root attribute and child element), `errorLevel`,
`findUnusedCode`, `findUnusedVariables`. Auto-discovery walks up from the current directory
and falls back to `psalm.xml` for drop-in Psalm compatibility.

### M16 — CLI

Completed: `--format` (text, JSON, GitHub Actions annotations, JUnit, SARIF),
`--set-baseline` / `--baseline` / `--update-baseline` / `--ignore-baseline`,
`--no-cache`, `--cache-dir`, `--clear-cache`, `--php-version`, `--find-dead-code`,
`--quiet`, `--verbose`, `--no-progress`, `--config`.

### M21 — LSP API surface

`symbol_at` for go-to-definition and find-references; `re_analyze_file` for incremental
single-file re-analysis with structural snapshot diffing; `inject_stub_slice` /
`StubSlice` / `CodebaseBuilder` for salsa-style pure Pass 1 computation;
`ParsedDocblock::is_inherit_doc` for hover/completion chain walking.
Location type unified across all crates, UTF-16/UTF-32 conversion at the LSP boundary.

### M22 — WASM playground

Interactive playground embedded in the docs site: PHP version selector (8.1–8.5), live
diagnostic underlay overlays, severity-colored cards. Shipped in v0.13.0.

### M23 — Psalm docblock parity

`@psalm-suppress`, `@psalm-assert`, `@psalm-assert-if-true`, `@psalm-assert-if-false`,
`@psalm-param`, `@psalm-return`, `@psalm-import-type`, `@psalm-require-extends`,
`@psalm-require-implements`, `@inheritDoc`. `InvalidDocblock` issue for unparseable
annotations. Shipped across v0.9.1–v0.14.0.

---

## Performance & Architecture Roadmap

### Phase 1 — Memory  ✅ Complete (v0.5.1)

**1. String interning** ✅
Reference keys interned as lock-free `u32` IDs, eliminating `Arc<str>` duplication across
`symbol_reference_locations`, `file_symbol_references`, and the dead-code sets.

**2. Flat `Vec<Ref>`** ✅
Nested map structure replaced by a single `Vec<(symbol_id, file_id, start, end)>` during
the build phase.

**3. `compact_reference_index()`** ✅
After Pass 2, the `Vec<Ref>` is sorted and two CSR index arrays are built — one keyed by
symbol, one by file. Delivered ~5× reduction in reference index memory.

---

### Phase 2 — Non-LSP incremental  ⚠️ Partial (v0.5.2)

**4. Cache Pass 1 results** ❌ Not started
Extend `CacheEntry` with `FileDefinitions`. On a cache hit, skip parsing and definition
collection entirely — not just body analysis. Biggest win for large projects where few
files change between runs.

**5. Cache finalization** ✅ (v0.5.2)
`re_analyze_file` captures a structural snapshot before file removal. If inheritance fields
are unchanged after Pass 1, `all_parents` is restored directly and ancestor recomputation
is skipped.

---

### Phase 3 — Remove the pass barrier  ✅ Complete

**6. Per-class `OnceLock` finalization** ✅ Complete
`ensure_finalized(fqcn)` computes ancestors lazily per class/interface and memoizes via
`DashMap<Arc<str>, OnceLock<Arc<[Arc<str>]>>>` with thread-local cycle detection.
`finalize()` is now a warm-all wrapper over `ensure_finalized`. `invalidate_finalization()`
clears the cache; `remove_file_definitions()` evicts only the affected entries granularly.

**7. Merge the pass loop** ✅ Complete
Pre-index and definition collection sub-passes merged into a single parallel `par_iter`,
eliminating the second parse per file. The eager `finalize()` barrier between Pass 1 and
Pass 2 is removed: `ensure_finalized()` is now called lazily at every `all_parents` read
site (`get_method_inner`, `get_property_inner`, `get_class_constant`,
`extends_or_implements`, `has_unknown_ancestor`, `collect_members_for_fqcn`,
`ClassAnalyzer::analyze_all`, `check_trait_constraints`, `argument_type_satisfies_param`,
`file_structural_snapshot`). Pass 1 result collection is the only barrier before Pass 2;
a second barrier remains between the G6 priming sweep and the issue-emitting Pass 2.

---

### Phase 4 — Symbol-level incremental + LSP (Salsa)  ⚠️ In progress

Current cache invalidation is file-level: if file A changes, all files importing anything
from A are evicted — even if only a private method body changed. A proper query system
tracks symbol-level dependencies and skips re-analysis when query outputs are unchanged.

The migration is broken into five sub-phases, each a shippable PR:

**S0. Database skeleton** ✅ (v0.16.0)
`salsa = "0.26"` added to the workspace. `MirDatabase` trait, `SourceFile` input, and
`MirDb` concrete database defined in `crates/mir-analyzer/src/db.rs`. No analysis logic
changed; this is the landing pad for subsequent sub-phases.

**S1. `collect_file_definitions` query** ✅ (v0.16.0)
`collect_file_definitions` Salsa tracked query wraps the existing `collect_slice` pure
variant. `StubSlice` result is memoized per `SourceFile`; consecutive in-process calls
with unchanged text skip parse and definition collection entirely (warm-path for LSP /
watch-mode re-analysis). Result is injected into `Codebase` via `inject_stub_slice`.
`re_analyze_file` (LSP incremental path) now goes through `collect_file_definitions`.
The batch `analyze()` path continues to use the direct parse route; Salsa memoization
for the batch path is deferred to S4 alongside the accumulator rewrite.

**S2. `class_ancestors` query** ✅ (v0.16.0)
`ClassNode` Salsa input (fqcn, active, parent, interfaces, traits, extends) and
`class_ancestors` tracked query with cycle recovery (PHP cycles return empty).
The structural snapshot triad (`file_structural_snapshot`, `structural_unchanged_after_pass1`,
`restore_all_parents`) is deleted from `Codebase`; `re_analyze_file` uses Salsa ancestry
comparison instead. Cold path (first LSP edit per file) falls back to `Codebase.all_parents`
for the old-state baseline, then calls `invalidate_finalization() + finalize()` on change.
Warm path (subsequent edits) skips `finalize()` when Salsa detects no ancestry change.
`finalization_cache` is kept for the batch path and `ensure_finalized`; full deletion is
deferred to S5 when `&dyn MirDatabase` is threaded through the analyzers.

**S3. `inferred_return_type` query** ❌ Blocked on S5
Replace `Pass2Driver::new_inference_only` and the G6 priming sweep with a Salsa tracked
query using fixpoint cycle recovery. Depth-N inferred return type chains resolve correctly.
Priming sweep (~2× Pass 2 CPU cost) is eliminated.

*Prerequisite:* S5 must land first. `inferred_return_type(db, FunctionNode) -> Union`
needs to call `StatementsAnalyzer`/`ExpressionAnalyzer`, which currently read from
`Codebase` directly (not tracked by Salsa). Without db threading, the query cannot observe
its own dependencies and will give stale results after the first evaluation.

**S4. `analyze_file` query + accumulators** ❌ Blocked on S5
Issues and reference locations become Salsa accumulators. `re_analyze_file` collapses to
two lines (set input + read accumulator). `AnalysisCache`, `build_reverse_deps`,
`evict_with_dependents`, and the compact CSR reference index are deleted. A private method
body change invalidates zero dependent files.

*Prerequisite:* S5 must land first, for the same reason as S3 — `analyze_file` must be
able to track all its Codebase reads through `&dyn MirDatabase`.

**S5. Thread `&dyn MirDatabase` through analyzers** ⚠️ In progress
Thread `&dyn MirDatabase` through `StatementsAnalyzer` / `ExpressionAnalyzer` /
`ClassAnalyzer`. Codebase lookups that feed into tracked queries (`inferred_return_type`,
`analyze_file`) become db-tracked reads. The `Codebase` struct shrinks incrementally as
fields move to Salsa inputs; `finalization_cache` is deleted as the last step.

*Note:* S5 does not need to be completed atomically. Individual Codebase fields can be
migrated one at a time (functions → methods → classes → …) with the remaining fields
still in `Codebase`. Each batch is a shippable PR. Full deletion of `Codebase` and the
two `Interner` fields is the final PR in this sub-phase.

Sub-PRs (each shippable, fixture suite green at every step):

- **PR1** ✅ `&dyn MirDatabase` threaded through `StatementsAnalyzer` /
  `ExpressionAnalyzer`; available as `ea.db: Option<&dyn MirDatabase>`.
- **PR2a / PR2b** ✅ `FunctionNode` input + register/deactivate; `ResolvedFn`
  helper; main metadata read in `call/function.rs` and the fn-existence
  use site migrated to db.
- **PR3a / PR3b** ✅ `MethodNode` input + register/deactivate; `ResolvedMethod`
  helper; method-call read sites migrated to db.
- **PR4a / PR4b** ✅ `PropertyNode` + `ClassConstantNode` inputs; property /
  constant read sites in `expr.rs` migrated via
  `find_property_node_in_chain` / `class_constant_exists_in_chain`.
- **PR5a / PR5b** ✅ `ClassNode` extended with `is_trait`, `is_enum`,
  `is_abstract`; traits and enums registered as `ClassNode`s.
  `class_kind_via_db` helper; the two `is_interface` / `is_abstract_class`
  read sites in `call/static_call.rs` and `call/method.rs` migrated to
  prefer db with codebase fallback.
- **PR6a / PR6b** ✅ `ClassNode` extended with `template_params`;
  populated for classes/interfaces/traits at upsert time.
  `type_exists_via_db` and `class_template_params_via_db` helpers added.
  All four read patterns in `call/args.rs` (`type_exists`,
  `interfaces.contains_key`, `traits.contains_key`,
  `get_class_template_params`) migrated to prefer db with codebase
  fallback via small private wrappers in `args.rs`.
- **PR7** ✅ `has_unknown_ancestor_db_or_codebase` helper in `db.rs`
  walks `class_ancestors` for db-registered classes and falls back to
  `Codebase::has_unknown_ancestor` otherwise; per-ancestor "known"
  check is `type_exists_via_db || codebase.type_exists` so bundled
  stubs are still respected. All seven read sites in `expr.rs`,
  `stmt/mod.rs`, `call/method.rs`, and `call/static_call.rs` migrated.
- **PR8** ✅ `MirDb::ingest_codebase(&Codebase)` mirrors the entire
  codebase symbol table (classes, interfaces, traits, enums,
  functions, their methods, properties, and constants) into the
  Salsa db.  Wired into `ProjectAnalyzer::load_stubs` so bundled and
  user stubs are db-visible the moment they land in `Codebase`.
- **PR9** ✅ `ingest_codebase` also called from the batch `analyze`
  path after Pass 1 + PSR-4 lazy-load complete.  Preparatory: today
  the batch `Pass2Driver` still passes `db: None`, so this changes
  no behavior — it sets up dropping the per-helper codebase
  fallbacks once `Pass2Driver` is wired with a shared db reference.
- **PR10a** ✅ `MirDb: Clone`.  Salsa 0.26's parallel pattern is
  per-thread cloning (each clone gets a fresh `ZalsaLocal`,
  underlying memoization is shared) — `salsa::Database: Send` but
  not `Sync`, so `&dyn MirDatabase` cannot be shared across
  `par_iter` workers.  Cloning is the prerequisite for threading
  the db through batch Pass 2.  Sanity test verifies a clone
  observes pre-clone upserts and resolves `class_ancestors`.

- **PR10b** ✅ Thread the cloned db through batch `Pass2Driver`
  (priming sweep + main sweep) using `for_each_with` /
  `map_with` so each rayon worker gets its own clone.
  `lazy_load_from_body_issues` stays on `db: None` for now (still
  has codebase fallbacks); a second `ingest_codebase` call after
  that lazy-load lands when the fallbacks are dropped.
  Collateral fixes: `ingest_codebase` now also registers enum
  cases (not just `own_constants`) as `ClassConstantNode`s so
  `class_constant_exists_in_chain` finds `Status::Active` and
  similar; `resolve_property_type` falls back to
  `Codebase::get_property` when the db lookup misses (db doesn't
  yet track docblock `@mixin` chains).

Remaining for S5 (rough order):
- Drop the codebase fallback in the prefer-db wrappers
  (`type_exists`, `is_interface`, `class_template_params`,
  `has_unknown_ancestor_db_or_codebase`) once batch Pass 2 reads
  the db.
- Remove `finalization_cache` and the structural snapshot fallback in
  `re_analyze_file` once no caller reaches `ensure_finalized` (gated
  on the per-field migrations finishing).
- Delete the remaining fields from `Codebase` (functions, methods,
  properties, constants, classes, interfaces, traits, enums) once no
  read site references them — one batch per field group, each a
  shippable PR.

Expected: sub-second re-analysis on save for LSP; precise invalidation across all query types.

---

### Phase dependencies

```
Phase 1 ──────────────────────── complete
Phase 2 ──────────────────────── item 4 subsumed by Phase 4 S1 (no longer worth doing separately)
Phase 3 ── complete; eager finalize() barrier removed, lazy ensure_finalized() at read sites
Phase 4 ── subsumes Phase 2 & 3  (Salsa makes manual caching redundant);
           S0–S2 complete; S5 in progress (PR1–PR5 landed)
           S3 and S4 unblocked only after S5 (db threading through analyzers)
           S5 → S3 → S4 is the correct execution order
```

---

## Analyzer Gaps

Known limitations embedded as explicit skips in the analyzer.
Each entry names the gap, the files/lines where the skip lives, and what lifting it requires.

---

### G1 — Full template inference for generic class instantiation

**What is skipped:**
Return type checks bail out when the declared type is a generic class instantiation
(`Result<string, void>`), an interface, or a class not in the codebase
(`declared_return_has_template`, `src/stmt.rs:1497–1505`). Param contravariance checks
similarly bail when either side contains a `TTemplateParam` (`src/class.rs:419`).
Expression-level checks skip template params to avoid false positives (`src/expr.rs:1868`).

**What lifting it requires:**
Full template inference: when a generic type is instantiated, substitute the concrete type
arguments into method signatures before comparing. Requires propagating the instantiation
context (`HashMap<template_name, Union>`) through `StatementsAnalyzer` and
`ExpressionAnalyzer` for every call and return-type check.

---

### G2 — Post-Pass-2 FQCN lazy loading (no `use` import)

**What is skipped:**
`#[ignore = "known gap: FQCN-without-use requires post-Pass-2 lazy loading"]`
(`tests/lazy_load.rs:227`). Fully-qualified class names referenced directly inside function
bodies (e.g. `new \Foo\Bar\Baz()`) without a `use` statement are only discovered during
Pass 2. The current lazy-load trigger runs before Pass 2 completes, so these classes are
never loaded on demand.

**What lifting it requires:**
A post-Pass-2 lazy-load phase: after all files complete Pass 2, collect still-missing FQCNs
and re-run loading + `ensure_finalized()`. Full inline resolution would require
`ensure_finalized()` to drive PSR-4 loading on cache miss — `Codebase` does not yet have
access to PSR-4 data, so that integration is a separate step.

---

### G3 — Override covariance with named objects and `self`/`static` ✅ Complete

**What was skipped:**
Return type covariance in `ClassAnalyzer` was skipped when either side involved a named
object (`involves_named_objects`) or `TSelf`/`TStaticObject` (`involves_self_static`).
This suppressed real violations alongside intended ones.

**How it was fixed:**
`named_object_return_compatible` (from `src/stmt.rs`) is now called inside the override
check when both unions consist entirely of object-like atoms (named objects, self, static,
parent, null, void, never, class-string). Mixed scalar+object unions still skip to avoid
false positives — that is the remaining G5 gap.

---

### G4 — Param contravariance with named objects in override checks

**What is skipped:**
The param contravariance loop in `ClassAnalyzer` skips pairs where either side contains a
named object (`src/class.rs:417`). A child method that illegally narrows a param from
`Animal` to `Cat` is not flagged.

**What lifting it requires:**
Use the codebase inheritance graph (`all_parents`, `all_interfaces`) to check whether
`child_param_type` is a subtype of `parent_param_type` for object types, mirroring how
`named_object_return_compatible` works. Depends on G3's infrastructure.

---

### G5 — Non-object type handling in `named_object_return_compatible`

**What is skipped:**
Falls through to a simple subtype check for non-object atomic types with the comment
"Non-object types: not handled here" (`src/stmt.rs:1368`). Union types mixing objects with
scalars (e.g. `string|MyClass`) may produce false negatives.

**What lifting it requires:**
Extend `named_object_return_compatible` to split union types: object atoms go through the
inheritance path, scalar atoms go through the existing simple subtype check.

---

### G6 — Cross-file inferred return types ✅ Complete (depth-1)

**What was skipped:**
`inferred_return_type` is written during the parallel Pass 2, so a file cannot see another
file's inferred return type if that file has not yet finished. Cross-file inference was
therefore incomplete when the calling file was analyzed before the callee.

**How it was fixed:**
A type-inference priming pass now runs before the issue-emitting Pass 2. The priming pass
runs all function and method bodies in parallel but skips reference tracking (so dead-code
and go-to-definition data are not double-counted); it only writes `inferred_return_type`
back to the codebase. By the time the main Pass 2 starts, every function's inferred return
type is already populated, eliminating the race for the common depth-1 case.

**Remaining gap:**
Depth-N chains (A→B→C where B's inferred type depends on C's) are still subject to
ordering within the priming pass. A fixed-point iteration or Phase 4 (Salsa) would resolve
this completely.

---

## False Positives

Known cases where the analyzer emits a diagnostic for correct PHP code.

---

### FP2 — Inner variable of `$$x` reported as `UnusedVariable`

**What fires incorrectly:**
In `$$key`, `$key` is read to determine the variable name at runtime, but the
`ExprKind::VariableVariable` handler (`src/expr.rs`) only returns `Union::mixed()` without
marking the inner variable as read. Any variable whose only use is as the operand of `$$`
is reported as `UnusedVariable`.

**Root cause:**
The variable-variable expression handler does not call `ctx.read_vars.insert(inner_name)`.

**What fixing it requires:**
When `ExprKind::VariableVariable(inner)` is a simple `Variable` node, extract its name and
insert it into `ctx.read_vars` before returning `Union::mixed()`.

---

### FP3 — `UnusedVariable` and `UnusedParam` always report line 1

**What fires incorrectly:**
Every `UnusedVariable` and `UnusedParam` diagnostic is emitted at `line: 1, col_start: 0`
regardless of where the variable was actually declared or assigned.

**Root cause:**
`emit_unused_variables` and `emit_unused_params` (`src/diagnostics.rs`) construct the
`Location` with hardcoded `line: 1` because the per-variable assignment location is not
tracked in `Context`.

**What fixing it requires:**
Add a `HashMap<String, Location>` (or similar) to `Context` that records the span of the
first assignment for each variable. Use that span when constructing the issue location in
`emit_unused_variables` / `emit_unused_params`.

---

## Refactoring

### R1 — Monolithic analysis files

`expr.rs` (~2 000 lines), `stmt.rs` (~1 600 lines), and `collector.rs` (~1 900 lines) are
candidates for the same sub-module split already applied to `call/` (args, function, method,
static_call). Splitting each into focused sub-modules would reduce compile times and make
targeted changes easier to review.

---

### R2 — Hot-path locking

`Codebase` uses `DashMap` throughout. After Pass 1 the symbol tables are read-only; freezing
them into `Arc<HashMap>` post-Pass-1 would eliminate per-read locking on the hottest path in
Pass 2.

---

### R3 — Type interning

**Needs profiling before attempting.**

The original premise — that singleton unions (`TString`, `TInt`, `TNull`, `mixed`) cause
allocator pressure — is off: `SmallVec<[Atomic; 2]>` keeps single-element unions fully
inline, so those copies are stack memcpys, not heap allocations.

The real candidate for Arc-sharing is stored return types (`FunctionStorage::return_type`,
`MethodStorage::return_type`): `effective_return_type().cloned()` in the call analyzer
clones the stored `Union` on every resolved call site. For widely-called functions this
multiplies any complex union (>2 atomics, which do heap-allocate) across all call sites.
Changing storage to `Arc<Union>` would reduce those to cheap ref-count bumps.

That change is pervasive (touches the entire value-type API surface) and the win is
speculative without a profile showing return-type cloning as a hot spot.
