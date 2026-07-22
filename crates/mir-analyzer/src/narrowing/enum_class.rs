//! Enum-case (`EnumName::CaseName`) and class-string (`Foo::class`) narrowing,
//! for variable, property, and static-property receivers.
use php_ast::owned::ExprKind;

use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{
    apply_prop_narrowed, extract_any_prop_access, extract_class_name, resolve_prop_current_type,
    resolve_static_prop_current_type, set_narrowed, UnionNarrowExt,
};
use super::instanceof_core::{named_object_matches_instanceof, project_type_params_onto_subclass};

/// If `ty` contains an atomic referring to the WHOLE enum `enum_fqcn` (a
/// plain `TNamedObject` — e.g. a `Status $s` parameter that was never
/// narrowed to individual cases), replace that atomic with a union of
/// `TLiteralEnumCase` for every case the enum declares. Atoms that already
/// are per-case literals, or refer to something else entirely, pass through
/// unchanged. Falls back to `ty` unchanged if the enum can't be resolved or
/// nothing needed expanding.
pub(super) fn expand_enum_to_cases(db: &dyn MirDatabase, ty: &Type, enum_fqcn: &str) -> Type {
    if !ty
        .types
        .iter()
        .any(|a| matches!(a, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == enum_fqcn))
    {
        return ty.clone();
    }
    let Some(crate::db::ClassLike::Enum(e)) =
        crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, enum_fqcn))
    else {
        return ty.clone();
    };
    let mut result = Type::empty();
    result.possibly_undefined = ty.possibly_undefined;
    result.from_docblock = ty.from_docblock;
    for atomic in &ty.types {
        match atomic {
            Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == enum_fqcn => {
                for case_name in e.cases.keys() {
                    result.add_type(Atomic::TLiteralEnumCase {
                        enum_fqcn: enum_fqcn.into(),
                        case_name: case_name.as_ref().into(),
                    });
                }
            }
            other => result.add_type(other.clone()),
        }
    }
    result
}

/// `$var->value === 'H'` / `123 === $var->value` — when `prop_expr` is
/// `$var->value` and `literal_expr` is a scalar literal, extract the
/// receiver variable, its backed enum's FQCN, and the specific case name
/// whose `->value` equals the literal. Returns `None` for anything else:
/// a non-`value` property, a non-literal comparand, an unresolvable or
/// non-backed enum, or a value matching zero or more than one case (the
/// latter would mean a duplicate backing value, which PHP itself rejects
/// at enum-declaration time, so it's treated as "can't happen" rather
/// than guessed at).
pub(super) fn extract_enum_value_case(
    prop_expr: &php_ast::owned::Expr,
    literal_expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
) -> Option<(String, String, String)> {
    let (var_name, prop) = extract_any_prop_access(prop_expr)?;
    if prop != "value" {
        return None;
    }
    let literal = match &literal_expr.kind {
        ExprKind::String(s) => Atomic::TLiteralString(std::sync::Arc::from(s.as_ref())),
        ExprKind::Int(n) => Atomic::TLiteralInt(*n),
        _ => return None,
    };
    let current = ctx.get_var(&var_name);
    let enum_fqcn = current.types.iter().find_map(|a| match a {
        Atomic::TNamedObject { fqcn, .. } => Some(fqcn.as_ref().to_string()),
        Atomic::TLiteralEnumCase { enum_fqcn, .. } => Some(enum_fqcn.to_string()),
        _ => None,
    })?;
    let crate::db::ClassLike::Enum(e) =
        crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, &enum_fqcn))?
    else {
        return None;
    };
    e.scalar_type.as_ref()?; // only a backed enum has a meaningful ->value
    let mut matched: Option<String> = None;
    for (name, case) in e.cases.iter() {
        let is_match = case
            .value
            .as_ref()
            .is_some_and(|v| v.types.contains(&literal));
        if is_match {
            if matched.is_some() {
                return None;
            }
            matched = Some(name.to_string());
        }
    }
    matched.map(|case_name| (var_name, enum_fqcn, case_name))
}

