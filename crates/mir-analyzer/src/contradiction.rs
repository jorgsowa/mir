//! Detection of statically-impossible comparisons against docblock-derived
//! types, surfaced as `DocblockTypeContradiction`.
//!
//! A docblock can pin a variable to a type that makes a later comparison or
//! assertion provably unsatisfiable — e.g. `@param int<5, max> $a` followed by
//! `assert($a < 4)`, or `@assert "a"|"b" $s` followed by `if ($s === "c")`.
//! We only flag comparisons against a literal when the controlling type came
//! from a docblock (`Type::from_docblock`); native-typed redundancy is left to
//! the existing `RedundantCondition` / `TypeDoesNotContainType` machinery.
//!
//! The analysis is deliberately conservative: an atomic is treated as "could
//! still match the literal" unless it is *definitely* incompatible, so unknown
//! or open atomics never produce a false contradiction.
use php_ast::ast::BinaryOp;
use php_ast::owned::{Expr, ExprKind};

use mir_types::{Atomic, Type};

use crate::flow_state::FlowState;

enum Lit {
    Int(i64),
    Str(String),
}

fn extract_var(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => extract_var(inner),
        _ => None,
    }
}

fn extract_lit(expr: &Expr) -> Option<Lit> {
    match &expr.kind {
        ExprKind::Int(n) => Some(Lit::Int(*n)),
        ExprKind::String(s) => Some(Lit::Str(s.to_string())),
        ExprKind::Parenthesized(inner) => extract_lit(inner),
        _ => None,
    }
}

fn lit_repr(lit: &Lit) -> String {
    match lit {
        Lit::Int(n) => n.to_string(),
        Lit::Str(s) => format!("\"{s}\""),
    }
}

/// Inclusive integer bounds of `ty` when *every* member is integer-like.
/// `None` on a side means unbounded; the whole result is `None` when any
/// member is not an integer (so ordering checks bail out rather than guess).
fn int_bounds(ty: &Type) -> Option<(Option<i64>, Option<i64>)> {
    if ty.types.is_empty() {
        return None;
    }
    let mut min: Option<i64> = Some(i64::MAX);
    let mut max: Option<i64> = Some(i64::MIN);
    for a in &ty.types {
        let (lo, hi) = match a {
            Atomic::TLiteralInt(n) => (Some(*n), Some(*n)),
            Atomic::TIntRange { min, max } => (*min, *max),
            Atomic::TInt | Atomic::TNumeric => (None, None),
            Atomic::TPositiveInt => (Some(1), None),
            Atomic::TNonNegativeInt => (Some(0), None),
            Atomic::TNegativeInt => (None, Some(-1)),
            _ => return None,
        };
        min = match (min, lo) {
            (Some(m), Some(l)) => Some(m.min(l)),
            _ => None,
        };
        max = match (max, hi) {
            (Some(m), Some(h)) => Some(m.max(h)),
            _ => None,
        };
    }
    Some((min, max))
}

/// Whether an atomic could compare strictly-equal (`===`) to `lit`. Returns
/// `true` for anything not *definitely* incompatible (mixed, scalar, the
/// matching general kind, objects, arrays, …).
fn atomic_can_equal(a: &Atomic, lit: &Lit) -> bool {
    match lit {
        Lit::Int(n) => match a {
            Atomic::TLiteralInt(m) => m == n,
            Atomic::TIntRange { min, max } => {
                min.is_none_or(|lo| lo <= *n) && max.is_none_or(|hi| *n <= hi)
            }
            Atomic::TPositiveInt => *n >= 1,
            Atomic::TNonNegativeInt => *n >= 0,
            Atomic::TNegativeInt => *n <= -1,
            // Scalars that can never be (strictly) an integer.
            Atomic::TString
            | Atomic::TNonEmptyString
            | Atomic::TNumericString
            | Atomic::TLiteralString(_)
            | Atomic::TClassString { .. }
            | Atomic::TFloat
            | Atomic::TLiteralFloat(..)
            | Atomic::TBool
            | Atomic::TTrue
            | Atomic::TFalse
            | Atomic::TNull => false,
            _ => true,
        },
        Lit::Str(s) => match a {
            Atomic::TLiteralString(t) => t.as_ref() == s.as_str(),
            // Scalars that can never be (strictly) a string.
            Atomic::TInt
            | Atomic::TLiteralInt(_)
            | Atomic::TIntRange { .. }
            | Atomic::TPositiveInt
            | Atomic::TNonNegativeInt
            | Atomic::TNegativeInt
            | Atomic::TFloat
            | Atomic::TLiteralFloat(..)
            | Atomic::TBool
            | Atomic::TTrue
            | Atomic::TFalse
            | Atomic::TNull => false,
            _ => true,
        },
    }
}

