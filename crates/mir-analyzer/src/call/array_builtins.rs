use std::sync::Arc;

use indexmap::IndexMap;
use php_ast::owned::{Expr, ExprKind};
use php_ast::Span;

use mir_issues::{IssueKind, Severity};
use mir_types::atomic::ArrayKey;
use mir_types::{Atomic, Type};

use crate::expr::ExpressionAnalyzer;
use crate::flow_state::FlowState;

/// True if `arg` is the array-callable pair form `[$obj, 'method']` /
/// `['Class', 'method']` — a 2-element shape (keys 0 and 1) whose first element
/// is an object / class-string / string and whose second element is a string.
/// PHP accepts this anywhere a `callable` is expected, including the `is_list`
/// shape produced by an `[$this, 'm']` literal.
pub(crate) fn is_callable_array_pair(arg: &Type) -> bool {
    arg.types.iter().any(|a| {
        let Atomic::TKeyedArray { properties, .. } = a else {
            return false;
        };
        if properties.len() != 2 {
            return false;
        }
        let first = properties.get(&mir_types::atomic::ArrayKey::Int(0));
        let second = properties.get(&mir_types::atomic::ArrayKey::Int(1));
        let (Some(first), Some(second)) = (first, second) else {
            return false;
        };
        // The first element must be an object (`[$this, 'm']`) or a
        // class-string. A plain/literal string is NOT accepted here: a literal
        // like `["one", "two"]` is only callable if "one" names a real class,
        // which this db-less predicate can't verify — leave that to the regular
        // checks so a non-class string pair is still rejected.
        let first_ok = first.ty.contains(|t| {
            matches!(
                t,
                Atomic::TNamedObject { .. }
                    | Atomic::TObject
                    | Atomic::TSelf { .. }
                    | Atomic::TStaticObject { .. }
                    | Atomic::TClassString(_)
                    | Atomic::TMixed
            )
        });
        let second_ok = second.ty.contains(|t| {
            matches!(
                t,
                Atomic::TString | Atomic::TNonEmptyString | Atomic::TLiteralString(_)
            )
        });
        first_ok && second_ok
    })
}

/// Validate array_map callback: arity must match the number of arrays passed.
/// array_map(callback, array1, array2, ...) → callback receives one element from each array.
pub(crate) fn check_array_map_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    arg_types: &[Type],
    arg_spans: &[Span],
) {
    if arg_types.is_empty() || arg_spans.is_empty() {
        return;
    }

    let callback_ty = &arg_types[0];
    let callback_span = arg_spans[0];

    if !super::callable::is_valid_callable_type(callback_ty) {
        ea.emit(
            IssueKind::InvalidArgument {
                param: "callback".to_string(),
                fn_name: "array_map".to_string(),
                expected: "callable".to_string(),
                actual: callback_ty.to_string(),
            },
            Severity::Error,
            callback_span,
        );
        return;
    }

    super::callable::record_callable_string_ref(ea, callback_ty, callback_span);

    if arg_types.len() > 1 {
        validate_callback_arity(ea, callback_ty, callback_span, arg_types.len() - 1);
    }
}

/// Generic callback arity validation for any function.
/// Emits TooFewArguments or TooManyArguments if the callback doesn't match expected arity.
fn validate_callback_arity(
    ea: &mut ExpressionAnalyzer<'_>,
    callback_ty: &Type,
    callback_span: Span,
    expected_arity: usize,
) {
    if let Some(params) = super::callable::extract_callable_params(callback_ty, ea) {
        let required_count = params
            .iter()
            .filter(|p| !p.is_optional && !p.is_variadic)
            .count();
        let has_variadic = params.iter().any(|p| p.is_variadic);
        let max_params = params.len();

        if required_count > expected_arity {
            let fn_name = callback_name_for_diagnostic(callback_ty);
            ea.emit(
                IssueKind::TooFewArguments {
                    fn_name,
                    expected: required_count,
                    actual: expected_arity,
                },
                Severity::Error,
                callback_span,
            );
        } else if !has_variadic && max_params < expected_arity {
            let fn_name = callback_name_for_diagnostic(callback_ty);
            ea.emit(
                IssueKind::TooManyArguments {
                    fn_name,
                    expected: max_params,
                    actual: expected_arity,
                },
                Severity::Error,
                callback_span,
            );
        }
    }
}

// PHP array_filter mode constants
const ARRAY_FILTER_USE_BOTH: i64 = 1; // pass value and key to callback
const ARRAY_FILTER_USE_KEY: i64 = 2; // pass only key to callback

/// Validate array_filter callback.
/// Expected arity depends on mode (arg_types[2]):
/// - ARRAY_FILTER_USE_BOTH (1): 2 args (value, key)
/// - ARRAY_FILTER_USE_KEY (2): 1 arg (key)
/// - else/missing: 1 arg (value)
pub(crate) fn check_array_filter_callback(
    ea: &mut ExpressionAnalyzer<'_>,
    arg_types: &[Type],
    arg_spans: &[Span],
) {
    if arg_types.len() < 2 || arg_spans.len() < 2 {
        return;
    }

    let callback_ty = &arg_types[1];
    let callback_span = arg_spans[1];

    if !super::callable::is_valid_callable_type(callback_ty) {
        ea.emit(
            IssueKind::InvalidArgument {
                param: "callback".to_string(),
                fn_name: "array_filter".to_string(),
                expected: "callable".to_string(),
                actual: callback_ty.to_string(),
            },
            Severity::Error,
            callback_span,
        );
        return;
    }

    super::callable::record_callable_string_ref(ea, callback_ty, callback_span);

    let expected_arity = if arg_types.len() > 2 {
        match arg_types[2].types.first() {
            Some(Atomic::TLiteralInt(ARRAY_FILTER_USE_BOTH)) => 2,
            Some(Atomic::TLiteralInt(ARRAY_FILTER_USE_KEY)) => 1,
            _ => 1,
        }
    } else {
        1
    };

    if let Some(params) = super::callable::extract_callable_params(callback_ty, ea) {
        let required_count = params
            .iter()
            .filter(|p| !p.is_optional && !p.is_variadic)
            .count();
        let has_variadic = params.iter().any(|p| p.is_variadic);
        let max_params = params.len();

        if required_count > expected_arity || (!has_variadic && max_params < expected_arity) {
            let actual_count = if has_variadic {
                required_count
            } else {
                max_params
            };
            let expected_plural = if expected_arity == 1 { "" } else { "s" };
            let actual_plural = if actual_count == 1 { "" } else { "s" };
            ea.emit(
                IssueKind::InvalidArgument {
                    param: "callback".to_string(),
                    fn_name: "array_filter".to_string(),
                    expected: format!(
                        "callable accepting {} argument{}",
                        expected_arity, expected_plural
                    ),
                    actual: format!(
                        "callable accepting {} argument{}",
                        actual_count, actual_plural
                    ),
                },
                Severity::Error,
                callback_span,
            );
        }
    }
}

