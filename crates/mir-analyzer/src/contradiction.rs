//! Detection of statically-impossible comparisons, surfaced as either
//! `DocblockTypeContradiction` or `ImpossibleIdenticalComparison`.
//!
//! **DocblockTypeContradiction** — A docblock can pin a variable to a type
//! that makes a later comparison provably unsatisfiable, e.g. `@param
//! int<5, max> $a` with `assert($a < 4)`, or `@assert "a"|"b" $s` with
//! `if ($s === "c")`. Only fired when the type is *closed and precise*
//! (bounded range or multi-literal union from a docblock).
//!
//! **ImpossibleIdenticalComparison** — A `===`/`!==` between two inferred
//! types that belong to different PHP type families (int vs string, object vs
//! null, …) is always constant and almost certainly a logic bug. Fired from
//! `expr/binary.rs` for every `===`/`!==` node whose operand types are
//! categorically disjoint.
//!
//! Both analyses are conservative: open atomics (mixed, scalar, callable,
//! template params) are never treated as disjoint.
use php_ast::ast::{BinaryOp, UnaryPrefixOp};
use php_ast::owned::{Expr, ExprKind};

use mir_types::{Atomic, Type};

use crate::flow_state::FlowState;
use crate::php_version::PhpVersion;

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
        // PHP parses `-1` as UnaryPrefix(Negate, Int(1)), not Int(-1).
        ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::Negate => {
            if let ExprKind::Int(n) = &u.operand.kind {
                n.checked_neg().map(Lit::Int)
            } else {
                None
            }
        }
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
            | Atomic::TIntegralFloat
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
            | Atomic::TIntegralFloat
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
pub(crate) fn is_closed_precise(ty: &Type) -> bool {
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
            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..) => "double",
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

// ---------------------------------------------------------------------------
// Strict-equality disjointness (ImpossibleIdenticalComparison)
// ---------------------------------------------------------------------------

/// PHP type family for `===` identity: two values can only be identical if
/// they belong to the same family.
#[derive(PartialEq)]
enum TypeFamily {
    Int,
    Float,
    String,
    Bool,
    Null,
    Array,
    Object,
}

/// Map an atomic to its PHP type family for `===` purposes.
/// Returns `None` for open / unknown atomics (mixed, scalar, numeric,
/// callable, template params, conditionals) — callers treat `None` as
/// "could be anything" and return `true` conservatively.
fn atomic_family(a: &Atomic) -> Option<TypeFamily> {
    Some(match a {
        Atomic::TInt
        | Atomic::TLiteralInt(_)
        | Atomic::TIntRange { .. }
        | Atomic::TPositiveInt
        | Atomic::TNegativeInt
        | Atomic::TNonNegativeInt => TypeFamily::Int,

        Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..) => TypeFamily::Float,

        Atomic::TString
        | Atomic::TLiteralString(_)
        | Atomic::TCallableString
        | Atomic::TClassString(_)
        | Atomic::TNonEmptyString
        | Atomic::TNumericString
        | Atomic::TInterfaceString(_)
        | Atomic::TEnumString
        | Atomic::TTraitString => TypeFamily::String,

        Atomic::TBool | Atomic::TTrue | Atomic::TFalse => TypeFamily::Bool,

        Atomic::TNull => TypeFamily::Null,

        Atomic::TArray { .. }
        | Atomic::TList { .. }
        | Atomic::TNonEmptyArray { .. }
        | Atomic::TNonEmptyList { .. }
        | Atomic::TKeyedArray { .. } => TypeFamily::Array,

        Atomic::TObject
        | Atomic::TNamedObject { .. }
        | Atomic::TStaticObject { .. }
        | Atomic::TSelf { .. }
        | Atomic::TParent { .. }
        | Atomic::TClosure { .. }
        | Atomic::TLiteralEnumCase { .. }
        | Atomic::TIntersection { .. } => TypeFamily::Object,

        // Open / unknown — cannot determine family.
        Atomic::TMixed
        | Atomic::TScalar
        | Atomic::TNumeric
        | Atomic::TVoid
        | Atomic::TNever
        | Atomic::TCallable { .. }
        | Atomic::TTemplateParam { .. }
        | Atomic::TConditional { .. } => return None,
    })
}

