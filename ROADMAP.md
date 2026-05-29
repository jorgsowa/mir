# Roadmap

## Honor phpstorm-stubs version attributes in the stub collector

### Objective & rationale

Make the collector resolve `#[LanguageLevelTypeAware]` and
`#[PhpStormStubsElementAvailable]` against the configured target `PhpVersion`.
Today both attributes are dropped, leaving ~2400 sites across 79 stub files with
untyped params or wrong-version signatures. Reading them (rather than migrating
them to phpdoc) keeps `stubs/` a **verbatim mirror** of upstream phpstorm-stubs,
so re-syncing remains a plain file copy.

### Confirmed upstream semantics

- **`PhpStormStubsElementAvailable($from, $to = null)`** — both bounds are
  **inclusive**. Verified empirically: `Error::__clone` is declared
  `from:"7.0", to:"8.0"` then `'8.1'`; the method exists in every version, so the
  only gap-free partition is inclusive `to` (7.0–8.0, then 8.1+).
- **`LanguageLevelTypeAware(array $map, string $default)`** — `default` is
  required. Pick the value at the highest map key &le; target; below the lowest
  key use `default`. Real case `['8.0' => 'int', '8.5' => 'int|null'], default: ''`
  confirms this, and shows `default: ''` (empty = "no type", must be treated as
  *no override*, never parsed into a type).
- No patch versions appear anywhere — all keys are `major.minor`.
- Targets: `ElementAvailable` → function/method/param; `LanguageLevelTypeAware` →
  those plus property.
- Arg forms: `from` = positional[0] or named; `to` = positional[1] or named;
  LLTA `default` = positional[1] or named `default:`. Attribute names may be
  aliased (`use ... as ...`).

### Scope

- **v1 includes:** param-level type (`LanguageLevelTypeAware`) and availability
  (`PhpStormStubsElementAvailable`); return-level type; **symbol-level**
  `PhpStormStubsElementAvailable` on functions/methods (cheap — reuses the same
  decoder at the existing `version_allows` site, and fixes duplicate-declaration
  pairs like `Error::__clone` that otherwise mis-resolve via "last declaration
  wins").
- **Deferred to v2:** `LanguageLevelTypeAware` on properties.
- **User code (`php_version == None`):** behavior unchanged — all attribute logic
  is gated on `Some`.

### Step 1 — Version range primitive

In `crates/mir-analyzer/src/php_version.rs`, add
`fn in_range(self, from: Option<&str>, to_inclusive: Option<&str>) -> bool`:
`available = (from.is_none() || self >= from) && (to.is_none() || self <= to)`.

Do **not** reuse `includes_symbol`: its `removed` bound is exclusive, and "the
minor after 8.0" is not well-defined across the 7.4 → 8.0 jump. Parse versions
with the existing `PhpVersion: FromStr`; ignore anything that fails to parse
(defensive), and truncate `x.y.z` → `x.y` if a patch version ever appears.

### Step 2 — Self-contained attribute decoder

New `crates/mir-analyzer/src/collector/version_attrs.rs`: pure functions over
`&[php_ast::owned::Attribute]` + `&use_aliases` + `target: PhpVersion`. No
collector state, so it is unit-testable in isolation.

- Constants `LLTA_FQN = "JetBrains\\PhpStorm\\Internal\\LanguageLevelTypeAware"`
  and `PSEA_FQN = "JetBrains\\PhpStorm\\Internal\\PhpStormStubsElementAvailable"`.
- **Name match:** resolve each `attr.name` through `use_aliases` to a canonical
  FQN, then compare to the constants. Never match on the bare short name (avoids
  collisions with user attributes of the same suffix).
- `is_available(attrs, target) -> bool`: find `PSEA`; read `from` (positional
  arg 0 or named `from`) and `to` (positional arg 1 or named `to`) as
  `ExprKind::String`; return `PhpVersion::in_range(from, to)`. Absent attribute
  ⇒ `true`.
- `type_aware(attrs, target) -> Option<String>`: find `LLTA`; positional arg 0 is
  an `ExprKind::Array`, each `ArrayElement` key/value is an `ExprKind::String`.
  Collect `(PhpVersion, type)` pairs, take the value at the highest key &le;
  target; if none match, use `default` (positional arg 1 or named `default:`).
  If the chosen string is empty, return `None` (no override). Absent attribute
  ⇒ `None`.

### Step 3 — Param loops (apply identically in both)

`collector/function.rs:58-98` (functions) and `collector/mod.rs:713-750`
(methods). Per param, the order is load-bearing:

1. **Availability first:** if `php_version` is `Some(v)` and
   `!is_available(&p.attributes, v)`, `continue` — omitting the param entirely
   (keeping it would corrupt arity checks). Do not even look at its type.
2. **Type next:** if `type_aware(&p.attributes, v)` returns `Some(s)`, run
   `parse_type_string(&s)` and use it as `ty`, overriding the (usually absent)
   hint/docblock type. Otherwise keep the current resolution.

### Step 4 — Return types

`function.rs:111-123` and `mod.rs:761-773`: before the existing `match`, if
`php_version` is `Some(v)` and the function/method node's own `attributes` yield
`type_aware -> Some(s)`, parse it and route through the same
`resolve_union_*` / `fill_self_static_parent` flow the docblock return type uses.

### Step 5 — Symbol-level availability

Extend `version_allows` (`collector/mod.rs:176-181`), or its callers for
functions/methods, so that in addition to `doc.since`/`doc.removed`, a
function/method node carrying `PhpStormStubsElementAvailable` is filtered via
`is_available`. This makes duplicate-declaration pairs (e.g. `Error::__clone`,
`from:"7.0", to:"8.0"` vs `'8.1'`) resolve to the correct declaration per target
instead of "last wins".

### Step 6 — Regression audit (before writing fixtures)

These params are untyped today, so wrong-type calls silently pass; real types
will newly fire diagnostics.

1. Run the full suite first; capture the diagnostic delta.
2. Triage each change: latent bug correctly surfaced vs. decoder bug vs. wrong
   upstream type. No bulk snapshot updates.
3. Accept the delta only once every change is explained.

### Step 7 — Tests

- **Unit (decoder):** multi-threshold map (`['8.0'=>'int','8.5'=>'int|null']`) at
  7.4 / 8.0 / 8.5; empty `default` ⇒ `None`; positional vs named `from`/`to`;
  aliased attribute name (`use ... as X`); `to` inclusive boundary
  (target == `to`).
- **Range primitive:** `in_range` boundaries including the 7.4 ↔ 8.0 jump.
- **phpt fixtures** (must hit the new path, not the duplicate-declaration path):
  - Param type via LLTA: `ArrayAccess::offsetGet($offset)` untyped → `mixed` at
    8.0 — `@mir-check` at 7.4 and 8.0.
  - Param availability via PSEA: a function whose param appears only `from:"8.0"`
    — assert arity differs across the boundary.
  - Symbol-level PSEA: `Error::__clone` resolves the right declaration on 7.4 vs
    8.1.

### Risk / non-goals

- Inclusive `to` is the highest-risk semantic; it is empirically and
  structurally confirmed, and Step 7 pins it with a boundary fixture.
- The stub corpus stays verbatim — no codegen, no divergence from upstream
  format.
- Touch surface is small and isolated: one primitive + one decoder module + four
  hook sites + one filter extension.
