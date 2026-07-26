//! `@psalm-assert-if-true`/`@psalm-assert-if-false` docblock-assertion
//! narrowing: applies a callee's declared assertions to the calling flow
//! state for free functions, methods, and static methods.
use php_ast::owned::ExprKind;

use mir_codebase::definitions::AssertionKind;
use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::arrays::{
    get_shape_path_type, resolve_shape_base_current_type, set_shape_base_narrowed, set_shape_path,
    ShapeBase,
};
use super::core::{
    extract_any_prop_access, extract_class_fqcn_from_expr, extract_prop_access,
    extract_static_prop_access, extract_var_name, narrow_receiver_non_null_on_prop_match,
    resolve_prop_current_type, resolve_static_prop_current_type,
};
use super::instanceof_core::filter_out_instanceof_match;

pub(super) fn apply_docblock_assertions(
    call: &php_ast::owned::FunctionCallExpr,
    ctx: &mut FlowState,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
    fn_name: &str,
) -> bool {
    let fn_name = fn_name
        .strip_prefix('\\')
        .map(|s| s.to_string())
        .unwrap_or_else(|| fn_name.to_string());
    let fn_active = |name: &str| -> bool {
        let here = crate::db::Fqcn::from_str(db, name);
        crate::db::find_function(db, here).is_some()
    };
    let resolved_fn_name = {
        let qualified = crate::db::resolve_name(db, file, &fn_name);
        if fn_active(qualified.as_str()) {
            qualified
        } else if fn_active(fn_name.as_str()) {
            fn_name.clone()
        } else {
            qualified
        }
    };

    let here = crate::db::Fqcn::from_str(db, resolved_fn_name.as_str());
    let Some(f) = crate::db::find_function(db, here) else {
        return false;
    };
    apply_assertions(
        &f.assertions,
        &f.params,
        &f.template_params,
        &call.args,
        ctx,
        is_true,
        db,
        file,
    )
}

/// Method-call counterpart of `apply_docblock_assertions` — the callee is
/// already resolved (via `resolve_method_from_db`, shared with both instance
/// and static method-call resolution) instead of looked up by free-function
/// name here.
pub(super) fn apply_method_docblock_assertions(
    call_args: &[php_ast::owned::Arg],
    resolved: &crate::call::method::ResolvedMethod,
    ctx: &mut FlowState,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) -> bool {
    if resolved.assertions.is_empty() {
        return false;
    }
    apply_assertions(
        &resolved.assertions,
        &resolved.params,
        &resolved.template_params,
        call_args,
        ctx,
        is_true,
        db,
        file,
    )
}

/// Shared assertion-application logic for both `@psalm-assert-if-true`/
/// `-if-false` docblock forms, used by both free functions
/// (`apply_docblock_assertions`) and methods/static methods
/// (`apply_method_docblock_assertions`) — narrows whichever argument each
/// matching assertion names to var/prop/static-prop.
#[allow(clippy::too_many_arguments)]
fn apply_assertions(
    assertions: &[mir_codebase::definitions::Assertion],
    params: &[mir_codebase::definitions::DeclaredParam],
    template_params: &[mir_codebase::definitions::TemplateParam],
    call_args: &[php_ast::owned::Arg],
    ctx: &mut FlowState,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) -> bool {
    let expected_kind = if is_true {
        AssertionKind::AssertIfTrue
    } else {
        AssertionKind::AssertIfFalse
    };

    // An assertion type written in terms of the callee's own `@template`s
    // (e.g. `@psalm-assert-if-true T $value` alongside `@param
    // class-string<T> $class`) must resolve T from this call's actual
    // arguments before narrowing — otherwise the variable narrows to the
    // bare, unresolved template atom instead of the concrete type.
    let template_bindings =
        compute_assertion_template_bindings(template_params, params, call_args, ctx, db, file);

    let mut applied = false;
    for assertion in assertions
        .iter()
        .filter(|a| a.kind == expected_kind || (is_true && a.kind == AssertionKind::Assert))
    {
        if apply_one_assertion(
            assertion,
            params,
            call_args,
            ctx,
            template_bindings.as_ref(),
            db,
            file,
        ) {
            applied = true;
        }
    }

    applied
}

