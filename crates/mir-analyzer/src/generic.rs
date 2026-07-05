/// Generic type inference — infer template bindings from argument types and
/// substitute them into return types.
use rustc_hash::FxHashMap;

use mir_codebase::storage::{FnParam, TemplateParam};
use mir_types::{atomic::ArrayKey, union::empty_type_params, Atomic, Name, Type};

use crate::db::MirDatabase;
use crate::subtype::is_subtype;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Infer template parameter bindings by matching parameter types against
/// argument types.
///
/// For example, given `function identity<T>(T $x): T` called with `"hello"`,
/// this returns `{ T → string }`.
pub fn infer_template_bindings(
    template_params: &[TemplateParam],
    params: &[FnParam],
    arg_types: &[Type],
) -> FxHashMap<Name, Type> {
    let mut bindings = infer_arg_template_bindings(template_params, params, arg_types);

    // For any template not bound through arguments, fall back to its bound
    // (or mixed if no bound is declared).
    for tp in template_params {
        bindings
            .entry(Name::from(tp.name.as_ref()))
            .or_insert_with(|| tp.bound.as_deref().cloned().unwrap_or_else(Type::mixed));
    }

    bindings
}

/// Infer template parameter bindings ONLY from the argument types, without the
/// bound/mixed fallback fill that [`infer_template_bindings`] applies.
///
/// A template that no argument binds is simply ABSENT from the returned map —
/// it is *not* inferred to its declared bound. This is the correct primitive for
/// parameterising a `new` receiver: a bounded template the constructor never
/// references must stay `mixed` (bare) downstream, so a later `T`-typed method
/// call does not falsely substitute the param to the bound and reject valid args.
pub fn infer_arg_template_bindings(
    template_params: &[TemplateParam],
    params: &[FnParam],
    arg_types: &[Type],
) -> FxHashMap<Name, Type> {
    let mut bindings: FxHashMap<Name, Type> = FxHashMap::default();
    let template_names: std::collections::HashSet<Name> = template_params
        .iter()
        .map(|tp| Name::from(tp.name.as_ref()))
        .collect();

    for (idx, arg_ty) in arg_types.iter().enumerate() {
        // A trailing variadic param collects every remaining argument: match
        // each one against it instead of stopping at params.len().
        let param = if idx < params.len() {
            &params[idx]
        } else {
            match params.last() {
                Some(p) if p.is_variadic => p,
                _ => break,
            }
        };
        if let Some(param_ty) = &param.ty {
            if param.is_variadic {
                // Variadic docblock types are written aggregate-style
                // (`@param array<X> $args`); each individual argument is an
                // `X`, so unwrap one array layer before matching.
                let elem = variadic_element_type(param_ty);
                infer_from_pair(elem, arg_ty, &template_names, &mut bindings);
            } else {
                infer_from_pair(param_ty, arg_ty, &template_names, &mut bindings);
            }
        }
    }

    bindings
}

/// For a variadic parameter declared aggregate-style (`@param array<X> $args`
/// or `list<X>`), return the element type `X` that each individual argument
/// must match. Types that aren't a single array/list atomic are returned
/// unchanged (e.g. `@param string ...$args` stores the bare element type).
fn variadic_element_type(ty: &Type) -> &Type {
    if ty.types.len() == 1 {
        match &ty.types[0] {
            Atomic::TArray { value, .. } | Atomic::TNonEmptyArray { value, .. } => value,
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => value,
            _ => ty,
        }
    } else {
        ty
    }
}

