mod args;
mod array_builtins;
mod builtin_purity;
mod callable;
mod function;
pub(crate) mod method;
mod opaque_callback;
mod static_call;

pub(crate) use args::substitute_static_in_return;
pub use args::{
    check_constructor_args, distinct_spans_for_expansion, expand_sole_spread_arg,
    spread_element_type, CheckArgsParams,
};
pub(crate) use callable::callable_string_token_span;
pub(crate) use function::extract_class_docblock;
pub(crate) use static_call::resolve_literal_class_scope;

pub struct CallAnalyzer;

/// Whether an argument of type `ty` holds an object a callee could mutate.
pub(crate) fn arg_object_is_mutable(db: &dyn crate::db::MirDatabase, ty: &mir_types::Type) -> bool {
    ty.types.iter().any(|atom| match atom {
        mir_types::Atomic::TNamedObject { fqcn, .. }
        | mir_types::Atomic::TSelf { fqcn }
        | mir_types::Atomic::TStaticObject { fqcn }
        | mir_types::Atomic::TParent { fqcn } => {
            !crate::db::object_type_is_immutable(db, fqcn.as_ref())
        }
        _ => false,
    })
}

/// Resolve a declared parameter's own textual argument position at a call
/// site — the index into `arg_types` (built in call-site TEXTUAL order),
/// not the parameter's own DECLARED index. The two differ when a named
/// argument reorders the call, so `resolve_conditional_returns`'s call sites
/// (`@return ($x is T ? A : B)`) must not index `arg_types` by declared
/// position. Mirrors `narrowing/assertions.rs`'s `arg_for_param_index`.
pub(crate) fn resolve_named_arg_type_index(
    params: &[mir_codebase::definitions::DeclaredParam],
    call_args: &[php_ast::owned::Arg],
    param_index: usize,
) -> Option<usize> {
    let param_name = params.get(param_index)?.name.as_ref();
    if let Some(idx) = call_args.iter().position(|a| {
        a.name
            .as_ref()
            .is_some_and(|n| crate::parser::name_to_string_owned(n) == param_name)
    }) {
        return Some(idx);
    }
    call_args
        .iter()
        .enumerate()
        .filter(|(_, a)| a.name.is_none())
        .nth(param_index)
        .map(|(i, _)| i)
}

/// The type a conditional return's `$param` discriminates on: the call-site
/// argument, or a `true`/`false`/`null` default when the argument is omitted.
pub(crate) fn conditional_param_type(
    params: &[mir_codebase::definitions::DeclaredParam],
    call_args: &[php_ast::owned::Arg],
    arg_types: &[mir_types::Type],
    param_name: &str,
) -> Option<mir_types::Type> {
    use mir_types::Atomic;
    let idx = params.iter().position(|p| p.name.as_ref() == param_name)?;
    if let Some(arg_idx) = resolve_named_arg_type_index(params, call_args, idx) {
        return arg_types.get(arg_idx).cloned();
    }
    let default = params[idx].default_text.as_deref()?.trim();
    let atomic = match default.to_ascii_lowercase().as_str() {
        "true" => Atomic::TTrue,
        "false" => Atomic::TFalse,
        "null" => Atomic::TNull,
        _ => return None,
    };
    Some(mir_types::Type::single(atomic))
}

/// `@return ($this is X ? A : B)` / `@return ($param is X ? A : B)` /
/// `@return (T is X ? A : B)` where
/// `X` is a CLASS name: `Type::resolve_conditional_returns` alone can never
/// resolve this, even once the discriminator's argument type is looked up
/// correctly — it's purely structural (`mir-types` has no `db`, so no
/// ancestor walk) and its closed predicate set (null/true/false/string/
/// list/array/int/float/bool) has no arm for an object atom at all, so an
/// object-typed subject always falls through to "unresolvable, widen to
/// the union of both branches" regardless of the looked-up argument.
/// Pre-resolves exactly that one shape (a single top-level `TConditional`
/// atom whose subject and looked-up argument are both object-like) via
/// `subtype::is_subtype`, then defers to the existing structural resolver
/// for every other shape (scalar discriminant, non-single-atomic type,
/// nested conditionals in the chosen branch) unchanged.
pub(crate) fn resolve_conditional_return<F>(
    ty: mir_types::Type,
    db: &dyn crate::db::MirDatabase,
    templates: Option<&rustc_hash::FxHashMap<mir_types::Name, mir_types::Type>>,
    lookup: F,
) -> mir_types::Type
where
    F: Fn(&str) -> Option<mir_types::Type>,
{
    resolve_conditional_dyn(ty, db, templates, &lookup)
}

