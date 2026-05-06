# mir Performance & Work Roadmap

---

## 🚨 PR4 Lessons & Strategic Pivot (May 6, 2026)

### What We Tried
PR4 attempted to migrate Pass 2 from a double-pass architecture (inference sweep + commit + analysis sweep) to a single-pass architecture using Salsa tracked queries (`inferred_function_return_type`, `inferred_method_return_type`) for on-demand inference.

### What Happened (Laravel benchmarks)

| Metric | Pre-PR4 | Post-PR4 | Verdict |
|--------|---------|----------|---------|
| Total allocated (full) | 3.38 GB | 3.71 GB | **+10% churn** ⚠️ |
| Total allocated (reanalysis leaf) | 393 MB | 1.42 GB | **+260% churn** ⚠️⚠️ |
| Throughput 1t | 559 elem/s | 485 elem/s | -13% |
| Throughput 12t | 360 elem/s | 333 elem/s | -7.5% |

**Action: revert `caa4f1a` + `d453b17` immediately.** See `S4_TRACKED_QUERY_ANALYSIS.md` for full root-cause analysis.

### Strategic Findings

#### 1. Total Allocation, Not Peak, Is the Right Metric for mir
Peak is bounded — host machines fit it. **What hurts is total allocation churn**: every alloc is allocator pressure + cache eviction + GC-equivalent free work. PR4 added +10% to full and +260% to reanalysis paths — that's the actual regression.

#### 2. Salsa Has Diminishing Returns for mir's Primary Use Case

**For batch CLI analysis (~95% of usage):**
- Every Salsa tracked query runs **exactly once** per process
- Memoization storage is allocated, populated, never re-read, then dropped
- This is pure overhead with no benefit

**For LSP (incremental):**
- File-level invalidation is sufficient (PHP edits are file-granular)
- Per-function/method memoization adds memory without proportional benefit

**Where Salsa earns its keep:**
| Use | Verdict |
|-----|---------|
| `class_ancestors` (cycle-safe inheritance) | ✅ Worth it — recursive, cycle handling is hard otherwise |
| `FunctionNode`/`ClassNode` as inputs | ✅ Worth it — stable identity for map keys |
| `collect_file_definitions` (Pass 1) | 🟡 Marginal — could be a plain function with content-hash cache |
| `analyze_file` (Pass 2 tracked) | ❌ Net negative — runs once, accumulator allocation per push |
| `inferred_function_return_type` (PR4) | ❌❌ Net very negative — forced re-parse + double-storage |

#### 3. The Biggest Lever Is Parse Count

Each PHP parse allocates a fresh `bumpalo::Bump` (~1-2 MB). The single biggest lever for total allocation:

| Architecture | Parses per file | Laravel parse churn (1500 files) |
|--------------|-----------------|-----------------------------------|
| Pre-PR4 (double-pass) | 2 | ~3 GB |
| Post-PR4 (lazy queries) | 1 + N (~4-6) | ~6-12 GB |
| Inference-in-walk target | 1 | ~1.5 GB |

#### 4. Why Parse Cache Attempts Failed
Rust lifetime constraints make a global parse cache impractical:
- `bumpalo::Bump` is `!Clone`
- `ParseResult<'arena, 'src>` has two lifetimes
- Thread-local storage requires `'static` → forces unsafe `transmute`
- Returning references into thread-local across function boundaries is unsound

`ParsedProjectFile` works only because it owns arena+parsed together and never hands out detached references — that pattern can't span Salsa query boundaries.

---

## Top Priorities for Total Allocation + Throughput (Post-Revert)

### 🥇 #1: Revert PR4
- `git revert d453b17 caa4f1a`
- Recovers baseline immediately
- Steps 1-5 (cycle recovery, registries, query skeletons) stay merged in earlier commits

### 🥈 #2: Inference-In-Walk Architecture (replaces PR4)
**Goal:** 1 parse per file (was 2× pre-PR4, 4-6× post-PR4 broken)