/// Extract the return type a callable union resolves to, if statically known:
/// - `TClosure` / `TCallable` carry their return type directly.
/// - `TLiteralString` is resolved as a function name and its declared return is used.
/// - `TIntersection` is searched part by part.
/// - A bare opaque `TCallable` (no declared signature) falls through to
///   `resolve_opaque_callback_via_callers` — if `callback_expr` is a
///   variable matching one of the *enclosing* function's own opaque
///   `callable` parameters, resolve its return type from how that
///   function's own callers actually invoke it.
///
/// Returns `None` when the callback's return type cannot be determined
/// (unknown string, `null`, an opaque callable with no resolvable callers,
/// …) so callers can fall back to the generic stub return type rather than
/// inventing a wrong element type.
fn callable_return_type(
    union: &Type,
    ea: &ExpressionAnalyzer<'_>,
    ctx: &FlowState,
    callback_expr: Option<&Expr>,
) -> Option<Type> {
    for atomic in &union.types {
        match atomic {
            Atomic::TClosure { data } => return Some(data.return_type.clone()),
            Atomic::TCallable {
                return_type: Some(rt),
                ..
            } => return Some((**rt).clone()),
            Atomic::TCallable {
                return_type: None, ..
            } => {
                if let Some(rt) = resolve_opaque_callback_via_callers(ea, ctx, callback_expr) {
                    return Some(rt);
                }
            }
            Atomic::TLiteralString(fn_name) if !fn_name.is_empty() => {
                let here = crate::db::Fqcn::from_str(ea.db, fn_name.as_ref());
                if let Some(f) = crate::db::find_function(ea.db, here) {
                    if let Some(rt) = &f.return_type {
                        return Some((**rt).clone());
                    }
                }
            }
            Atomic::TIntersection { parts } => {
                for part in parts.iter() {
                    if let Some(rt) = callable_return_type(part, ea, ctx, callback_expr) {
                        return Some(rt);
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// When `callback_expr` is a bare `$variable` matching one of the enclosing
/// function's own declared parameters, and that parameter's declared type is
/// itself a bare opaque `callable` (no docblock refinement), resolve its
/// return type from how the enclosing function's own callers actually invoke
/// it. Scoped to plain function parameters — see the `opaque_callback`
/// module docs for why method parameters are out of scope.
fn resolve_opaque_callback_via_callers(
    ea: &ExpressionAnalyzer<'_>,
    ctx: &FlowState,
    callback_expr: Option<&Expr>,
) -> Option<Type> {
    let ExprKind::Variable(name) = &callback_expr?.kind else {
        return None;
    };
    let var_name = name.trim_start_matches('$');
    let fqn = ctx.current_function_fqn.as_ref()?;
    let f = crate::db::find_function(ea.db, crate::db::Fqcn::from_str(ea.db, fqn))?;
    let index = f.params.iter().position(|p| p.name.as_ref() == var_name)?;
    // Only meaningful when the *declared* param type is itself a bare opaque
    // callable — if it already carries a signature, the arms above already
    // resolved it, so this avoids re-deriving a perfectly good declared
    // signature through the (slower) caller scan.
    let is_bare_callable = f.params[index].ty.as_ref().is_some_and(|t| {
        t.types.len() == 1
            && matches!(
                &t.types[0],
                Atomic::TCallable {
                    return_type: None,
                    ..
                }
            )
    });
    if !is_bare_callable {
        return None;
    }
    let callee = super::opaque_callback::CalleeKey::Function(Arc::clone(fqn));
    super::opaque_callback::opaque_callback_return_type(ea.db, &callee, index as u16)
}

/// Infer the return type of `array_fill($start_index, $count, $value)`.
///
/// When the count argument is provably >= 1, the result is
/// `non-empty-list<T>` if `$start_index` is provably `0`, or
/// `non-empty-array<int, T>` if it's provably nonzero (T is the type of
/// `$value`). Falls through to `None` otherwise so the stub's generic
/// `array` is used.
pub(crate) fn array_fill_return_type(arg_types: &[Type]) -> Option<Type> {
    let start = arg_types.first()?;
    let count = arg_types.get(1)?;
    let value = arg_types.get(2)?;
    let count_is_positive = !count.types.is_empty()
        && count.types.iter().all(|a| match a {
            Atomic::TLiteralInt(n) => *n >= 1,
            Atomic::TPositiveInt => true,
            Atomic::TIntRange { min, .. } => min.is_some_and(|m| m >= 1),
            _ => false,
        });
    if !count_is_positive {
        return None;
    }
    // A list (keys 0..count-1) only when $start_index is provably exactly
    // 0 — any other start makes the result a non-list int-keyed array (PHP
    // also special-cases a NEGATIVE start_index: only the first key keeps
    // it, the rest restart from 0 — still never a list, so it's covered by
    // the same "not zero" fallback below rather than needing its own arm).
    let start_is_zero = matches!(start.types.as_slice(), [Atomic::TLiteralInt(0)]);
    if start_is_zero {
        return Some(Type::single(Atomic::TNonEmptyList {
            value: Box::new(value.clone()),
        }));
    }
    let start_is_known_non_zero = !start.types.is_empty()
        && start.types.iter().all(|a| match a {
            Atomic::TLiteralInt(n) => *n != 0,
            Atomic::TPositiveInt | Atomic::TNegativeInt => true,
            Atomic::TIntRange { min, max } => {
                min.is_some_and(|m| m > 0) || max.is_some_and(|m| m < 0)
            }
            _ => false,
        });
    if start_is_known_non_zero {
        return Some(Type::single(Atomic::TNonEmptyArray {
            key: Box::new(Type::single(Atomic::TInt)),
            value: Box::new(value.clone()),
        }));
    }
    None
}

/// Infer the return type of `array_keys($array)`.
///
/// When the argument is a statically non-empty array, upgrades `list<K>` in
/// the stub's template-resolved return type to `non-empty-list<K>` so the key
/// type from Psalm-style template inference is preserved. Returns the stub
/// return unchanged when the source is not provably non-empty.
pub(crate) fn array_keys_return_type(arg_types: &[Type], return_ty: &Type) -> Type {
    let Some(arr) = arg_types.first() else {
        return return_ty.clone();
    };
    if !super::callable::is_non_empty_collection(arr) {
        return return_ty.clone();
    }
    // Upgrade list<K> → non-empty-list<K> while keeping the stub's key type.
    let mut result = Type::empty();
    result.from_docblock = return_ty.from_docblock;
    for atomic in &return_ty.types {
        match atomic {
            Atomic::TList { value } => {
                result.add_type(Atomic::TNonEmptyList {
                    value: value.clone(),
                });
            }
            other => result.add_type(other.clone()),
        }
    }
    if result.is_empty() {
        return_ty.clone()
    } else {
        result
    }
}

/// Infer the return type of `array_reverse($array)`.
///
/// Preserves non-emptiness: a non-empty array reversed is still non-empty.
/// Uses the same value type as the source array.
pub(crate) fn array_reverse_return_type(arg_types: &[Type]) -> Option<Type> {
    let arr = arg_types.first()?;
    if arr.is_mixed() {
        return None;
    }
    let (key, value) = crate::stmt::infer_foreach_types(arr);
    if value.is_mixed() {
        return None;
    }
    // `array_reverse` always preserves string keys; `$preserve_keys` (2nd
    // arg) only controls whether INT keys are renumbered from 0 (false,
    // the default) or kept as-is (true). Only an int-keyed-only source with
    // preserve_keys false/absent becomes a re-indexed list — everything
    // else keeps its original key type (values just get reordered).
    let preserve_keys = arg_types.get(1).is_some_and(|t| {
        t.types
            .iter()
            .any(|a| matches!(a, Atomic::TTrue | Atomic::TBool))
            && !t
                .types
                .iter()
                .any(|a| matches!(a, Atomic::TFalse | Atomic::TNull))
    });
    let key_is_int_only = !key.is_mixed() && key.types.iter().all(Atomic::is_int);
    if key_is_int_only && !preserve_keys {
        let atomic = if super::callable::is_non_empty_collection(arr) {
            Atomic::TNonEmptyList {
                value: Box::new(value),
            }
        } else {
            Atomic::TList {
                value: Box::new(value),
            }
        };
        return Some(Type::single(atomic));
    }
    if key.is_mixed() {
        return None;
    }
    let atomic = if super::callable::is_non_empty_collection(arr) {
        Atomic::TNonEmptyArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    } else {
        Atomic::TArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    };
    Some(Type::single(atomic))
}

/// Infer the result type of `array_map($callback, $array, ...)`.
///
/// PHP semantics: `array_map` applies `$callback` to each element and returns
/// an array of the callback's return values. With a single source array the
/// keys are preserved; with multiple arrays the result is re-indexed with
/// integer keys. A `null` callback (zip mode) is not modeled — we return
/// `None` so the generic stub `array` return is kept.
///
/// Returns `None` when the callback return type is unknown, so the caller falls
/// back to the stub return type instead of fabricating `array<…, mixed>`.
pub(crate) fn infer_array_map_return(
    ea: &ExpressionAnalyzer<'_>,
    arg_types: &[Type],
    ctx: &FlowState,
    callback_expr: Option<&Expr>,
) -> Option<Type> {
    let callback = arg_types.first()?;
    // `array_map(null, ...)` (zip mode) and other non-callable first args are
    // out of scope; only proceed for a genuinely callable first argument.
    if callback.types.iter().any(|a| matches!(a, Atomic::TNull)) {
        return None;
    }
    let value = callable_return_type(callback, ea, ctx, callback_expr)?;
    // A `void`/`never` callback is degenerate (the runtime fills `null`); don't
    // fabricate an `array<…, void>` element type that would surface dubious
    // downstream diagnostics — keep the generic stub `array`.
    if value
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TVoid | Atomic::TNever))
    {
        return None;
    }

    // Single-array mode: detect list-ness and non-emptiness from the source.
    // Multi-array (zip) mode: always produces array<int, T>.
    if arg_types.len() == 2 {
        let source = &arg_types[1];
        // A literal array (`[1, 2, 3]`) is represented as a `TKeyedArray` with
        // `is_list: true`, not as `TList`/`TNonEmptyList` — without this arm,
        // `array_map` over a literal list lost its list-ness/non-emptiness and
        // fell back to a generic `array<int, T>`.
        let src_is_list = !source.types.is_empty()
            && source.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TList { .. }
                        | Atomic::TNonEmptyList { .. }
                        | Atomic::TKeyedArray { is_list: true, .. }
                )
            });
        let src_is_non_empty = !source.types.is_empty()
            && source.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TNonEmptyArray { .. } | Atomic::TNonEmptyList { .. }
                ) || matches!(a, Atomic::TKeyedArray { properties, .. } if !properties.is_empty())
            });
        let atom = match (src_is_list, src_is_non_empty) {
            (true, true) => Atomic::TNonEmptyList {
                value: Box::new(value),
            },
            (true, false) => Atomic::TList {
                value: Box::new(value),
            },
            (false, true) => {
                let (k, _) = crate::stmt::infer_foreach_types(source);
                let key = if k.is_mixed() { Type::array_key() } else { k };
                Atomic::TNonEmptyArray {
                    key: Box::new(key),
                    value: Box::new(value),
                }
            }
            (false, false) => {
                let (k, _) = crate::stmt::infer_foreach_types(source);
                let key = if k.is_mixed() { Type::array_key() } else { k };
                Atomic::TArray {
                    key: Box::new(key),
                    value: Box::new(value),
                }
            }
        };
        Some(Type::single(atom))
    } else {
        // Multi-array zip mode: integer-keyed, not a list (multi-array semantics).
        let src_is_non_empty = arg_types.get(1).is_some_and(|t| {
            !t.types.is_empty()
                && t.types.iter().all(|a| {
                    matches!(
                        a,
                        Atomic::TNonEmptyArray { .. } | Atomic::TNonEmptyList { .. }
                    )
                })
        });
        let key = Type::single(Atomic::TInt);
        let atom = if src_is_non_empty {
            Atomic::TNonEmptyArray {
                key: Box::new(key),
                value: Box::new(value),
            }
        } else {
            Atomic::TArray {
                key: Box::new(key),
                value: Box::new(value),
            }
        };
        Some(Type::single(atom))
    }
}