fn resolve_conditional_dyn(
    ty: mir_types::Type,
    db: &dyn crate::db::MirDatabase,
    templates: Option<&rustc_hash::FxHashMap<mir_types::Name, mir_types::Type>>,
    lookup: &dyn Fn(&str) -> Option<mir_types::Type>,
) -> mir_types::Type {
    let mut resolved = mir_types::Type::empty();
    for atomic in ty.types {
        let mir_types::Atomic::TConditional { data } = &atomic else {
            resolved.add_type(atomic);
            continue;
        };
        match resolve_class_subject_branch(data, db, templates, lookup) {
            Some(branch) => resolved.merge_with(&branch),
            None => resolved
                .merge_with(&mir_types::Type::single(atomic).resolve_conditional_returns(lookup)),
        }
    }
    resolved
}

/// The chosen branch of a conditional whose subject and discriminant are both
/// object types, resolved through `is_subtype`.
fn resolve_class_subject_branch(
    data: &mir_types::atomic::ConditionalData,
    db: &dyn crate::db::MirDatabase,
    templates: Option<&rustc_hash::FxHashMap<mir_types::Name, mir_types::Type>>,
    lookup: &dyn Fn(&str) -> Option<mir_types::Type>,
) -> Option<mir_types::Type> {
    use mir_types::Atomic;
    let is_object = |ty: &mir_types::Type| {
        !ty.types.is_empty()
            && ty.types.iter().all(|a| {
                matches!(a, Atomic::TLiteralEnumCase { .. })
                    || a.named_object_fqcn().is_some_and(|f| !f.contains("::"))
            })
    };
    let subject = crate::subtype::canonical_enum_cases(db, &data.subject);
    if !is_object(&subject) {
        return None;
    }
    let param_name = data.param_name.as_ref()?;
    // `T is X`: `T` is a template, bound rather than a parameter.
    let arg_ty = lookup(param_name.as_ref())
        .or_else(|| templates.and_then(|t| t.get(param_name).cloned()))?;
    let arg_ty = crate::subtype::canonical_enum_cases(db, &arg_ty);
    if !is_object(&arg_ty) {
        return None;
    }
    let branch = if crate::subtype::is_subtype(db, &arg_ty, &subject) {
        data.if_true.clone()
    } else {
        let disjoint = arg_ty
            .types
            .iter()
            .all(|a| subject.types.iter().all(|s| provably_disjoint(db, a, s)));
        if !disjoint {
            return None;
        }
        data.if_false.clone()
    };
    Some(resolve_conditional_dyn(branch, db, templates, lookup))
}

/// True when no value can be both `a` and `s`, given `a` is not a subtype of `s`.
/// Unknown or enum-vs-class pairs are not provably disjoint.
fn provably_disjoint(
    db: &dyn crate::db::MirDatabase,
    a: &mir_types::Atomic,
    s: &mir_types::Atomic,
) -> bool {
    use mir_types::Atomic;
    if let (Atomic::TLiteralEnumCase { .. }, Atomic::TLiteralEnumCase { .. }) = (a, s) {
        return a != s;
    }
    let (Some(a), Some(s)) = (a.named_object_fqcn(), s.named_object_fqcn()) else {
        return false;
    };
    let kind = |f: &str| crate::db::class_kind(db, f);
    let (Some(ak), Some(sk)) = (kind(a), kind(s)) else {
        return false;
    };
    if a == s || crate::db::extends_or_implements(db, s, a) {
        return false;
    }
    // A subclass of a non-final class or interface may also implement an interface.
    crate::db::is_final(db, a)
        || crate::db::is_final(db, s)
        || !(ak.is_interface || sk.is_interface)
}

/// An assignment expression in argument position (`f($x = expr)`,
/// `->andReturn($mock = m::mock(...))`) has its value consumed by the call —
/// the write is used even if the variable is never read again.
pub(crate) fn consume_arg_assignment(
    expr: &php_ast::owned::Expr,
    ctx: &mut crate::flow_state::FlowState,
) {
    if let php_ast::owned::ExprKind::Assign(a) = &expr.kind {
        if let php_ast::owned::ExprKind::Variable(name) = &a.target.kind {
            let n = name.trim_start_matches('$');
            ctx.read_vars.insert(mir_types::Name::from(n));
            ctx.mark_consumed(n);
        }
    }
}