**Design:**
```
analyze_file(file):
  parse once
  walk AST: when visiting FunctionDecl/MethodDecl
    → call infer_one_function/infer_one_method with already-parsed AST
    → store in DashMap<FunctionNode, Arc<Union>> on MirDb (non-Salsa)

inferred_function_return_type(node):  // tracked query, FALLBACK ONLY
  if map.get(node) → return it       // warm path: 95%+ of calls
  else → re-parse                    // cold path: cross-file LSP queries only
```

**Expected impact:**
- Total allocation (full): **3.38 GB → ~1.7 GB (−50%)**
- Total allocation (reanalysis): **393 MB → ~280 MB (−30%)**
- Throughput: **+15–25%** (one parse + walk vs. two)

### 🥉 #3: Demote `analyze_file` from Tracked Query to Plain Function
**Why:** `analyze_file` runs exactly once per file per analysis. Salsa memoization is paid but never reused.

**Current overhead:**
- Per-query memoization storage (one entry per `(file, input)`)
- Accumulator allocations per `IssueAccumulator(issue).accumulate(db)` push (millions per Laravel run)
- Dependency edge tracking

**Replacement:**
```rust
fn analyze_file(db, file, input) -> (Vec<Issue>, Vec<Symbol>, Vec<RefLoc>)
```
Direct return values. Issues, symbols, ref-locs are file-scoped outputs, not shared dependencies — they don't need Salsa.

**Expected impact:** 5–10% total allocation reduction, lower per-query latency.

### #4: Remove `inferred_return_type` Field from FunctionNode/MethodNode
After #2 lands, the field is no longer needed (DashMap holds the values during analysis, dropped at end). Eliminates double-storage. Touches Salsa input definitions and 8 upsert paths.

### #5: Audit Other Salsa Tracked Queries for Run-Once Pattern
Any tracked query that runs exactly once per process is a candidate for demotion. Specifically check:
- `collect_file_definitions` (Pass 1)
- Any `*_via_db` helper that delegates to a single tracked query

### #6: Existing P0 Wins (still valid post-revert)
- **(*t).clone() at call sites** (`call/function.rs:42`, `call/method.rs:52`) — 20–50 MiB LSP, 1–2d effort
- **Method signature sharing** (param list dedup) — 100–150 MiB cold start, 2–3d effort
- **Lazy type resolution** (vendor) — 300–500 MiB cold start, 3–4d effort

---

## Recommended Sequence (Post-Revert)

```
Step 1: Revert PR4 (1 commit)               → restore baseline
Step 2: Inference-in-walk (separate PR)     → -50% total alloc full, -30% reanalysis
Step 3: Demote analyze_file (separate PR)   → -5–10% additional
Step 4: Drop inferred_return_type field     → cleanup, small gain
Step 5: Existing P0 perf items (#1–3)       → 50–70 MiB LSP improvement
```

Each step is independently measurable, reversible, and has its own benchmark gate.

### Process Improvements for Future Perf Work

1. **Benchmark gate at every step**, not just at the end. If a step regresses >5%, halt and reassess before proceeding.
2. **Plan mitigations are load-bearing.** When a plan calls out a risk and proposes a mitigation, treat the mitigation as required, not optional.
3. **Test the warm path explicitly.** A gate like "explicit return type bypasses inference" doesn't validate the body-inference path.
4. **Stop after 3 failed variants.** If a third attempt at a workaround fails the same way, the approach is wrong — rethink, don't iterate.

---

## Current Measurements (May 5, 2026)

| Scenario | Peak live | Total allocated |
|---|---|---|
| Full analysis (cold start) | 388.6 MiB | 3318.8 MiB |
| Vendor collection only | ~205 MiB | 2309.3 MiB |
| LSP re-analysis — high-fanout file | 198.8 MiB | 801.1 MiB |
| LSP re-analysis — leaf file | 174.3 MiB | 383.6 MiB |

Benchmark: `cargo bench --bench analyze_real_world -- bench_full_analysis_detailed`  
LSP target: `reanalysis_project_only` (measures only `analyze()` with vendor pre-loaded).

---

## Open GitHub Issues by Category (May 5, 2026)

### 🏗️ Architecture & Migration (Phase A: Blocking other work)

| # | Title | Priority | Dependencies |
|---|---|---|---|
| 115 | Build reverse dependency index for file invalidation | Critical | — |
| 116 | Incremental re-analysis for a single changed file | Critical | Requires #115 |