/// Infer the result type of `array_reduce($array, $callback, $initial = null)`.
///
/// The result is either the callback's return type (if the array is
/// non-empty) or `$initial` as-is (if it's empty) — union both since mir
/// doesn't track non-emptiness precisely enough here to pick one.
pub(crate) fn infer_array_reduce_return(
    ea: &ExpressionAnalyzer<'_>,
    arg_types: &[Type],
    ctx: &FlowState,
    callback_expr: Option<&Expr>,
) -> Option<Type> {
    let callback = arg_types.get(1)?;
    let mut result = callable_return_type(callback, ea, ctx, callback_expr)?;
    if result
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TVoid | Atomic::TNever))
    {
        return None;
    }
    match arg_types.get(2) {
        Some(initial) => result.merge_with(initial),
        None => result.merge_with(&Type::null()),
    }
    Some(result)
}

/// Infer the result type of `array_filter($array, $callback?, ...)`.
///
/// Filtering never changes the element types — it only removes entries — so the
/// result carries the source array's key and value types (made possibly-empty;
/// list-ness is dropped because filtering can leave gaps). Returns `None` when
/// the source element types are unknown so the generic stub `array` is kept.
pub(crate) fn infer_array_filter_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (key, value) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() && value.is_mixed() {
        return None;
    }
    Some(Type::single(Atomic::TArray {
        key: Box::new(key),
        value: Box::new(value),
    }))
}