fn can_equal(ty: &Type, lit: &Lit) -> bool {
    ty.types.iter().any(|a| atomic_can_equal(a, lit))
}

/// Whether `ty` is a closed set precise enough that an out-of-set comparison is
/// a genuine contradiction: a bounded int range (`int<5, max>`) or a union of
/// at least two literals (`1|2|3`, `"a"|"b"`).
///
/// A *lone* literal (`0`, `"x"`) is deliberately rejected: it is too often a
/// loop-carried under-approximation — e.g. `$i = 0; while (…) { $r = $i++;
/// if ($r > 3) … }` infers `$r` as `0` because loop-variable widening is not
/// modelled, which would otherwise flag the live `$r > 3` as impossible.
fn is_closed_precise(ty: &Type) -> bool {
    if ty.types.is_empty() {
        return false;
    }
    let all_precise = ty.types.iter().all(|a| match a {
        Atomic::TLiteralInt(_) | Atomic::TLiteralString(_) | Atomic::TLiteralFloat(..) => true,
        Atomic::TIntRange { min, max } => min.is_some() || max.is_some(),
        Atomic::TPositiveInt | Atomic::TNonNegativeInt | Atomic::TNegativeInt => true,
        _ => false,
    });
    if !all_precise {
        return false;
    }
    let has_range = ty.types.iter().any(|a| {
        matches!(
            a,
            Atomic::TIntRange { .. }
                | Atomic::TPositiveInt
                | Atomic::TNonNegativeInt
                | Atomic::TNegativeInt
        )
    });
    has_range || ty.types.len() >= 2
}

/// Whether `$var op N` can never hold, given `$var`'s integer bounds.
fn ordering_impossible(ty: &Type, n: i64, op: BinaryOp) -> bool {
    let Some((min, max)) = int_bounds(ty) else {
        return false;
    };
    match op {
        // every value ≥ min, so none is `< n` when min ≥ n
        BinaryOp::Less => min.is_some_and(|lo| lo >= n),
        BinaryOp::LessOrEqual => min.is_some_and(|lo| lo > n),
        BinaryOp::Greater => max.is_some_and(|hi| hi <= n),
        BinaryOp::GreaterOrEqual => max.is_some_and(|hi| hi < n),
        _ => false,
    }
}

/// Flip a comparison operator so the variable is conceptually on the left
/// (used when the source wrote `N < $var`).
fn flip(op: BinaryOp) -> BinaryOp {
    match op {
        BinaryOp::Less => BinaryOp::Greater,
        BinaryOp::LessOrEqual => BinaryOp::GreaterOrEqual,
        BinaryOp::Greater => BinaryOp::Less,
        BinaryOp::GreaterOrEqual => BinaryOp::LessOrEqual,
        other => other,
    }
}

fn op_str(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Identical => "===",
        BinaryOp::Less => "<",
        BinaryOp::LessOrEqual => "<=",
        BinaryOp::Greater => ">",
        BinaryOp::GreaterOrEqual => ">=",
        _ => "?",
    }
}

// ---------------------------------------------------------------------------
// gettype() switch/match dead-arm analysis (UnevaluatedCode)
// ---------------------------------------------------------------------------

/// The exact strings `gettype()` can return. A `case`/arm comparing against any
/// other string is dead — most often `"int"`/`"float"`/`"bool"`/`"null"` where
/// the author meant the longer canonical form.
const GETTYPE_VALUES: &[&str] = &[
    "boolean",
    "integer",
    "double",
    "string",
    "array",
    "object",
    "resource",
    "resource (closed)",
    "NULL",
    "unknown type",
];

/// Is `s` a string `gettype()` can actually return?
pub(crate) fn gettype_is_valid(s: &str) -> bool {
    GETTYPE_VALUES.contains(&s)
}

