use std::sync::Arc;

use php_ast::Span;

use mir_codebase::definitions::{DeclaredParam, TemplateParam, Visibility};
use mir_issues::{IssueKind, Severity};
use mir_types::{ArrayKey, Atomic, Name, Type};

use crate::expr::ExpressionAnalyzer;

mod counts;
mod nullability;
mod types;

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

pub(crate) struct ArgBinding {
    pub(crate) param_idx: usize,
    pub(crate) arg_ty: Type,
    pub(crate) arg_span: Span,
    pub(crate) arg_idx: usize,
}

pub struct CheckArgsParams<'a> {
    pub fn_name: &'a str,
    pub params: &'a [DeclaredParam],
    pub arg_types: &'a [Type],
    pub arg_spans: &'a [Span],
    pub arg_names: &'a [Option<String>],
    pub arg_can_be_byref: &'a [bool],
    pub call_span: Span,
    pub has_spread: bool,
    /// True when the total argument count can't be trusted for a
    /// TooFew/TooManyArguments check — normally identical to `has_spread`,
    /// but a sole spread arg that [`expand_sole_spread_arg`] resolved into
    /// concrete per-element bindings sets `has_spread` to `false` (so
    /// `check_counts` processes every expanded element instead of stopping
    /// after the first) while this stays `true`, preserving the existing
    /// "don't flag arity through a spread" behavior.
    pub arity_unknown: bool,
    pub template_params: &'a [TemplateParam],
    /// True when the function/method is tagged `@no-named-arguments`.
    pub no_named_arguments: bool,
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

pub fn check_constructor_args(
    ea: &mut ExpressionAnalyzer<'_>,
    class_name: &str,
    p: CheckArgsParams<'_>,
) {
    let ctor_name = format!("{class_name}::__construct");
    check_args(
        ea,
        CheckArgsParams {
            fn_name: &ctor_name,
            ..p
        },
    );
}

/// For a spread (`...`) argument, return the union of value types across all array atomics.
/// E.g. `array<int, int>` → `int`, `list<string>` → `string`, `mixed` → `mixed`.
pub fn spread_element_type(db: &dyn crate::db::MirDatabase, arr_ty: &Type) -> Type {
    let mut result = Type::empty();
    for atomic in arr_ty.types.iter() {
        match atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                for t in value.types.iter() {
                    result.add_type(t.clone());
                }
            }
            Atomic::TKeyedArray { properties, .. } => {
                for (_key, prop) in properties.iter() {
                    for t in prop.ty.types.iter() {
                        result.add_type(t.clone());
                    }
                }
            }
            // Traversable<TKey, TValue>/Iterator/IteratorAggregate/Generator — resolve
            // the real item types via the class's own `@implements` annotation
            // (or `current()`/`key()`/`getIterator()` chain), not a naive
            // `type_params[1]` positional guess.
            Atomic::TNamedObject { fqcn, type_params } => {
                if let Some((_key, value)) =
                    crate::stmt::resolve_iterator_item_types(db, fqcn, type_params, 4)
                {
                    for t in value.types.iter() {
                        result.add_type(t.clone());
                    }
                } else {
                    return Type::mixed();
                }
            }
            _ => return Type::mixed(),
        }
    }
    if result.types.is_empty() {
        Type::mixed()
    } else {
        result
    }
}

/// When a call's sole argument is a spread (`f(...$arr)`) and `$arr` resolves
/// to a literal, sequentially int-keyed (0..n-1), closed shape, return one
/// type per element instead of the single merged [`spread_element_type`].
///
/// Without this, `needsTwoInts(...$pair)` binds the merged union of ALL of
/// `$pair`'s element types to just the first parameter (via the existing
/// "stop after the first arg once a spread is seen" arity logic) and never
/// checks the remaining parameters at all — `$pair[1]` being definitely
/// wrong-typed for `$b` went completely undetected.
///
/// Returns `None` (fall back to the single-merged-type behavior) for a
/// dynamic-length array, an open shape, or a non-shape array type.
pub fn expand_sole_spread_arg(arr_ty: &Type) -> Option<Vec<Type>> {
    let mut per_index: Vec<Type> = Vec::new();
    if arr_ty.types.is_empty() {
        return None;
    }
    for atomic in &arr_ty.types {
        // `is_list` is only set for the `list{...}` docblock spelling — a
        // shape written as `array{0: int, 1: string}` describes the exact
        // same sequential-int-key structure but parses with `is_list:
        // false`, so don't require it; the explicit per-index key lookup
        // below already enforces the sequential-key structure that matters.
        let Atomic::TKeyedArray {
            properties,
            is_open: false,
            ..
        } = atomic
        else {
            return None;
        };
        if per_index.is_empty() {
            for i in 0..properties.len() {
                let prop = properties.get(&ArrayKey::Int(i as i64))?;
                per_index.push(prop.ty.clone());
            }
        } else {
            if properties.len() != per_index.len() {
                return None;
            }
            for (i, slot) in per_index.iter_mut().enumerate() {
                let prop = properties.get(&ArrayKey::Int(i as i64))?;
                slot.merge_with(&prop.ty);
            }
        }
    }
    if per_index.is_empty() {
        None
    } else {
        Some(per_index)
    }
}