/// Can two atomics ever be strictly identical (`===`)?
///
/// Returns `true` conservatively when either atomic has no definite family.
/// Within the same family, also checks specific literal disjointness
/// (`TTrue !== TFalse`, `TLiteralInt(5) !== TLiteralInt(6)`, etc.).
fn atomics_can_be_identical(left: &Atomic, right: &Atomic) -> bool {
    let (Some(lf), Some(rf)) = (atomic_family(left), atomic_family(right)) else {
        return true;
    };
    if lf != rf {
        return false;
    }
    // Same family — check specific literal disjointness.
    match (left, right) {
        (Atomic::TTrue, Atomic::TFalse) | (Atomic::TFalse, Atomic::TTrue) => false,
        (Atomic::TLiteralInt(a), Atomic::TLiteralInt(b)) => a == b,
        (Atomic::TLiteralString(a), Atomic::TLiteralString(b)) => a == b,
        (Atomic::TLiteralFloat(a1, a2), Atomic::TLiteralFloat(b1, b2)) => a1 == b1 && a2 == b2,
        (
            Atomic::TLiteralEnumCase {
                enum_fqcn: ef1,
                case_name: cn1,
            },
            Atomic::TLiteralEnumCase {
                enum_fqcn: ef2,
                case_name: cn2,
            },
        ) => ef1 == ef2 && cn1 == cn2,
        _ => true,
    }
}

/// Whether any atom in `left` could ever be strictly identical (`===`) to any
/// atom in `right`.
///
/// Returns `true` (conservative: comparison is possible) when either type is
/// empty (never/unknown) or when any atom pair has an unknown family.
pub(crate) fn types_can_be_identical(left: &Type, right: &Type) -> bool {
    if left.types.is_empty() || right.types.is_empty() {
        return true;
    }
    left.types.iter().any(|la| {
        right
            .types
            .iter()
            .any(|ra| atomics_can_be_identical(la, ra))
    })
}

// ---------------------------------------------------------------------------
// Loose-equality disjointness (ImpossibleLooseComparison)
// ---------------------------------------------------------------------------

fn is_open_atomic(a: &Atomic) -> bool {
    matches!(
        a,
        Atomic::TMixed
            | Atomic::TScalar
            | Atomic::TNumeric
            | Atomic::TVoid
            | Atomic::TNever
            | Atomic::TCallable { .. }
            | Atomic::TTemplateParam { .. }
            | Atomic::TConditional { .. }
    )
}

fn is_object_atomic(a: &Atomic) -> bool {
    matches!(
        a,
        Atomic::TObject
            | Atomic::TNamedObject { .. }
            | Atomic::TStaticObject { .. }
            | Atomic::TSelf { .. }
            | Atomic::TParent { .. }
            | Atomic::TClosure { .. }
            | Atomic::TLiteralEnumCase { .. }
            | Atomic::TIntersection { .. }
    )
}

fn is_array_atomic(a: &Atomic) -> bool {
    matches!(
        a,
        Atomic::TArray { .. }
            | Atomic::TList { .. }
            | Atomic::TNonEmptyArray { .. }
            | Atomic::TNonEmptyList { .. }
            | Atomic::TKeyedArray { .. }
    )
}

fn is_nonempty_array_atomic(a: &Atomic) -> bool {
    matches!(
        a,
        Atomic::TNonEmptyArray { .. } | Atomic::TNonEmptyList { .. }
    )
}

fn is_int_atomic(a: &Atomic) -> bool {
    matches!(
        a,
        Atomic::TInt
            | Atomic::TLiteralInt(_)
            | Atomic::TIntRange { .. }
            | Atomic::TPositiveInt
            | Atomic::TNonNegativeInt
            | Atomic::TNegativeInt
    )
}

fn is_float_atomic(a: &Atomic) -> bool {
    matches!(
        a,
        Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
    )
}

/// Whether `s` could be a PHP numeric string (conservative: returns `true` when unsure).
///
/// PHP considers a string numeric if it consists of optional whitespace, an optional
/// sign, digits, and an optional decimal / exponent part. Used to determine whether
/// a literal-string atom can be loosely equal to an integer or float.
fn is_php_numeric_string(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    if t.parse::<i64>().is_ok() {
        return true;
    }
    // Reject inf/nan: PHP does not consider those numeric strings.
    if let Ok(f) = t.parse::<f64>() {
        return f.is_finite();
    }
    false
}

