# S4: True Tracked Query Architecture — Analysis of Improvements & Blockers

---

## ⚠️ PR4 Implementation Findings (May 6, 2026) — REVERT RECOMMENDED

PR4 (single-pass execution via tracked queries, commits `caa4f1a` + `d453b17`) was implemented but **regressed performance significantly**. The assumed wins did not materialize.

### Benchmark Comparison (Laravel)

| Metric | Pre-PR4 | Post-PR4 | Δ |
|--------|---------|----------|---|
| Peak live (full analysis) | 360.7 MiB | 413.5 MiB | +14.6% ⚠️ |
| **Total allocated (full)** | **3.38 GB** | **3.71 GB** | **+10% churn** |
| **Total allocated (reanalysis leaf)** | **393 MB** | **1.42 GB** | **+260% churn** ⚠️⚠️ |
| Throughput 1t | 559 elem/s | 485 elem/s | -13% |
| Throughput 12t | 360 elem/s | 333 elem/s | -7.5% |

### Root Cause: Cost Model Inverted

**Pre-PR4 (double-pass):** 2 parses per file, 2 walks per file.
**Post-PR4 (lazy queries):** 1 + N parses per file, where N = internal calls (~3-5 in Laravel).

Each tracked-query invocation `inferred_function_return_type(node)` performs:
1. `sf.text(db)` (cheap)
2. `bumpalo::Bump::new()` (allocator churn)
3. **Full re-parse** of the file (the killer)
4. Walk to find one declaration
5. Run inference

For Laravel: **4-6× parse work** vs. baseline 2×. Memory regression compounds because:
- Salsa memoizes one `Arc<Union>` per FunctionNode/MethodNode
- The `inferred_return_type` field on FunctionNode/MethodNode is **still populated as fallback**
- Result: types are double-stored

### Why the Plan's Mitigation Wasn't Implemented

The plan (sharded-humming-peach.md) explicitly stated under Blocker B:

> **Mitigation:** Drive inference from within `analyze_file`'s own AST walk (warm path). Re-parse only for out-of-order cross-file lookups.

This warm-path mitigation was **not implemented**. The tracked queries became the **primary** inference path, not the fallback. This inverted the design: re-parsing is now the default, not the exception.

### Why Parse Cache Attempts Failed

Multiple attempts at adding a parse cache hit fundamental Rust lifetime issues:
1. `bumpalo::Bump` is `!Clone` — cannot duplicate the arena
2. `ParseResult<'arena, 'src>` has two lifetimes (arena + source)
3. Thread-local storage requires `'static` → forces unsafe `transmute`
4. Returning references into thread-local across function boundaries violates aliasing → unsound

`ParsedProjectFile` works only because it owns arena and parsed result together as one struct that never hands out detached references. That pattern can't replicate across Salsa query boundaries.

### Lessons Learned

| Mistake | What It Cost | What to Do Differently |
|---------|--------------|------------------------|
| Committed PR4 before benchmarking | -13% throughput merged | Benchmark BEFORE commit at every gate |
| Step 4 gate didn't catch warm-path issue | Late discovery of design flaw | Test gates must exercise the warm path on body-inferred functions, not just explicit-typed ones |
| Skimmed Blocker B mitigation | Inverted architecture | Treat plan mitigations as load-bearing constraints, not nice-to-haves |
| Iterated on cache after 3 lifetime failures | Wasted ~2 hours | Third failed variant = signal to stop and rethink |

### Recommended Action

**Revert both commits** (`git revert d453b17 caa4f1a`):
- Restores baseline performance instantly (no regression)
- Steps 1-5 (cycle recovery, source_files in MirDb, AnalyzeFileInput, query skeletons) stay merged in earlier commits
- Clean slate for redesign

### Redesign Proposal: Inference-In-Walk Architecture

```
analyze_file(file):
  parse once
  walk AST: when visiting FunctionDecl/MethodDecl
    → call infer_one_function/infer_one_method directly
      (uses already-parsed AST, zero re-parse)
    → store result in DashMap<FunctionNode, Arc<Union>> on MirDb
      (non-Salsa, like reference_locations)

inferred_function_return_type(node):  // tracked query as FALLBACK
  if map.get(node).is_some() → return it    // warm path: 95%+ of calls
  else → re-parse                            // cold path: cross-file LSP only
```