**Status:** #115 and #116 unlock core incremental analysis. Foundational for LSP editor integration.

**Deferred to LSP phase (not core mir responsibility):**
- **#112** Expose definition location lookup by SymbolKind — mir provides `symbol_location()` and `member_location()`; LSP implements dispatch
- **#114** Add related locations to Issue — LSP diagnostic enhancement; defer until LSP integration phase

---

### 🏗️ Architecture & Migration (Future phases)

*All future-phase architecture work is blocked on Phase A (#115, #116) completion.*

---

### 🔍 Missing Analysis Checks (24 open issues)

#### Implemented issues in need of emission (easy wins)
- **#125** `MissingReturnType` — check in collector; gate by severity level
- **#126** `MissingParamType` — check in collector; skip magic methods
- **#137** `MissingThrowsDocblock` — collect declared `@throws`, check call graph
- **#141** `UnnecessaryVarAnnotation` — compare `@var` type vs inferred type

#### Type-related checks (medium effort)
- **#117** `NullableReturnStatement` — narrow null-vs-non-nullable case
- **#119** `MismatchingDocblockReturnType` / `MismatchingDocblockParamType` — compare docblock vs native hint
- **#118** `InvalidPropertyAssignment` — type check property writes (like InvalidArgument)
- **#122** `TypeDoesNotContainType` — intersection of === operands is empty
- **#127** `MixedArgument`, `MixedAssignment`, `MixedPropertyFetch` — emit when mixed propagates (gate by level)

#### Casting & coercion checks (medium effort)
- **#133** `InvalidCast` — object/array to scalar casts
- **#135** `RedundantCast` — cast to already-known type
- **#134** `InvalidOperand` — incompatible operator usage (arithmetic on arrays, concat on non-strings)
- ~~**#140** `ImplicitToStringCast`~~ — ✅ Completed
- **#11** Implicit type coercions — float→int loss, string→bool

#### Array checks (low-medium effort)
- **#120** `NonExistentArrayOffset` — literal keys not in array shape
- **#121** `InvalidArrayOffset` — non-int/string array key types
- **#138** `DuplicateArrayKey` — literal key appears twice

#### Enum & match checks (low effort)
- **#129** `MatchNotExhaustive` — match on enum missing cases
- **#143** `PropertyNotSetInConstructor` — non-nullable typed property not initialized in constructor

#### Narrowing gaps (medium effort)
- **#36** Missing type guard functions — `array_key_exists`, `is_scalar`, `is_iterable`, `is_countable`, `is_numeric`, `ctype_*`, `str_contains`
- **#145** Nullsafe call narrowing — `$x?->method() !== null` narrows `$x` to non-null

#### Visibility & API boundaries (low-medium effort)
- **#9** Visibility violations (private/protected) — already mentioned as check #9
- **#136** `InternalMethod` — `@internal` method calls across boundaries
- **#123** `UnusedFunction` — extend dead-code pass to top-level functions
- **#130** / **#131** `MissingPropertyType` — properties without type hints
- **#54** `UnusedClass` — classes never instantiated or referenced
- **#55** Respect `@psalm-api` / `@api` — exclude from dead-code (gate to visibility & api checks)

#### Taint checks (already partially implemented)
- **#128** `TaintedInput` — generic taint source entry point (completes taint source-to-sink chain)
- **#33** Sanitizer functions — `htmlspecialchars()`, `intval()`, `escapeshellarg()` untaint values
- **#56** Inter-procedural data flow — DataFlowGraph for taint (foundation for #33, #57)
- **#57** `@psalm-taint-*` annotations — custom sources, sinks, sanitizers

**Summary:** ~24 checks defined but not emitted. Clustering into 4–5 small PRs per category (checks + tests) would cover most of this work.

---

### 💡 Type Inference Improvements (4 open issues)

| # | Title | Effort | Status |
|---|---|---|---|
| **35** | Infer TClosure from arrow functions / anonymous functions | 1–2d | Unlocks array_map/filter inference |
| **34** | Superglobal arrays with precise types | 4h | String/int key shapes per superglobal |
| **29** | `@var` type params threaded to variable TypeEnv | 2d | Requires #26 (template subst at call sites) |
| **28** | Static method calls missing template substitution | 1–2d | Mirror function call pattern |

**Note:** #29 and #28 depend on completing #26 (receiver type params at call sites, not listed as open here — may be completed or in progress).

---

### 🐛 Known Analysis Bugs (1 open)

| # | Title | Effort | Impact |
|---|---|---|---|
| **30** | Division by zero / modulo by zero detection | 4h | Low — rare in practice |

**Completed:**
- ~~**#37** Loop widening should use union instead of mixed~~ — ✅ Moderate impact; reduces false negatives in loops
- ~~**#152** Generic template narrowing through `instanceof`~~ — ✅ High impact; unlocks conditional return types

---

### 📚 Documentation (3 open)

| # | Title | Priority |
|---|---|---|
| **14** | Add PHP code examples to every issue kind | High |
| **17** | Restructure SUMMARY.md into sections | Low |
| **18** | Add custom theme with logo | Low |

---

### 🔌 Extensibility & Configuration (5 open)

| # | Title | Priority | Notes |
|---|---|---|---|
| **59** | Parse `.phpstub` files for custom type definitions | Medium | Allows users to add types without Rust |
| **58** | Plugin system (custom rules & type resolvers) | Low | Deferred (complex API surface) |
| **55** | Respect `@psalm-api` / `@api` annotation | Medium | Gating for dead-code and public API |
| **12** | Gate diagnostics by PHP version | Medium | Union types, enums, match, readonly, DNF |
| **52** | (not in list) Find-dead-code CLI flag | — | Related to #54, #55 |

---

### 🎯 Recent Completions (as of May 6, 2026)

- **v0.17.3** (commit 13a1839): Deduplication wins across perf work
  - PR1–PR33: Salsa S5 method resolution finalization
  - PR31: enum implicit-method synthesis
  - PR32/PR33: drop codebase.get_method() fallbacks
- **#140** (commit d3ec173): `ImplicitToStringCast` emission — objects without `__toString` in string context
- **#152** (commit 36095c4): Generic template type narrowing through `instanceof` check
- Cumulative: **−28% peak, −4.5% total** from original baseline

---

## Priority Clarification

**README states:** "Reduce `UndefinedMethod` / `InvalidArgument` false positives" is the #1 next priority.

**This roadmap** (MEMORY_OPTIMIZATION_ROADMAP) focuses on **performance & architecture**.

**New roadmap** (FALSE_POSITIVES_ROADMAP.md) focuses on **eliminating 1,356 ignored tests** that represent user-facing false-positives.

**Execution strategy:** Both roadmaps run in parallel:
- **Phase 0 (blocking):** #115 / #116 (reverse dependency index + incremental re-analysis) — required for both roadmaps
- **Phases 1–2 (P1):** FALSE_POSITIVES work (callable validation, inheritance, read tracking) can run alongside performance work
- **Phase B onwards:** Perf work can continue independently

See FALSE_POSITIVES_ROADMAP.md for detailed false-positive elimination phases and PR breakdown.

---

## Next Priorities

### 1. Eliminate `(*t).clone()` at call sites — **20–50 MiB LSP savings** (estimated) [P0]

**Where:** `call/function.rs:42` and `call/method.rs:52`

```rust
let return_ty_raw = node.return_type(db)
    .or(inferred)
    .map(|t| (*t).clone())   // ← clones Union out of Arc on every call site
    .unwrap_or_else(Union::mixed);
```

Every function/method call during Pass 2 unconditionally clones the return `Union` out of its `Arc`. For the 90%+ of non-generic calls, that `Union` is never modified — yet always cloned.

**Fix:** Keep `return_ty_raw: Arc<Union>` in `ResolvedFn` / `ResolvedMethod`. Only clone when template substitution is actually needed (check `template_params.is_empty()`). The final `return_ty` handed back to the expression analyzer would still be an owned `Union`, but the clone is deferred to the rare generic case.

**Scope:** `call/function.rs`, `call/method.rs`, and the downstream consumers of `return_ty_raw` in each.  
**Risk:** Medium — touches return type flow through `ResolvedFn`, `ResolvedMethod`.  
**Effort:** 1–2 days.

---

### 2. Method signature sharing — **100–150 MiB cold-start savings** (estimated)

Many PHP framework methods share identical parameter lists (`(string $arg, array $opts)` appears thousands of times). Currently each gets its own `Arc<[FnParam]>` allocation.

**Fix:** After vendor collection, hash each method's param list, deduplicate into a shared `Arc<[FnParam]>`, replace per-method copies with a pointer into the shared table.

**Scope:** `crates/mir-codebase/src/storage.rs`, `crates/mir-analyzer/src/db.rs`, `project.rs`.  
**Risk:** Low — isolated to the collection phase, doesn't affect analysis.  
**Effort:** 2–3 days.

---

### 3. Lazy type resolution — **300–500 MiB cold-start savings** (estimated)

During vendor collection, full `Union` types are resolved for all 25k vendor classes × 6–8 methods, even though only 5–10% are referenced by a typical project. The 2.3 GiB vendor collection cost is dominated by this work.

**Fix:** Store type hints as a lightweight `TypeHint` enum during collection (`Named(Arc<str>)`, `Union(Vec<Arc<str>>)`, `Docblock(Arc<str>)`). Resolve to `Union` lazily via a Salsa query on first access.

**Scope:** `storage.rs` (MethodStorage, FunctionStorage), `collector/mod.rs`, `db.rs` (new Salsa query), 20+ call sites.  
**Risk:** Medium — touches the entire parameter/return type access surface.  
**Effort:** 3–4 days.

---

### 4. Analyzer gaps (functional, not memory)

**G2 — Post-Pass-2 FQCN lazy loading**  
Fully-qualified class names referenced without a `use` import (e.g. `new \Foo\Bar\Baz()`) are never lazy-loaded because the trigger runs before Pass 2 completes. Test: `tests/lazy_load.rs:227` (currently `#[ignore]`).  
Fix: add a post-Pass-2 sweep that collects still-missing FQCNs and re-runs loading.

**G4 — Param contravariance for named objects in override checks**  
A child method that illegally narrows a param from `Animal` to `Cat` is not flagged. The contravariance loop in `class.rs:417` skips pairs where either side contains a named object.  
Fix: use the inheritance graph (`all_parents`, `all_interfaces`) to check subtype direction, mirroring `named_object_return_compatible`.

**G5 — Non-object types in `named_object_return_compatible`**  
Union types mixing objects with scalars (e.g. `string|MyClass`) may produce false negatives. The function falls through to a simple check for non-object atomics.  
Fix: split the union — object atoms go through the inheritance path, scalar atoms through the simple subtype check.

---

### 5. M20 — Plugin system

Not started.

---

## Completed Work

| Optimization | Impact | Notes |
|---|---|---|
| Avoid Salsa cache for vendor collection | **−25.6% peak** (547 → 407 MiB) | Biggest single win |
| Arc<Union> param type interning | −2.0% total | OnceLock for 7 common scalars |
| Arc<Union> return type interning | −0.56% total | Same mechanism; low yield — 70% of vendor methods have no explicit return type |
| FnParam::default → has_default bool | **−4.1% peak, −1.8% total** | 208 → 32 bytes per FnParam; 205k params × 176 bytes freed |
| SimpleType enum for callable params | −0.08% total | Low yield — callables are rare in vendor code |
| Deduplicate resolve_fn per call site | −1.0 MiB LSP re-analysis | analyze_function_call was calling resolve_fn twice; now once |

**Cumulative: −28% peak, −4.5% total from original baseline.**

---

## What Didn't Work

**Global string interning (reverted, commit d76d6af):**  
DashMap contention under parallel Pass 2 caused an 18% multi-threaded perf regression with <1% memory savings. Any string interning must be confined to the sequential ingestion phase to avoid this.

**Interning inferred return types at commit time:**  
`commit_inferred_return_types` runs *after* Pass 2 body analysis. The clone budget is already spent by then. Saving <0.2 MiB here doesn't move the needle.

**Extending Arc<Union> interning to all types via DashMap:**  
Scalars are already interned. Class types appear in too many distinct forms to deduplicate meaningfully with a global map. The hot clones happen at call sites during Pass 2, not at collection time.

---

## Sequencing & Dependencies

### Phase A: Unblock LSP (Week of May 5)
1. **#115** Reverse dependency index — enables incremental analysis plumbing
2. **#116** Incremental re-analysis — tie it to #115; orchestrate via reverse index

*Effort:* ~1–2 weeks  
*Payoff:* Core incremental analysis ready; LSP can build on solid foundation

**Removed from Phase A (LSP concerns, not mir core):**
- ~~#112 Definition lookup~~ → LSP implements the SymbolKind dispatch using mir's existing APIs
- ~~#114 Related locations~~ → LSP diagnostic enhancement; defer to LSP integration phase

### Phase B: Performance Stabilization (Week of May 12)
1. **P0 perf** (#1–3 above) — target 50–70 MiB LSP improvement
2. Benchmark against real-world codebases (Laravel, Symfony, WordPress)

*Effort:* ~1–2 weeks  
*Payoff:* Usable performance for large projects

### Phase C: Analysis Completeness (Week of May 19+)
1. **Quick wins:** Emit-but-not-implemented checks (#125, #126, #137, #141) — batch into 1–2 PRs
2. **Type inference:** #35 (TClosure), #34 (superglobals), #28 (static methods)
3. **Narrowing:** #36 (type guards), #145 (nullsafe narrowing)
4. **Generic:** #152 (instanceof narrowing for templates)

*Effort:* ~2–3 weeks for ~15 checks + inference fixes  
*Payoff:* Dramatically higher diagnostic coverage on real code

### Phase D: Configuration & Stubs (June)
1. **#59** `.phpstub` file parsing — enables custom type definitions
2. **#12** PHP version gating — configure min version, flag version-specific syntax
3. **#55** `@psalm-api` gating — dead-code respects intent

*Effort:* ~1 week  
*Payoff:* More professional UX, configuration parity with Psalm

### Phase E: Docs & Polish (June+)
1. **#14** Examples for every issue kind
2. **#18** Custom theme with logo
3. Issue kind pages with links to Psalm references

*Effort:* ~4–5 days  
*Payoff:* Professional first impression; users understand what each check does

---

## Batch Suggestions for Issue PRs

### Batch 1: Missing-Type Checks (P1)
#125, #126 → 1 PR — collect once, emit MissingReturnType / MissingParamType with severity gating

### Batch 2: Docblock Consistency (P2)
#119, #141, #137 → 1 PR — docblock parsing already complete; add checks for mismatch / unnecessary / missing throws

### Batch 3: Null & Optionality (P2)
#117, #145 → 1 PR — nullable return statements + nullsafe call narrowing

### Batch 4: Type Safety (P2)
#118, #122, #127 → 1 PR — property type checks, type intersection, mixed propagation (all type-system gating)

### Batch 5: Casts & Operators (P3)
#133, #134, #135, #140 → 1 PR — cast validity, operator checks, implicit coercions

### Batch 6: Arrays (P3)
#120, #121, #138 → 1 PR — array offset validity, duplicate keys, shape exhaustiveness

### Batch 7: Dead Code & API Boundaries (P3)
#123, #129, #130, #131, #143, #54, #55, #136 → 2 PRs — extend dead-code pass, property initialization, visibility, @api gating

**Total:** ~8 PRs covering 24 check emissions — realistic over 3–4 weeks with concurrent perf work.

---

## Benchmark Guide

Always compare the same benchmark before and after a change. Use `git stash` to get a clean baseline.

- **LSP hot path:** `reanalysis_project_only/laravel_high_fanout` and `laravel_leaf_file`
- **Cold start:** `full_analysis/laravel`
- **Vendor collection only:** `vendor_collection/laravel`

```bash
# Before
git stash
cargo bench --bench analyze_real_world -- bench_full_analysis_detailed 2>&1 | grep "\[memory\]"
git stash pop

# After
cargo bench --bench analyze_real_world -- bench_full_analysis_detailed 2>&1 | grep "\[memory\]"
```
