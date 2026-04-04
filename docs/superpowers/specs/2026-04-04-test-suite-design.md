# Test Suite Design — mir static analyzer

**Date:** 2026-04-04  
**Scope:** Per-rule integration tests for the 8 Psalm-parity rules

---

## Goals

- Every Psalm-parity rule has a dedicated test file with positive (fires) and negative (no false positive) cases.
- Tests assert exact `IssueKind` variant, line number, and `col_start`.
- No temp-file boilerplate in test bodies — hidden inside the harness.
- Harness is reusable across crates.

---

## Architecture

### New crate: `crates/mir-test-utils`

A `lib.rs`-only crate with no binary. Depends on `mir-analyzer` and `mir-issues`.

**Public API:**

```rust
/// Run the analyzer on an inline PHP string. Returns all unsuppressed issues.
pub fn check(src: &str) -> Vec<Issue>;

/// Assert exactly one issue matching `kind` at `line`:`col_start`.
/// Panics with full issue list if not found.
pub fn assert_issue(issues: &[Issue], kind: IssueKind, line: u32, col_start: u16);

/// Assert no issue with name `kind_name` (e.g. "UndefinedFunction") exists.
/// Panics with full issue list if any match.
pub fn assert_no_issue(issues: &[Issue], kind_name: &str);
```

`check()` creates a temp file internally, runs `ProjectAnalyzer`, deletes the temp file, and returns unsuppressed issues. This detail is invisible to test authors.

### Test file location

Rust integration tests under `crates/mir-analyzer/tests/rules/`:

```
crates/mir-analyzer/tests/
  rules/
    undefined_function.rs
    undefined_class.rs
    invalid_argument.rs
    possibly_invalid_array_access.rs
    redundant_condition.rs
    undefined_method.rs
    invalid_return_type.rs
    method_signature_mismatch.rs
```

### Test pattern

```rust
use mir_test_utils::{check, assert_issue, assert_no_issue};
use mir_issues::IssueKind;

#[test]
fn reports_call_to_missing_function() {
    let issues = check("<?php\nfoo();");
    assert_issue(&issues, IssueKind::UndefinedFunction { name: "foo".into() }, 2, 0);
}

#[test]
fn does_not_report_builtin_function() {
    let issues = check("<?php\nstrlen('hello');");
    assert_no_issue(&issues, "UndefinedFunction");
}
```

---

## Test Inventory (~63 tests across 8 files)

### `undefined_function.rs` (~8 tests)
1. Call to completely unknown function → fires
2. Call to PHP builtin (`strlen`, `array_map`) → no fire
3. Call to user-defined function in same file → no fire
4. Call with namespace prefix (`\foo()`) where not defined → fires
5. Call to `unpack()` → no fire (builtin)
6. Suppressed with `@psalm-suppress UndefinedFunction` → no fire
7. Call inside a method body → fires with correct line/col
8. Multiple calls to same undefined function → fires once per call site

### `undefined_class.rs` (~8 tests)
1. `new UnknownClass()` → fires
2. `new stdClass()` → no fire
3. User-defined class in same file → no fire
4. Type hint `function f(UnknownClass $x)` → fires
5. `instanceof UnknownClass` → fires
6. `use ast\Node` where `ast` extension not loaded → fires
7. Class aliased via `use Foo as Bar`; `new Bar()` → no fire
8. Suppressed → no fire

### `invalid_argument.rs` (~10 tests)
1. Pass `string` where `int` expected → fires
2. Pass `int` where `int` expected → no fire
3. Pass `null` where `string` expected → fires
4. Pass union `int|string` where `int` expected → fires
5. Pass subclass where parent expected → no fire
6. Pass wrong type to builtin (`strlen(42)`) → fires
7. Named argument wrong type → fires
8. Variadic argument wrong type → fires
9. Pass `mixed` → no fire
10. Correct union passed to union param → no fire

### `invalid_return_type.rs` (~8 tests)
1. Return `string` from `int`-declared function → fires
2. Return correct type → no fire
3. Return `null` from non-nullable declared return → fires
4. Return union where single type declared → fires
5. Implicit `return;` from non-void function → fires
6. Return subclass of declared return type → no fire
7. Return `mixed` → no fire
8. Void function with explicit `return null` → fires

### `undefined_method.rs` (~9 tests)
1. Call `$obj->nonExistent()` on known class → fires
2. Call defined method → no fire
3. Call on `null` → `NullMethodCall`, not `UndefinedMethod`
4. Call on interface type that declares method → no fire
5. Call on abstract class that defines method → no fire
6. Call on `mixed` → no fire
7. Static call `Foo::bar()` where `bar` not defined → fires
8. `parent::` call where method exists → no fire
9. Call on generic type parameter → no fire

### `method_signature_mismatch.rs` (~7 tests)
1. Override narrows parameter type → fires
2. Override widens return type → fires
3. Override with compatible signature → no fire
4. Override adds required parameter → fires
5. Override changes default to required → fires
6. Interface implementation with wrong signature → fires
7. Abstract method implementation with correct signature → no fire

### `possibly_invalid_array_access.rs` (~7 tests)
1. Destructure `array|false` → fires
2. Destructure `array` → no fire
3. Destructure `false` → fires
4. `[$a, $b] = get()` where `get(): array|false` → fires on both
5. False-checked before destructure (`if ($r !== false) { [$a] = $r; }`) → no fire
6. `unpack()` result destructured → fires (documents known gap)
7. Correct array offset access → no fire

### `redundant_condition.rs` (~6 tests)
1. `if ($x === null)` where `$x: string` → fires
2. `if ($x !== null)` where `$x: string` → fires
3. `if ($x === null)` where `$x: string|null` → no fire
4. `if (is_string($x))` where `$x: string` → fires
5. `if (is_string($x))` where `$x: string|int` → no fire
6. Redundant second check after type narrowing → fires

---

## Non-goals

- Tests for the remaining 49 `IssueKind` variants (deferred).
- Snapshot testing.
- Column-end (`col_end`) assertions.
- Performance benchmarks.
