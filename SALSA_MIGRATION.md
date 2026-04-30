# Salsa migration — current status & open questions

## Shipped through PR33

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

## Architectural blocker uncovered this session

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

This is **the** blocker for finishing S5.  Until it's resolved, every
remaining S5 PR that needs to mirror Codebase state into Salsa-tracked
fields after a parallel pass will hit the same deadlock.

## Resolution candidates (each requires a design decision)

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

## Method-resolution status: codebase-free

All `Codebase::get_method`-style fallbacks in method/constructor
resolution paths are gone (PR27/PR28/PR29/PR30/PR31/PR32/PR33).
`db::lookup_method_in_chain` is the single canonical walker, and it
is at full parity with `Codebase::get_method`'s order: own → mixins
(recursive) → traits (transitive) → ancestors (with each ancestor's
own + traits + mixins).

The only remaining `Codebase::get_method` reads in the analyzer are:
- `call/method.rs:51` — fetches `inferred_return_type` from
  `MethodStorage`.  Stays until S3 promotes the field to a tracked
  query.

Properties and class constants still have a few `Codebase::get_property`
/ `get_class_constant` sites in `expr.rs` (lines 830, 1317, 1473).
Those are the next surface to migrate; same pattern (chain helper,
mixin walk if needed for properties).

## Other S5 work remaining

- Migrate the residual `Codebase::get_method` / `get_property` /
  `get_class_constant` reads in `expr.rs:830` (constant), `expr.rs:1317`
  (property), `expr.rs:1473` (readonly fallback — different shape, on
  class storage).  Each needs a `lookup_*_in_chain` helper similar to
  the method one PR28 added; each will keep a codebase fallback until
  the lazy-load hook lands.
- Remove `Codebase::ensure_finalized` and the `finalization_cache`
  once no read site reaches them.  Gated on the per-field migrations
  finishing.
- Remove the structural-snapshot fallback in `re_analyze_file`'s
  cold path.  Same gating.
- The `lazy_load_from_body_issues` post-Pass-2 sweep still uses
  `Pass2Driver::new` (full pass, including issue emission and
  reference tracking) on lazy-loaded files, then reanalyzes the
  triggering files.  Confirm the inferred return types written here
  reach the main pass's reads — if option 3 above is taken, this
  is automatic.  If options 1/2/4 are taken, a sync call is needed
  here too.

## Open questions for the next session

1. **Which resolution candidate above does the project want?**
   The choice cascades into the next 5–8 PRs.

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

The 745-test fixture suite passes after every PR.  No new tests were
added for the db helpers themselves beyond the existing `db.rs` unit
tests for `class_ancestors`, `class_template_params_via_db`, etc.
Adding focused unit tests for `method_exists_via_db`,
`method_is_concretely_implemented`, and
`extends_or_implements_via_db` would harden the migration further;
they're tested transitively through fixtures today.
