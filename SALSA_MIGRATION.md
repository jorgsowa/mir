# Salsa migration — current status & open questions

## Shipped through PR38

S0 (db skeleton) and S1 (`collect_file_definitions` query) are complete.
S2 (`class_ancestors` query) is complete and is the LSP warm-path
short-circuit.  S5 has shipped 26 incremental PRs migrating analyzer
reads onto the db:

- PR1 — `&dyn MirDatabase` threaded through analyzers.
- PR2–6 — `FunctionNode`, `MethodNode`, `PropertyNode`,
  `ClassConstantNode`, `ClassNode` Salsa inputs + helpers.
- PR7–9 — `ingest_codebase` mirrors stubs/PSR-4 lazy-loads into the db.
- PR10–11 — db threaded through batch `Pass2Driver` (cloned per worker
  for rayon parallelism).
- PR12–13 — codebase fallbacks dropped from prefer-db wrappers; helpers
  renamed.
- PR14–15 — `ClassAnalyzer.db` lifecycle cleanup; trait-constraint
  ancestor reads via `class_ancestors`.
- PR16–17 — `Codebase::traits.get` reads in trait-constraint validation
  removed; `ClassAnalyzer` reads ancestors from db.
- PR18–20 — `require_extends`/`require_implements` on `ClassNode`,
  `extends_or_implements_via_db` migrating ~30 call sites,
  `method_is_concretely_implemented` in db.
- PR21 — fast-skip identical re-ingest (resolved a 60s+ deadlock).
- PR22 — `method_exists_via_db` + 4 magic-method existence sites.
- PR23 — drop codebase fallback in `resolve_fn` / `fn_exists`.
- PR24 — db-track class/enum match guards in `resolve_property_type`.
- PR25 — thread db through `narrowing.rs` (11 sites).
- PR26 — drop dead `Codebase::extends_or_implements`.
- PR27 — drop redundant codebase fallback in 4 magic-method existence
  checks (`__call`, `__callStatic`, `__invoke` x2): `method_exists_via_db`
  alone now answers them.
- PR28 — read method-body params/return_type from db in 4 `pass2.rs`
  sites + 1 `stmt/mod.rs` site (own-class lookups while seeding
  `Context::for_method`).  Adds `db::lookup_method_in_chain` helper
  (own → ancestors walk, mirroring `Codebase::get_method`).