/// Can two atomics ever be loosely equal (`==`) in PHP?
///
/// Conservative: returns `true` for open atomics (mixed, scalar, etc.) and for
/// complex scalar-vs-scalar coercions (null vs 0, "" vs false, etc.) — PHP's
/// type juggling is too intricate to enumerate safely there.
///
/// Only returns `false` for provably-impossible cases:
/// - any object vs null, false, int, float, string, or array
/// - any array vs null, int, float, string, or object
/// - a non-empty array vs false (non-empty arrays are always truthy)
/// - a non-numeric literal string vs any integer/float (PHP 8.0+)
/// - a non-numeric literal string vs a non-zero literal integer (all versions)
fn atomics_can_be_loose_equal(left: &Atomic, right: &Atomic, php_version: PhpVersion) -> bool {
    if is_open_atomic(left) || is_open_atomic(right) {
        return true;
    }

    let left_obj = is_object_atomic(left);
    let right_obj = is_object_atomic(right);
    let left_arr = is_array_atomic(left);
    let right_arr = is_array_atomic(right);

    // Object vs ...
    if left_obj || right_obj {
        let other = if left_obj { right } else { left };
        return match other {
            // obj == another object: possible (same class, same identity)
            _ if is_object_atomic(other) => true,
            // obj == true: always true (objects are truthy) — possible
            Atomic::TTrue | Atomic::TBool => true,
            // Everything else: null, false, int, float, string, array → always false
            _ => false,
        };
    }

    // Array vs ...
    if left_arr || right_arr {
        let arr_side = if left_arr { left } else { right };
        let other = if left_arr { right } else { left };
        return match other {
            // arr == another array: possible (same contents)
            _ if is_array_atomic(other) => true,
            // arr == true: possible (non-empty array is truthy)
            Atomic::TTrue | Atomic::TBool => true,
            // arr == false: only possible when the array could be empty;
            // a non-empty array is always truthy so == false is impossible
            Atomic::TFalse => !is_nonempty_array_atomic(arr_side),
            // arr vs null/int/float/string/object → always false in PHP
            _ => false,
        };
    }

    // Scalar vs scalar: most coercions are complex, but literal-string vs
    // integer/float cases are statically decidable.
    //
    // PHP 8.0+ changed string-vs-int comparison: when one operand is a
    // non-numeric string, the integer is converted *to* string instead of
    // the string being converted to an integer.  A non-numeric literal like
    // "foo" can therefore never equal any integer/float in PHP 8+.
    //
    // In all PHP versions: a non-numeric string converts to int(0) under the
    // PHP 7 rules, so comparing it against a non-zero literal int is still
    // impossible regardless of version.
    if php_version >= PhpVersion::new(8, 0) {
        if let Atomic::TLiteralString(s) = left {
            if !is_php_numeric_string(s) && (is_int_atomic(right) || is_float_atomic(right)) {
                return false;
            }
        }
        if let Atomic::TLiteralString(s) = right {
            if !is_php_numeric_string(s) && (is_int_atomic(left) || is_float_atomic(left)) {
                return false;
            }
        }
    } else {
        // PHP < 8: non-numeric string → (int)0 under comparison, so it can
        // equal 0 but never any non-zero literal integer.
        match (left, right) {
            (Atomic::TLiteralString(s), Atomic::TLiteralInt(n))
            | (Atomic::TLiteralInt(n), Atomic::TLiteralString(s))
                if !is_php_numeric_string(s) && *n != 0 =>
            {
                return false;
            }
            _ => {}
        }
    }

    true
}

/// Whether any atom in `left` could ever be loosely equal (`==`) to any atom
/// in `right`.
///
/// Returns `true` conservatively when either type is empty or when any atom
/// pair might be loosely equal.
pub(crate) fn types_can_be_loose_equal(left: &Type, right: &Type, php_version: PhpVersion) -> bool {
    if left.types.is_empty() || right.types.is_empty() {
        return true;
    }
    left.types.iter().any(|la| {
        right
            .types
            .iter()
            .any(|ra| atomics_can_be_loose_equal(la, ra, php_version))
    })
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