/// Pre-mark variables passed to by-reference parameters as defined.
///
/// Passing an as-yet-undefined variable to an out-parameter (e.g. `&$matches`
/// in `preg_match`, or a user method's `&$out`) defines it, so it must not be
/// reported as `UndefinedVariable`. Variadic by-ref params (`&...$rest`) cover
/// every argument from their position onward. Must run before the arguments are
/// analyzed so the read side never sees the variable as undefined.
///
/// Binds each arg to its param the same way `call/args/counts.rs::check_counts`
/// does — honoring named arguments that reorder which param a given arg feeds
/// — rather than assuming `args[i]` always feeds `params[i]`. That fuller
/// binder can't be reused directly here since it needs each arg's inferred
/// type, which isn't available yet (premarking must run before args are
/// analyzed).
pub(crate) fn premark_byref_arg_vars(
    params: &[mir_codebase::definitions::DeclaredParam],
    args: &[php_ast::owned::Arg],
    ctx: &mut crate::flow_state::FlowState,
) {
    use php_ast::owned::ExprKind;

    fn premark_one(
        param: &mir_codebase::definitions::DeclaredParam,
        arg: &php_ast::owned::Arg,
        ctx: &mut crate::flow_state::FlowState,
    ) {
        if !param.is_byref {
            return;
        }
        let Some(value) = &arg.value else { return };
        if let ExprKind::Variable(name) = &value.kind {
            let var_name = name.trim_start_matches('$');
            if !ctx.var_is_defined(var_name) {
                // Prefer @param-out type if declared; fall back to declared
                // in-type, then mixed.
                let ty = param
                    .out_ty
                    .as_ref()
                    .or(param.ty.as_ref())
                    .map(|t| (**t).clone())
                    .unwrap_or_else(mir_types::Type::mixed);
                ctx.set_var(var_name, ty);
            }
        }
    }

    let variadic_index = params.iter().position(|p| p.is_variadic);
    let max_positional = variadic_index.unwrap_or(params.len());
    let mut positional = 0usize;

    for arg in args {
        // A spread (`...$rest`) can't be statically bound to a single param.
        if arg.unpack {
            break;
        }
        if let Some(name) = &arg.name {
            let name = crate::parser::name_to_string_owned(name);
            if let Some(param) = params.iter().find(|p| p.name.as_ref() == name.as_str()) {
                premark_one(param, arg, ctx);
            } else if let Some(vi) = variadic_index {
                premark_one(&params[vi], arg, ctx);
            }
            continue;
        }
        let param = if positional < max_positional {
            params.get(positional)
        } else {
            variadic_index.map(|vi| &params[vi])
        };
        if let Some(param) = param {
            premark_one(param, arg, ctx);
        }
        positional += 1;
    }
}

/// Type a by-ref argument holds after the call: `@param-out`, else the declared
/// param type (the callee may have written anything the type admits).
pub(crate) fn byref_written_type(
    param: &mir_codebase::definitions::DeclaredParam,
) -> Option<&std::sync::Arc<mir_types::Type>> {
    param.out_ty.as_ref().or(if param.is_byref {
        param.ty.as_ref()
    } else {
        None
    })
}

/// True when `ty` still names an unbound template param.
pub(crate) fn has_unbound_template(ty: &mir_types::Type) -> bool {
    use mir_types::Atomic;
    ty.types.iter().any(|a| match a {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
            has_unbound_template(key) || has_unbound_template(value)
        }
        Atomic::TList { value } | Atomic::TNonEmptyList { value } => has_unbound_template(value),
        Atomic::TNamedObject { type_params, .. } => type_params.iter().any(has_unbound_template),
        _ => false,
    })
}

// Reusable per-thread buffer for arg_types collection. The Option lets
// reentrant calls (foo(bar(baz()))) detect they can't borrow the same buffer
// and fall back to a fresh allocation.
thread_local! {
    pub(crate) static ARG_TYPES_BUF: std::cell::RefCell<Option<Vec<mir_types::Type>>> =
        const { std::cell::RefCell::new(Some(Vec::new())) };
}

/// Param types a callee passes to a closure bound to `param` (`callable(E)` /
/// `Closure(E)`), with `bindings` substituted.
pub(crate) fn callback_param_hints(
    ea: &crate::expr::ExpressionAnalyzer<'_>,
    param: &mir_codebase::definitions::DeclaredParam,
    bindings: &rustc_hash::FxHashMap<mir_types::Name, mir_types::Type>,
) -> Option<Vec<Option<mir_types::Type>>> {
    use mir_types::Atomic;
    let ty = param.ty.as_ref()?.substitute_templates(bindings);
    let mut callables = ty.types.iter().filter_map(|a| match a {
        Atomic::TCallable {
            params: Some(params),
            ..
        } => Some(params),
        Atomic::TClosure { data } => Some(&data.params),
        _ => None,
    });
    let fn_params = callables.next()?;
    if callables.next().is_some() {
        return None;
    }
    Some(
        fn_params
            .iter()
            .map(|p| {
                p.ty.as_ref()
                    .map(|t| callable::resolve_enum_case_refs(&t.to_union(), ea))
            })
            .collect(),
    )
}