/// Apply a single already-selected assertion to its matching argument(s) —
/// the per-assertion body shared between `apply_assertions`'s conditional
/// if-true/if-false narrowing (pre-filtered by kind + branch) and a bare,
/// unconditional `@psalm-assert` statement call (which always applies,
/// regardless of any condition, and must never fall through to the generic
/// per-call-site `AssertionKind::Assert` handling that used to be
/// hand-duplicated in `call/function.rs`, `call/method.rs`, and
/// `call/static_call.rs` — none of which ever read `assertion.param_key`,
/// handled a variadic param, or resolved a named argument, unlike this
/// shared body).
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_one_assertion(
    assertion: &mir_codebase::definitions::Assertion,
    params: &[mir_codebase::definitions::DeclaredParam],
    call_args: &[php_ast::owned::Arg],
    ctx: &mut FlowState,
    template_bindings: Option<&rustc_hash::FxHashMap<mir_types::Name, Type>>,
    db: &dyn MirDatabase,
    file: &str,
) -> bool {
    let Some(index) = params.iter().position(|p| p.name == assertion.param) else {
        return false;
    };
    let mut applied = false;
    // A variadic param's assertion applies to every trailing positional
    // arg it swallows (`assertVariadic(...$values)` asserted over each
    // of `assertVariadic($a, $b, $c)`), not just the first one —
    // `arg_for_param_index` only ever resolves a single positional arg.
    let variadic_args: Vec<&php_ast::owned::Arg>;
    let args_to_check: &[&php_ast::owned::Arg] = if params[index].is_variadic {
        variadic_args = call_args
            .iter()
            .filter(|a| a.name.is_none())
            .skip(index)
            .collect();
        &variadic_args
    } else {
        variadic_args = arg_for_param_index(params, call_args, index)
            .into_iter()
            .collect();
        &variadic_args
    };
    for arg in args_to_check {
        // `@psalm-assert-if-true Type $arr['key']` — the assertion
        // targets a specific key of this parameter, not the whole
        // argument. Build a shape-path target from the argument
        // expression + the asserted key (rather than narrowing the
        // argument's own whole value) and SET that key's type,
        // adding it to the shape if not already present — the
        // array-key-refinement machinery `isset()`/`empty()` already
        // use for NARROWING an existing key, applied here as an
        // ASSIGN instead.
        if !assertion.param_key.is_empty() {
            let path = &assertion.param_key;
            let base = if let Some(name) = extract_var_name(&arg.value) {
                Some(ShapeBase::Var(name))
            } else if let Some((obj, prop)) = extract_any_prop_access(&arg.value) {
                Some(ShapeBase::Prop(obj, prop))
            } else {
                extract_static_prop_access(&arg.value, ctx, db, file)
                    .map(|(fqcn, prop)| ShapeBase::Static(fqcn, prop))
            };
            if let Some(base) = base {
                let current = resolve_shape_base_current_type(ctx, &base, db, file);
                let ty = match template_bindings {
                    Some(b) => assertion.ty.substitute_templates(b),
                    None => assertion.ty.clone(),
                };
                let ty = if assertion.negated {
                    let current_leaf = get_shape_path_type(&current, path);
                    negate_assertion_type(&current_leaf, &ty, db)
                } else {
                    ty
                };
                let narrowed = set_shape_path(&current, path, &ty);
                set_shape_base_narrowed(ctx, &base, current, narrowed);
                applied = true;
            }
            continue;
        }
        if let Some(var_name) = extract_var_name(&arg.value) {
            let ty = match template_bindings {
                Some(b) => assertion.ty.substitute_templates(b),
                None => assertion.ty.clone(),
            };
            let ty = if assertion.negated {
                negate_assertion_type(&ctx.get_var(&var_name), &ty, db)
            } else {
                ty
            };
            ctx.set_var(&var_name, ty);
            applied = true;
        } else if let Some((obj, prop)) = extract_any_prop_access(&arg.value) {
            let ty = match template_bindings {
                Some(b) => assertion.ty.substitute_templates(b),
                None => assertion.ty.clone(),
            };
            let ty = if assertion.negated {
                let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                negate_assertion_type(&current, &ty, db)
            } else {
                ty
            };
            // `$obj->prop` on a null `$obj` reads as null, so proving
            // the property itself is non-nullable also proves `$obj`
            // wasn't null.
            let proved_prop_non_null = !ty.is_nullable();
            ctx.set_prop_refined(&obj, &prop, ty);
            narrow_receiver_non_null_on_prop_match(ctx, &obj, proved_prop_non_null);
            applied = true;
        } else if let Some((fqcn, prop)) = extract_static_prop_access(&arg.value, ctx, db, file) {
            let ty = match template_bindings {
                Some(b) => assertion.ty.substitute_templates(b),
                None => assertion.ty.clone(),
            };
            let ty = if assertion.negated {
                let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                negate_assertion_type(&current, &ty, db)
            } else {
                ty
            };
            ctx.set_prop_refined(&fqcn, &prop, ty);
            applied = true;
        }
    }
    applied
}

