# mir False-Positives Elimination Roadmap

**Rationale:** mir's README states "Reduce `UndefinedMethod` / `InvalidArgument` false positives" as the #1 next priority. Current state: **1,356 ignored tests** spanning 24 categories. This roadmap breaks down the work into phases and specific PRs to systematically eliminate the highest-impact false-positives.

---

## False-Positive Inventory (Current, May 6, 2026)

| Issue Type | Count | Root Cause | Est. Impact |
|---|---|---|---|
| **invalid_argument** | 101 | Callable/builtin param types, array callback validation | High |
| **unused_variable** | 75 | Nullsafe operator tracking, closure captures | High |
| **undefined_class** | 73 | Cross-file imports, namespace resolution | High |
| **undefined_method** | 48 | Multi-level inheritance, @method docblocks | High |
| **stub_behavior** | 38 | Built-in function signatures | Medium |
| **method_signature_mismatch** | 35 | Covariance/contravariance, docblock merging | Medium |
| **possibly_invalid_argument** | 32 | Union narrowing in conditionals | Medium |
| **invalid_docblock** | 29 | @var/@return/@throws parsing gaps | Medium |
| **undefined_function** | 28 | Dynamic namespace, class-string callables | Medium |
| **invalid_return_type** | 25 | Generic return type substitution | Medium |
| **invalid_return_statement** | 24 | Nullable/mixed return narrowing | Medium |
| **Other (12 categories)** | 308 | Various (redundant conditions, dead code, etc.) | Low–Medium |

---

## Execution Strategy

### Phase 0: Foundation (Blocking Work) — **Critical, start now**

Before reducing false-positives, two architectural changes are required:

1. **#115 / #116** (from MEMORY_OPTIMIZATION_ROADMAP) — Reverse dependency index + incremental re-analysis
   - Needed so test fixtures can be run in isolation without relying on cross-file inference state
   - **Effort:** ~1–2 weeks
   - **Blocker:** Until this lands, multi-file fixture tests are unreliable

2. **Salsa S3 completion** (from MEMORY_OPTIMIZATION_ROADMAP) — Resolve deadlock in `inferred_return_type` promotion
   - Needed for accurate inferred return types in method resolution
   - **Effort:** ~3–5 days (rayon::scope fix attempt)
   - **Impact:** Unlocks 15–20 false-positive fixes in method call checking

**Decision:** Start #115 / #116 in parallel with S3 deadlock fix. Both unblock false-positive work.

---

### Phase 1: High-Impact Quick Wins — **P1, 2–3 weeks**

**Goal:** Eliminate ~150 false-positives (invalid_argument + undefined_method)

#### 1a. Callable & Array Callback Parameter Validation
**Issues:** 101 invalid_argument cases (array_map, array_reduce, array_filter, usort callbacks)

**Root cause:** Callback type signatures not validated against expected closure signatures.

**Fix strategy:**
1. Enhance `CallableType` to store param/return type metadata from docblocks & Closures
2. When analyzing `array_map($fn, $array)`, resolve `$fn` type and validate:
   - Closure param count matches expected
   - Param types are assignable from array value type
3. Cache resolved callables in Salsa to avoid re-inference

**Scope:** `crates/mir-analyzer/src/call/callable.rs` (new), update `call/function.rs` for builtin handling

**Test coverage:** 35–40 phpt fixtures in `invalid_argument/` directory

**Effort:** 2–3 days | **PR:** `fix(false-pos): callable parameter validation for array_*`

---

#### 1b. Multi-Level Inheritance Method Resolution
**Issues:** 48 undefined_method cases (cross-file inheritance, 3+ levels deep)

**Root cause:** Method lookup stops at immediate parent; doesn't walk `GrandParent → Parent → Child` chain.

**Fix strategy:**
1. Implement `lookup_method_in_chain` in Salsa (mirrors existing `db::lookup_method_in_chain` but returns all ancestors)
2. Walk full inheritance chain in `resolve_method_from_db`
3. Add test cases for `GrandParent.php → Parent.php → Child.php` three-level inheritance