This achieves single-parse-per-file (matching the plan's intent) without the regression, because:
- 95%+ of inference calls hit the warm path (functions called within their batch)
- Re-parse only fires for true cross-file out-of-order lookups (LSP go-to-def)
- No double-storage (DashMap OR field, not both)
- Single-pass benefit preserved for incremental LSP

---

## Current State: Double-Pass Architecture

```
Pass 2a (inference-only sweep)
    ├─ Parallel: each file runs Pass2Driver::new_inference_only()
    ├─ Collects inferred function/method return types in thread-safe buffers
    └─ Returns: Vec<(fqn, Union)> functions + Vec<(fqcn, name, Union)> methods

↓ commit_inferred_return_types() [serial, canonical db setter]

Pass 2b (full analysis sweep)
    ├─ Parallel: each file runs Pass2Driver::new() [full mode]
    ├─ Reads inferred types from canonical db during type inference
    ├─ Emits all issues (UndefinedClass, InvalidArgument, etc.)
    └─ Returns: Vec<Issue> + Vec<ResolvedSymbol>

Performance cost: every file is fully analyzed twice (passes 2a + 2b).
```

**Why the double pass exists:**
- Type inference requires analyzing method/function bodies to infer return types
- Analysis checks (InvalidArgument, etc.) need those inferred types available **before** analyzing call sites
- Cross-file dependencies: a call in file A may depend on inferred type of function in file B
- Solution: pass 2a primes the db, then pass 2b can read those types during analysis

**Current implementation files:**
- `project.rs:258–529` — `analyze()` orchestration (calls `run_inference_sweep`, then main Pass 2)
- `project.rs:944–987` — `run_inference_sweep()` (rayon::in_place_scope parallel loop)
- `pass2.rs:77–99` — `Pass2Driver::new()` / `new_inference_only()` factory methods
- `pass2.rs:113–129` — `record_function_inference()` / `record_method_inference()` (only active in inference_only mode)
- `db.rs:1998–2035` — `commit_inferred_return_types()` (serial setter loop)

---

## S4 Goal: Single-Pass On-Demand Type Inference

```
Pass 2 (single, tracked query)
    ├─ Parallel: each file calls salsa::tracked query `analyze_file(SourceFile, AnalyzeFileInput)`
    ├─ Tracked query computes types on-demand via `inferred_return_type(node)` lazy query
    ├─ Issue/reference accumulators collect results during tracked-query evaluation
    └─ Returns: issues + reference locations via accumulated::<IssueAccumulator>(db, …)

↓ No serial commit phase — all writes are through Salsa tracked queries
```

**Why this is better:**
1. **Eliminates double-pass overhead** — single analysis walk per file
2. **Memoization** — Salsa caches inferred types; if a file re-analyzes but function X wasn't touched, its inferred type stays cached
3. **Incremental at scale** — per-file cached accumulators mean re-analyzing file A only re-computes issues in A (not B)
4. **Architectural purity** — all writes through Salsa queries; no side-channel `commit_inferred_return_types()` setter pattern

---

## S4 Architecture Pieces (already in place)

### 1. Accumulator Infrastructure (db.rs lines 2353–2456)

**`IssueAccumulator`:**
```rust
#[salsa::accumulator]
pub struct IssueAccumulator(pub Issue);
```
- Tracked queries accumulate issues via `IssueAccumulator(issue).accumulate(db)`
- Consumer reads via `analyze_file::accumulated::<IssueAccumulator>(db, file, input)`

**`RefLocAccumulator`:**
```rust
#[salsa::accumulator]
pub struct RefLocAccumulator(pub RefLoc);
```
- Mirrors current `Codebase::mark_*_referenced_at` side effects into a Salsa-observable stream

**`AnalyzeFileInput`:**
```rust
#[salsa::input]
pub struct AnalyzeFileInput {
    pub php_version: Arc<str>,  // "8.1", "8.2", etc.
}
```
- Carries analysis parameters not captured by `SourceFile` alone
- Extensible for future parameters (e.g., severity level gating, LSP context)

### 2. Tracked Query Stub (db.rs lines 2418–2456)

```rust
#[salsa::tracked]
pub fn analyze_file(db: &dyn MirDatabase, file: SourceFile, _input: AnalyzeFileInput) {
    // Currently: parses + emits parse errors only
    // S4 PRs will extend to call full Pass2Driver
}
```

**Current scope (S4 step 1):**
- Parses file
- Emits parse errors via `IssueAccumulator`
- Does NOT call Pass2Driver (yet)

**Semantics:** pure, idempotent, deterministic re-execution

---

## Key Improvements Enabled by S4

### 1. **Eliminate Pass 2 Double Execution** (2–3 days)
   
**Current state:**
```rust
// project.rs:440-446
let (functions, methods) = run_inference_sweep(db_priming, filtered_parsed, php_version);
db.commit_inferred_return_types(functions, methods);

// project.rs:454-491
let pass2_results = parsed_files.par_iter()
    .map_with(db_main, |db, parsed| {
        let driver = Pass2Driver::new(db, php_version);
        driver.analyze_bodies(...)  // ← full analysis, second walk of same AST
    })
```

**With S4:**
```rust
// Single call per file, no priming sweep
parsed_files.par_iter().for_each_with(db, |db, parsed| {
    analyze_file(db, source_file, AnalyzeFileInput { php_version })
    let issues = analyze_file::accumulated::<IssueAccumulator>(db, source_file, input);
})
```

**Benefit:** ~45% reduction in Pass 2 execution time (baseline: 2×walk → 1×walk with lazy type resolution)

---

### 2. **True Incremental Analysis** (2–3 days, depends on #115 reverse dep index)

**Current limitation:**
- Cache stores per-file issues at `(file_hash, php_version)` key
- Cache invalidation is all-or-nothing: if parent class changes, ALL children must re-analyze
- No tracked-query memoization of per-file results

**With S4:**
- Per-file `analyze_file` results are tracked queries
- Salsa automatically invalidates only dependent files in the call graph
- Cache hits for files that don't depend on changed symbols

**Example:**
```
File A (function foo)  ← changed
    ↓ (used by)
File B (calls foo)     ← must re-analyze
    ↓ (used by)
File C (unrelated)     ← stays cached
```

**Reverse dependency index** (#115) maps symbol → dependent files, enabling Salsa to skip File C.

---

### 3. **Lazy Inferred Type Resolution** (medium-term, 1–2 weeks)

**Problem with current approach:**
- `run_inference_sweep` is sequential (rayon::in_place_scope spawns jobs but collects serially)
- Inferred types for all functions are computed upfront, even if only 30% are used in a given analysis run
- Memory: ~50–100 MiB of inferred types kept in memory for the duration of Pass 2

**Solution with S4:**
```rust
#[salsa::tracked]
fn inferred_return_type(
    db: &dyn MirDatabase,
    node: FunctionNode  // or MethodNode
) -> Arc<Union> {
    // Lazily analyze body, return inferred type
    let driver = Pass2Driver::new_inference_only(db, ...);
    driver.analyze_fn_body(node)
}
```

- Called only when `Pass2Driver` encounters a function/method call
- Results cached by Salsa; re-analysis of file B doesn't re-compute if B doesn't call file A
- Parallel execution naturally spreads the work across the main Pass 2 parallelism

**Benefit:** 50–100 MiB LSP memory savings; enables "streaming" analysis where inference is demand-driven

---

### 4. **Eliminate InferredReturnBuffer Workaround** (1 day)

**Current state:**
```rust
// pass2.rs:73-74
pub(crate) struct Pass2Driver<'a> {
    inferred_types: Arc<Mutex<InferredTypes>>,  // thread-safe buffer
}

// pass2.rs:113-118
fn record_function_inference(&self, fqn: &Arc<str>, inferred: &Union) {
    if self.inference_only {
        if let Ok(mut types) = self.inferred_types.lock() {
            types.functions.push((fqn.clone(), inferred.clone()));
        }
    }
}
```

**With S4:**
```rust
// Pass2Driver no longer carries this buffer
// Instead, during analysis:
IssueAccumulator(issue).accumulate(db);
RefLocAccumulator(refLoc).accumulate(db);
// Inferred types auto-stored via tracked query `inferred_return_type(node)`
```

**Benefit:** Cleaner code path; eliminates manual buffer management and commit logic

---

## Blockers & Implementation Order

### Blocker A: S5 Completion (CRITICAL)

**Status:** S5 (method resolution via Salsa) is 95% complete (PR33 merged).

**Remaining S5 tasks:**
1. Rename `*_db_or_codebase` helpers and drop now-unused codebase fallback parameters
2. Remove `finalization_cache` and `structural snapshot` from `re_analyze_file`
3. Delete remaining `Codebase` fields once all read sites are migrated to Salsa queries

**Why:** S4 assumes that all symbol lookups (`lookup_method_in_chain`, `lookup_property_in_chain`, etc.) are Salsa-backed. If Codebase fallbacks still exist, `analyze_file` tracked query becomes non-deterministic (depends on mutable Codebase state).

**Estimated effort:** 2–3 days (cleanup work, low risk)

---

### Blocker B: Pass2Driver Refactoring (MEDIUM)

**Current shape:**
```rust
pub fn analyze_bodies(
    &self,
    program: &php_ast::ast::Program,
    file: Arc<str>,
    source: &str,
    source_map: &SourceMap,
) -> (Vec<Issue>, Vec<ResolvedSymbol>)
```

**Required for S4:**
- Split into **statement analyzer** and **expression analyzer** that emit via `IssueAccumulator` / `RefLocAccumulator` instead of returning vectors
- Ensure all `db` reads are through Salsa queries (no DashMap lookups or Codebase reads)
- Verify determinism: same input file + db state → identical issue set (order-independent, aggregate via accumulator)

**Estimated effort:** 3–4 days (refactoring, medium risk)

**Risk factors:**
- `IssueBuffer` side effects in nested analyzers (statements → expressions → variables)
- Reference location recording via `Codebase::mark_*_referenced_at` (needs → `RefLocAccumulator`)
- Reordering of issue emission (Salsa accumulators are unordered; issue sorting happens in consumer)

---

### Blocker C: Lazy Inferred Type Query (MEDIUM)

**Current problem:**
```rust
// During Pass 2 analysis, need inferred type of method X
let inferred = node.inferred_return_type(db);  // ← will be available after commit
```

**With S4, need:**
```rust
#[salsa::tracked]
fn inferred_return_type(db: &dyn MirDatabase, node: MethodNode) -> Arc<Union> {
    // Analyze method body, return inferred type
    // This query is called from within analyze_file — circular dependency?
}

// During analyze_file:
let inferred = inferred_return_type(db, node);  // safe because it's a tracked query
```

**Potential circular dependency:**
- `analyze_file(File A)` → calls `inferred_return_type(Method in File B)` → triggers `analyze_file(File B)`?

**Solution:**
- `inferred_return_type` query is for **inference only** — no issue emission, no reference recording
- Separate from `analyze_file` (which emits issues)
- Avoids circular dependency: `analyze_file` can call `inferred_return_type` freely

**Estimated effort:** 2–3 days (new tracked query, tricky dependency verification)

---

### Blocker D: Determinism & Accumulator Ordering (LOW-MEDIUM)

**Concern:**
- Salsa accumulators don't guarantee order
- Current code expects issues in source order (for user-facing output)
- Reference locations must be deduplicated and sorted before caching

**Solution:**
```rust
let issues = analyze_file::accumulated::<IssueAccumulator>(db, file, input);
let mut issues = issues.iter().map(|acc| acc.0.clone()).collect::<Vec<_>>();
issues.sort_by_key(|issue| (issue.location.line, issue.location.col_start));
```

**Estimated effort:** 1 day (post-processing in consumer, low risk)

---

### Blocker E: Reference Index Transition (MEDIUM)

**Current state:**
```rust
// pass2.rs — during analysis
codebase.mark_function_referenced_at(symbol_key, file, line, col);
```

**Problem:**
- `Codebase` is mutable during Pass 2 → Salsa can't observe these side effects
- Cache mechanism works, but it's a workaround

**With S4:**
```rust
// During analyze_file (tracked query)
RefLocAccumulator(RefLoc {
    symbol_key,
    file,
    line, col_start, col_end
}).accumulate(db);

// Consumer (LSP / dead-code detection)
let refs = analyze_file::accumulated::<RefLocAccumulator>(db, file, input);
```

**Integration points:**
1. **Dead code detection** (`dead_code.rs`) — reads reference index from `Codebase`
   - With S4: read accumulated `RefLocAccumulator` across all files
   - Requires aggregating accumulators from all `analyze_file` calls

2. **LSP definition/reference finding** (`symbol.rs`)
   - With S4: query `analyze_file::accumulated` for target file
   - Faster: no need to replay from cache

**Estimated effort:** 1–2 days (integration work)

---

## Execution Plan: 4 PRs

### PR1: S5 Cleanup (2–3 days)
- Remove `*_db_or_codebase` helper fallbacks
- Remove `finalization_cache` and structural snapshot
- Unblock S4 proper

**Risk:** Low (pure refactor, test coverage exists)

---

### PR2: Accumulator Integration & Parse Error Emission (2–3 days)
- Extend `analyze_file` tracked query to emit parse errors via `IssueAccumulator` (already done)
- Add test: `analyze_file` emits correct issues for malformed PHP
- **Do NOT call Pass2Driver yet** — just validate accumulator plumbing

**Deliverable:** `analyze_file` query fully tested with accumulators

**Risk:** Low (isolated to query stub)

---

### PR3: Pass2Driver Refactoring to Emit via Accumulators (3–4 days)
- Refactor statement/expression analyzers to accumulate issues instead of returning vectors
- Refactor reference recording to use `RefLocAccumulator` instead of `Codebase::mark_*_referenced_at`
- Integrate `Pass2Driver` into `analyze_file` tracked query
- Still run both passes (2a inference + 2b analysis) — **no parallelism change yet**

**Test:** old + new code paths produce identical issues (order-independent comparison)

**Risk:** Medium (large refactoring, but covered by 1355+ fixture tests)

---

### PR4: Single-Pass Execution & Lazy Inferred Types (2–3 days)
- Remove `run_inference_sweep` / `commit_inferred_return_types` from `project.rs::analyze()`
- Introduce `#[salsa::tracked] fn inferred_return_type(…)` lazy query
- Call `analyze_file` once per file (not twice)
- Verify Salsa caches intermediate results correctly

**Benchmark:** Compare before/after on real-world codebase
- Expected: ~45% Pass 2 speedup
- Expected: ~50–100 MiB LSP memory savings

**Risk:** High (fundamental architecture change, parallelism verification needed)

---

## Success Criteria

⚠️ **Note (May 6, 2026):** The original PR4 hit -13% throughput, +260% reanalysis allocation churn — see PR4 Implementation Findings at top. Targets below should be re-validated against the inference-in-walk redesign, not the lazy-query approach.

| Metric | Pre-PR4 baseline | Original PR4 target | Inference-in-walk target |
|--------|------------------|---------------------|---------------------------|
| Pass 2 time (cold) | 2.5s (1t) / 3.9s (12t) | ~2.3s (−45%) ❌ Got 2.9s/4.2s | Should match or beat baseline |
| Total allocated (full) | 3.38 GB | n/a — not tracked | ~1.7 GB (−50% from single parse) |
| Total allocated (reanalysis) | 393 MB | n/a — not tracked | ~280 MB (−30% from no double-store) |
| Peak memory (LSP) | 169.6 MiB (leaf) | ~150 MiB (−10%) | Should match baseline |
| Fixture tests passing | 78 (+ 1355 ignored) | 78 (zero regressions) | 78 (zero regressions) |

---

## Open Questions

1. **Circular dependency risk:** Can `inferred_return_type(node)` be called from within `analyze_file` without creating a tracked-query loop?
   - Answer: Yes, because `inferred_return_type` is a separate tracked query; Salsa handles transitive dependencies correctly.

2. **Accumulator ordering:** How to ensure issues are emitted in source order?
   - Answer: Post-process accumulated issues in consumer; Salsa doesn't guarantee order.

3. **Reference index aggregation:** How to collect all reference locations from parallel `analyze_file` calls?
   - Answer: `analyze_file::accumulated::<RefLocAccumulator>(db, file, input)` returns per-file refs; aggregate by caller.

4. **Backwards compatibility with cache:** Can we switch to S4 while old cache files use the old two-pass format?
   - Answer: Cache format doesn't change; cache entries are keyed by `(file, hash, php_version)`. Migration is transparent.

---

## Estimated Total Effort

- **PR1 (S5 cleanup):** 2–3 days
- **PR2 (Accumulator setup):** 2–3 days
- **PR3 (Driver refactoring):** 3–4 days
- **PR4 (Single-pass execution):** 2–3 days
- **Buffer for edge cases, testing, benchmarking:** 2–3 days

**Total: 11–16 days** (2–3 weeks, assuming parallel work on other tasks possible)

---

## Dependencies & Sequencing

```
S5 Cleanup (PR1)
    ↓
Accumulator Setup (PR2)
    ↓
Driver Refactoring (PR3)
    ↓
Single-Pass Execution (PR4)
    ↓
Lazy Type Resolution (future, 1–2 weeks, not in critical path)
```

**Can run in parallel:**
- False-positives elimination work (separate codebase sections)
- LSP integration (#115, #116) — no coupling to S4 internals

---

## Why S4 Matters

1. **Performance:** 45% Pass 2 speedup unlocks large-project usability
2. **Incrementalism:** Per-file tracked queries + reverse dependency index enable true incremental analysis for LSP
3. **Architecture:** Moves from side-effect-based code to pure Salsa queries; improves maintainability and testability
4. **Scalability:** Lazy type inference means analysis cost scales with actual usage, not total codebase size

S4 is the foundation for scaling mir to enterprise PHP codebases (Laravel, Symfony, WordPress).