/// Synthesize one span per expanded element of a sole spread argument, since
/// the source array has no per-element location of its own. Reusing the
/// SAME span for every element would collide in the issue de-duplication key
/// (kind + file + line + col_start — see `mir_issues::IssueBuffer::add`) and
/// silently drop every diagnostic after the first mismatching element.
/// Nudges `start` forward by up to `span.len() - 1` bytes per index so each
/// element gets a distinct (if approximate) column within the same expression.
pub fn distinct_spans_for_expansion(span: Span, count: usize) -> Vec<Span> {
    let max_nudge = span.len().saturating_sub(1);
    (0..count)
        .map(|i| {
            let start = span.start + (i as u32).min(max_nudge);
            Span {
                start,
                end: span.end.max(start),
            }
        })
        .collect()
}

fn substitute_static_in_type(
    t: Type,
    receiver_fqcn: &Arc<str>,
    receiver_type_params: &[Type],
) -> Type {
    let from_docblock = t.from_docblock;
    let types: Vec<Atomic> = t
        .types
        .into_iter()
        .map(|a| substitute_static_atom(a, receiver_fqcn, receiver_type_params))
        .collect();
    let mut result = Type::from_vec(types);
    result.from_docblock = from_docblock;
    result
}

fn substitute_static_atom(a: Atomic, fqcn: &Arc<str>, receiver_type_params: &[Type]) -> Atomic {
    match a {
        // Preserve the receiver's own inferred type params (e.g. `Box<int>`)
        // rather than erasing them to a bare `Box` — otherwise a fluent
        // `: static`-returning method loses generic tracking for the rest of
        // the call chain (`$box->withValue($v)->get()` would return `mixed`
        // instead of the receiver's actual template binding).
        Atomic::TStaticObject { .. } | Atomic::TSelf { .. } => Atomic::TNamedObject {
            fqcn: Name::from(fqcn.as_ref()),
            type_params: mir_types::union::vec_to_type_params(receiver_type_params.to_vec()),
        },
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(substitute_static_in_type(
                *value,
                fqcn,
                receiver_type_params,
            )),
        },
        Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
            value: Box::new(substitute_static_in_type(
                *value,
                fqcn,
                receiver_type_params,
            )),
        },
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(substitute_static_in_type(*key, fqcn, receiver_type_params)),
            value: Box::new(substitute_static_in_type(
                *value,
                fqcn,
                receiver_type_params,
            )),
        },
        Atomic::TNonEmptyArray { key, value } => Atomic::TNonEmptyArray {
            key: Box::new(substitute_static_in_type(*key, fqcn, receiver_type_params)),
            value: Box::new(substitute_static_in_type(
                *value,
                fqcn,
                receiver_type_params,
            )),
        },
        // `self<T>`/`static<T>`/`parent<T>`/`$this<T>` sentinel written in a
        // self-out annotation (see `parse_self_out_type` in the docblock
        // parser) — unlike a bare `TSelf`/`TStaticObject`, this carries the
        // annotation's own `<T>` args (often a method-level template, e.g.
        // `self<U>`), which must survive into the resolved receiver class so
        // the caller's later `substitute_templates` can still replace `U`
        // with this call's inferred binding, instead of being erased like
        // the top-level `self`/`static` arms above.
        Atomic::TNamedObject {
            fqcn: obj_fqcn,
            type_params,
        } if matches!(obj_fqcn.as_ref(), "self" | "static" | "parent" | "$this") => {
            let substituted: Vec<Type> = type_params
                .iter()
                .map(|t| substitute_static_in_type(t.clone(), fqcn, receiver_type_params))
                .collect();
            Atomic::TNamedObject {
                fqcn: Name::from(fqcn.as_ref()),
                type_params: mir_types::union::vec_to_type_params(substituted),
            }
        }
        // `static`/`self` nested in generic arguments (`Builder<static>`) must
        // resolve like the top-level forms, or the unresolved atom degrades
        // every downstream template binding on the returned object to mixed.
        Atomic::TNamedObject {
            fqcn: obj_fqcn,
            type_params,
        } if !type_params.is_empty() => {
            let substituted: Vec<Type> = type_params
                .iter()
                .map(|t| substitute_static_in_type(t.clone(), fqcn, receiver_type_params))
                .collect();
            Atomic::TNamedObject {
                fqcn: obj_fqcn,
                type_params: mir_types::union::vec_to_type_params(substituted),
            }
        }
        other => other,
    }
}