**Scope:** `crates/mir-analyzer/src/db.rs` (new Salsa query), `crates/mir-analyzer/src/call/method.rs`

**Test coverage:** 15–20 phpt fixtures in `undefined_method/cross_file_*`

**Effort:** 1–2 days | **PR:** `fix(false-pos): resolve methods through full inheritance chain`

---

#### 1c. Docblock @method / @var Pseudo-Method Resolution
**Issues:** Part of invalid_argument (101) + undefined_method (48)

**Root cause:** `@method void setName(string $name)` annotations parsed but not used during call checking.

**Fix strategy:**
1. During `collect_file_definitions`, extract `@method` docblock entries and register as synthetic methods on the class
2. Store as pseudo-MethodNode in Salsa (active=true, synthetic=true, visibility=public)
3. During method call resolution, include synthetic methods in lookup
4. Validate `@method` parameter types during InvalidArgument checks

**Scope:** `crates/mir-analyzer/src/collector/class.rs`, `db.rs` (synthetic method registration)

**Test coverage:** 10–15 phpt fixtures in `invalid_argument/annotation_*` and `undefined_method/docblock_*`

**Effort:** 2 days | **PR:** `feat(false-pos): support @method docblock pseudo-methods`

---

### Phase 2: Variable Read Tracking — **P2, 2–3 weeks**

**Goal:** Eliminate ~75 false-positives (unused_variable)

#### 2a. Nullsafe Operator Call Tracking
**Issues:** 75 unused_variable cases (variables passed to nullsafe method calls `$obj?->method($var)`)

**Root cause:** Nullsafe operator (`?->`) skips argument read tracking.

**Fix strategy:**
1. Treat `$obj?->method($args)` same as `$obj->method($args)` for argument read tracking
2. Mark each argument variable as "read" even if the call might not execute
3. Add assertion tracking for `$obj?->method() !== null` to narrow type

**Scope:** `crates/mir-analyzer/src/statements/stmt.rs` (nullsafe call handling)

**Test coverage:** 20–25 phpt fixtures in `unused_variable/nullsafe_*`

**Effort:** 1 day | **PR:** `fix(false-pos): track variable reads in nullsafe method calls`

---

#### 2b. Closure Capture Read Tracking
**Issues:** Part of unused_variable (75)

**Root cause:** Variables captured in closures passed to `array_map`, `array_filter`, `usort` callbacks marked as unused.

**Fix strategy:**
1. Enhance closure analysis to mark captured variables as "read"
2. When analyzing callback arguments, walk closure body and mark all captured variables
3. Special handling for `use($var)` explicit captures vs implicit captures

**Scope:** `crates/mir-analyzer/src/expr.rs` (closure handling), `statements/stmt.rs` (use clause)

**Test coverage:** 15–20 phpt fixtures in `unused_variable/closure_*`

**Effort:** 1–2 days | **PR:** `fix(false-pos): mark variables read when captured in closures`

---

#### 2c. Assignment-Side Read Tracking
**Issues:** Part of unused_variable (75) (variables on RHS of assignment)

**Root cause:** `$x = $y` marks `$y` as read only sometimes; inconsistent across statement types.

**Fix strategy:**
1. Audit all assignment statement handlers to ensure RHS variables are marked as read
2. Special handling for reference assignments (`$x =& $y`)
3. Add tests for complex assignment patterns (destructuring, foreach with reference)

**Scope:** `crates/mir-analyzer/src/statements/stmt.rs` (assignment handling)

**Test coverage:** 10–15 phpt fixtures in `unused_variable/assignment_*`

**Effort:** 1 day | **PR:** `fix(false-pos): consistently track reads on assignment RHS`

---

### Phase 3: Cross-File Resolution — **P3, 3–4 weeks**

**Goal:** Eliminate ~100 false-positives (undefined_class + namespace issues)