- PR29 — prefer db for `__construct` param lookup in `expr.rs`'s `new
  Foo(...)` arity check.
- PR30 — `lookup_method_in_chain` now walks trait-of-traits
  (transitively) and ancestor traits, mirroring `method_exists_via_db`'s
  semantics.  Closes the trait-walk gap behind three of the four
  fixtures that failed when the codebase fallback was first dropped
  in `resolve_method_from_db`'s callers.
- PR31 — closes the last two semantic gaps in
  `lookup_method_in_chain`: enum implicit methods (`cases` always,
  `from`/`tryFrom` for backed enums) are now synthesized as
  `MethodNode`s at ingest time; docblock `@mixin` chains have a new
  `mixins: Arc<[Arc<str>]>` field on `ClassNode` plus a recursive walk
  in the chain helper.  Brings the helper to full parity with
  `Codebase::get_method`'s walk.
- PR32 — `resolve_method_from_db` now calls
  `db::lookup_method_in_chain` directly; the private
  `find_method_node_in_chain` helper (own + ancestors only) is
  removed, and the `or_else(|| codebase.get_method(...))` fallbacks
  in its two callers (`call/method.rs:299`,
  `call/static_call.rs:56`) are deleted.  All four originally-failing
  fixtures pass.
- PR33 — drop the `__construct` codebase fallback in `expr.rs`
  introduced by PR29.  The chain helper covers it after PR30/PR31.
- PR34 — drop codebase fallback for class-constant existence at
  `expr.rs:830`.  `class_constant_exists_in_chain` (moved from `expr.rs`
  into `db.rs` next to its peers) walks own + `class_ancestors`, which
  already includes parents, interfaces, and direct traits — full parity
  with `Codebase::get_class_constant` for existence purposes.
- PR35 — `db::lookup_property_in_chain` replaces `find_property_node_in_chain`
  in `expr.rs`, extending it to walk `@mixin` chains (own + each
  ancestor's mixins, recursive).  Drops the
  `.or_else(|| codebase.get_property(...))` fallback in
  `resolve_property_type`.  Uses the same `mixins` field on `ClassNode`
  added in PR31 and is cycle-safe via a per-call visited set.
- PR36 — drop the redundant `cls.get_property(...)` fallback in the
  readonly-assignment check at `expr.rs:1473`.  Both arms read
  own_properties only (no chain walk); `ingest_codebase` mirrors every
  class's own_properties into `PropertyNode` inputs, so the db path
  is at parity for this site.
- PR38 — focused unit tests for the chain helpers
  (`lookup_method_in_chain`, `lookup_property_in_chain`,
  `class_constant_exists_in_chain`): own-vs-ancestor precedence,
  trait-of-traits, `@mixin` walks, mutual-mixin cycles, case sensitivity,
  inactive-class handling.
- PR37 — drops the last `Codebase::get_method` fallback in the analyzer
  (`call/method.rs:51`).  `lookup_method_in_chain` already returns the
  *owner* of the resolved method, so reading `inferred_return_type` only
  needed a direct (non-walking, non-finalizing) storage lookup; introduced
  `Codebase::method_inferred_return_type` mirroring the
  `call/function.rs:36` pattern for free functions.
- PR39 — drops `Codebase::ensure_finalized`, the `finalization_cache`
  field, the `compute_all_parents` private helper, and the public
  inheritance-walking lookups (`get_method`, `get_property`,
  `get_class_constant`).  Inlines the only two remaining external uses
  of those walks (`has_magic_get`, `get_member_location`) as direct
  walks of `own_methods`/`own_properties` + `all_parents`/own traits.
  `Codebase::finalize()` now computes ancestors directly (single
  recursive walk per class/interface) instead of memoizing per-class.
  Also adds an explicit `self.codebase.finalize()` call in
  `ProjectAnalyzer::analyze` between Pass 1 (with PSR-4 lazy load) and
  Pass 2, since `all_parents` is no longer populated lazily on first
  read.  Resolves the long-standing PR37 gating issue documented below.

## Architectural blocker (still relevant for S3, no longer blocks S5)

Promoting `MethodStorage::inferred_return_type` /
`FunctionStorage::inferred_return_type` to tracked fields on
`MethodNode` / `FunctionNode` (the smallest concrete unit toward S3)
**deadlocks** when the post-priming sync runs.

Stack trace at the hang:

```
sync_inferred_return_types
  MethodNode::set_inferred_return_type
    salsa::Storage::cancel_others
      Condvar::wait — never wakes