/// Infer the result type of `array_slice($array, $offset, $length, $preserve_keys)`.
///
/// When the source is a list type and `preserve_keys` is false (the default),
/// the result is re-indexed to start from 0 → returns `list<TValue>`.
/// When the source is a generic array, key and value types are preserved.
/// The result is always possibly-empty (we cannot know whether the slice is
/// non-empty without evaluating the offset/length at analysis time).
pub(crate) fn array_slice_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    // Check if preserve_keys is explicitly true (4th arg is TTrue or TBool).
    let preserve_keys = arg_types.get(3).is_some_and(|t| {
        t.types
            .iter()
            .any(|a| matches!(a, Atomic::TTrue | Atomic::TBool))
            && !t
                .types
                .iter()
                .any(|a| matches!(a, Atomic::TFalse | Atomic::TNull))
    });

    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }

    // When the source is a list and preserve_keys is false (default), the
    // result is also a list (integer keys are re-indexed from 0). A literal
    // array (`[1, 2, 3]`) is a `TKeyedArray` with `is_list: true`, not a
    // `TList`/`TNonEmptyList` — without that arm, slicing a literal list lost
    // its list-ness and fell back to a generic `array<K, V>`.
    let is_source_list = source.types.iter().all(|a| {
        matches!(
            a,
            Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { is_list: true, .. }
        )
    });

    if is_source_list && !preserve_keys {
        return Some(Type::single(Atomic::TList {
            value: Box::new(value),
        }));
    }

    // Generic array: preserve key type too.
    let (key, _) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() {
        return None;
    }
    Some(Type::single(Atomic::TArray {
        key: Box::new(key),
        value: Box::new(value),
    }))
}

/// Infer the result type of `array_values($array)`.
///
/// Re-indexing produces a `list<TValue>` (or `non-empty-list<TValue>` when the
/// source is provably non-empty). Returns `None` when element types are unknown
/// so the generic stub `list<TValue>` binding falls back to `list<mixed>`.
pub(crate) fn infer_array_values_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }
    let atomic = if super::callable::is_non_empty_collection(source) {
        Atomic::TNonEmptyList {
            value: Box::new(value),
        }
    } else {
        Atomic::TList {
            value: Box::new(value),
        }
    };
    Some(Type::single(atomic))
}