#### 3a. Lazy-Load FQCN Post-Pass-2
**Issues:** 73 undefined_class cases (fully-qualified names not loaded before Pass 2 finishes)

**Root cause:** Class loader triggered before Pass 2 complete; `new \Foo\Bar\Baz()` references aren't in the graph yet.

**Fix strategy:**
1. After Pass 2 completes, sweep for all unresolved class references
2. Trigger lazy-load for any FQCN not yet in the database
3. Re-analyze only affected files to surface newly-resolved methods/properties

**Scope:** `crates/mir-analyzer/src/project.rs` (post-Pass-2 hook)

**Test coverage:** Existing ignored test `tests/lazy_load.rs:227`

**Effort:** 2 days | **PR:** `fix(false-pos): lazy-load FQCNs after Pass 2`

---

#### 3b. Namespace Import Resolution
**Issues:** Part of undefined_class (73) (`use App\Foo; new Foo()` misresolves)

**Root cause:** `use` aliases not consistently applied during type resolution.

**Fix strategy:**
1. During `collect_file_definitions`, store each file's import namespace in Salsa
2. When resolving a bare class name, check file imports before falling back to current namespace
3. Add test for cross-file `use` alias resolution

**Scope:** `crates/mir-analyzer/src/db.rs` (file imports storage), `crates/mir-analyzer/src/types.rs` (resolution)

**Test coverage:** 10–15 phpt fixtures in `undefined_class/namespace_*`

**Effort:** 1–2 days | **PR:** `fix(false-pos): apply namespace imports during type resolution`

---

#### 3c. Composite: Class String Callables
**Issues:** 28 undefined_function cases (`call_user_func('ClassName::method')`)

**Root cause:** String callables like `'ClassName::staticMethod'` and `'ClassName::method'` not parsed into method references.

**Fix strategy:**
1. When analyzing `call_user_func($callable)` where `$callable` is a string literal, parse as `ClassName::method`
2. Resolve `ClassName` via current namespace + imports
3. Mark as method reference to prevent dead-code false-positives

**Scope:** `crates/mir-analyzer/src/call/function.rs` (call_user_func handling)

**Test coverage:** 5–8 phpt fixtures in `undefined_function/call_user_func_*`

**Effort:** 1 day | **PR:** `fix(false-pos): parse string callables as method references`

---

### Phase 4: Built-In Function Stubs & Docblock Consistency — **P4, 2–3 weeks**

**Goal:** Eliminate ~70 false-positives (stub_behavior + invalid_docblock + method_signature_mismatch)

#### 4a. Enhance Built-In Stub Accuracy
**Issues:** 38 stub_behavior cases (variadic params, byref params, type narrowing)

**Root cause:** JetBrains phpstorm-stubs don't capture all nuances (e.g., `sscanf()` output vars are byref).

