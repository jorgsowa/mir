//! `EnumName::CaseName` / `get_class()`/`get_debug_type()`/`get_parent_class()`/
//! `$obj::class` comparisons against `Foo::class`/a static property/an enum
//! case, for variable, property, and static-property receivers.
use php_ast::owned::ExprKind;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{
    extract_any_prop_access, extract_static_prop_access, extract_var_name,
    narrow_receiver_non_null_on_prop_match, ScalarArgTarget,
};
use super::instanceof_core::narrow_static_prop_is_subclass_of;
use super::{
    extract_class_const_fqcn, extract_dynamic_class_const_static_prop_var,
    extract_dynamic_class_const_var, extract_enum_case, extract_get_class_arg,
    extract_get_class_static_prop_arg, extract_get_debug_type_arg,
    extract_get_debug_type_static_prop_arg, extract_get_parent_class_arg,
    extract_get_parent_class_static_prop_arg, narrow_from_get_parent_class_literal,
    narrow_prop_to_class_string, narrow_prop_to_literal_enum_case, narrow_prop_to_specific_class,
    narrow_static_prop_to_class_string, narrow_static_prop_to_literal_enum_case,
    narrow_static_prop_to_specific_class, narrow_var_to_class_string,
    narrow_var_to_literal_enum_case, narrow_var_to_specific_class,
};