/// Infer the result type of `array_merge($arr1, $arr2, ...)`.
///
/// When ALL arguments are list types, the merged result is also a list (PHP
/// re-indexes integer keys from 0 for array_merge). The result is non-empty
/// if at least one argument is provably non-empty.
/// Falls back to `None` when any arg is a non-list array (string-keyed arrays
/// or mixed-key arrays) — the generic stub handles those cases.
pub(crate) fn infer_array_merge_return(arg_types: &[Type]) -> Option<Type> {
    if arg_types.is_empty() {
        return None;
    }
    // All args must be list types for us to produce a list result. A literal
    // array (`[1, 2, 3]`) is a `TKeyedArray` with `is_list: true`, not a
    // `TList`/`TNonEmptyList` — without that arm, merging a literal list
    // argument lost its list-ness and fell back to a generic `array<K, V>`.
    let all_lists = arg_types.iter().all(|t| {
        !t.types.is_empty()
            && t.types.iter().all(|a| {
                matches!(
                    a,
                    Atomic::TList { .. }
                        | Atomic::TNonEmptyList { .. }
                        | Atomic::TKeyedArray { is_list: true, .. }
                )
            })
    });
    if !all_lists {
        return None;
    }
    // Compute the union of all element types.
    let mut value = Type::empty();
    for arg in arg_types {
        let (_, v) = crate::stmt::infer_foreach_types(arg);
        value.merge_with(&v);
    }
    if value.is_empty() || value.is_mixed() {
        return None;
    }
    // Non-empty if ANY arg is non-empty.
    let any_non_empty = arg_types
        .iter()
        .any(super::callable::is_non_empty_collection);
    let atomic = if any_non_empty {
        Atomic::TNonEmptyList {
            value: Box::new(value),
        }
    } else {
        Atomic::TList {
            value: Box::new(value),
        }
    };
    Some(Type::single(atomic))
}

/// Infer the return type of `array_merge_recursive($arr1, $arr2, ...)`.
///
/// For int keys, `array_merge_recursive` behaves exactly like `array_merge`
/// (int keys are always freshly appended/renumbered, regardless of the
/// recursive string-key merging that gives this function its name — no
/// int-key collision can ever occur). So when every argument is a list, the
/// two functions produce identical results. The general string-keyed case is
/// genuinely complex (colliding scalars get wrapped into a new array,
/// colliding arrays deep-merge) and is intentionally not modeled here.
pub(crate) fn array_merge_recursive_return_type(arg_types: &[Type]) -> Option<Type> {
    infer_array_merge_return(arg_types)
}

/// Infer the return type of `array_unique($array)`.
///
/// `array_unique` preserves keys and drops duplicates; a non-empty input always
/// yields a non-empty result. List-ness is NOT preserved (keys can have gaps after
/// deduplication). Returns `None` to fall back to the stub for unknown inputs.
pub(crate) fn array_unique_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (key, value) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() && value.is_mixed() {
        return None;
    }
    let atomic = if super::callable::is_non_empty_collection(source) {
        Atomic::TNonEmptyArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    } else {
        Atomic::TArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    };
    Some(Type::single(atomic))
}

/// Infer the return type of `array_key_first($array)` / `array_key_last($array)`.
///
/// For non-empty collections the result is always `string|int` (never null).
/// For `list` / `non-empty-list` it is always `int`.
/// Returns `None` to fall back to the stub when the collection is possibly empty.
pub(crate) fn array_key_first_last_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() || !super::callable::is_non_empty_collection(source) {
        return None;
    }
    let all_list = source
        .types
        .iter()
        .all(|a| matches!(a, Atomic::TNonEmptyList { .. } | Atomic::TList { .. }));
    if all_list {
        Some(Type::single(Atomic::TInt))
    } else {
        let mut ty = Type::single(Atomic::TInt);
        ty.add_type(Atomic::TString);
        Some(ty)
    }
}

/// Infer the return type of `array_pop($array)` / `array_shift($array)`.
///
/// When the collection element type is known and the collection is provably
/// non-empty, the return is `T` (not `T|null`). When the collection is typed
/// but possibly empty, returns `T|null`. Falls back to `None` (→ stub) for
/// unknown / `mixed` sources.
pub(crate) fn array_pop_shift_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }
    if super::callable::is_non_empty_collection(source) {
        Some(value)
    } else {
        let mut ty = value;
        ty.add_type(Atomic::TNull);
        Some(ty)
    }
}

/// Infer the return type of `reset($array)` / `end($array)`.
///
/// Both reposition the internal pointer to a known first/last slot
/// regardless of any prior pointer state, so — like `array_pop_shift_return`
/// — a provably non-empty source can drop the `false` failure case.
pub(crate) fn array_reset_end_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }
    if super::callable::is_non_empty_collection(source) {
        Some(value)
    } else {
        let mut ty = value;
        ty.add_type(Atomic::TFalse);
        Some(ty)
    }
}

/// Infer the return type of `current($array)` / `next($array)` / `prev($array)`.
///
/// Unlike `reset()`/`end()`, these depend on the pointer's position left by
/// PRIOR calls (not tracked here) — always includes `false`, even for a
/// provably non-empty source.
pub(crate) fn array_current_next_prev_return(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }
    let mut ty = value;
    ty.add_type(Atomic::TFalse);
    Some(ty)
}

/// Infer the by-ref array type after an in-place sort function.
///
/// All sort functions preserve element types. Re-indexing sorts (`sort`, `rsort`, `usort`,
/// `shuffle`) also re-index integer keys from 0, making the result a `list<T>`.
/// Key-preserving sorts (`asort`, `arsort`, `ksort`, `krsort`, `uasort`, `uksort`) leave
/// key types unchanged — so the original type is returned as-is.
///
/// The main benefit over the stub's generic `array` is that element types are not lost.
pub(crate) fn sort_byref_type(arr: &Type, reindex: bool) -> Type {
    if arr.is_mixed() {
        return arr.clone();
    }
    if !reindex {
        return arr.clone();
    }
    let (_, value) = crate::stmt::infer_foreach_types(arr);
    if value.is_mixed() {
        return arr.clone();
    }
    let atom = if super::callable::is_non_empty_collection(arr) {
        Atomic::TNonEmptyList {
            value: Box::new(value),
        }
    } else {
        Atomic::TList {
            value: Box::new(value),
        }
    };
    Type::single(atom)
}