/// The canonical `gettype()` value an author likely meant when they wrote an
/// invalid one (`"int"` → `"integer"`).
pub(crate) fn gettype_suggestion(s: &str) -> Option<&'static str> {
    Some(match s {
        "int" | "long" => "integer",
        "float" | "real" => "double",
        "bool" => "boolean",
        "null" | "Null" | "none" => "NULL",
        _ => return None,
    })
}

/// If `expr` is a `gettype($x)` call, return its argument expression.
pub(crate) fn gettype_call_arg(expr: &Expr) -> Option<&Expr> {
    let ExprKind::FunctionCall(call) = &expr.kind else {
        return None;
    };
    let name = match &call.name.kind {
        ExprKind::Identifier(n) => n.as_ref(),
        _ => return None,
    };
    if !name
        .trim_start_matches('\\')
        .eq_ignore_ascii_case("gettype")
    {
        return None;
    }
    call.args.first().map(|a| &a.value)
}

/// The set of `gettype()` strings a value of type `ty` could yield, or `None`
/// when the type is too open (mixed/scalar) to decide — in which case callers
/// must not report a dead arm on the "type can't yield this" basis.
pub(crate) fn gettype_possible_values(ty: &Type) -> Option<Vec<&'static str>> {
    if ty.types.is_empty() {
        return None;
    }
    let mut out: Vec<&'static str> = Vec::new();
    for a in &ty.types {
        let v = match a {
            Atomic::TInt
            | Atomic::TLiteralInt(_)
            | Atomic::TIntRange { .. }
            | Atomic::TPositiveInt
            | Atomic::TNonNegativeInt
            | Atomic::TNegativeInt => "integer",
            Atomic::TFloat | Atomic::TLiteralFloat(..) => "double",
            Atomic::TString
            | Atomic::TLiteralString(_)
            | Atomic::TNonEmptyString
            | Atomic::TNumericString
            | Atomic::TClassString { .. } => "string",
            Atomic::TBool | Atomic::TTrue | Atomic::TFalse => "boolean",
            Atomic::TNull => "NULL",
            // Anything open or that we don't model precisely: bail out.
            _ => return None,
        };
        if !out.contains(&v) {
            out.push(v);
        }
    }
    Some(out)
}

/// If `expr` is a comparison of a docblock-typed variable against a literal
/// that can never be satisfied, return `(rendered_comparison, declared_type)`
/// for a `DocblockTypeContradiction`.
pub(crate) fn impossible_comparison(expr: &Expr, ctx: &FlowState) -> Option<(String, String)> {
    let ExprKind::Binary(b) = &expr.kind else {
        return None;
    };
    if !matches!(
        b.op,
        BinaryOp::Identical
            | BinaryOp::Less
            | BinaryOp::LessOrEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterOrEqual
    ) {
        return None;
    }

    let (var_name, lit, var_on_left) =
        if let (Some(v), Some(l)) = (extract_var(&b.left), extract_lit(&b.right)) {
            (v, l, true)
        } else if let (Some(v), Some(l)) = (extract_var(&b.right), extract_lit(&b.left)) {
            (v, l, false)
        } else {
            return None;
        };

    let ty = ctx.get_var(&var_name);
    // Only judge against a *closed, precise* type — a literal union (`1|2|3`,
    // `"a"|"b"`) or a bounded int range (`int<5, max>`). These only arise from
    // docblocks (`@param`, `@assert`) or literal/range inference, never from a
    // bare native `int`/`string` hint, so a contradiction here is real and not
    // an artefact of imprecise widening.
    if !is_closed_precise(&ty) {
        return None;
    }

    let impossible = match b.op {
        BinaryOp::Identical => !can_equal(&ty, &lit),
        _ => {
            let Lit::Int(n) = &lit else {
                return None;
            };
            let op = if var_on_left { b.op } else { flip(b.op) };
            ordering_impossible(&ty, *n, op)
        }
    };
    if !impossible {
        return None;
    }

    let var_s = format!("${var_name}");
    let lit_s = lit_repr(&lit);
    let rendered = if var_on_left {
        format!("{var_s} {} {lit_s}", op_str(b.op))
    } else {
        format!("{lit_s} {} {var_s}", op_str(b.op))
    };
    Some((rendered, format!("{ty}")))
}