/// Shared by the strict `===`/`!==` arm and the loose `==`/`!=` arm: narrows
/// `$x`/`$this->prop` against an enum-case (`EnumName::CaseName`) or a
/// `get_class()`/`get_debug_type()`/`get_parent_class()`/`$obj::class`
/// comparison against `Foo::class`. Sound for loose comparison too — enum
/// cases are singleton objects (`==` and `===` agree on them), and these
/// functions always return plain strings, so a string==string loose
/// comparison can never diverge from strict here. NOT a general green light
/// for loose class-string comparison: a bare `$x == 'literal'` where `$x`'s
/// type admits bool/object atoms is a separate case intentionally left
/// unhandled (loose/strict diverge there), so this function must only ever
/// be called for the specific shapes it matches on below.
pub(super) fn narrow_from_static_or_class_const_comparison(
    ctx: &mut FlowState,
    b: &php_ast::owned::BinaryExpr,
    effective_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    if let ExprKind::StaticPropertyAccess(_) = &b.right.kind {
        if let Some((enum_fqcn, case_name)) = extract_enum_case(
            &b.right,
            ctx.self_fqcn.as_deref(),
            ctx.parent_fqcn.as_deref(),
            db,
            file,
        ) {
            if let Some(var_name) = extract_var_name(&b.left) {
                narrow_var_to_literal_enum_case(
                    db,
                    ctx,
                    &var_name,
                    &enum_fqcn,
                    &case_name,
                    effective_true,
                );
            }
        }
        // `b.right` structurally matched `StaticPropertyAccess` but wasn't a
        // declared enum case above — it's a genuine `self::$prop`/
        // `static::$prop` receiver being compared against `b.left`
        // (`EnumName::CaseName === self::$prop` / `Foo::class === self::$prop`).
        else if let Some((fqcn, prop)) = extract_static_prop_access(&b.right, ctx, db, file) {
            if let Some((enum_fqcn, case_name)) = extract_enum_case(
                &b.left,
                ctx.self_fqcn.as_deref(),
                ctx.parent_fqcn.as_deref(),
                db,
                file,
            ) {
                narrow_static_prop_to_literal_enum_case(
                    db,
                    ctx,
                    &fqcn,
                    &prop,
                    (&enum_fqcn, &case_name),
                    effective_true,
                );
            } else if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(class_fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_to_class_string(
                        ctx,
                        &fqcn,
                        &prop,
                        &class_fqcn,
                        effective_true,
                        db,
                    );
                }
            }
        }
    } else if let ExprKind::StaticPropertyAccess(_) = &b.left.kind {
        if let Some((enum_fqcn, case_name)) = extract_enum_case(
            &b.left,
            ctx.self_fqcn.as_deref(),
            ctx.parent_fqcn.as_deref(),
            db,
            file,
        ) {
            if let Some(var_name) = extract_var_name(&b.right) {
                narrow_var_to_literal_enum_case(
                    db,
                    ctx,
                    &var_name,
                    &enum_fqcn,
                    &case_name,
                    effective_true,
                );
            }
        }
        // Symmetric fallback: `b.left` is a genuine static-property receiver
        // being compared against `b.right` (`self::$prop === EnumName::CaseName`
        // / `self::$prop === Foo::class`).
        else if let Some((fqcn, prop)) = extract_static_prop_access(&b.left, ctx, db, file) {
            if let Some((enum_fqcn, case_name)) = extract_enum_case(
                &b.right,
                ctx.self_fqcn.as_deref(),
                ctx.parent_fqcn.as_deref(),
                db,
                file,
            ) {
                narrow_static_prop_to_literal_enum_case(
                    db,
                    ctx,
                    &fqcn,
                    &prop,
                    (&enum_fqcn, &case_name),
                    effective_true,
                );
            } else if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(class_fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_to_class_string(
                        ctx,
                        &fqcn,
                        &prop,
                        &class_fqcn,
                        effective_true,
                        db,
                    );
                }
            }
        }
    }
    // `$x === EnumName::CaseName` (real enum-case access parses as
    // ClassConstAccess, the same node `Foo::class` uses — try case
    // narrowing first, falling back to the class-string case below).
    else if let ExprKind::ClassConstAccess(_) = &b.right.kind {
        if let Some(var_name) = extract_var_name(&b.left) {
            if let Some((enum_fqcn, case_name)) = extract_enum_case(
                &b.right,
                ctx.self_fqcn.as_deref(),
                ctx.parent_fqcn.as_deref(),
                db,
                file,
            ) {
                narrow_var_to_literal_enum_case(
                    db,
                    ctx,
                    &var_name,
                    &enum_fqcn,
                    &case_name,
                    effective_true,
                );
            } else if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_var_to_class_string(ctx, &var_name, &fqcn, effective_true, db);
                }
            }
        }
        // `$this->prop === EnumName::CaseName` / `$obj->prop === ...`
        // — property-access counterpart of the plain-variable case
        // above, only reached once extract_var_name on the left fails.
        else if let Some((obj_var, prop)) = extract_any_prop_access(&b.left) {
            if let Some((enum_fqcn, case_name)) = extract_enum_case(
                &b.right,
                ctx.self_fqcn.as_deref(),
                ctx.parent_fqcn.as_deref(),
                db,
                file,
            ) {
                narrow_prop_to_literal_enum_case(
                    db,
                    ctx,
                    &obj_var,
                    &prop,
                    file,
                    (&enum_fqcn, &case_name),
                    effective_true,
                );
                narrow_receiver_non_null_on_prop_match(ctx, &obj_var, effective_true);
            } else if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                // `$this->prop === Foo::class` — plain class-string
                // comparison, not an enum case; property counterpart of
                // the plain-variable `narrow_var_to_class_string` case.
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_prop_to_class_string(
                        ctx,
                        &obj_var,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                    narrow_receiver_non_null_on_prop_match(ctx, &obj_var, effective_true);
                }
            }
        }
        // `get_class($x) === Foo::class` — the far more idiomatic
        // counterpart of the `get_class($x) === 'Foo'` string-literal
        // case above; only reached once extract_var_name on the left
        // fails (i.e. the left side is the get_class(...) call itself).
        else if let Some(target) = extract_get_class_arg(&b.left) {
            if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                }
            }
        }
        // `get_class(self::$prop) === Foo::class` — static-property
        // counterpart of the block above.
        else if let Some((fqcn_recv, prop)) =
            extract_get_class_static_prop_arg(&b.left, ctx, db, file)
        {
            if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                }
            }
        }
        // `get_debug_type($x) === Foo::class` — same idiom as
        // `get_class($x) === Foo::class` above, PHP 8's replacement for get_class().
        else if let Some(target) = extract_get_debug_type_arg(&b.left) {
            if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    match &target {
                        ScalarArgTarget::Var(obj_var_name) => narrow_var_to_specific_class(
                            ctx,
                            obj_var_name,
                            &fqcn,
                            effective_true,
                            db,
                        ),
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                obj,
                                prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, obj, effective_true);
                        }
                    }
                }
            }
        }
        // `get_debug_type(self::$prop) === Foo::class` — static-property
        // counterpart of the block above.
        else if let Some((fqcn_recv, prop)) =
            extract_get_debug_type_static_prop_arg(&b.left, ctx, db, file)
        {
            if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                }
            }
        }
        // `get_parent_class($x) === Foo::class` — same idiom as
        // `get_class($x) === Foo::class` above, proving $x a strict
        // subclass instance of Foo (see narrow_from_get_parent_class_literal).
        else if let Some(target) = extract_get_parent_class_arg(&b.left) {
            if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                }
            }
        }
        // `get_parent_class(self::$prop) === Foo::class` — ScalarArgTarget has
        // no static-property variant (tracked as S19), so extract it call-site-
        // locally instead, reusing the existing narrow_static_prop_is_subclass_of.
        else if let Some((fqcn_recv, prop)) =
            extract_get_parent_class_static_prop_arg(&b.left, ctx, db, file)
        {
            if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_is_subclass_of(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        db,
                        effective_true,
                    );
                }
            }
        }
        // `$obj::class === Foo::class` — dynamic get_class()-equivalent.
        else if let Some(target) = extract_dynamic_class_const_var(&b.left) {
            if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                }
            }
        }
        // `self::$prop::class === Foo::class` — static-property counterpart
        // of the block above.
        else if let Some((fqcn_recv, prop)) =
            extract_dynamic_class_const_static_prop_var(&b.left, ctx, db, file)
        {
            if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                }
            }
        }
        // `Foo::class === $obj::class` — reached here (rather than the
        // `b.left is ClassConstAccess` arm below) because `$obj::class`
        // also parses as ClassConstAccess, matching this arm's guard first.
        else if let Some(target) = extract_dynamic_class_const_var(&b.right) {
            if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                }
            }
        }
        // `Foo::class === self::$prop::class` — static-property counterpart.
        else if let Some((fqcn_recv, prop)) =
            extract_dynamic_class_const_static_prop_var(&b.right, ctx, db, file)
        {
            if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                }
            }
        }
    } else if let ExprKind::ClassConstAccess(_) = &b.left.kind {
        if let Some(var_name) = extract_var_name(&b.right) {
            if let Some((enum_fqcn, case_name)) = extract_enum_case(
                &b.left,
                ctx.self_fqcn.as_deref(),
                ctx.parent_fqcn.as_deref(),
                db,
                file,
            ) {
                narrow_var_to_literal_enum_case(
                    db,
                    ctx,
                    &var_name,
                    &enum_fqcn,
                    &case_name,
                    effective_true,
                );
            } else if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_var_to_class_string(ctx, &var_name, &fqcn, effective_true, db);
                }
            }
        }
        // `EnumName::CaseName === $this->prop` — property-access
        // counterpart, symmetric with the left-side case above.
        else if let Some((obj_var, prop)) = extract_any_prop_access(&b.right) {
            if let Some((enum_fqcn, case_name)) = extract_enum_case(
                &b.left,
                ctx.self_fqcn.as_deref(),
                ctx.parent_fqcn.as_deref(),
                db,
                file,
            ) {
                narrow_prop_to_literal_enum_case(
                    db,
                    ctx,
                    &obj_var,
                    &prop,
                    file,
                    (&enum_fqcn, &case_name),
                    effective_true,
                );
                narrow_receiver_non_null_on_prop_match(ctx, &obj_var, effective_true);
            } else if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                // `Foo::class === $this->prop` — symmetric counterpart
                // of the plain class-string case above.
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_prop_to_class_string(
                        ctx,
                        &obj_var,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                    narrow_receiver_non_null_on_prop_match(ctx, &obj_var, effective_true);
                }
            }
        }
        // `Foo::class === get_class($x)` — symmetric counterpart.
        else if let Some(target) = extract_get_class_arg(&b.right) {
            if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    match target {
                        ScalarArgTarget::Var(name) => {
                            narrow_var_to_specific_class(ctx, &name, &fqcn, effective_true, db)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                &obj,
                                &prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                        }
                    }
                }
            }
        }
        // `Foo::class === get_class(self::$prop)` — symmetric counterpart.
        else if let Some((fqcn_recv, prop)) =
            extract_get_class_static_prop_arg(&b.right, ctx, db, file)
        {
            if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                }
            }
        }
        // `Foo::class === get_debug_type($x)` — symmetric counterpart.
        else if let Some(target) = extract_get_debug_type_arg(&b.right) {
            if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    match &target {
                        ScalarArgTarget::Var(obj_var_name) => narrow_var_to_specific_class(
                            ctx,
                            obj_var_name,
                            &fqcn,
                            effective_true,
                            db,
                        ),
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_to_specific_class(
                                ctx,
                                obj,
                                prop,
                                &fqcn,
                                effective_true,
                                db,
                                file,
                            );
                            narrow_receiver_non_null_on_prop_match(ctx, obj, effective_true);
                        }
                    }
                }
            }
        }
        // `Foo::class === get_debug_type(self::$prop)` — symmetric counterpart.
        else if let Some((fqcn_recv, prop)) =
            extract_get_debug_type_static_prop_arg(&b.right, ctx, db, file)
        {
            if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_to_specific_class(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        effective_true,
                        db,
                    );
                }
            }
        }
        // `Foo::class === get_parent_class($x)` — symmetric counterpart.
        else if let Some(target) = extract_get_parent_class_arg(&b.right) {
            if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                }
            }
        }
        // `Foo::class === get_parent_class(self::$prop)` — symmetric counterpart.
        else if let Some((fqcn_recv, prop)) =
            extract_get_parent_class_static_prop_arg(&b.right, ctx, db, file)
        {
            if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(fqcn) = extract_class_const_fqcn(
                    cca,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                    db,
                    file,
                ) {
                    narrow_static_prop_is_subclass_of(
                        ctx,
                        &fqcn_recv,
                        &prop,
                        &fqcn,
                        db,
                        effective_true,
                    );
                }
            }
        }
    }
}