/// Infer the return type of `array_search($needle, $haystack)`.
///
/// The stub returns `string|int|false`. When the haystack has a known key type
/// (e.g. `list` or `array<string, …>`), the success case is narrowed to that key
/// type, reducing the union from `string|int|false` to `key_type|false`.
pub(crate) fn array_search_return_type(arg_types: &[Type]) -> Option<Type> {
    let haystack = arg_types.get(1)?;
    if haystack.is_mixed() {
        return None;
    }
    let (key, _) = crate::stmt::infer_foreach_types(haystack);
    if key.is_mixed() {
        return None;
    }
    let mut result = key;
    result.add_type(Atomic::TFalse);
    Some(result)
}

/// Infer the return type of `key($array)`.
///
/// The stub returns unrefined `int|string|null`. Narrowed to the array's own
/// key type when known. Always includes `null` — the internal pointer's
/// position isn't tracked across calls, so even a provably non-empty array
/// may have its pointer already past the end.
pub(crate) fn array_key_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (key, _) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() {
        return None;
    }
    let mut result = key;
    result.add_type(Atomic::TNull);
    Some(result)
}

/// Infer the return type of `array_fill_keys(array $keys, mixed $value)`.
///
/// The result has one entry per element in `$keys`. The key type comes from
/// the *value* type of `$keys` (since those values become the result's keys),
/// and the value type is the type of `$value`. Non-empty `$keys` → non-empty result.
pub(crate) fn array_fill_keys_return_type(arg_types: &[Type]) -> Option<Type> {
    let keys_arr = arg_types.first()?;
    let value_ty = arg_types.get(1)?;
    if keys_arr.is_mixed() || value_ty.is_mixed() {
        return None;
    }
    let (_, key_of_result) = crate::stmt::infer_foreach_types(keys_arr);
    if key_of_result.is_mixed() {
        return None;
    }
    let atom = if super::callable::is_non_empty_collection(keys_arr) {
        Atomic::TNonEmptyArray {
            key: Box::new(key_of_result),
            value: Box::new(value_ty.clone()),
        }
    } else {
        Atomic::TArray {
            key: Box::new(key_of_result),
            value: Box::new(value_ty.clone()),
        }
    };
    Some(Type::single(atom))
}

/// Infer the return type of `array_chunk($array, $length, $preserve_keys?)`.
///
/// `array_chunk` splits an array into sub-arrays of at most `$length` elements:
/// - The outer array is a `list<…>` (always re-indexed by chunk index).
/// - When `$preserve_keys` is false (default) each chunk is also a `list<T>`.
/// - When `$preserve_keys` is true each chunk preserves the original keys → `array<K,T>`.
/// - Non-empty source → non-empty outer list (there is at least one chunk).
///
/// Falls back to stub `array` when value type is unknown.
pub(crate) fn array_chunk_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (key, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }
    let preserve_keys = arg_types.get(2).is_some_and(|t| {
        t.types
            .iter()
            .any(|a| matches!(a, Atomic::TTrue | Atomic::TBool))
            && !t
                .types
                .iter()
                .any(|a| matches!(a, Atomic::TFalse | Atomic::TNull))
    });
    let chunk_atom = if preserve_keys {
        if key.is_mixed() {
            return None;
        }
        Atomic::TArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    } else {
        Atomic::TList {
            value: Box::new(value),
        }
    };
    let chunk_ty = Type::single(chunk_atom);
    let outer_atom = if super::callable::is_non_empty_collection(source) {
        Atomic::TNonEmptyList {
            value: Box::new(chunk_ty),
        }
    } else {
        Atomic::TList {
            value: Box::new(chunk_ty),
        }
    };
    Some(Type::single(outer_atom))
}

/// Infer the return type of the `array_diff`/`array_intersect` function
/// family (and their `_key`/`_assoc`/`u*`/`uassoc` variants).
///
/// Every member of this family filters `$array` (the first argument) down to
/// a subset of its own entries — by value, key, or both, optionally via a
/// user callback that only decides membership — never altering the surviving
/// entries' keys or values. So the result's key/value types are exactly
/// `$array`'s. The result can never be proven non-empty (every entry could be
/// filtered out) and is never a list (the original — possibly
/// non-sequential — int keys are preserved verbatim).
pub(crate) fn array_diff_intersect_like_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (key, value) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() && value.is_mixed() {
        return None;
    }
    Some(Type::single(Atomic::TArray {
        key: Box::new(key),
        value: Box::new(value),
    }))
}

/// Infer the return type of `array_combine(array $keys, array $values)`.
///
/// PHP pairs each value of `$keys` (coerced to a legal array-key type) with
/// the value at the same position in `$values`, positionally. Since PHP 8
/// throws a `ValueError` when the counts differ, on the successful-return
/// path a non-empty `$keys` guarantees a non-empty result. The result is
/// never a list — its keys come from `$keys`'s arbitrary values, not
/// sequential indices.
pub(crate) fn array_combine_return_type(arg_types: &[Type]) -> Option<Type> {
    let keys_arr = arg_types.first()?;
    let values_arr = arg_types.get(1)?;
    let (_, keys_values) = crate::stmt::infer_foreach_types(keys_arr);
    let key = crate::expr::helpers::coerce_array_key_type(&keys_values);
    let (_, value) = crate::stmt::infer_foreach_types(values_arr);
    if key.is_mixed() && value.is_mixed() {
        return None;
    }
    let atomic = if super::callable::is_non_empty_collection(keys_arr) {
        Atomic::TNonEmptyArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    } else {
        Atomic::TArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    };
    Some(Type::single(atomic))
}