**Fix strategy:**
1. Audit the 10 most common builtin functions flagged in ignored tests
2. Cross-reference against PHP manual for byref/variadic handling
3. Add `.phpstub` extension support (issue #59) to allow user overrides without Rust rebuild
4. Create baseline stubs for commonly-checked functions (array_map, array_filter, preg_match, etc.)

**Scope:** `crates/mir-analyzer/src/stubs/` (new `.phpstub` parser), stub files

**Test coverage:** 20–25 phpt fixtures in `stub_behavior/*`

**Effort:** 2–3 days | **PR:** `fix(stubs): improve builtin function accuracy for common functions`

---

#### 4b. Docblock Type Consistency Checking
**Issues:** 29 invalid_docblock + 35 method_signature_mismatch cases

**Root cause:** Docblock types (`@param`, `@return`, `@method`) not validated against declared hints or compared across inheritance.

**Fix strategy:**
1. After Pass 2, emit warnings for docblock-to-hint mismatches:
   - `@return string` but native return type is `int`
   - `@param int $x` but native param type is `string`
2. Walk inheritance chain and check docblock consistency:
   - Parent has `@return string`, child has `@return int` (invalid narrowing)
   - Parent param `@param Animal $x`, child param `@param Cat $x` (invalid narrowing)

**Scope:** `crates/mir-analyzer/src/analysis/class.rs` (inheritance walk), collector

**Test coverage:** 15–20 phpt fixtures in `invalid_docblock/*` and `method_signature_mismatch/*`

**Effort:** 2 days | **PR:** `fix(false-pos): validate docblock consistency with hints and inheritance`

---

### Phase 5: Type Narrowing & Conditional Checks — **P5, 3–4 weeks**

**Goal:** Eliminate ~60 false-positives (possibly_invalid_argument + possibly_undefined_variable + type_does_not_contain_type)

#### 5a. Union Type Narrowing in Conditionals
**Issues:** 32 possibly_invalid_argument cases (union types not narrowed in conditional branches)

**Root cause:** Type narrowing after `instanceof`, `is_string()`, etc. not applied to union types containing mixed.

**Fix strategy:**
1. Enhance `narrow_for_assertion` to handle:
   - `if ($x instanceof Foo) { /* $x: Foo */ }`
   - `if (is_string($x)) { /* $x: string */ }` for union types
   - `if ($x !== null) { /* remove null from union */ }`
2. Apply narrowed types to parameter checking in conditional branches
3. Add tracking for union branches that are never satisfied (dead code)

**Scope:** `crates/mir-analyzer/src/statements/expr_assert.rs`, `analysis/type_narrowing.rs`

**Test coverage:** 15–20 phpt fixtures in `possibly_invalid_argument/*`

**Effort:** 2–3 days | **PR:** `fix(false-pos): improve union type narrowing in conditionals`

---

#### 5b. Nullsafe Call Result Narrowing
**Issues:** Part of possibly_undefined_variable (14)

**Root cause:** `$obj?->method() !== null` narrows `$obj` but not the result type.

**Fix strategy:**
1. When analyzing `($x?->method()) !== null`, narrow:
   - `$x` to non-null in the true branch
   - Result type to remove null in the true branch
2. Add tracking for nullsafe call return type propagation

**Scope:** `crates/mir-analyzer/src/analysis/type_narrowing.rs`

**Test coverage:** 5–8 phpt fixtures in `possibly_undefined_variable/nullsafe_*`

**Effort:** 1 day | **PR:** `fix(false-pos): narrow nullsafe call results`

---

#### 5c. Type Guard Functions
**Issues:** Part of type_does_not_contain_type (12)

**Root cause:** Custom type guard functions (`array_key_exists()`, `is_scalar()`, `is_iterable()`) not recognized.

**Fix strategy:**
1. Add Salsa registry of known type guard functions with their narrowing behavior
2. When analyzing `if (array_key_exists($key, $arr))`, register key in array type shape
3. Support `@param-out bool` docblock annotation for custom type guards (issue #36)

**Scope:** `crates/mir-analyzer/src/analysis/type_narrowing.rs`, db.rs

**Test coverage:** 10–15 phpt fixtures in `type_does_not_contain_type/*`

**Effort:** 2 days | **PR:** `feat(false-pos): recognize type guard functions and custom @param-out`

---

## Priority Matrix & Sequencing

```
Phase 0 (Foundation)     ← BLOCKING (start now)
  ↓
Phase 1 (Quick Wins)     ← HIGH IMPACT (2–3 weeks)
  ↓
Phase 2 (Read Tracking)  ← HIGH IMPACT (2–3 weeks)
  ↓
Phase 3 (Cross-File)     ← MEDIUM IMPACT (3–4 weeks)
  ↓
Phase 4 (Stubs & Docs)   ← MEDIUM IMPACT (2–3 weeks)
  ↓
Phase 5 (Narrowing)      ← MEDIUM IMPACT (3–4 weeks)
```

**Parallel track:** Continue perf work from MEMORY_OPTIMIZATION_ROADMAP alongside Phase 1–2.

---

## PR Batch Breakdown

### Batch 1 (Week 1–2 of Phase 1)
1. `fix(false-pos): callable parameter validation for array_*` — 35–40 tests
2. `fix(false-pos): resolve methods through full inheritance chain` — 15–20 tests

### Batch 2 (Week 2–3 of Phase 1)
3. `feat(false-pos): support @method docblock pseudo-methods` — 10–15 tests

### Batch 3 (Week 1–2 of Phase 2)
4. `fix(false-pos): track variable reads in nullsafe method calls` — 20–25 tests
5. `fix(false-pos): mark variables read when captured in closures` — 15–20 tests

### Batch 4 (Week 2–3 of Phase 2)
6. `fix(false-pos): consistently track reads on assignment RHS` — 10–15 tests

### Batch 5 (Week 1–2 of Phase 3)
7. `fix(false-pos): lazy-load FQCNs after Pass 2` — existing test
8. `fix(false-pos): apply namespace imports during type resolution` — 10–15 tests

### Batch 6 (Week 2–3 of Phase 3)
9. `fix(false-pos): parse string callables as method references` — 5–8 tests

### Batch 7 (Week 1 of Phase 4)
10. `fix(stubs): improve builtin function accuracy for common functions` — 20–25 tests

### Batch 8 (Week 2 of Phase 4)
11. `fix(false-pos): validate docblock consistency with hints and inheritance` — 15–20 tests

### Batch 9 (Week 1–2 of Phase 5)
12. `fix(false-pos): improve union type narrowing in conditionals` — 15–20 tests
13. `fix(false-pos): narrow nullsafe call results` — 5–8 tests

### Batch 10 (Week 2–3 of Phase 5)
14. `feat(false-pos): recognize type guard functions and custom @param-out` — 10–15 tests

**Total:** 14 PRs over ~12–14 weeks, reducing false-positives from 1,356 → ~400–500 (estimated).

---

## Success Metrics

| Milestone | Target | Current | Expected Date |
|---|---|---|---|
| Phase 0 complete (#115, #116, S3) | ✓ Foundation | In progress | May 20 |
| Phase 1 + 2 complete (270 FP) | ~1,086 remaining | 1,356 | June 3 |
| Phase 3 + 4 complete (170 FP) | ~916 remaining | 1,356 | July 1 |
| Phase 5 complete (60 FP) | ~856 remaining | 1,356 | July 22 |
| Sub-500 false-positives | <500 | 1,356 | August 1 |

---

## Integration with MEMORY_OPTIMIZATION_ROADMAP

- **Do not defer Phase 0 for perf work.** #115 / #116 are blocking for both incremental analysis AND reliable multi-file fixture testing.
- **Can parallelize Phases 1–2 with perf work** (Phase B in MEMORY_OPTIMIZATION_ROADMAP). They don't conflict.
- **Phase 3–5 should defer until Phase B completes** (perf stabilization). High-complexity type narrowing + cross-file resolution is easier with stable performance baselines.

---

## Known Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| S3 deadlock unresolved | Blocks inferred return type promotion; limits method call checking accuracy | Attempt rayon::scope fix; if unsuccessful, defer to next release |
| Multi-file fixtures flaky without #115 / #116 | False positives hard to verify | Implement incremental re-analysis first |
| Callable type validation adds perf cost | Phase 2 impact unknown | Profile array_map/filter workloads; consider lazy resolution |
| Docblock parsing incomplete | Some @method/@var cases slip through | Add parser tests for each variant (param position, generics, nullable, etc.) |

---

## Links to Related Issues

- Builtin function enhancements: [#59 (.phpstub files)](https://github.com/jorgsowa/mir/issues/59)
- Type guard functions: [#36](https://github.com/jorgsowa/mir/issues/36)
- Nullsafe narrowing: [#145](https://github.com/jorgsowa/mir/issues/145)
- Generic instanceof narrowing: [#152](https://github.com/jorgsowa/mir/issues/152) ← Already completed
- Loop widening: [#37](https://github.com/jorgsowa/mir/issues/37) ← Already completed