pub(super) fn narrow_var_to_literal_enum_case(
    db: &dyn MirDatabase,
    ctx: &mut FlowState,
    name: &str,
    enum_fqcn: &str,
    case_name: &str,
    is_case: bool,
) {
    let current = ctx.get_var(name);
    let narrowed = if is_case {
        Type::single(Atomic::TLiteralEnumCase {
            enum_fqcn: enum_fqcn.into(),
            case_name: case_name.into(),
        })
    } else {
        // For !== comparison with enum case, remove that specific case from
        // the union. `current` may not already be decomposed into per-case
        // TLiteralEnumCase atoms (e.g. a plain `Status $s` parameter typed
        // as the whole enum) — expand it first, or the filter below matches
        // nothing and the exclusion silently does nothing.
        expand_enum_to_cases(db, &current, enum_fqcn).filter(|t| {
            !matches!(t, Atomic::TLiteralEnumCase { enum_fqcn: fqcn, case_name: c }
                if fqcn.as_ref() == enum_fqcn && c.as_ref() == case_name)
        })
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

/// Property-access counterpart of `narrow_var_to_literal_enum_case`, for
/// `$this->prop === EnumName::CaseName` (or any `$obj->prop` receiver).
pub(super) fn narrow_prop_to_literal_enum_case(
    db: &dyn MirDatabase,
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    file: &str,
    (enum_fqcn, case_name): (&str, &str),
    is_case: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    let narrowed = if is_case {
        Type::single(Atomic::TLiteralEnumCase {
            enum_fqcn: enum_fqcn.into(),
            case_name: case_name.into(),
        })
    } else {
        expand_enum_to_cases(db, &current, enum_fqcn).filter(|t| {
            !matches!(t, Atomic::TLiteralEnumCase { enum_fqcn: fqcn, case_name: c }
                if fqcn.as_ref() == enum_fqcn && c.as_ref() == case_name)
        })
    };
    // The exclusion branch (`$obj->prop !== Status::Active`) is also satisfied
    // whenever $obj itself is null (`null !== <enum case>` is true), so a
    // nullable receiver means an empty narrowed-out result here isn't a real
    // contradiction — same reasoning as `narrow_prop_to_specific_class`.
    let mark_diverges = is_case || !ctx.get_var(obj_var).is_nullable();
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_to_literal_enum_case`, for
/// `self::$prop === EnumName::CaseName` (and `static::$prop`/`Class::$prop`).
pub(super) fn narrow_static_prop_to_literal_enum_case(
    db: &dyn MirDatabase,
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    (enum_fqcn, case_name): (&str, &str),
    is_case: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    let narrowed = if is_case {
        Type::single(Atomic::TLiteralEnumCase {
            enum_fqcn: enum_fqcn.into(),
            case_name: case_name.into(),
        })
    } else {
        expand_enum_to_cases(db, &current, enum_fqcn).filter(|t| {
            !matches!(t, Atomic::TLiteralEnumCase { enum_fqcn: fqcn, case_name: c }
                if fqcn.as_ref() == enum_fqcn && c.as_ref() == case_name)
        })
    };
    // No separate receiver-nullability concern for a static property —
    // self::/static:: is never itself null, unlike an instance receiver.
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
}

/// Static-property counterpart of `narrow_prop_to_class_string`, for
/// `self::$prop === Foo::class` (and `static::$prop`/`Class::$prop`).
pub(super) fn narrow_static_prop_to_class_string(
    ctx: &mut FlowState,
    fqcn_key: &str,
    prop: &str,
    fqcn: &str,
    is_class: bool,
    db: &dyn MirDatabase,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn_key, prop, db);
    let narrowed = if is_class {
        Type::single(Atomic::TClassString(Some(mir_types::Name::from(fqcn))))
    } else {
        current.filter(|t| {
            !matches!(t, Atomic::TClassString(Some(f)) if f.as_ref() == fqcn && crate::db::is_final(db, fqcn))
        })
    };
    apply_prop_narrowed(ctx, fqcn_key, prop, current, narrowed, true);
}

/// `$cls === Foo::class` / `!== Foo::class` narrowing. Unlike `instanceof`/
/// `is_a`, `Foo::class` (like `get_class()`) names the *exact* runtime
/// class — so on the false branch, a `class-string<Foo>` atom (which,
/// everywhere else in this file, means "Foo or any subclass") can only be
/// dropped outright when `Foo` is `final` and so provably has no subclass
/// that could still satisfy `!== Foo::class`.
pub(super) fn narrow_var_to_class_string(
    ctx: &mut FlowState,
    name: &str,
    fqcn: &str,
    is_class: bool,
    db: &dyn MirDatabase,
) {
    let current = ctx.get_var(name);
    let narrowed = if is_class {
        Type::single(Atomic::TClassString(Some(mir_types::Name::from(fqcn))))
    } else {
        current.filter(|t| {
            !matches!(t, Atomic::TClassString(Some(f)) if f.as_ref() == fqcn && crate::db::is_final(db, fqcn))
        })
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

/// Property-access counterpart of `narrow_var_to_class_string`, for
/// `$this->prop === Foo::class` (a plain class-string comparison, not the
/// enum-case idiom `narrow_prop_to_literal_enum_case` already handles).
pub(super) fn narrow_prop_to_class_string(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    fqcn: &str,
    is_class: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    let narrowed = if is_class {
        Type::single(Atomic::TClassString(Some(mir_types::Name::from(fqcn))))
    } else {
        current.filter(|t| {
            !matches!(t, Atomic::TClassString(Some(f)) if f.as_ref() == fqcn && crate::db::is_final(db, fqcn))
        })
    };
    // The exclusion branch (`$obj->prop !== Foo::class`) is also satisfied
    // whenever $obj itself is null (`null !== 'Foo'` is true), so a nullable
    // receiver means an empty narrowed-out result here isn't a real
    // contradiction — same reasoning as `narrow_prop_to_specific_class`.
    let mark_diverges = is_class || !ctx.get_var(obj_var).is_nullable();
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// `get_class($x)`/`get_debug_type($x)`/`$x::class` compared to a literal —
/// see `narrow_var_to_class_string`'s doc comment for why the false branch
/// can only drop a matching `TNamedObject` atom when the class is `final`.
/// For an exact-class narrowing (`get_class($x) === Foo::class`), find a
/// concrete type_params list to attach to the narrowed `Foo` atom by
/// reusing/projecting from any object atom already in `current` that `Foo`
/// is a subtype of (or equal to) — mirrors how
/// `narrow_instanceof_preserving_subtypes` preserves/projects type params
/// instead of discarding them, so `get_class($x) === Foo::class` on a
/// `Box<int>`-typed `$x` narrows to `Foo<int>`, not a raw `Foo`.
pub(super) fn type_params_for_exact_class(
    current: &Type,
    target_fqcn: &str,
    db: &dyn MirDatabase,
) -> std::sync::Arc<[Type]> {
    for t in &current.types {
        if let Atomic::TNamedObject { fqcn, type_params } = t {
            if type_params.is_empty() {
                continue;
            }
            if fqcn.as_ref() == target_fqcn {
                return type_params.clone();
            }
            if named_object_matches_instanceof(target_fqcn, fqcn, db) {
                return project_type_params_onto_subclass(db, fqcn, type_params, target_fqcn);
            }
        }
    }
    mir_types::union::empty_type_params()
}

pub(super) fn narrow_var_to_specific_class(
    ctx: &mut FlowState,
    name: &str,
    fqcn: &str,
    is_exact_class: bool,
    db: &dyn MirDatabase,
) {
    let current = ctx.get_var(name);
    let narrowed = if is_exact_class {
        Type::single(Atomic::TNamedObject {
            fqcn: fqcn.into(),
            type_params: type_params_for_exact_class(&current, fqcn, db),
        })
    } else {
        current.filter(|t| match t {
            Atomic::TNamedObject { fqcn: obj_fqcn, .. }
            | Atomic::TSelf { fqcn: obj_fqcn }
            | Atomic::TStaticObject { fqcn: obj_fqcn }
            | Atomic::TParent { fqcn: obj_fqcn } => {
                obj_fqcn.as_ref() != fqcn || !crate::db::is_final(db, fqcn)
            }
            _ => true,
        })
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

/// Property-access counterpart of `narrow_var_to_specific_class`, for
/// `get_class($this->prop) === 'ClassName'`/`get_debug_type($this->prop) ===
/// 'ClassName'`-style exact-class narrowing on a property receiver.
pub(super) fn narrow_prop_to_specific_class(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    fqcn: &str,
    is_exact_class: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    let narrowed = if is_exact_class {
        Type::single(Atomic::TNamedObject {
            fqcn: fqcn.into(),
            type_params: type_params_for_exact_class(&current, fqcn, db),
        })
    } else {
        current.filter(|t| match t {
            Atomic::TNamedObject { fqcn: obj_fqcn, .. }
            | Atomic::TSelf { fqcn: obj_fqcn }
            | Atomic::TStaticObject { fqcn: obj_fqcn }
            | Atomic::TParent { fqcn: obj_fqcn } => {
                obj_fqcn.as_ref() != fqcn || !crate::db::is_final(db, fqcn)
            }
            _ => true,
        })
    };
    // The exclusion branch (`get_debug_type($obj->prop) !== 'Foo'`) is also
    // satisfied whenever $obj itself is null: get_debug_type(null) returns
    // the string 'null', which is never equal to a real class name — so a
    // nullable receiver means an empty narrowed-out result here isn't a real
    // contradiction. The is_exact_class branch is unaffected (never empty).
    let mark_diverges = is_exact_class || !ctx.get_var(obj_var).is_nullable();
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_to_specific_class`. There's no
/// nullable-receiver variable for a static property, so unlike the instance
/// version this always marks a divergence on an empty result.
pub(super) fn narrow_static_prop_to_specific_class(
    ctx: &mut FlowState,
    fqcn_receiver: &str,
    prop: &str,
    fqcn: &str,
    is_exact_class: bool,
    db: &dyn MirDatabase,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn_receiver, prop, db);
    let narrowed = if is_exact_class {
        Type::single(Atomic::TNamedObject {
            fqcn: fqcn.into(),
            type_params: type_params_for_exact_class(&current, fqcn, db),
        })
    } else {
        current.filter(|t| match t {
            Atomic::TNamedObject { fqcn: obj_fqcn, .. }
            | Atomic::TSelf { fqcn: obj_fqcn }
            | Atomic::TStaticObject { fqcn: obj_fqcn }
            | Atomic::TParent { fqcn: obj_fqcn } => {
                obj_fqcn.as_ref() != fqcn || !crate::db::is_final(db, fqcn)
            }
            _ => true,
        })
    };
    apply_prop_narrowed(ctx, fqcn_receiver, prop, current, narrowed, true);
}

pub(super) fn extract_enum_case(
    expr: &php_ast::owned::Expr,
    self_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(String, String)> {
    // Real `EnumName::CaseName` syntax parses as `ClassConstAccess` (the same
    // node shape used for `Foo::class` and plain class constants) — not
    // `StaticPropertyAccess`, which is reserved for `Foo::$prop` (the `$`
    // sigil enum-case access never has). Accept both node kinds structurally
    // and disambiguate by confirming the target actually is a declared case
    // of a real enum, so `Foo::BAR` (a plain class constant) and `Foo::class`
    // aren't misread as case narrowing.
    let spa = match &expr.kind {
        ExprKind::StaticPropertyAccess(spa) => spa,
        ExprKind::ClassConstAccess(cca) => cca,
        _ => return None,
    };
    let enum_short_name = extract_class_name(&spa.class, self_fqcn, parent_fqcn)?;
    let enum_fqcn = crate::db::resolve_name(db, file, &enum_short_name);
    let ExprKind::Identifier(case_name) = &spa.member.kind else {
        return None;
    };
    let is_declared_case = matches!(
        crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, &enum_fqcn)),
        Some(crate::db::ClassLike::Enum(e)) if e.cases.contains_key(case_name.as_ref())
    );
    if !is_declared_case {
        return None;
    }
    Some((enum_fqcn, case_name.to_string()))
}

pub(super) fn extract_class_const_fqcn(
    cca: &php_ast::owned::StaticAccessExpr,
    self_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<String> {
    let is_class = matches!(&cca.member.kind, ExprKind::Identifier(n) if n.as_ref() == "class");
    if !is_class {
        return None;
    }
    let short = extract_class_name(&cca.class, self_fqcn, parent_fqcn)?;
    Some(crate::db::resolve_name(db, file, &short))
}