/// Infer the return type of `array_count_values(array $array)`.
///
/// The result's keys are the distinct *values* of `$array` (a value→key role
/// swap) — PHP restricts these to `int|string` and throws a `TypeError` for
/// any other value type since PHP 8.0, so a source whose value type isn't
/// entirely int/string is a runtime-error path, not worth modeling here.
/// The result's values are always counts, i.e. `int<1, max>` for any key
/// that's present at all.
pub(crate) fn array_count_values_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() || value.types.is_empty() {
        return None;
    }
    if !value.types.iter().all(|a| a.is_int() || a.is_string()) {
        return None;
    }
    let key = crate::expr::helpers::coerce_array_key_type(&value);
    let count = Type::single(Atomic::TIntRange {
        min: Some(1),
        max: None,
    });
    let atomic = if super::callable::is_non_empty_collection(source) {
        Atomic::TNonEmptyArray {
            key: Box::new(key),
            value: Box::new(count),
        }
    } else {
        Atomic::TArray {
            key: Box::new(key),
            value: Box::new(count),
        }
    };
    Some(Type::single(atomic))
}

/// Infer the return type of `array_change_key_case(array $array, int $case = CASE_LOWER)`.
///
/// Only STRING keys are case-folded; int keys and all values pass through
/// unchanged. For any source that isn't a `TKeyedArray` shape (a plain
/// `array<K, V>`/`list<V>`/non-empty variant), case-folding a `string` key
/// type doesn't change its *type* — so the source type itself is already the
/// correct, unchanged result. For a `TKeyedArray` shape with at least one
/// string key, the case is only rewritten when `$case` resolves to a known
/// literal `CASE_LOWER` (0) or `CASE_UPPER` (1); anything else (a union of
/// atoms, an unresolved `$case`) falls back to the generic stub.
pub(crate) fn array_change_key_case_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let is_shape = source.types.len() == 1 && matches!(source.types[0], Atomic::TKeyedArray { .. });
    if !is_shape {
        // Generic array (or union of non-shape array atoms): folding a
        // string key's TYPE is a no-op, and values are untouched.
        if source.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TArray { .. }
                    | Atomic::TList { .. }
                    | Atomic::TNonEmptyArray { .. }
                    | Atomic::TNonEmptyList { .. }
            )
        }) {
            return Some(source.clone());
        }
        return None;
    }
    let Atomic::TKeyedArray {
        properties,
        is_open,
        is_list,
    } = &source.types[0]
    else {
        unreachable!("is_shape checked above");
    };
    let has_string_key = properties.keys().any(|k| matches!(k, ArrayKey::String(_)));
    if !has_string_key {
        return Some(source.clone());
    }
    let case_arg = arg_types.get(1);
    let lower = match case_arg {
        None => true,
        Some(t) => match t.types.as_slice() {
            [Atomic::TLiteralInt(0)] => true,
            [Atomic::TLiteralInt(1)] => false,
            _ => return None,
        },
    };
    let mut new_properties = IndexMap::new();
    for (k, prop) in properties.iter() {
        let new_key = match k {
            ArrayKey::String(s) => {
                let folded: Arc<str> = if lower {
                    s.to_lowercase().into()
                } else {
                    s.to_uppercase().into()
                };
                ArrayKey::String(folded)
            }
            ArrayKey::Int(_) => k.clone(),
        };
        new_properties.insert(new_key, prop.clone());
    }
    Some(Type::single(Atomic::TKeyedArray {
        properties: Box::new(new_properties),
        is_open: *is_open,
        is_list: *is_list,
    }))
}

/// Infer the return type of `array_splice(&$array, $offset, $length?, $replacement?)`.
///
/// Returns the array of REMOVED elements. Unlike `array_slice`, there is no
/// `preserve_keys` parameter — `array_splice` always renumbers int keys from
/// 0 in its return value (string keys pass through unchanged), i.e.
/// structurally identical to `array_slice_return_type`'s `preserve_keys =
/// false` path.
pub(crate) fn array_splice_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, value) = crate::stmt::infer_foreach_types(source);
    if value.is_mixed() {
        return None;
    }
    let is_source_list = source.types.iter().all(|a| {
        matches!(
            a,
            Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { is_list: true, .. }
        )
    });
    if is_source_list {
        return Some(Type::single(Atomic::TList {
            value: Box::new(value),
        }));
    }
    let (key, _) = crate::stmt::infer_foreach_types(source);
    if key.is_mixed() {
        return None;
    }
    Some(Type::single(Atomic::TArray {
        key: Box::new(key),
        value: Box::new(value),
    }))
}

/// Infer the return type of `array_pad(array $array, int $length, mixed $value)`.
///
/// SCOPED to a pure-list source (only sequential int keys `0..n-1`, matching
/// `TList`/`TNonEmptyList`/`TKeyedArray{is_list: true}`): padding a list to
/// `abs($length)` elements always renumbers int keys, on either side, so the
/// result is a fresh list regardless of pad direction. A source with any
/// string key would need to keep those keys unrenumbered while still
/// left/right-padding the int ones — genuinely ambiguous to model generically
/// here, so falls back to the generic stub instead of guessing.
pub(crate) fn array_pad_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let is_source_list = !source.types.is_empty()
        && source.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TList { .. }
                    | Atomic::TNonEmptyList { .. }
                    | Atomic::TKeyedArray { is_list: true, .. }
            )
        });
    if !is_source_list {
        return None;
    }
    let (_, source_value) = crate::stmt::infer_foreach_types(source);
    if source_value.is_mixed() {
        return None;
    }
    let pad_value = arg_types.get(2)?.clone();
    let mut value = source_value;
    value.merge_with(&pad_value);

    // Non-empty either because the source already is, or because a known
    // non-zero literal $length guarantees at least one element is padded in.
    let length_forces_non_empty = arg_types
        .get(1)
        .is_some_and(|t| matches!(t.types.as_slice(), [Atomic::TLiteralInt(n)] if *n != 0));
    let non_empty = super::callable::is_non_empty_collection(source) || length_forces_non_empty;

    let atomic = if non_empty {
        Atomic::TNonEmptyList {
            value: Box::new(value),
        }
    } else {
        Atomic::TList {
            value: Box::new(value),
        }
    };
    Some(Type::single(atomic))
}