```

Same root cause as PR21: Salsa's `zalsa_mut` waits for *all other*
clones of the database to drop before allowing a write.  Rayon's
`for_each_with` clones the db per-worker; rayon's thread pool retains
worker threads (and apparently their thread-local Salsa state) after
the parallel sweep returns.  The strong-count never drops to 1, so
the setter waits forever.

PR21 worked around this by fast-skipping setter calls when values match
(no setter ⇒ no `cancel_others`).  That trick doesn't apply to the
inferred-return-type sync because the values genuinely change
(`None` → `Some(union)` is the whole point).

For S5, PR37 sidestepped this by keeping `inferred_return_type` outside
Salsa: the analyzer reads it directly from `Codebase` storage on the
already-resolved owner FQCN (no inheritance walk, no finalization).
S3 still has to confront this — the field has to enter Salsa's
dependency graph for cycle detection, fixpoint, and on-demand
evaluation — but S5 no longer waits on it.

## Resolution candidates for S3 (each requires a design decision)

1. **Avoid `for_each_with` for the priming sweep.** Use `rayon::scope`
   or manual scoped threads so worker clones are guaranteed dropped
   before the sync runs.  Risk: rayon may still retain thread-local
   Salsa state via its persistent thread pool; needs verification.

2. **Move the priming-sweep writeback into Salsa setters directly,
   from inside the parallel closure**, using `&mut MirDb` clones.
   Salsa supports `&mut` writes from each clone independently because
   they share the underlying storage via Arc.  But: each setter call
   acquires a write lock — turning a parallel pass into a contended
   serial one.  Likely a perf regression; probably worth measuring.

3. **Skip the in-memory mirror; let S3 read inferred types directly
   through a non-tracked trait method.**  Add
   `fn function_inferred_return(&self, fqn: &str) -> Option<Union>`
   to `MirDatabase`, implemented as a `Codebase` lookup.  Tracking is
   then *outside* Salsa for this one field — it works, but won't help
   when S3 promotes `inferred_return_type` to an actual tracked query
   (cycle detection, fixpoint, on-demand evaluation all need the value
   inside Salsa's dependency graph).

4. **Reorder the migration: do the Codebase storage promotion *first*,
   before any inferred-return-type promotion.**  If `FunctionStorage`
   and `MethodStorage` themselves become Salsa inputs, the priming
   sweep writes via setters from inside the parallel closure (option
   2), but each clone has its own `Storage` snapshot for write
   contention purposes.  This is what S5's documented end-state
   already requires; treat the deadlock as one more reason to do that
   refactor sooner rather than later.  Largest scope.

5. **Don't promote inferred_return_type at all; keep S3 as a tracked
   query that reads directly from `Codebase` storage via a custom
   trait method.**  Salsa won't observe codebase mutations
   automatically (codebase is interior-mutable via DashMap), so the
   tracked query gets stale results unless we manually invalidate.
   Effectively gives up on S3's perf win.

## Method/property/constant resolution status: codebase-free

All `Codebase::get_method` / `get_property` / `get_class_constant`
calls from analyzer code are gone (PR27–PR33 for methods; PR34–PR36
for properties and constants; PR37 for the last
`inferred_return_type` read).  PR39 deletes those three public methods
from `Codebase` entirely, along with `ensure_finalized` and the
per-class `finalization_cache`.  Inheritance walks now live in two
places only:

- **`db.rs` (analyzer-facing, Salsa-memoized)**:
  - `db::lookup_method_in_chain` — own → mixins (recursive) → traits
    (transitive) → ancestors (with each ancestor's own + traits + mixins).
  - `db::lookup_property_in_chain` — own → mixins (recursive) → ancestors
    (with each ancestor's mixins).
  - `db::class_constant_exists_in_chain` — own + `class_ancestors`.
- **`Codebase::finalize`** populates `all_parents` once per class /
  interface.  `has_magic_get`, `has_unknown_ancestor`,
  `get_member_location`, and `get_inherited_template_bindings` walk
  that pre-computed list directly — no lazy/recursive resolver
  remains in `mir-codebase`.

## Open questions for the next session

1. **For S3 (inferred-return-type promotion), which deadlock
   resolution candidate does the project want?**  PR37 unblocked S5
   without picking one, but S3 still has to.  See "Resolution
   candidates for S3" above.

2. **For S3 itself, what's the body-AST sourcing strategy?**  The
   tracked query needs a way to walk a function/method body without
   the bumpalo arena lifetime that Pass 2 currently uses.  Re-parsing
   via the memoized `collect_file_definitions` then walking by FQN is
   the natural fit (shape matches S1) — confirm before committing.

3. **For S4 (analyze_file query + accumulators), how do we handle
   reference locations and the dead-code index?**  Both currently
   live in `Codebase` interior-mutable storage and are written from
   the parallel Pass 2.  Salsa accumulators are the natural target,
   but the migration touches every `mark_*_referenced_at` call site.

## Test coverage status

The 745-test fixture suite passes after every PR.  Focused unit tests
exist in `db.rs` for `class_ancestors`,
`class_template_params_via_db`, `method_exists_via_db`,
`method_is_concretely_implemented`, `extends_or_implements_via_db`,
`has_unknown_ancestor_via_db`, and (added in PR38)
`lookup_method_in_chain`, `lookup_property_in_chain`, and
`class_constant_exists_in_chain` (own/ancestor precedence,
mixin walks, mutual-mixin cycles, trait-of-traits, case sensitivity,
inactive-class handling).