/// Compute the `@template`-resolved bindings for an assertion's callee, the
/// same "resolve T from this call's actual arguments" logic
/// `apply_assertions` needs internally — exposed so a bare, unconditional
/// `@psalm-assert` statement call (which never goes through
/// `apply_assertions` at all, since that dispatch only ever runs for a call
/// used as a boolean CONDITION) can compute the identical bindings before
/// calling `apply_one_assertion` per matching assertion.
pub(crate) fn compute_assertion_template_bindings(
    template_params: &[mir_codebase::definitions::TemplateParam],
    params: &[mir_codebase::definitions::DeclaredParam],
    call_args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<rustc_hash::FxHashMap<mir_types::Name, Type>> {
    if template_params.is_empty() {
        return None;
    }
    let arg_types: Vec<Type> = call_args
        .iter()
        .map(|arg| assertion_arg_type(&arg.value, ctx, db, file))
        .collect();
    let arg_names: Vec<Option<String>> = call_args
        .iter()
        .map(|arg| arg.name.as_ref().map(crate::parser::name_to_string_owned))
        .collect();
    Some(
        crate::generic::infer_template_bindings(
            db,
            template_params,
            params,
            &arg_types,
            &arg_names,
        )
        .0,
    )
}

/// Resolve a method-call receiver's exact class FQCN for dispatching a
/// `@psalm-assert-if-true`/`-if-false` docblock assertion — only handles a
/// receiver resolved to a single concrete class atom, or a `TIntersection`
/// whose parts unambiguously agree on which one declares `method_name`
/// (mirroring `narrow_nullsafe_method_call_null`'s same conservative scope;
/// a union of multiple UNRELATED classes could resolve the same method name
/// to different signatures, so that case still falls through). Handles both
/// a bare-variable receiver (`$v->isInt($p)`) and a property-access receiver
/// (`$this->validator->isInt($p)`, a very common real-world shape) — the
/// latter previously fell through unresolved, silently no-oping the whole
/// assertion. A chained-call-result receiver (`$h->getValidator()->isInt($p)`)
/// stays unresolved, mirroring the same, already-accepted scope limit
/// `@psalm-self-out` documents for a non-variable receiver.
pub(super) fn method_call_receiver_fqcn(
    object: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
    method_name: &str,
) -> Option<std::sync::Arc<str>> {
    let obj_ty = if let Some(obj_var) = extract_var_name(object) {
        ctx.get_var(&obj_var)
    } else if let Some((obj_var, prop)) = extract_any_prop_access(object) {
        // `extract_any_prop_access` also matches a nullsafe (`?->`) receiver
        // chain, unlike the plain-`->`-only `extract_prop_access` this used
        // to call — mirrors the same fix already applied to the self-out
        // write-back's receiver resolution in `call/method.rs`.
        resolve_prop_current_type(ctx, &obj_var, &prop, db, file)
    } else if let Some((fqcn, prop)) = extract_static_prop_access(object, ctx, db, file) {
        // `self::$validator->isValid($x)` — a static-property receiver is a
        // first-class shape everywhere else in this file (the assertion
        // TARGET side already resolves one), but this receiver-resolution
        // helper only ever tried a bare variable or an instance-property
        // chain, silently no-oping assert-if-true/-false narrowing for it.
        resolve_static_prop_current_type(ctx, &fqcn, &prop, db)
    } else {
        return None;
    };
    let non_null_atoms: Vec<&Atomic> = obj_ty
        .types
        .iter()
        .filter(|t| !matches!(t, Atomic::TNull))
        .collect();
    match non_null_atoms.as_slice() {
        [Atomic::TNamedObject { fqcn, .. }]
        | [Atomic::TSelf { fqcn }]
        | [Atomic::TStaticObject { fqcn }]
        | [Atomic::TParent { fqcn }] => Some(std::sync::Arc::from(fqcn.as_ref())),
        // `Foo&Bar` — ordinary method-call resolution (`call/method.rs`'s
        // own `TIntersection` arm) already dispatches to whichever part
        // declares the method, so assertion-tag narrowing riding along the
        // same call must resolve the identical FQCN instead of silently
        // no-oping just because no single member atom matched above.
        [Atomic::TIntersection { parts }] => {
            parts
                .iter()
                .flat_map(|p| p.types.iter())
                .find_map(|atomic| match atomic {
                    Atomic::TNamedObject { fqcn, .. } => {
                        let resolved = crate::db::resolve_name(db, file, fqcn.as_ref());
                        let resolved: std::sync::Arc<str> = std::sync::Arc::from(resolved.as_str());
                        crate::db::has_method_in_chain(db, &resolved, method_name)
                            .then_some(resolved)
                    }
                    _ => None,
                })
        }
        _ => None,
    }
}

/// Resolve a static-method call's class-name expression (`Foo::bar()`,
/// `self::bar()`, `static::bar()`, `parent::bar()`) to a FQCN — the bare-
/// identifier counterpart of `extract_static_prop_access_parts`'s class
/// resolution (that one matches a `StaticPropertyAccess`'s `.class` field;
/// this one matches a `StaticMethodCall`'s). `extract_class_fqcn_from_expr`
/// is the wrong tool here: it resolves `Foo::class`/a string literal, not a
/// bare class-name identifier used directly as a call target.
pub(super) fn resolve_static_call_class_fqcn(
    class_expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<str>> {
    let ExprKind::Identifier(id) = &class_expr.kind else {
        return None;
    };
    let resolved = crate::db::resolve_name(db, file, id.as_ref());
    match resolved.as_str() {
        "self" | "static" => Some(std::sync::Arc::from(
            ctx.self_fqcn.as_deref().or(ctx.static_fqcn.as_deref())?,
        )),
        "parent" => Some(std::sync::Arc::from(ctx.parent_fqcn.as_deref()?)),
        s => Some(std::sync::Arc::from(s)),
    }
}

/// Compute the narrowed type for a negated assertion (`@psalm-assert !Type
/// $x` — "$x is asserted NOT to be this type"): `current` minus `asserted`
/// for the shapes that can be precisely subtracted — `null`, `false`, and a
/// named class/interface (via the same subclass-aware exclusion a
/// `!($x instanceof C)` guard already uses). A union/intersection target
/// (`!A|B`) subtracts each atom in turn — "$x is not A|B" means neither A
/// nor B — instead of bailing out entirely just because the target has more
/// than one atom. Any atom kind not recognized is left unchanged rather than
/// risk excluding too much.
pub(crate) fn negate_assertion_type(current: &Type, asserted: &Type, db: &dyn MirDatabase) -> Type {
    if current.is_mixed_not_template() {
        return current.clone();
    }
    let mut result = current.clone();
    for atomic in &asserted.types {
        result = match atomic {
            Atomic::TNull => result.remove_null(),
            Atomic::TFalse => result.remove_false(),
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn } => filter_out_instanceof_match(&result, fqcn, db),
            _ => result,
        };
    }
    result
}

/// Resolve the call argument that actually feeds `params[param_index]`,
/// honoring named-argument reordering: a named argument binds by name
/// wherever it sits textually, so `call_args[param_index]` is only correct
/// when every argument is positional.
fn arg_for_param_index<'a>(
    params: &[mir_codebase::definitions::DeclaredParam],
    call_args: &'a [php_ast::owned::Arg],
    param_index: usize,
) -> Option<&'a php_ast::owned::Arg> {
    let param_name = params.get(param_index)?.name.as_ref();
    if let Some(arg) = call_args.iter().find(|a| {
        a.name
            .as_ref()
            .is_some_and(|n| crate::parser::name_to_string_owned(n) == param_name)
    }) {
        return Some(arg);
    }
    call_args
        .iter()
        .filter(|a| a.name.is_none())
        .nth(param_index)
}

/// Best-effort type of a call argument for inferring `@template` bindings on
/// an assert-if-true/-false narrowing call — not a full expression
/// evaluator, just enough to resolve the common `class-string<T>`/`T
/// $x`-typed guard-function shapes (e.g. `isInstanceOf($value,
/// Foo::class)`). Anything else falls back to `mixed`, which leaves the
/// template unbound rather than mis-bound.
fn assertion_arg_type(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Type {
    if let Some(var_name) = extract_var_name(expr) {
        return ctx.get_var(&var_name);
    }
    if let Some((obj_var, prop)) = extract_prop_access(expr) {
        return resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
    }
    if let Some(fqcn) = extract_class_fqcn_from_expr(
        expr,
        ctx.self_fqcn.as_deref(),
        ctx.static_fqcn.as_deref(),
        ctx.parent_fqcn.as_deref(),
        db,
        file,
    ) {
        return Type::single(Atomic::TClassString(Some(mir_types::Name::from(
            fqcn.as_ref(),
        ))));
    }
    Type::mixed()
}