/// Infer the return type of `array_column($array, $column_key, $index_key = null)`.
///
/// SCOPED to a single resolvable row shape and a literal `string`/`int`/`null`
/// `$column_key`: `infer_foreach_types` must yield exactly one `TKeyedArray`
/// atom (a union of row shapes, or non-shape rows, isn't modeled). A row
/// missing `$column_key` is silently excluded at runtime rather than included
/// as null — reflected here by only treating the result as non-empty when the
/// column property isn't optional. `$column_key === null` (the "whole rows"
/// form) has no such exclusion: every row is always present, so it carries no
/// `.optional` dependency of its own.
pub(crate) fn array_column_return_type(arg_types: &[Type]) -> Option<Type> {
    let source = arg_types.first()?;
    if source.is_mixed() {
        return None;
    }
    let (_, row) = crate::stmt::infer_foreach_types(source);
    if row.types.len() != 1 {
        return None;
    }
    let Atomic::TKeyedArray { properties, .. } = &row.types[0] else {
        return None;
    };

    let column_key_ty = arg_types.get(1)?;
    // `None` here means "whole rows" ($column_key === null): the value is the
    // row itself, with no column-presence exclusion to account for.
    let column_key = match column_key_ty.types.as_slice() {
        [Atomic::TLiteralString(s)] => Some(ArrayKey::String(s.clone())),
        [Atomic::TLiteralInt(i)] => Some(ArrayKey::Int(*i)),
        [Atomic::TNull] => None,
        _ => return None,
    };
    let (value, column_optional) = match &column_key {
        Some(key) => {
            let column_prop = properties.get(key)?;
            (column_prop.ty.clone(), column_prop.optional)
        }
        None => (row.clone(), false),
    };

    // Omitted or an explicit literal `null` both mean "no $index_key": a
    // fresh 0-indexed list result.
    let index_arg = arg_types.get(2);
    let is_no_index = match index_arg {
        None => true,
        Some(t) => matches!(t.types.as_slice(), [Atomic::TNull]),
    };
    if is_no_index {
        let non_empty = super::callable::is_non_empty_collection(source) && !column_optional;
        let atomic = if non_empty {
            Atomic::TNonEmptyList {
                value: Box::new(value),
            }
        } else {
            Atomic::TList {
                value: Box::new(value),
            }
        };
        return Some(Type::single(atomic));
    }

    let index_key_ty = index_arg?;
    let index_key = match index_key_ty.types.as_slice() {
        [Atomic::TLiteralString(s)] => ArrayKey::String(s.clone()),
        [Atomic::TLiteralInt(i)] => ArrayKey::Int(*i),
        _ => return None,
    };
    let index_prop = properties.get(&index_key)?;
    let key = crate::expr::helpers::coerce_array_key_type(&index_prop.ty);

    let non_empty = super::callable::is_non_empty_collection(source)
        && !column_optional
        && !index_prop.optional;
    let atomic = if non_empty {
        Atomic::TNonEmptyArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    } else {
        Atomic::TArray {
            key: Box::new(key),
            value: Box::new(value),
        }
    };
    Some(Type::single(atomic))
}

/// Helper: extract a readable function name from union for diagnostic output.
fn callback_name_for_diagnostic(callback_ty: &Type) -> String {
    if let Some(Atomic::TLiteralString(fn_name)) = callback_ty.types.first() {
        fn_name.to_string()
    } else {
        "(closure)".to_string()
    }
}

pub(crate) fn array_push_unshift_byref_type(
    arr: &Type,
    push_types: &[Type],
    inside_loop: bool,
) -> Type {
    if arr.is_mixed() || push_types.is_empty() {
        return arr.clone();
    }
    // A genuinely empty source (`$arr = []` before the call, or the closed
    // `array{}` a proven-empty narrow produces) carries no per-property info
    // for `infer_foreach_types` to fold, so it falls back to `mixed` there and
    // this function would otherwise bail out to `arr.clone()` — leaving the
    // variable typed as still-empty even though it definitely isn't anymore.
    // Prepending onto nothing is identical to appending onto nothing (both
    // just produce the pushed values in order, 0-indexed), so route through
    // the same shape-preserving growth `$arr[] = …` uses for both functions.
    if !arr.types.is_empty()
        && arr
            .types
            .iter()
            .all(|a| matches!(a, Atomic::TKeyedArray { properties, .. } if properties.is_empty()))
    {
        let mut current = arr.clone();
        for pushed in push_types {
            if pushed.is_mixed() {
                return arr.clone();
            }
            current =
                crate::expr::helpers::widen_array_as_list(&current, pushed, inside_loop, None);
        }
        return current;
    }
    let (_, src_value) = crate::stmt::infer_foreach_types(arr);
    let mut value = src_value;
    for pushed in push_types {
        if pushed.is_mixed() {
            return arr.clone();
        }
        value.merge_with(pushed);
    }
    if value.is_empty() || value.is_mixed() {
        return arr.clone();
    }
    // A literal array (`[1, 2, 3]`) is a `TKeyedArray` with `is_list: true`,
    // not a `TList`/`TNonEmptyList` — without that arm, pushing/unshifting
    // onto a literal list lost its list-ness and fell back to a generic
    // `array<K, V>`.
    let is_src_list = !arr.types.is_empty()
        && arr.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TList { .. }
                    | Atomic::TNonEmptyList { .. }
                    | Atomic::TKeyedArray { is_list: true, .. }
            )
        });
    if is_src_list {
        return Type::single(Atomic::TNonEmptyList {
            value: Box::new(value),
        });
    }
    let (key, _) = crate::stmt::infer_foreach_types(arr);
    if key.is_mixed() {
        return arr.clone();
    }
    Type::single(Atomic::TNonEmptyArray {
        key: Box::new(key),
        value: Box::new(value),
    })
}