/// Replace `TStaticObject` / `TSelf` in a method's return type with the actual receiver FQCN.
/// Also recurses into array and list value types so `@return static[]` is correctly resolved.
/// `receiver_type_params` carries the receiver's own inferred type params (empty for a
/// receiver with no known params, e.g. a plain `Foo::bar()` static call) so a `: static`
/// return doesn't erase them.
pub(crate) fn substitute_static_in_return(
    ret: Type,
    receiver_fqcn: &Arc<str>,
    receiver_type_params: &[Type],
) -> Type {
    substitute_static_in_type(ret, receiver_fqcn, receiver_type_params)
}

pub(crate) fn check_method_visibility(
    ea: &mut ExpressionAnalyzer<'_>,
    visibility: Visibility,
    owner_fqcn: &Arc<str>,
    method_name: &Arc<str>,
    ctx: &crate::flow_state::FlowState,
    span: Span,
) {
    check_method_visibility_with_magic(ea, visibility, owner_fqcn, method_name, ctx, span, "__call")
}

/// Same as [`check_method_visibility`], but lets the caller pick which magic
/// method intercepts an inaccessible call at runtime — instance calls fall
/// back to `__call`, static calls to `__callStatic`.
pub(crate) fn check_method_visibility_with_magic(
    ea: &mut ExpressionAnalyzer<'_>,
    visibility: Visibility,
    owner_fqcn: &Arc<str>,
    method_name: &Arc<str>,
    ctx: &crate::flow_state::FlowState,
    span: Span,
    magic_method: &str,
) {
    let disallowed = match visibility {
        Visibility::Private => {
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            let from_trait =
                crate::db::class_kind(ea.db, owner_fqcn.as_ref()).is_some_and(|k| k.is_trait);
            !(caller_fqcn == owner_fqcn.as_ref()
                || (from_trait
                    && crate::db::extends_or_implements(ea.db, caller_fqcn, owner_fqcn.as_ref())))
        }
        Visibility::Protected => {
            let caller_fqcn = ctx.self_fqcn.as_deref().unwrap_or("");
            // Anonymous classes are never collected into the class-hierarchy DB
            // (the collector skips them), so `extends_or_implements` can't see
            // their `extends` clause. `ctx.parent_fqcn` is derived straight from
            // the AST for them (see `analyze_class_decl_stmt`), so fall back to
            // it: if the immediate parent is or extends the owner, `self` does too.
            let related_via_parent = ctx.parent_fqcn.as_deref().is_some_and(|parent| {
                parent == owner_fqcn.as_ref()
                    || crate::db::extends_or_implements(ea.db, parent, owner_fqcn.as_ref())
            });
            caller_fqcn.is_empty()
                || !(caller_fqcn == owner_fqcn.as_ref()
                    || crate::db::extends_or_implements(ea.db, caller_fqcn, owner_fqcn.as_ref())
                    || related_via_parent)
        }
        Visibility::Public => false,
    };
    // An inaccessible method call is dispatched to `__call` at runtime when
    // the class (chain) defines one — e.g. Laravel's Router::prefix() is
    // protected and external callers go through Macroable::__call.
    if disallowed && !crate::db::has_method_in_chain(ea.db, owner_fqcn, magic_method) {
        ea.emit(
            IssueKind::UndefinedMethod {
                class: owner_fqcn.to_string(),
                method: method_name.to_string(),
            },
            Severity::Error,
            span,
        );
    }
}