/// Check that each binding satisfies the template's declared bound, using
/// the codebase to resolve class inheritance chains. This is inheritance-aware
/// and will accept subclasses that satisfy their parent's bound.
/// Returns a list of `(template_name, inferred_type, bound)` for violations.
pub fn check_template_bounds_with_inheritance<'a>(
    db: &dyn MirDatabase,
    bindings: &'a FxHashMap<Name, Type>,
    template_params: &'a [TemplateParam],
) -> Vec<(&'a Name, &'a Type, &'a Type)> {
    // An inferred type that still contains unresolved template placeholders or
    // self/static cannot be meaningfully checked against the bound here — it
    // resolves only at a concrete call site (e.g. Eloquent's TRelatedModel
    // bound by `self`/`static` inside the defining class).
    let is_unresolved = |ty: &Type| {
        ty.types.iter().any(|a| match a {
            Atomic::TTemplateParam { .. }
            | Atomic::TSelf { .. }
            | Atomic::TStaticObject { .. }
            | Atomic::TParent { .. } => true,
            Atomic::TNamedObject { fqcn, type_params } => {
                (type_params.is_empty() && !fqcn.contains('\\') && {
                    let name = fqcn.as_str();
                    name.eq_ignore_ascii_case("self")
                        || name.eq_ignore_ascii_case("static")
                        || name.eq_ignore_ascii_case("parent")
                        || template_params.iter().any(|tp| tp.name.as_ref() == name)
                }) || type_params
                    .iter()
                    .any(|t| is_unresolved_shallow(t, template_params))
            }
            _ => false,
        })
    };

    let mut violations = Vec::new();
    for tp in template_params {
        if let Some(bound) = &tp.bound {
            if let Some(inferred) = bindings.get(&tp.name) {
                // Substitute already-bound template params into the bound before
                // comparing — handles `@template B of A` where A itself is a
                // template that was just bound from another argument.
                let resolved_bound = bound.substitute_templates(bindings);
                if !resolved_bound.is_mixed()
                    && !inferred.is_mixed()
                    && !is_unresolved(inferred)
                    && !is_subtype(db, inferred, &resolved_bound)
                {
                    violations.push((&tp.name, inferred, bound.as_ref()));
                }
            }
        }
    }
    violations
}

/// Shallow variant of the unresolved-placeholder check for nested type params
/// (one level is enough: `Collection<TKey, ...>` with a bare `TKey` inside).
/// Must check the name against the actual declared `template_params` — an
/// ordinary concrete class name is also a bare, namespace-less `TNamedObject`
/// and must not be mistaken for an unbound placeholder.
fn is_unresolved_shallow(ty: &Type, template_params: &[TemplateParam]) -> bool {
    ty.types.iter().any(|a| match a {
        Atomic::TTemplateParam { .. }
        | Atomic::TSelf { .. }
        | Atomic::TStaticObject { .. }
        | Atomic::TParent { .. } => true,
        Atomic::TNamedObject { fqcn, type_params } => {
            type_params.is_empty() && !fqcn.contains('\\') && {
                let name = fqcn.as_str();
                name.eq_ignore_ascii_case("self")
                    || name.eq_ignore_ascii_case("static")
                    || name.eq_ignore_ascii_case("parent")
                    || template_params.iter().any(|tp| tp.name.as_ref() == name)
            }
        }
        _ => false,
    })
}

