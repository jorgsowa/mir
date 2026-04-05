# Spec: MethodSignatureMismatch — parameter type narrowing detection

**Date:** 2026-04-05
**Issue:** #41

## Problem

`MethodSignatureMismatch` is emitted for return type widening and arity changes, but not for
parameter type narrowing in overrides or interface implementations. Narrowing a parameter type
(e.g. parent accepts `string`, child accepts only `int`) violates contravariance and must be
rejected.

## Design

### Location

`crates/mir-analyzer/src/class.rs` — `check_override_compat` method.

### Change

After existing section `d.` (required param count check), add section `e.`:

```
For each positional index i in 0..min(parent.params.len(), own_method.params.len()):
  - Skip if either param's ty is None (no type hint on one side — PHP allows this)
  - Skip if either type is mixed
  - Skip if either type involves named objects (subtype check needs codebase/inheritance graph)
  - Skip if either type involves self/static
  - Violation: !parent_param_ty.is_subtype_of_simple(child_param_ty)
    (Contravariance: parent type must be a subtype of child type; if not, child narrows)
  - Emit MethodSignatureMismatch with detail naming the param index and the two types
```

The guards mirror the return-type check (section `c.`) to avoid false positives from generics,
named objects, and self/static references.

### Tests

Remove `#[ignore]` from the two existing ignored tests:
- `reports_override_narrowing_param_type`
- `reports_interface_implementation_wrong_signature`

Add new tests to cover the guards and edge cases:

| Test name | Scenario | Expected |
|-----------|----------|----------|
| `does_not_report_override_widening_param_type` | parent: `string`, child: `string\|int` | no issue |
| `does_not_report_override_no_type_hint` | parent: no hint, child: `int` | no issue |
| `reports_override_narrowing_second_param` | first param matches, second narrows | 1 issue |

## Acceptance criteria

`cargo test -p mir-analyzer --test method_signature_mismatch` passes with no ignored tests
for the two currently-failing cases.
