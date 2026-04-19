# mir Roadmap

Current version: **v0.5.2**

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
| M15 — Configuration | ⚠️ Partial |
| M16 — CLI | ⚠️ Partial (`--set-baseline`, `--no-cache` missing) |
| M17 — Cache layer (Pass 2, content-hash) | ✅ Complete |
| M18 — Dead code detection | ✅ Complete |
| M19 — Taint analysis | ✅ Complete |
| M20 — Plugin system | ❌ Not started |

---

## Performance & Architecture Roadmap

### Phase 1 — Memory (independent, `mir-codebase` only)

The reference index uses `DashMap<Arc<str>, HashMap<Arc<str>, HashSet<(u32,u32)>>>`.
The innermost `HashSet` carries ~72 bytes of overhead per entry; actual data is 8 bytes.

**1. String interning**
Replace `Arc<str>` keys across all reference maps with `u32` IDs backed by a two-way interner.
Eliminates key duplication across `symbol_reference_locations`, `file_symbol_references`, and
the three dead-code `DashSet`s.

**2. Flat `Vec<Ref>`**
Replace the nested map structure with a single `Vec<(symbol_id, file_id, start, end)>` during
the build phase. All three overlapping maps collapse into one.

**3. `compact_reference_index()`**
After Pass 2, sort the `Vec<Ref>` and build two CSR (Compressed Sparse Row) index arrays —
one keyed by symbol, one by file. Drop the build-phase hash maps.

Expected: ~5× reduction in reference index memory. No behavioral change.

---

### Phase 2 — Non-LSP incremental (`mir-cache`)

Pass 2 results are already cached by content hash (M17). The missing increment:

**4. Cache Pass 1 results**
Extend `CacheEntry` with `FileDefinitions`. On a cache hit, skip parsing and definition
collection entirely — not just body analysis. Biggest win for large projects where few
files change between runs.

**5. Cache finalization**
Hash each class's definition inputs (parent FQCN + interface list). Skip
`collect_class_ancestors` if the class definition is unchanged. Store the computed
`all_parents` and `all_methods` tables alongside the definition cache entry.

Expected: near-zero cost for CLI runs where few files changed.

---

### Phase 3 — Remove the pass barrier (`mir-codebase`, `mir-analyzer`)

The global `finalize()` is a serial barrier: all of Pass 1 must complete before any file
can start Pass 2. Files whose dependency chains are short are blocked unnecessarily.

**6. Per-class `OnceLock` finalization**
Replace the global `finalize()` with `ensure_finalized(fqcn)` that computes lazily per class
and memoizes the result. Use a `DashMap<Arc<str>, OnceLock<Arc<FinalizedClass>>>` with
thread-local cycle detection.

**7. Merge the pass loop**
Single rayon scan: each file task does Pass 1 → `ensure_finalized()` for its dependencies →
Pass 2, all without a global barrier. `lazy_load_missing_classes` becomes automatic — missing
classes are loaded on demand inside `ensure_finalized()`.

Expected: 20–40% wall-time reduction for large projects. Simplifies incremental re-analysis.

---

### Phase 4 — Symbol-level incremental + LSP (Salsa)

Current cache invalidation is file-level: if file A changes, all files importing anything from
A are evicted — even if only a private method body changed. A proper query system tracks
symbol-level dependencies and skips re-analysis when query outputs are unchanged.

**8. Introduce Salsa**
Define `parse_file`, `file_definitions`, `finalized_class`, and `analyze_file` as tracked
queries backed by the `salsa` crate. Salsa handles memoization, cycle detection, parallel
evaluation, and precise invalidation automatically.

**9. Replace `re_analyze_file`**
Update a `SourceFile` input; Salsa invalidates only the affected subgraph. The LSP path
simplifies to a single call with no manual cache eviction or definition removal.

**10. Replace file-level dep graph**
Salsa's automatic dependency tracking replaces `file_symbol_references` and the reverse dep
cache. Symbol-level precision: a change to a private method body invalidates zero other files.

Expected: sub-second re-analysis on save for LSP; precise invalidation across all query types.

---

### Phase dependencies

```
Phase 1 ──────────────────────── ships alone, no blockers
Phase 2 ──────────────────────── ships alone (cache is additive)
Phase 3 ── benefits from Phase 1 (flat Vec friendlier to per-class memoization)
Phase 4 ── subsumes Phase 2 & 3  (Salsa makes manual caching redundant)
```

Phases 1 and 2 deliver value independently. Phase 3 is the stepping stone toward Phase 4.
Phase 4 is the right long-term foundation for both LSP and CLI incremental analysis.