/// Build template bindings from a receiver's concrete type params.
///
/// Zips `class_template_params` (e.g. `[T]` declared on the class) with
/// `receiver_type_params` (e.g. `[User]` from `Collection<User>`) to produce
/// `{ T → User }`. If the receiver supplies fewer type params than the class
/// declares, the trailing template params are left unbound. If the receiver
/// supplies more, the extras are ignored.
pub fn build_class_bindings(
    class_template_params: &[TemplateParam],
    receiver_type_params: &[Type],
) -> FxHashMap<Name, Type> {
    class_template_params
        .iter()
        .zip(receiver_type_params.iter())
        .map(|(tp, ty)| (Name::from(tp.name.as_ref()), ty.clone()))
        .collect()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// If `param_ty` is a union mixing template placeholders with concrete atomics,
/// return `arg_ty` with the concrete atomics filtered out — what the template
/// should actually bind to. Returns `None` when no filtering is needed.
fn compute_template_residual(
    param_ty: &Type,
    arg_ty: &Type,
    template_names: &std::collections::HashSet<Name>,
) -> Option<Type> {
    let mut has_template = false;
    let mut has_template_class_string = false;
    let mut concrete: Vec<&Atomic> = Vec::new();
    for a in &param_ty.types {
        if is_template_atomic(a, template_names) {
            has_template = true;
        } else if matches!(a, Atomic::TClassString(Some(n)) | Atomic::TInterfaceString(Some(n)) if template_names.contains(n))
        {
            // `class-string<T>`/`interface-string<T>` alongside a bare `T`
            // (Mockery's `class-string<TMock>|TMock` pattern): the
            // class-string/interface-string alternative binds those args
            // itself, so the bare template must not also absorb them.
            has_template_class_string = true;
        } else {
            concrete.push(a);
        }
    }
    if !has_template || (concrete.is_empty() && !has_template_class_string) {
        return None;
    }
    let mut residual = Type::empty();
    residual.from_docblock = arg_ty.from_docblock;
    residual.possibly_undefined = arg_ty.possibly_undefined;
    let mut class_string_consumed = false;
    for a in &arg_ty.types {
        let consumed_by_class_string = has_template_class_string
            && matches!(a, Atomic::TClassString(_) | Atomic::TInterfaceString(_))
            || matches!(a, Atomic::TLiteralString(s) if has_template_class_string && literal_is_class_like(s));
        if consumed_by_class_string {
            class_string_consumed = true;
            continue;
        }
        if !concrete.iter().any(|c| atomics_match_for_filter(c, a)) {
            residual.add_type(a.clone());
        }
    }
    if residual.types.is_empty() {
        // An EMPTY residual is meaningful when a `class-string<T>` alternative
        // consumed the args: the bare template binds nothing at all. Otherwise
        // (all args matched concrete atomics) keep the legacy behavior of
        // binding the full arg type.
        return class_string_consumed.then_some(residual);
    }
    if residual.types.len() == arg_ty.types.len() && !class_string_consumed {
        return None;
    }
    Some(residual)
}

/// Whether a string literal is shaped like a class reference: backslash-
/// separated identifier segments, with at least one backslash or an
/// uppercase first letter. Filters out Mockery's `'alias:Foo'` /
/// `'overload:Foo'` prefixes and plain lowercase words.
fn literal_is_class_like(s: &str) -> bool {
    let t = s.trim_start_matches('\\');
    if t.is_empty() {
        return false;
    }
    let shape_ok = t.split('\\').all(|seg| {
        !seg.is_empty()
            && seg
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || !c.is_ascii())
            && !seg.chars().next().is_some_and(|c| c.is_ascii_digit())
    });
    shape_ok && (s.contains('\\') || t.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
}

fn is_template_atomic(a: &Atomic, template_names: &std::collections::HashSet<Name>) -> bool {
    match a {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TNamedObject { fqcn, type_params } => {
            type_params.is_empty() && !fqcn.contains('\\') && template_names.contains(fqcn)
        }
        _ => false,
    }
}

/// Conservative atomic-kind match for filtering arg atomics out of a residual.
/// Returns true when an arg atomic is "covered" by a concrete param atomic so
/// the template need not absorb it. Only matches the simple kinds we expect to
/// see paired with templates in unions (null, bool, int, string, etc.).
fn atomics_match_for_filter(concrete: &Atomic, arg: &Atomic) -> bool {
    matches!(
        (concrete, arg),
        (Atomic::TNull, Atomic::TNull)
            | (Atomic::TBool, Atomic::TBool)
            | (Atomic::TBool, Atomic::TTrue)
            | (Atomic::TBool, Atomic::TFalse)
            | (Atomic::TTrue, Atomic::TTrue)
            | (Atomic::TFalse, Atomic::TFalse)
            | (Atomic::TInt, Atomic::TInt)
            | (Atomic::TFloat, Atomic::TFloat)
            | (Atomic::TIntegralFloat, Atomic::TIntegralFloat)
            | (Atomic::TFloat, Atomic::TIntegralFloat)
            | (Atomic::TIntegralFloat, Atomic::TFloat)
            | (Atomic::TString, Atomic::TString)
    )
}

/// Recursively match `param_ty` (which may contain template placeholders)
/// against `arg_ty` (a concrete type), updating `bindings`.
///
/// `template_names` is the set of template names declared on the surrounding
/// function/method. Bare unqualified `TNamedObject` references whose fqcn is in
/// that set are treated as template-param references — the docblock parser
/// emits them that way because it lacks template context at parse time
/// (mirrors the workaround in `Type::substitute_templates`).
fn infer_from_pair(
    param_ty: &Type,
    arg_ty: &Type,
    template_names: &std::collections::HashSet<Name>,
    bindings: &mut FxHashMap<Name, Type>,
) {
    // When the parameter is a union mixing template placeholders with concrete
    // atomics (e.g. `T|null` against `Bar|null`), the template should bind to
    // the residual after subtracting matching concrete atomics — otherwise
    // `T` ends up as `Bar|null` instead of `Bar`.
    let template_residual = compute_template_residual(param_ty, arg_ty, template_names);

    for p_atomic in &param_ty.types {
        match p_atomic {
            // Direct template placeholder: T → bind T = residual(arg_ty)
            Atomic::TTemplateParam { name, .. } => {
                let bind = template_residual.as_ref().unwrap_or(arg_ty);
                if bind.types.is_empty() {
                    // Empty residual: every arg atomic was consumed by another
                    // union alternative (e.g. `class-string<T>`); nothing left
                    // for the bare template to bind.
                    continue;
                }
                let entry = bindings.entry(*name).or_insert_with(Type::empty);
                entry.merge_with(bind);
            }

            // non-empty-array<K, V> matched against array<k_ty, v_ty>, array{...}
            // or a list<t_ty> (a list is a subtype of array<int, t_ty>).
            // Same inference logic as TArray below — delegates to the TArray handler.
            Atomic::TNonEmptyArray { key: pk, value: pv } => {
                for a_atomic in &arg_ty.types {
                    match a_atomic {
                        Atomic::TArray { key: ak, value: av }
                        | Atomic::TNonEmptyArray { key: ak, value: av } => {
                            infer_from_pair(pk, ak, template_names, bindings);
                            infer_from_pair(pv, av, template_names, bindings);
                        }
                        Atomic::TList { value: av } | Atomic::TNonEmptyList { value: av } => {
                            infer_from_pair(
                                pk,
                                &Type::single(Atomic::TInt),
                                template_names,
                                bindings,
                            );
                            infer_from_pair(pv, av, template_names, bindings);
                        }
                        Atomic::TKeyedArray { properties, .. } => {
                            let mut key_union = Type::empty();
                            let mut val_union = Type::empty();
                            for (k, prop) in properties {
                                let key_atomic = match k {
                                    ArrayKey::String(_) => Atomic::TString,
                                    ArrayKey::Int(_) => Atomic::TInt,
                                };
                                key_union.add_type(key_atomic);
                                val_union.merge_with(&prop.ty);
                            }
                            if !key_union.types.is_empty() {
                                infer_from_pair(pk, &key_union, template_names, bindings);
                                infer_from_pair(pv, &val_union, template_names, bindings);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // array<K, V> matched against array<k_ty, v_ty>, array{...} or
            // list<t_ty> (a list is a subtype of array<int, t_ty>).
            Atomic::TArray { key: pk, value: pv } => {
                for a_atomic in &arg_ty.types {
                    match a_atomic {
                        Atomic::TArray { key: ak, value: av }
                        | Atomic::TNonEmptyArray { key: ak, value: av } => {
                            infer_from_pair(pk, ak, template_names, bindings);
                            infer_from_pair(pv, av, template_names, bindings);
                        }
                        Atomic::TList { value: av } | Atomic::TNonEmptyList { value: av } => {
                            infer_from_pair(
                                pk,
                                &Type::single(Atomic::TInt),
                                template_names,
                                bindings,
                            );
                            infer_from_pair(pv, av, template_names, bindings);
                        }
                        Atomic::TKeyedArray { properties, .. } => {
                            let mut key_union = Type::empty();
                            let mut val_union = Type::empty();
                            for (k, prop) in properties {
                                let key_atomic = match k {
                                    ArrayKey::String(_) => Atomic::TString,
                                    ArrayKey::Int(_) => Atomic::TInt,
                                };
                                key_union.add_type(key_atomic);
                                val_union.merge_with(&prop.ty);
                            }
                            if !key_union.types.is_empty() {
                                infer_from_pair(pk, &key_union, template_names, bindings);
                                infer_from_pair(pv, &val_union, template_names, bindings);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // list<T> matched against list<t_ty> or a literal/keyed array whose
            // shape is a list (`array_is_list()`-true `TKeyedArray`, e.g. array
            // literals like `['a', 'b']`, which never construct `TList` directly).
            Atomic::TList { value: pv } | Atomic::TNonEmptyList { value: pv } => {
                for a_atomic in &arg_ty.types {
                    match a_atomic {
                        Atomic::TList { value: av } | Atomic::TNonEmptyList { value: av } => {
                            infer_from_pair(pv, av, template_names, bindings);
                        }
                        Atomic::TKeyedArray {
                            properties,
                            is_list: true,
                            ..
                        } => {
                            let mut val_union = Type::empty();
                            for prop in properties.values() {
                                val_union.merge_with(&prop.ty);
                            }
                            if !val_union.types.is_empty() {
                                infer_from_pair(pv, &val_union, template_names, bindings);
                            }
                        }
                        _ => {}
                    }
                }
            }

            // ClassName<T> matched against ClassName<t_ty> — or, if the bare
            // name is itself a declared template, bind it to arg_ty.
            Atomic::TNamedObject {
                fqcn: pfqcn,
                type_params: pp,
            } => {
                if pp.is_empty() && !pfqcn.contains('\\') && template_names.contains(pfqcn) {
                    let bind = template_residual.as_ref().unwrap_or(arg_ty);
                    if bind.types.is_empty() {
                        continue; // see TTemplateParam arm above
                    }
                    let entry = bindings.entry(*pfqcn).or_insert_with(Type::empty);
                    entry.merge_with(bind);
                    continue;
                }
                for a_atomic in &arg_ty.types {
                    if let Atomic::TNamedObject {
                        fqcn: afqcn,
                        type_params: ap,
                    } = a_atomic
                    {
                        if pfqcn == afqcn {
                            for (p_param, a_param) in pp.iter().zip(ap.iter()) {
                                infer_from_pair(p_param, a_param, template_names, bindings);
                            }
                        }
                    }
                }
            }

            // Closure(T1, T2): R matched against Closure(t1, t2): r
            Atomic::TClosure {
                params: p_params,
                return_type: p_ret,
                ..
            } => {
                for a_atomic in &arg_ty.types {
                    match a_atomic {
                        Atomic::TClosure {
                            params: a_params,
                            return_type: a_ret,
                            ..
                        } => {
                            for (pp, ap) in p_params.iter().zip(a_params.iter()) {
                                if let (Some(pt), Some(at)) = (pp.ty.as_ref(), ap.ty.as_ref()) {
                                    infer_from_pair(
                                        &pt.to_union(),
                                        &at.to_union(),
                                        template_names,
                                        bindings,
                                    );
                                }
                            }
                            infer_from_pair(p_ret, a_ret, template_names, bindings);
                        }
                        Atomic::TCallable {
                            params: Some(a_params),
                            return_type: Some(a_ret),
                        } => {
                            for (pp, ap) in p_params.iter().zip(a_params.iter()) {
                                if let (Some(pt), Some(at)) = (pp.ty.as_ref(), ap.ty.as_ref()) {
                                    infer_from_pair(
                                        &pt.to_union(),
                                        &at.to_union(),
                                        template_names,
                                        bindings,
                                    );
                                }
                            }
                            infer_from_pair(p_ret, a_ret, template_names, bindings);
                        }
                        _ => {}
                    }
                }
            }

            // callable(T1, T2): R matched against callable(t1, t2): r or Closure(...)
            Atomic::TCallable {
                params: Some(p_params),
                return_type: Some(p_ret),
            } => {
                for a_atomic in &arg_ty.types {
                    match a_atomic {
                        Atomic::TCallable {
                            params: Some(a_params),
                            return_type: Some(a_ret),
                        } => {
                            for (pp, ap) in p_params.iter().zip(a_params.iter()) {
                                if let (Some(pt), Some(at)) = (pp.ty.as_ref(), ap.ty.as_ref()) {
                                    infer_from_pair(
                                        &pt.to_union(),
                                        &at.to_union(),
                                        template_names,
                                        bindings,
                                    );
                                }
                            }
                            infer_from_pair(p_ret, a_ret, template_names, bindings);
                        }
                        Atomic::TClosure {
                            params: a_params,
                            return_type: a_ret,
                            ..
                        } => {
                            for (pp, ap) in p_params.iter().zip(a_params.iter()) {
                                if let (Some(pt), Some(at)) = (pp.ty.as_ref(), ap.ty.as_ref()) {
                                    infer_from_pair(
                                        &pt.to_union(),
                                        &at.to_union(),
                                        template_names,
                                        bindings,
                                    );
                                }
                            }
                            infer_from_pair(p_ret, a_ret, template_names, bindings);
                        }
                        _ => {}
                    }
                }
            }

            // A&B intersection — recurse each part against the arg. Use the
            // residual-filtered arg when the surrounding union computed one:
            // atomics consumed by sibling alternatives (e.g. `class-string<T>`)
            // must not leak into bare-template parts of the intersection.
            Atomic::TIntersection { parts } => {
                let arg = template_residual.as_ref().unwrap_or(arg_ty);
                if arg.types.is_empty() {
                    continue;
                }
                for part in parts.iter() {
                    infer_from_pair(part, arg, template_names, bindings);
                }
            }

            // class-string<T> matched against class-string<SomeClass>
            Atomic::TClassString(Some(param_name)) if template_names.contains(param_name) => {
                for a_atomic in &arg_ty.types {
                    let cls_ty = match a_atomic {
                        Atomic::TClassString(Some(arg_cls)) => {
                            Some(Type::single(Atomic::TNamedObject {
                                fqcn: *arg_cls,
                                type_params: empty_type_params(),
                            }))
                        }
                        Atomic::TClassString(None) => Some(Type::single(Atomic::TObject)),
                        // A class-name-shaped string literal coerces to
                        // class-string (Psalm-style): `m::mock('Foo\Bar')`.
                        Atomic::TLiteralString(s) if literal_is_class_like(s) => {
                            Some(Type::single(Atomic::TNamedObject {
                                fqcn: Name::new(s.trim_start_matches('\\')),
                                type_params: empty_type_params(),
                            }))
                        }
                        _ => None,
                    };
                    if let Some(cls_ty) = cls_ty {
                        let entry = bindings.entry(*param_name).or_insert_with(Type::empty);
                        entry.merge_with(&cls_ty);
                    }
                }
            }

            // interface-string<T> matched against interface-string<SomeIface> or
            // class-string<SomeIface> (e.g. `SomeIface::class` types as class-string).
            Atomic::TInterfaceString(Some(param_name)) if template_names.contains(param_name) => {
                for a_atomic in &arg_ty.types {
                    let cls_ty = match a_atomic {
                        Atomic::TInterfaceString(Some(arg_cls))
                        | Atomic::TClassString(Some(arg_cls)) => {
                            Some(Type::single(Atomic::TNamedObject {
                                fqcn: *arg_cls,
                                type_params: empty_type_params(),
                            }))
                        }
                        Atomic::TInterfaceString(None) | Atomic::TClassString(None) => {
                            Some(Type::single(Atomic::TObject))
                        }
                        Atomic::TLiteralString(s) if literal_is_class_like(s) => {
                            Some(Type::single(Atomic::TNamedObject {
                                fqcn: Name::new(s.trim_start_matches('\\')),
                                type_params: empty_type_params(),
                            }))
                        }
                        _ => None,
                    };
                    if let Some(cls_ty) = cls_ty {
                        let entry = bindings.entry(*param_name).or_insert_with(Type::empty);
                        entry.merge_with(&cls_ty);
                    }
                }
            }

            // TConditional in param position is intentionally unsupported —
            // binding a template from a conditional type requires a constraint
            // solver that doesn't exist here.
            _ => {}
        }
    }
}