pub(crate) fn expr_can_be_passed_by_reference_owned(expr: &php_ast::owned::Expr) -> bool {
    matches!(
        expr.kind,
        php_ast::owned::ExprKind::Variable(_)
            | php_ast::owned::ExprKind::ArrayAccess(_)
            | php_ast::owned::ExprKind::PropertyAccess(_)
            | php_ast::owned::ExprKind::NullsafePropertyAccess(_)
            | php_ast::owned::ExprKind::StaticPropertyAccess(_)
            | php_ast::owned::ExprKind::StaticPropertyAccessDynamic { .. }
    )
}

// ---------------------------------------------------------------------------
// Orchestrator
// ---------------------------------------------------------------------------

pub(crate) fn check_args(ea: &mut ExpressionAnalyzer<'_>, p: CheckArgsParams<'_>) {
    let CheckArgsParams {
        fn_name,
        params,
        arg_types,
        arg_spans,
        arg_names,
        arg_can_be_byref,
        call_span,
        has_spread,
        arity_unknown,
        template_params,
        no_named_arguments,
    } = p;

    let bindings = counts::check_counts(
        ea,
        fn_name,
        params,
        arg_types,
        arg_spans,
        arg_names,
        call_span,
        has_spread,
        arity_unknown,
        no_named_arguments,
    );

    for ArgBinding {
        param_idx,
        arg_ty,
        arg_span,
        arg_idx,
    } in &bindings
    {
        let param = &params[*param_idx];

        if param.is_byref && !arg_can_be_byref.get(*arg_idx).copied().unwrap_or(false) {
            ea.emit(
                IssueKind::InvalidPassByReference {
                    fn_name: fn_name.to_string(),
                    param: param.name.to_string(),
                },
                Severity::Error,
                *arg_span,
            );
        }

        if let Some(raw_param_ty) = &param.ty {
            // A variadic param's docblock type may be spelled either as the
            // bare element (`string ...$args`) or as an aggregate array/list
            // (`array<int, V> ...$args`, `list<V> ...$args`) — each individual
            // argument must be checked against the element type either way.
            // Reuses generic.rs's inference-side unwrapper so both call paths
            // agree on which spellings count as "aggregate".
            let param_ty: &Type = if param.is_variadic {
                crate::generic::variadic_element_type(raw_param_ty)
            } else {
                raw_param_ty
            };

            // types::check_one handles the full per-binding sequence: callable-sig validations,
            // null checks (via nullability::check_one), and type-compat checks.
            types::check_one(
                ea,
                fn_name,
                &param.name,
                param_ty,
                arg_ty,
                *arg_span,
                *arg_idx,
                template_params,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Shared predicate (used by types.rs and nullability.rs via super::)
// ---------------------------------------------------------------------------

fn param_contains_template_or_unknown(
    param_ty: &Type,
    arg_ty: &Type,
    ea: &ExpressionAnalyzer<'_>,
    template_params: &[TemplateParam],
) -> bool {
    let template_names: rustc_hash::FxHashSet<&str> =
        template_params.iter().map(|tp| tp.name.as_ref()).collect();

    fn has_template_param(union: &Type, template_names: &rustc_hash::FxHashSet<&str>) -> bool {
        union.types.iter().any(|atomic| match atomic {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TNamedObject { fqcn, type_params } => {
                // Check if this name is a template parameter
                if !fqcn.contains('\\') && template_names.contains(fqcn.as_ref()) {
                    return true;
                }
                // Check nested type_params for template parameters only
                type_params
                    .iter()
                    .any(|tp| has_template_param(tp, template_names))
            }
            Atomic::TClassString(Some(inner)) | Atomic::TInterfaceString(Some(inner)) => {
                !inner.contains('\\') && template_names.contains(inner.as_ref())
            }
            _ => false,
        })
    }

    param_ty.types.iter().any(|atomic| match atomic {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TNamedObject { fqcn, type_params } => {
            // Check if this name is a template parameter
            if !fqcn.contains('\\') && template_names.contains(fqcn.as_ref()) {
                return true;
            }
            // Check if this is an unknown type
            if !fqcn.contains('\\') && !crate::db::class_exists(ea.db, fqcn.as_ref()) {
                return true;
            }
            // Check nested type_params for template parameters only
            if type_params.is_empty() || !has_template_param(param_ty, &template_names) {
                return false;
            }
            // A generic param like `Bar<T>` should only forgive the argument when
            // it could plausibly BE a `Bar` regardless of what `T` resolves to —
            // mirroring how the TIntersection arm below forgives only the
            // templated part while still enforcing the concrete parts. An
            // argument whose own type can never satisfy `Bar` at the class level
            // (a bare scalar, or an unrelated, unrelated-by-inheritance class) is
            // still a real mismatch no `T` binding could ever paper over.
            arg_ty.types.iter().any(|arg_atomic| match arg_atomic {
                Atomic::TNamedObject { fqcn: arg_fqcn, .. }
                | Atomic::TStaticObject { fqcn: arg_fqcn }
                | Atomic::TSelf { fqcn: arg_fqcn } => {
                    arg_fqcn == fqcn
                        || crate::db::extends_or_implements(ea.db, arg_fqcn.as_ref(), fqcn.as_ref())
                        || crate::db::extends_or_implements(ea.db, fqcn.as_ref(), arg_fqcn.as_ref())
                        || crate::db::has_unknown_ancestor(ea.db, arg_fqcn.as_ref())
                }
                Atomic::TObject | Atomic::TMixed | Atomic::TTemplateParam { .. } => true,
                _ => false,
            })
        }
        Atomic::TClassString(Some(inner)) | Atomic::TInterfaceString(Some(inner)) => {
            // Check if this name is a template parameter
            if !inner.contains('\\') && template_names.contains(inner.as_ref()) {
                return true;
            }
            // Check if this is an unknown type
            !inner.contains('\\') && !crate::db::class_exists(ea.db, inner.as_ref())
        }
        Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
            fn contains_template_or_unknown(
                t: &Type,
                ea: &ExpressionAnalyzer<'_>,
                template_names: &rustc_hash::FxHashSet<&str>,
            ) -> bool {
                t.types.iter().any(|v| match v {
                    Atomic::TTemplateParam { .. } => true,
                    Atomic::TNamedObject { fqcn, .. } => {
                        if !fqcn.contains('\\') && template_names.contains(fqcn.as_ref()) {
                            return true;
                        }
                        !fqcn.contains('\\') && !crate::db::class_exists(ea.db, fqcn.as_ref())
                    }
                    _ => false,
                })
            }
            // A templated/unknown array KEY (e.g. `@template TKey of array-key`
            // in `array<TKey, TValue>`) must also be forgiven — previously only
            // the value type was checked, so a key-only template left this arm
            // returning false and a legitimate arg risked a false InvalidArgument.
            contains_template_or_unknown(key, ea, &template_names)
                || contains_template_or_unknown(value, ea, &template_names)
        }
        Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
            value.types.iter().any(|v| match v {
                Atomic::TTemplateParam { .. } => true,
                Atomic::TNamedObject { fqcn, .. } => {
                    if !fqcn.contains('\\') && template_names.contains(fqcn.as_ref()) {
                        return true;
                    }
                    !fqcn.contains('\\') && !crate::db::class_exists(ea.db, fqcn.as_ref())
                }
                _ => false,
            })
        }
        // For A&B intersections containing a template, only suppress the
        // InvalidArgument if the arg satisfies all the concrete (non-template)
        // parts. If a concrete part is violated (e.g. arg doesn't implement
        // Taggable), the error is a true positive and should still fire.
        Atomic::TIntersection { parts } => {
            let has_template = parts
                .iter()
                .any(|part| has_template_param(part, &template_names));
            if !has_template {
                return false;
            }
            // Check that every concrete (non-template) part is satisfied by arg_ty.
            parts.iter().all(|part| {
                if has_template_param(part, &template_names) {
                    return true; // template part — forgiven
                }
                // Concrete part: arg_ty must satisfy it via extends/implements.
                // Also flatten TIntersection in arg_ty (e.g. Box<string>&Taggable as arg).
                part.types.iter().all(|part_atomic| {
                    let part_fqcn = match part_atomic {
                        Atomic::TNamedObject { fqcn, .. } => fqcn,
                        _ => return true,
                    };
                    let arg_satisfies = |arg_fqcn: &Name| {
                        arg_fqcn == part_fqcn
                            || crate::db::extends_or_implements(
                                ea.db,
                                arg_fqcn.as_ref(),
                                part_fqcn.as_ref(),
                            )
                    };
                    arg_ty.types.iter().any(|arg_atomic| match arg_atomic {
                        Atomic::TNamedObject { fqcn, .. } => arg_satisfies(fqcn),
                        Atomic::TIntersection { parts: arg_parts } => arg_parts
                            .iter()
                            .any(|ap| ap.types.iter().any(|a| matches!(a, Atomic::TNamedObject { fqcn, .. } if arg_satisfies(fqcn)))),
                        _ => false,
                    })
                })
            })
        }
        _ => false,
    })
}
