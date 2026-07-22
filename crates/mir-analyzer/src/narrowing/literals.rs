//! Int/string/bool/null literal & range narrowing: `$x === 5`, `$x < 5`,
//! `$x === 'lit'`, `$x === true`/`false`, `$x === null` and their loose
//! (`==`/`!=`) counterparts, for variable, property, and static-property
//! receivers.
use php_ast::ast::{BinaryOp, UnaryPrefixOp};
use php_ast::owned::ExprKind;

use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{
    apply_prop_narrowed, extract_static_prop_access, is_numeric_string,
    narrow_receiver_non_null_on_prop_match, peel_parens, resolve_prop_current_type,
    resolve_static_prop_current_type, set_narrowed, ScalarArgTarget, UnionNarrowExt,
};

/// Returns true if `expr` is the boolean literal `true`.
pub(super) fn is_truthy_bool_literal(expr: &php_ast::owned::Expr) -> bool {
    matches!(expr.kind, php_ast::owned::ExprKind::Bool(true))
}

/// True when `expr` is a non-empty string literal, or a variable/property
/// already narrowed to one — shared by `str_contains()`/`str_starts_with()`/
/// `str_ends_with()` and the `strpos()`-family false-comparable narrowing,
/// both of which only narrow their haystack when the needle is provably
/// non-empty (an empty needle is trivially "found" at offset 0).
pub(super) fn expr_is_nonempty_string_literal(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> bool {
    match &expr.kind {
        ExprKind::String(s) => !s.is_empty(),
        _ => match ScalarArgTarget::extract(expr) {
            Some(ScalarArgTarget::Var(name)) => {
                matches!(ctx.get_var(&name).types.as_slice(), [Atomic::TLiteralString(s)] if !s.is_empty())
            }
            Some(ScalarArgTarget::Prop(obj, prop)) => matches!(
                resolve_prop_current_type(ctx, &obj, &prop, db, file)
                    .types
                    .as_slice(),
                [Atomic::TLiteralString(s)] if !s.is_empty()
            ),
            // ScalarArgTarget has no static-property variant (tracked as
            // S19) — extract it call-site-locally instead, mirroring the
            // haystack side's existing static-prop arm right below.
            None => match extract_static_prop_access(expr, ctx, db, file) {
                Some((fqcn, prop)) => matches!(
                    resolve_static_prop_current_type(ctx, &fqcn, &prop, db)
                        .types
                        .as_slice(),
                    [Atomic::TLiteralString(s)] if !s.is_empty()
                ),
                None => false,
            },
        },
    }
}

/// Extract a signed integer literal from an expression, handling negation.
pub(super) fn extract_int_literal(expr: &php_ast::owned::Expr) -> Option<i64> {
    let e = peel_parens(expr);
    match &e.kind {
        ExprKind::Int(n) => Some(*n),
        ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::Negate => {
            if let ExprKind::Int(n) = &u.operand.kind {
                n.checked_neg()
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Flip a comparison operator for when operands are swapped (`5 > $x` → `$x < 5`).
pub(super) fn flip_comparison_op(op: BinaryOp) -> BinaryOp {
    match op {
        BinaryOp::Less => BinaryOp::Greater,
        BinaryOp::LessOrEqual => BinaryOp::GreaterOrEqual,
        BinaryOp::Greater => BinaryOp::Less,
        BinaryOp::GreaterOrEqual => BinaryOp::LessOrEqual,
        other => other,
    }
}

/// Narrow a variable by a comparison `$var op n` being `is_true`.
/// The range constraint implied by `$x <op> n` resolving to `is_true`.
/// Negation (`!is_true`) flips the constraint (e.g. NOT `< N` becomes `>= N`).
/// Sentinel bound pair meaning "no `i64` value can satisfy this range" — a
/// deliberately inverted (`min > max`) pair rather than a distinct enum
/// variant, so every existing consumer (`intersect_int_range_into`'s
/// `lo > hi` check, and `TLiteralInt`'s direct bounds check in
/// `narrow_type_to_int_range`) already treats it as empty with no extra
/// plumbing.
const IMPOSSIBLE_BOUNDS: (Option<i64>, Option<i64>) = (Some(1), Some(0));

fn int_comparison_bounds(
    op: BinaryOp,
    n: i64,
    is_true: bool,
) -> Option<(Option<i64>, Option<i64>)> {
    match (op, is_true) {
        // `$x < i64::MIN` (or its `!($x >= i64::MIN)` negation) can never be
        // true — `n - 1` would underflow, so treat that as a genuinely empty
        // range instead of silently falling back to an unconstrained upper
        // bound.
        (BinaryOp::Less, true) | (BinaryOp::GreaterOrEqual, false) => Some(
            n.checked_sub(1)
                .map_or(IMPOSSIBLE_BOUNDS, |hi| (None, Some(hi))),
        ),
        (BinaryOp::LessOrEqual, true) | (BinaryOp::Greater, false) => Some((None, Some(n))),
        // Mirror image: `$x > i64::MAX` can never be true — `n + 1` overflows.
        (BinaryOp::Greater, true) | (BinaryOp::LessOrEqual, false) => Some(
            n.checked_add(1)
                .map_or(IMPOSSIBLE_BOUNDS, |lo| (Some(lo), None)),
        ),
        (BinaryOp::GreaterOrEqual, true) | (BinaryOp::Less, false) => Some((Some(n), None)),
        _ => None,
    }
}

/// Whether `$x <op> n` resolving to `is_true` proves `$x` isn't `null`,
/// independent of `n`'s value. PHP's ordering-comparison table converts a
/// `null` operand to `bool` (`false`) and the int literal to `bool` (`n !=
/// 0`) — `null > n` and `null <= n` (its negation) always compare
/// `false <op> bool(n)` in a way that can never hold `null`'s side true
/// regardless of `n`, so those two directions exclude `null` unconditionally.
/// `>=`/`<` are the opposite: whether `null` survives depends on whether
/// `n == 0`, so those stay untouched here (deferred, `n`-dependent).
/// Whether a null-valued receiver is excluded by `$x op N` holding
/// `is_true`. PHP compares `null` to an int via its ordering-comparison
/// table by converting `null` to `bool(false)` and the int literal to
/// `bool(N)` (`true` iff `N != 0`), then comparing the two bools.
///
/// `Greater`/`LessOrEqual` are N-independent: `false > bool(N)` is always
/// false, and `false <= bool(N)` is always true, regardless of N — so
/// `$x > N` true (or `$x <= N` false) always excludes null, and `$x > N`
/// false (or `$x <= N` true) never does.
///
/// `Less`/`GreaterOrEqual` are N-dependent: `false < bool(N)` and
/// `false >= bool(N)` each flip on whether `N == 0`, so whether null
/// survives hinges on N too, not just the (op, is_true) shape.
fn int_comparison_excludes_null(op: BinaryOp, n: i64, is_true: bool) -> bool {
    match (op, is_true) {
        (BinaryOp::Greater, true) | (BinaryOp::LessOrEqual, false) => true,
        (BinaryOp::Greater, false) | (BinaryOp::LessOrEqual, true) => false,
        (BinaryOp::Less, true) | (BinaryOp::GreaterOrEqual, false) => n == 0,
        (BinaryOp::Less, false) | (BinaryOp::GreaterOrEqual, true) => n != 0,
        _ => false,
    }
}

pub(super) fn narrow_var_int_comparison(
    ctx: &mut FlowState,
    name: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some((min, max)) = int_comparison_bounds(op, n, is_true) else {
        return;
    };
    let current = ctx.get_var(name);
    let mut narrowed = narrow_type_to_int_range(&current, min, max);
    if int_comparison_excludes_null(op, n, is_true) {
        narrowed = narrowed.remove_null();
    }
    // Mark the branch unreachable only when the current type is "closed precise"
    // (a bounded int range, named int subtype, or literal union) — these only arise
    // from docblocks/inference, so an empty intersection is a real contradiction.
    // A plain `int` narrowed to an empty range is just conservative widening, not a bug.
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    set_narrowed(ctx, name, &current, narrowed, mark_diverges);
}

/// Property-access counterpart of `narrow_var_int_comparison`, for
/// `$this->prop < N` (or any `$obj->prop` receiver).
#[allow(clippy::too_many_arguments)]
pub(super) fn narrow_prop_int_comparison(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some((min, max)) = int_comparison_bounds(op, n, is_true) else {
        return;
    };
    // Whether the comparison excludes null is a fact about op/n/is_true alone,
    // independent of the property's own (possibly mixed) type — apply it even
    // when the value-narrowing below bails out on a mixed property.
    narrow_receiver_non_null_on_prop_match(
        ctx,
        obj_var,
        int_comparison_excludes_null(op, n, is_true),
    );
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    let mut narrowed = narrow_type_to_int_range(&current, min, max);
    if int_comparison_excludes_null(op, n, is_true) {
        narrowed = narrowed.remove_null();
    }
    // A nullable $obj means the comparison can also be satisfied by the
    // receiver itself being null: PHP's ordering-comparison table converts a
    // null operand and the int literal to bool and compares those (`false`
    // for null, `N != 0` for the literal), which can make `<`/`<=`/`>`/`>=`
    // true regardless of the property's own (precise, out-of-range) type —
    // same reasoning as narrow_prop_instanceof/narrow_prop_is_a's gate.
    let mark_diverges =
        crate::contradiction::is_closed_precise(&current) && !ctx.get_var(obj_var).is_nullable();
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_int_comparison`, for
/// `self::$prop < N` (or `static::$prop`/`Class::$prop`). Unlike the
/// instance-property case, a static property has no separate receiver
/// variable whose nullability could also satisfy the comparison —
/// `self::`/`static::` is never itself null — so mark_diverges only
/// depends on `is_closed_precise`, matching the plain-variable case.
pub(super) fn narrow_static_prop_int_comparison(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some((min, max)) = int_comparison_bounds(op, n, is_true) else {
        return;
    };
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let mut narrowed = narrow_type_to_int_range(&current, min, max);
    if int_comparison_excludes_null(op, n, is_true) {
        narrowed = narrowed.remove_null();
    }
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, mark_diverges);
}

/// Apply integer bounds `[min, max]` to all integer components of a type.
///
/// Integer atoms (`int`, `int<a,b>`, literal ints) that fall within the bounds
/// are kept (possibly tightened); those that provably fall outside are removed.
/// Non-integer atoms pass through unchanged so the narrowing is always safe.
fn narrow_type_to_int_range(ty: &Type, min: Option<i64>, max: Option<i64>) -> Type {
    let in_bounds = |v: i64| min.is_none_or(|lo| v >= lo) && max.is_none_or(|hi| v <= hi);
    let mut result = Type::empty();
    result.from_docblock = ty.from_docblock;
    for atomic in &ty.types {
        match atomic {
            Atomic::TInt => {
                // Route through the same intersection helper as the named
                // int-subtype/range arms below (treating plain `int` as the
                // fully-unbounded range) so an impossible `min > max` result
                // (see `IMPOSSIBLE_BOUNDS`) is dropped instead of being
                // constructed as a nonsensical `int<min, max>` atom.
                intersect_int_range_into(&mut result, None, None, min, max);
            }
            // Named int subtypes carry implicit bounds; intersect rather than replace.
            Atomic::TPositiveInt => {
                intersect_int_range_into(&mut result, Some(1), None, min, max);
            }
            Atomic::TNonNegativeInt => {
                intersect_int_range_into(&mut result, Some(0), None, min, max);
            }
            Atomic::TNegativeInt => {
                intersect_int_range_into(&mut result, None, Some(-1), min, max);
            }
            Atomic::TIntRange {
                min: cur_min,
                max: cur_max,
            } => {
                intersect_int_range_into(&mut result, *cur_min, *cur_max, min, max);
            }
            Atomic::TLiteralInt(v) => {
                if in_bounds(*v) {
                    result.add_type(atomic.clone());
                }
            }
            _ => {
                result.add_type(atomic.clone());
            }
        }
    }
    result
}

/// Intersect `(existing_min, existing_max)` with `(narrow_min, narrow_max)` and push
/// the result into `out`, skipping the intersection if it is provably empty.
fn intersect_int_range_into(
    out: &mut Type,
    existing_min: Option<i64>,
    existing_max: Option<i64>,
    narrow_min: Option<i64>,
    narrow_max: Option<i64>,
) {
    let new_min = match (existing_min, narrow_min) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (None, v) | (v, None) => v,
    };
    let new_max = match (existing_max, narrow_max) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (None, v) | (v, None) => v,
    };
    if let (Some(lo), Some(hi)) = (new_min, new_max) {
        if lo > hi {
            return; // Empty intersection — this arm is unreachable
        }
    }
    out.add_type(Atomic::TIntRange {
        min: new_min,
        max: new_max,
    });
}

/// Narrow all `TString` atoms to `TNonEmptyString`, preserving other atoms —
/// except the empty-string literal, which a proven non-empty result rules
/// out entirely (unlike `TString`, a literal can't be "tightened", only
/// kept or dropped). Used when a condition proves the string is non-empty.
///
/// `pub(super)`: also used by the array `in_array`/`array_search` narrowing
/// in `mod.rs` and by the strpos-family narrowing in `strings.rs`.
pub(super) fn narrow_string_to_non_empty(ty: &Type) -> Type {
    let mut result = Type::empty();
    result.from_docblock = ty.from_docblock;
    for t in &ty.types {
        match t {
            Atomic::TString => result.add_type(Atomic::TNonEmptyString),
            Atomic::TLiteralString(s) if s.as_ref().is_empty() => {}
            _ => result.add_type(t.clone()),
        }
    }
    result
}

/// Drop every atom a length check proves impossible once the string is known
/// to be exactly empty: `non-empty-string` and any atom that can never be
/// `""` at all (a numeric/class/callable string, or a non-empty literal) —
/// mirrors `Type::narrow_to_empty_collection` for arrays.
///
/// `pub(super)`: also used by the `strlen()`/`mb_strlen()` narrowing in
/// `strings.rs`.
pub(super) fn narrow_string_to_empty(ty: &Type) -> Type {
    ty.filter(|t| match t {
        Atomic::TNonEmptyString
        | Atomic::TNumericString
        | Atomic::TClassString(_)
        | Atomic::TCallableString => false,
        Atomic::TLiteralString(s) => s.as_ref().is_empty(),
        _ => true,
    })
}

pub(super) fn narrow_var_null(ctx: &mut FlowState, name: &str, is_null: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

/// Loose-equality counterpart of `narrow_var_null`, for `$x == null`/`!= null`.
/// PHP's loose-null comparison is also true for `false`/`0`/`0.0`/`""`/`[]`, not
/// just `null` itself, so narrowing the true branch all the way down to `TNull`
/// (as the strict `=== null` form does) would be unsound. `narrow_to_falsy` is
/// the same safe superset already used for `== true`/`== false`
/// (`narrow_var_loose_bool`) — it over-includes the string `"0"` (falsy but not
/// loose-null-equal), the standard approximation this file already accepts
/// elsewhere. The false direction (`$x != null`) stays exactly as sound as the
/// strict form: if `$x` were null the comparison would be true, so a false
/// result still proves `$x` isn't the null value.
pub(super) fn narrow_var_loose_null(ctx: &mut FlowState, name: &str, is_null: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_null {
        current.narrow_to_falsy()
    } else {
        current.remove_null()
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

/// Property-access counterpart of `narrow_var_loose_null`. Like
/// `narrow_prop_null`, a *nullable* `$obj` receiver means `$obj->prop == null`
/// can be true purely because the receiver is null (PHP 8's plain `->`
/// access on a null receiver warns and evaluates to `null`, same as `?->`) —
/// so the `is_null=true` direction must never mark divergence in that case.
/// The `is_null=false` direction stays sound either way and additionally
/// proves the receiver itself is non-null when it was nullable.
pub(super) fn narrow_prop_loose_null(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_falsy()
    } else {
        current.remove_null()
    };
    let receiver_nullable = ctx.get_var(obj_var).is_nullable();
    let mark_diverges = !receiver_nullable || !is_null;
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
    if !is_null && receiver_nullable {
        narrow_var_null(ctx, obj_var, false);
    }
}

/// Narrow `name` to truthy (`want_truthy`) or falsy, for the loose
/// `$x == true`/`$x == false` idiom — distinct from `narrow_var_bool`, which
/// handles the strict `$x === true`/`$x === false` identity check.
pub(super) fn narrow_var_loose_bool(ctx: &mut FlowState, name: &str, want_truthy: bool) {
    let current = ctx.get_var(name);
    let narrowed = if want_truthy {
        current.narrow_to_truthy()
    } else {
        current.narrow_to_falsy()
    };
    // mark_diverges=false: `impossible_loose_comparison` already owns detecting
    // an always-true/always-false `== true`/`== false` contradiction (and does
    // so conservatively); asserting divergence here would double-report the
    // same fact as an unrelated RedundantCondition.
    set_narrowed(ctx, name, &current, narrowed, false);
}

/// Property-access counterpart of `narrow_var_loose_bool`, for
/// `$this->prop == true`/`false` (or any `$obj->prop` receiver).
pub(super) fn narrow_prop_loose_bool(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    want_truthy: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    let narrowed = if want_truthy {
        current.narrow_to_truthy()
    } else {
        current.narrow_to_falsy()
    };
    // mark_diverges=false: matches narrow_var_loose_bool's rationale — a
    // separate contradiction pass already owns flagging an always-true/false compare.
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

pub(super) fn narrow_var_bool(ctx: &mut FlowState, name: &str, value: bool, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = bool_narrow_type(&current, value, is_value);
    set_narrowed(ctx, name, &current, narrowed, false);
}

/// Property-access counterpart of `narrow_var_bool`, for
/// `$this->prop === true`/`false` (or any `$obj->prop` receiver).
pub(super) fn narrow_prop_bool(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    value: bool,
    is_value: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    let narrowed = bool_narrow_type(&current, value, is_value);
    // mark_diverges=false: matches narrow_var_bool's rationale — a separate
    // contradiction pass already owns flagging an always-true/false compare.
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

/// `TBool` (PHP `bool`) must be split into TTrue/TFalse rather than kept wholesale.
/// e.g. `$x: bool; if ($x === false)` → true-branch should be `false`, not `bool`.
pub(super) fn bool_narrow_type(current: &Type, value: bool, is_value: bool) -> Type {
    let mut narrowed = Type::empty();
    narrowed.from_docblock = current.from_docblock;
    for t in &current.types {
        let keep = match t {
            Atomic::TBool => {
                // Split: narrow TBool to the specific literal being tested.
                if is_value {
                    let lit = if value { Atomic::TTrue } else { Atomic::TFalse };
                    narrowed.add_type(lit);
                } else {
                    let lit = if value { Atomic::TFalse } else { Atomic::TTrue };
                    narrowed.add_type(lit);
                }
                false // handled above — don't fall through
            }
            Atomic::TTrue => is_value == value,
            Atomic::TFalse => is_value != value,
            Atomic::TMixed | Atomic::TScalar => {
                // On a match, substitute the specific literal being tested (mirrors
                // literal_string_narrow_type/literal_int_narrow_type); on a non-match,
                // keep the atom unchanged — excluding one bool value doesn't narrow a
                // type that can also hold non-bool values.
                if is_value {
                    let lit = if value { Atomic::TTrue } else { Atomic::TFalse };
                    narrowed.add_type(lit);
                } else {
                    narrowed.add_type(t.clone());
                }
                false // handled above — don't fall through
            }
            _ => !is_value, // non-bool atoms: keep only when narrowing away
        };
        if keep {
            narrowed.add_type(t.clone());
        }
    }
    narrowed
}

pub(super) fn narrow_var_literal_string(
    ctx: &mut FlowState,
    name: &str,
    value: &str,
    is_value: bool,
) {
    let current = ctx.get_var(name);
    let narrowed = literal_string_narrow_type(&current, value, is_value);
    // For closed-precise types (literal-string unions, ...), an empty result
    // means the exclusion is a genuine contradiction — mark divergence, same
    // as narrow_var_literal_int below.
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    set_narrowed(ctx, name, &current, narrowed, mark_diverges);
}

/// Property-access counterpart of `narrow_var_literal_string`, for
/// `$this->prop === 'literal'` (or any `$obj->prop` receiver).
pub(super) fn narrow_prop_literal_string(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    value: &str,
    is_value: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    let narrowed = literal_string_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

pub(super) fn literal_string_narrow_type(current: &Type, value: &str, is_value: bool) -> Type {
    if is_value {
        let lit: std::sync::Arc<str> = std::sync::Arc::from(value);
        let mut result = Type::empty();
        result.from_docblock = current.from_docblock;
        for t in &current.types {
            match t {
                Atomic::TLiteralString(s) if s.as_ref() == value => {
                    result.add_type(t.clone());
                }
                // Generic/wide string types: the literal could satisfy them — narrow
                // to the literal exactly like every narrower sibling atom below does
                // (TNonEmptyString, TNumericString, TCallableString, ...).
                Atomic::TString | Atomic::TScalar | Atomic::TMixed => {
                    result.add_type(Atomic::TLiteralString(lit.clone()));
                }
                // String subtypes: the literal could satisfy them — narrow to the literal.
                Atomic::TNonEmptyString if !value.is_empty() => {
                    result.add_type(Atomic::TLiteralString(lit.clone()));
                }
                Atomic::TNumericString if is_numeric_string(value) => {
                    result.add_type(Atomic::TLiteralString(lit.clone()));
                }
                Atomic::TCallableString
                | Atomic::TClassString(_)
                | Atomic::TInterfaceString(_)
                | Atomic::TEnumString
                | Atomic::TTraitString => {
                    result.add_type(Atomic::TLiteralString(lit.clone()));
                }
                _ => {} // non-string or non-matching literal — filtered out
            }
        }
        result
    } else {
        current.filter(|t| !matches!(t, Atomic::TLiteralString(s) if s.as_ref() == value))
    }
}

pub(super) fn narrow_var_literal_int(ctx: &mut FlowState, name: &str, value: i64, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = literal_int_narrow_type(&current, value, is_value);
    // For closed-precise types (bounded ranges, named int subtypes, literal unions),
    // an empty result means the exclusion is a genuine contradiction — mark divergence.
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    set_narrowed(ctx, name, &current, narrowed, mark_diverges);
}

/// Whether `a` can safely participate in a loose `== value`/`!= value` int
/// narrowing: either already int-like, or `TNull` where `value != 0` (`null
/// == 0` is PHP's one surprising loose-equality case; `null` compared
/// loosely against any other int literal behaves exactly like the strict
/// comparison already does, so `literal_int_narrow_type`'s int-only atoms
/// handle it correctly once let through). Any other atom (string/float/bool)
/// could loosely equal the same numeric value in a way strict comparison
/// wouldn't, so those cases are left unnarrowed.
pub(super) fn atom_safe_for_loose_int_narrowing(a: &Atomic, value: i64) -> bool {
    a.is_int() || (matches!(a, Atomic::TNull) && value != 0)
}

/// Loose-equality counterpart of `narrow_var_literal_int`, for `$x == 42` /
/// `$x != 42`.
pub(super) fn narrow_var_loose_int(ctx: &mut FlowState, name: &str, value: i64, is_value: bool) {
    let current = ctx.get_var(name);
    if current.types.is_empty()
        || !current
            .types
            .iter()
            .all(|a| atom_safe_for_loose_int_narrowing(a, value))
    {
        return;
    }
    let narrowed = literal_int_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    set_narrowed(ctx, name, &current, narrowed, mark_diverges);
}

/// Property-access counterpart of `narrow_var_literal_int`, for
/// `$this->prop === 42` (or any `$obj->prop` receiver).
pub(super) fn narrow_prop_literal_int(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    value: i64,
    is_value: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    let narrowed = literal_int_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// Loose-equality counterpart of `narrow_prop_literal_int`, for
/// `$this->prop == 42` / `$obj->prop != 42` — property-access sibling of
/// `narrow_var_loose_int`, same safety gate.
pub(super) fn narrow_prop_loose_int(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    value: i64,
    is_value: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.types.is_empty()
        || !current
            .types
            .iter()
            .all(|a| atom_safe_for_loose_int_narrowing(a, value))
    {
        return;
    }
    let narrowed = literal_int_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

pub(super) fn literal_int_narrow_type(current: &Type, value: i64, is_value: bool) -> Type {
    if is_value {
        let int_contains = |min: Option<i64>, max: Option<i64>| {
            min.is_none_or(|lo| value >= lo) && max.is_none_or(|hi| value <= hi)
        };
        let mut result = Type::empty();
        result.from_docblock = current.from_docblock;
        for t in &current.types {
            match t {
                Atomic::TLiteralInt(n) if *n == value => {
                    result.add_type(t.clone());
                }
                // Generic/wide int types: the literal could satisfy them — narrow to
                // the literal exactly like every narrower sibling atom below does
                // (TIntRange, TPositiveInt, TNonNegativeInt, TNegativeInt).
                Atomic::TInt | Atomic::TScalar | Atomic::TNumeric | Atomic::TMixed => {
                    result.add_type(Atomic::TLiteralInt(value));
                }
                Atomic::TIntRange { min, max } if int_contains(*min, *max) => {
                    result.add_type(Atomic::TLiteralInt(value));
                }
                Atomic::TPositiveInt if int_contains(Some(1), None) => {
                    result.add_type(Atomic::TLiteralInt(value));
                }
                Atomic::TNonNegativeInt if int_contains(Some(0), None) => {
                    result.add_type(Atomic::TLiteralInt(value));
                }
                Atomic::TNegativeInt if int_contains(None, Some(-1)) => {
                    result.add_type(Atomic::TLiteralInt(value));
                }
                _ => {}
            }
        }
        result
    } else {
        // Remove the excluded literal from the type.  For named int subtypes and
        // int ranges, also tighten the bound when the excluded value sits exactly
        // at the lower or upper edge.
        let tighten = |min: Option<i64>, max: Option<i64>| {
            let (new_min, new_max) = if min == Some(value) {
                match value.checked_add(1) {
                    Some(v) => (Some(v), max),
                    // value was i64::MAX — no larger value exists, so excluding it
                    // leaves the range empty. An inverted (lo > hi) range is the
                    // existing empty-range encoding the TIntRange caller below
                    // already detects and skips; `None` here would wrongly read as
                    // "unbounded" instead.
                    None => (Some(i64::MAX), Some(i64::MIN)),
                }
            } else if max == Some(value) {
                match value.checked_sub(1) {
                    Some(v) => (min, Some(v)),
                    None => (Some(i64::MAX), Some(i64::MIN)),
                }
            } else {
                return None; // excluded value is not on an edge — keep as-is
            };
            // Canonicalise tightened ranges back to named subtypes.
            let atom = match (new_min, new_max) {
                (Some(1), None) => Atomic::TPositiveInt,
                (Some(0), None) => Atomic::TNonNegativeInt,
                (None, Some(-1)) => Atomic::TNegativeInt,
                (None, None) => Atomic::TInt,
                (min, max) => Atomic::TIntRange { min, max },
            };
            Some(atom)
        };
        let mut result = Type::empty();
        result.from_docblock = current.from_docblock;
        for t in &current.types {
            match t {
                Atomic::TLiteralInt(n) if *n == value => {} // excluded
                Atomic::TIntRange { min, max } => {
                    if let Some(tightened) = tighten(*min, *max) {
                        // Skip the atom entirely when tightening produced an empty
                        // range (lo > hi). This correctly empties the result for
                        // single-point ranges like `int<1,1>` when excluding 1.
                        let is_empty_range = matches!(
                            &tightened,
                            Atomic::TIntRange { min: Some(lo), max: Some(hi) } if lo > hi
                        );
                        if !is_empty_range {
                            result.add_type(tightened);
                        }
                    } else {
                        result.add_type(t.clone());
                    }
                }
                Atomic::TPositiveInt => {
                    if let Some(tightened) = tighten(Some(1), None) {
                        result.add_type(tightened);
                    } else {
                        result.add_type(t.clone());
                    }
                }
                Atomic::TNonNegativeInt => {
                    if let Some(tightened) = tighten(Some(0), None) {
                        result.add_type(tightened);
                    } else {
                        result.add_type(t.clone());
                    }
                }
                Atomic::TNegativeInt => {
                    if let Some(tightened) = tighten(None, Some(-1)) {
                        result.add_type(tightened);
                    } else {
                        result.add_type(t.clone());
                    }
                }
                _ => result.add_type(t.clone()),
            }
        }
        result
    }
}
