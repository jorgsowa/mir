/// Type narrowing — refines variable types based on conditional expressions.
///
/// Given a condition expression and a branch direction (true/false), this
/// module updates the `FlowState` to narrow variable types accordingly.
use php_ast::ast::{AssignOp, BinaryOp, UnaryPrefixOp};
use php_ast::owned::ExprKind;

use mir_codebase::definitions::AssertionKind;
use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

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
fn narrow_from_static_or_class_const_comparison(
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
    }
}

/// Narrow the types in `ctx` as if `expr` evaluates to `is_true`.
pub fn narrow_from_condition(
    expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    match &expr.kind {
        // Parenthesized — unwrap and narrow the inner expression
        ExprKind::Parenthesized(inner) => {
            narrow_from_condition(inner, ctx, is_true, db, file);
        }

        // !expr  →  narrow as if expr is !is_true
        ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::BooleanNot => {
            narrow_from_condition(&u.operand, ctx, !is_true, db, file);
        }

        // $a && $b  →  if true: narrow both; if false: no constraint
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanAnd || b.op == BinaryOp::LogicalAnd => {
            if is_true {
                narrow_from_condition(&b.left, ctx, true, db, file);
                narrow_from_condition(&b.right, ctx, true, db, file);
                // When A && B is true, both sides were evaluated.
                // Promote variables from possibly_assigned to assigned for side effects in each.
                promote_assignment_effects(&b.left, ctx, db, file);
                promote_assignment_effects(&b.right, ctx, db, file);
            }
        }

        // $a || $b  →  if false: narrow both; if true: try to narrow same-var instanceof union
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            if !is_true {
                narrow_from_condition(&b.left, ctx, false, db, file);
                narrow_from_condition(&b.right, ctx, false, db, file);
                // When A || B is false, both sides were evaluated.
                // Promote variables from possibly_assigned to assigned for side effects in each.
                promote_assignment_effects(&b.left, ctx, db, file);
                promote_assignment_effects(&b.right, ctx, db, file);
            } else {
                // For `$x instanceof A || $x instanceof B` in true-branch: narrow $x to A|B
                narrow_or_instanceof_true(&b.left, &b.right, ctx, db, file);

                // For `!isset($x) || RHS` in true-branch: narrow RHS as if isset($x) is true
                narrow_or_isset_true(&b.left, &b.right, ctx, db, file);
            }
        }

        // $x === null / $x !== null
        ExprKind::Binary(b) if b.op == BinaryOp::Identical || b.op == BinaryOp::NotIdentical => {
            let is_identical = b.op == BinaryOp::Identical;
            let effective_true = if is_identical { is_true } else { !is_true };

            // `count($arr) === N` / `strlen($str) !== N`, etc. — independent of
            // the rest of this arm, mirrors the `<`/`<=`/`>`/`>=` handling below.
            narrow_count_or_strlen_equality(ctx, db, file, &b.left, &b.right, b.op, is_true);

            // `($x ?? FALLBACK) === FALLBACK` — on the false branch, $x was defined
            // Must be checked before literal comparisons because `b.right` matching a literal
            // would otherwise consume the arm before we check for NullCoalesce on `b.left`.
            if let Some(nc) = extract_null_coalesce(&b.left) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        // A non-FALLBACK result also proves the receiver itself wasn't
                        // null (a null receiver's `?->`/`->` access coalesces straight
                        // to FALLBACK) — without this, a later plain read of the same
                        // property re-admits null via the receiver's own nullability.
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, !effective_true);
                    }
                }
            } else if let Some(nc) = extract_null_coalesce(&b.right) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, !effective_true);
                    }
                }
            }
            // `$x === null`
            else if matches!(b.right.kind, ExprKind::Null) {
                if let Some(target) = extract_array_key_first_or_last_arg(&b.left) {
                    match target {
                        ScalarArgTarget::Var(arr_var) => {
                            narrow_array_key_first_or_last_null(ctx, &arr_var, effective_true)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_array_key_first_or_last_null(
                                ctx,
                                &obj,
                                &prop,
                                db,
                                file,
                                effective_true,
                            )
                        }
                    }
                } else if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_nullsafe_prop_access(&b.left) {
                    narrow_nullsafe_prop_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((obj, prop)) = extract_prop_access(&b.left) {
                    narrow_prop_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_null(ctx, &fqcn, &prop, db, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(target) = extract_array_key_first_or_last_arg(&b.right) {
                    match target {
                        ScalarArgTarget::Var(arr_var) => {
                            narrow_array_key_first_or_last_null(ctx, &arr_var, effective_true)
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            narrow_prop_array_key_first_or_last_null(
                                ctx,
                                &obj,
                                &prop,
                                db,
                                file,
                                effective_true,
                            )
                        }
                    }
                } else if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_nullsafe_prop_access(&b.right) {
                    narrow_nullsafe_prop_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((obj, prop)) = extract_prop_access(&b.right) {
                    narrow_prop_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_null(ctx, &fqcn, &prop, db, effective_true);
                }
            }
            // `$x === true` / `$x === false`
            else if matches!(b.right.kind, ExprKind::Bool(true)) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_bool(ctx, &name, true, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_bool(ctx, &obj, &prop, db, file, true, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_bool(ctx, &fqcn, &prop, db, true, effective_true);
                } else {
                    // `is_string($x) === true` / `($x instanceof Y) === true` — the
                    // left side is guaranteed to be a real bool (not just truthy),
                    // so narrowing it as a bare condition is exactly as sound as
                    // strict equality requires; re-entering the top-level match
                    // dispatches to whatever arm (FunctionCall, Instanceof, Isset,
                    // ...) already handles that expression shape directly.
                    narrow_from_condition(&b.left, ctx, effective_true, db, file);
                }
            } else if matches!(b.right.kind, ExprKind::Bool(false)) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_bool(ctx, &name, false, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_bool(ctx, &obj, &prop, db, file, false, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_bool(ctx, &fqcn, &prop, db, false, effective_true);
                } else {
                    // `strpos($h, $n) !== false` / `array_search($n, $h) === false`
                    narrow_from_false_comparable_call(&b.left, ctx, db, file, effective_true);
                    // `is_string($x) === false` — see the `=== true` comment above.
                    narrow_from_condition(&b.left, ctx, !effective_true, db, file);
                }
            }
            // `true === $x` / `false === $x` — symmetric; extract_var_name looks through
            // assignment exprs, so this also handles `false === ($x = expr)`.
            else if matches!(b.left.kind, ExprKind::Bool(true)) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_bool(ctx, &name, true, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_bool(ctx, &obj, &prop, db, file, true, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_bool(ctx, &fqcn, &prop, db, true, effective_true);
                } else {
                    narrow_from_condition(&b.right, ctx, effective_true, db, file);
                }
            } else if matches!(b.left.kind, ExprKind::Bool(false)) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_bool(ctx, &name, false, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_bool(ctx, &obj, &prop, db, file, false, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_bool(ctx, &fqcn, &prop, db, false, effective_true);
                } else {
                    narrow_from_false_comparable_call(&b.right, ctx, db, file, effective_true);
                    narrow_from_condition(&b.right, ctx, !effective_true, db, file);
                }
            }
            // `get_class($x) === 'ClassName'` — check before literal strings so it takes precedence
            else if let ExprKind::String(class_name_str) = &b.right.kind {
                if let Some(target) = extract_get_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
                } else if let Some(target) = extract_gettype_arg(&b.left) {
                    narrow_from_gettype_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_debug_type_arg(&b.left) {
                    narrow_from_get_debug_type_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_parent_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_dynamic_class_const_var(&b.left) {
                    // `$obj::class === 'ClassName'`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
                } else if let Some(name) = extract_var_name(&b.left) {
                    // `$x === 'literal'`
                    narrow_var_literal_string(ctx, &name, class_name_str, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    // `$this->prop === 'literal'`
                    narrow_prop_literal_string(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        class_name_str,
                        effective_true,
                    );
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    // `self::$prop === 'literal'`
                    narrow_static_prop_literal_string(
                        ctx,
                        &fqcn,
                        &prop,
                        db,
                        class_name_str,
                        effective_true,
                    );
                }
            } else if let ExprKind::String(class_name_str) = &b.left.kind {
                if let Some(target) = extract_get_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
                } else if let Some(target) = extract_gettype_arg(&b.right) {
                    narrow_from_gettype_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_debug_type_arg(&b.right) {
                    narrow_from_get_debug_type_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_parent_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_dynamic_class_const_var(&b.right) {
                    // `'ClassName' === $obj::class`
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
                } else if let Some(name) = extract_var_name(&b.right) {
                    // `$x === 'literal'`
                    narrow_var_literal_string(ctx, &name, class_name_str, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    // `'literal' === $this->prop`
                    narrow_prop_literal_string(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        class_name_str,
                        effective_true,
                    );
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    // `'literal' === self::$prop`
                    narrow_static_prop_literal_string(
                        ctx,
                        &fqcn,
                        &prop,
                        db,
                        class_name_str,
                        effective_true,
                    );
                }
            }
            // `$x === 42`
            else if let ExprKind::Int(n) = &b.right.kind {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_literal_int(ctx, &name, *n, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_literal_int(ctx, &obj, &prop, db, file, *n, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_literal_int(ctx, &fqcn, &prop, db, *n, effective_true);
                }
            } else if let ExprKind::Int(n) = &b.left.kind {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_literal_int(ctx, &name, *n, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_literal_int(ctx, &obj, &prop, db, file, *n, effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_literal_int(ctx, &fqcn, &prop, db, *n, effective_true);
                }
            }
            // `$arr === []` narrows $arr to empty; `$arr !== []` narrows to non-empty.
            // Checked before the enum-case/class-const guard below: a bare
            // `self::$prop === []` would otherwise match that guard's
            // "either side is a StaticPropertyAccess" condition and be
            // silently swallowed (that function only handles enum-case/
            // get_class-family comparisons, not array-emptiness).
            else if let ExprKind::Array(elems) = &b.right.kind {
                if elems.is_empty() {
                    if let Some(var_name) = extract_var_name(&b.left) {
                        let current = ctx.get_var(&var_name);
                        let narrowed = if effective_true {
                            current.narrow_to_empty_collection()
                        } else {
                            current.narrow_to_non_empty_collection()
                        };
                        if !narrowed.is_empty() && narrowed != current {
                            ctx.set_var(&var_name, narrowed);
                        }
                    } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                        narrow_prop_array_empty(ctx, &obj, &prop, db, file, effective_true);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                    } else if let Some((fqcn, prop)) =
                        extract_static_prop_access(&b.left, ctx, db, file)
                    {
                        narrow_static_prop_array_empty(ctx, &fqcn, &prop, db, effective_true);
                    }
                }
            } else if let ExprKind::Array(elems) = &b.left.kind {
                if elems.is_empty() {
                    if let Some(var_name) = extract_var_name(&b.right) {
                        let current = ctx.get_var(&var_name);
                        let narrowed = if effective_true {
                            current.narrow_to_empty_collection()
                        } else {
                            current.narrow_to_non_empty_collection()
                        };
                        if !narrowed.is_empty() && narrowed != current {
                            ctx.set_var(&var_name, narrowed);
                        }
                    } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                        narrow_prop_array_empty(ctx, &obj, &prop, db, file, effective_true);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                    } else if let Some((fqcn, prop)) =
                        extract_static_prop_access(&b.right, ctx, db, file)
                    {
                        narrow_static_prop_array_empty(ctx, &fqcn, &prop, db, effective_true);
                    }
                }
            }
            // `$x === EnumName::CaseName` / get_class()/get_debug_type()/
            // get_parent_class()/$obj::class compared against `Foo::class` —
            // factored into narrow_from_static_or_class_const_comparison so
            // the loose `==`/`!=` arm can reuse it (see that function's doc
            // comment for why loose comparison is sound here specifically).
            else if matches!(b.right.kind, ExprKind::StaticPropertyAccess(_))
                || matches!(b.left.kind, ExprKind::StaticPropertyAccess(_))
                || matches!(b.right.kind, ExprKind::ClassConstAccess(_))
                || matches!(b.left.kind, ExprKind::ClassConstAccess(_))
            {
                narrow_from_static_or_class_const_comparison(ctx, b, effective_true, db, file);
            }
        }

        // $x < N, $x <= N, $x > N, $x >= N — comparison-driven integer range narrowing
        ExprKind::Binary(b)
            if matches!(
                b.op,
                BinaryOp::Less
                    | BinaryOp::LessOrEqual
                    | BinaryOp::Greater
                    | BinaryOp::GreaterOrEqual
            ) =>
        {
            // `$x < N` / `N < $x` (and `$this->prop < N` / `N < $this->prop`) —
            // normalize so the variable/property is always on the left,
            // flipping the operator when the literal was on the left instead.
            if let Some(var_name) = extract_var_name(&b.left) {
                if let Some(n) = extract_int_literal(&b.right) {
                    narrow_var_int_comparison(ctx, &var_name, b.op, n, is_true);
                }
            } else if let Some(var_name) = extract_var_name(&b.right) {
                if let Some(n) = extract_int_literal(&b.left) {
                    narrow_var_int_comparison(ctx, &var_name, flip_comparison_op(b.op), n, is_true);
                }
            } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                if let Some(n) = extract_int_literal(&b.right) {
                    narrow_prop_int_comparison(ctx, &obj, &prop, db, file, b.op, n, is_true);
                }
            } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                if let Some(n) = extract_int_literal(&b.left) {
                    narrow_prop_int_comparison(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        flip_comparison_op(b.op),
                        n,
                        is_true,
                    );
                }
            } else if let Some((fqcn, prop)) = extract_static_prop_access(&b.left, ctx, db, file) {
                if let Some(n) = extract_int_literal(&b.right) {
                    narrow_static_prop_int_comparison(ctx, &fqcn, &prop, db, b.op, n, is_true);
                }
            } else if let Some((fqcn, prop)) = extract_static_prop_access(&b.right, ctx, db, file) {
                if let Some(n) = extract_int_literal(&b.left) {
                    narrow_static_prop_int_comparison(
                        ctx,
                        &fqcn,
                        &prop,
                        db,
                        flip_comparison_op(b.op),
                        n,
                        is_true,
                    );
                }
            }
            // count($arr) op N  /  N op count($arr) — normalize so count call is on left.
            let (count_expr, count_cmp_op, count_lit) = if extract_count_arg(&b.left).is_some() {
                (&b.left, b.op, &b.right)
            } else {
                (&b.right, flip_comparison_op(b.op), &b.left)
            };
            if let (Some(target), Some(n)) = (
                extract_count_arg(count_expr),
                extract_int_literal(count_lit),
            ) {
                match target {
                    ScalarArgTarget::Var(arr_var) => {
                        narrow_array_count_comparison(ctx, &arr_var, count_cmp_op, n, is_true)
                    }
                    ScalarArgTarget::Prop(obj, prop) => narrow_prop_array_count_comparison(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        count_cmp_op,
                        n,
                        is_true,
                    ),
                }
            }
            // strlen($str) op N  /  N op strlen($str) — same normalization.
            let (strlen_expr, strlen_cmp_op, strlen_lit) = if extract_strlen_arg(&b.left).is_some()
            {
                (&b.left, b.op, &b.right)
            } else {
                (&b.right, flip_comparison_op(b.op), &b.left)
            };
            if let (Some(target), Some(n)) = (
                extract_strlen_arg(strlen_expr),
                extract_int_literal(strlen_lit),
            ) {
                match target {
                    ScalarArgTarget::Var(str_var) => {
                        narrow_string_strlen_comparison(ctx, &str_var, strlen_cmp_op, n, is_true)
                    }
                    ScalarArgTarget::Prop(obj, prop) => narrow_prop_string_strlen_comparison(
                        ctx,
                        &obj,
                        &prop,
                        db,
                        file,
                        strlen_cmp_op,
                        n,
                        is_true,
                    ),
                }
            }
        }

        // $x == null  (loose equality)
        ExprKind::Binary(b) if b.op == BinaryOp::Equal || b.op == BinaryOp::NotEqual => {
            let is_equal = b.op == BinaryOp::Equal;
            let effective_true = if is_equal { is_true } else { !is_true };

            // `count($arr) == N` / `strlen($str) != N`, etc.
            narrow_count_or_strlen_equality(ctx, db, file, &b.left, &b.right, b.op, is_true);

            // `($x ?? FALLBACK) == FALLBACK` / `!= FALLBACK` — same reasoning as the
            // strict `===`/`!==` arm: `same_literal` requires an identical literal
            // AST, so when $x is null the coalesce result is exactly FALLBACK,
            // making `FALLBACK == FALLBACK` trivially true regardless of loose vs
            // strict — a `!=` result still proves $x wasn't null. Must be checked
            // before the literal-comparison arms below for the same reason as the
            // strict arm (a literal on the other side would otherwise consume it).
            if let Some(nc) = extract_null_coalesce(&b.left) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, !effective_true);
                    }
                }
            } else if let Some(nc) = extract_null_coalesce(&b.right) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        let narrowed = current.remove_null();
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, !effective_true);
                    }
                }
            } else if matches!(b.right.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_loose_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_loose_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.left, ctx, db, file)
                {
                    narrow_static_prop_loose_null(ctx, &fqcn, &prop, db, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_loose_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_loose_null(ctx, &obj, &prop, db, file, effective_true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&b.right, ctx, db, file)
                {
                    narrow_static_prop_loose_null(ctx, &fqcn, &prop, db, effective_true);
                }
            }
            // `$x == true` / `$x == false` (and negated forms) — PHP defines loose
            // comparison to a bool literal as `(bool)$x === value`, i.e. identical
            // to the truthy/falsy narrowing a bare `if ($x)` already gets.
            else if let ExprKind::Bool(value) = &b.right.kind {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_loose_bool(ctx, &name, *value == effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                    narrow_prop_loose_bool(ctx, &obj, &prop, db, file, *value == effective_true);
                    // `null == true` is false, so a true match against the
                    // literal `true` also proves the receiver wasn't null —
                    // same reasoning as the strict `===` sibling arm above.
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, *value == effective_true);
                } else {
                    if !value {
                        // `strpos($h, $n) == false` / `!= false` — same
                        // false-comparable narrowing as the strict `===`/`!==` arm.
                        narrow_from_false_comparable_call(&b.left, ctx, db, file, effective_true);
                    }
                    // `is_string($x) == true` / `== false` — loose comparison to a
                    // bool literal is `(bool)$x === value` for ANY $x (not just a
                    // guaranteed-bool expression), i.e. exactly the bare truthy/
                    // falsy narrowing `narrow_from_condition` itself provides.
                    narrow_from_condition(&b.left, ctx, *value == effective_true, db, file);
                }
            } else if let ExprKind::Bool(value) = &b.left.kind {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_loose_bool(ctx, &name, *value == effective_true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                    narrow_prop_loose_bool(ctx, &obj, &prop, db, file, *value == effective_true);
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, *value == effective_true);
                } else {
                    if !value {
                        narrow_from_false_comparable_call(&b.right, ctx, db, file, effective_true);
                    }
                    narrow_from_condition(&b.right, ctx, *value == effective_true, db, file);
                }
            }
            // `$arr == []` / `$arr != []` — loose array equality requires identical
            // key/value pairs, so this is exactly as sound as the strict `===` case.
            else if let ExprKind::Array(elems) = &b.right.kind {
                if elems.is_empty() {
                    if let Some(var_name) = extract_var_name(&b.left) {
                        let current = ctx.get_var(&var_name);
                        let narrowed = if effective_true {
                            current.narrow_to_empty_collection()
                        } else {
                            current.narrow_to_non_empty_collection()
                        };
                        if !narrowed.is_empty() && narrowed != current {
                            ctx.set_var(&var_name, narrowed);
                        }
                    } else if let Some((obj, prop)) = extract_any_prop_access(&b.left) {
                        narrow_prop_array_empty(ctx, &obj, &prop, db, file, effective_true);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                    } else if let Some((fqcn, prop)) =
                        extract_static_prop_access(&b.left, ctx, db, file)
                    {
                        narrow_static_prop_array_empty(ctx, &fqcn, &prop, db, effective_true);
                    }
                }
            } else if let ExprKind::Array(elems) = &b.left.kind {
                if elems.is_empty() {
                    if let Some(var_name) = extract_var_name(&b.right) {
                        let current = ctx.get_var(&var_name);
                        let narrowed = if effective_true {
                            current.narrow_to_empty_collection()
                        } else {
                            current.narrow_to_non_empty_collection()
                        };
                        if !narrowed.is_empty() && narrowed != current {
                            ctx.set_var(&var_name, narrowed);
                        }
                    } else if let Some((obj, prop)) = extract_any_prop_access(&b.right) {
                        narrow_prop_array_empty(ctx, &obj, &prop, db, file, effective_true);
                        narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
                    } else if let Some((fqcn, prop)) =
                        extract_static_prop_access(&b.right, ctx, db, file)
                    {
                        narrow_static_prop_array_empty(ctx, &fqcn, &prop, db, effective_true);
                    }
                }
            }
            // `get_class($x) == 'ClassName'` / `get_debug_type($x) == 'ClassName'` /
            // `gettype($x) == 'ClassName'` / `$x::class == 'ClassName'` — loose `==`
            // mirrors the `===` handling below the `Identical`/`NotIdentical` arm:
            // class/type names are never numeric-looking strings, so loose
            // comparison agrees with strict comparison here.
            else if let ExprKind::String(class_name_str) = &b.right.kind {
                if let Some(target) = extract_get_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
                } else if let Some(target) = extract_gettype_arg(&b.left) {
                    narrow_from_gettype_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_debug_type_arg(&b.left) {
                    narrow_from_get_debug_type_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_parent_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_dynamic_class_const_var(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
            } else if let ExprKind::String(class_name_str) = &b.left.kind {
                if let Some(target) = extract_get_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
                } else if let Some(target) = extract_gettype_arg(&b.right) {
                    narrow_from_gettype_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_debug_type_arg(&b.right) {
                    narrow_from_get_debug_type_literal(
                        ctx,
                        &target,
                        class_name_str,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_get_parent_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_from_get_parent_class_literal(
                        ctx,
                        &target,
                        &fqcn,
                        effective_true,
                        db,
                        file,
                    );
                } else if let Some(target) = extract_dynamic_class_const_var(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
            // `$x == EnumName::CaseName` / `get_class($x) == Foo::class` etc. —
            // the `Foo::class`/enum-case counterpart of the string-literal arms
            // above, reusing the same logic the strict `===` arm uses (sound for
            // loose comparison — see narrow_from_static_or_class_const_comparison's
            // doc comment for why).
            else if matches!(b.right.kind, ExprKind::StaticPropertyAccess(_))
                || matches!(b.left.kind, ExprKind::StaticPropertyAccess(_))
                || matches!(b.right.kind, ExprKind::ClassConstAccess(_))
                || matches!(b.left.kind, ExprKind::ClassConstAccess(_))
            {
                narrow_from_static_or_class_const_comparison(ctx, b, effective_true, db, file);
            }
        }

        // $x instanceof ClassName  /  $this->prop instanceof ClassName
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            if let Some(var_name) = extract_var_name(&b.left) {
                if let Some(raw_name) = extract_class_name(
                    &b.right,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                ) {
                    // Resolve the short name to its FQCN using file imports
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    let current = ctx.get_var(&var_name);
                    let narrowed = if is_true {
                        narrow_instanceof_preserving_subtypes(
                            &current,
                            &class_name,
                            db,
                            &ctx.template_param_names,
                        )
                    } else {
                        filter_out_instanceof_match(&current, &class_name, db)
                    };
                    set_narrowed(ctx, &var_name, &current, narrowed, true);
                }
            } else if let Some((obj, prop)) = extract_nullsafe_prop_access(&b.left) {
                if let Some(raw_name) = extract_class_name(
                    &b.right,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                ) {
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    narrow_prop_instanceof(ctx, &obj, &prop, &class_name, db, file, is_true);
                    // `null instanceof X` is always false, so a true result also
                    // proves the receiver itself is non-null (see
                    // `narrow_nullsafe_prop_null` for the same reasoning).
                    if is_true {
                        narrow_var_null(ctx, &obj, false);
                    }
                }
            } else if let Some((obj, prop)) = extract_prop_access(&b.left) {
                if let Some(raw_name) = extract_class_name(
                    &b.right,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                ) {
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    narrow_prop_instanceof(ctx, &obj, &prop, &class_name, db, file, is_true);
                    // Same reasoning as the nullsafe arm above: `null instanceof X`
                    // is always false, so proving it true also proves `$obj` itself
                    // wasn't null (PHP 8 reads `$obj->prop` on a null `$obj` as a
                    // warning, still evaluating to null).
                    if is_true {
                        narrow_var_null(ctx, &obj, false);
                    }
                }
            } else if let Some((fqcn, prop)) = extract_static_prop_access(&b.left, ctx, db, file) {
                if let Some(raw_name) = extract_class_name(
                    &b.right,
                    ctx.self_fqcn.as_deref(),
                    ctx.parent_fqcn.as_deref(),
                ) {
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    narrow_static_prop_instanceof(ctx, &fqcn, &prop, &class_name, db, is_true);
                }
            }
        }

        // is_string($x), is_int($x), is_null($x), is_array($x), etc.
        // Also handles assert($x instanceof Y) — narrows like a bare condition.
        ExprKind::FunctionCall(call) => {
            // `ExprKind::Variable` (a dynamic `$fn()` call) is deliberately not
            // matched here: `name` would be the callable variable's own
            // identifier text, not its runtime string value, so every builtin
            // check below would only ever match by coincidence (e.g. a
            // variable literally named `$is_array`) while never actually
            // calling that builtin.
            let fn_name_opt: Option<&str> = match &call.name.kind {
                ExprKind::Identifier(name) => Some(name.as_ref()),
                _ => None,
            };
            if let Some(fn_name) = fn_name_opt {
                let bare = fn_name.trim_start_matches('\\');
                if matches!(
                    bare.to_ascii_lowercase().as_str(),
                    "class_exists" | "interface_exists" | "trait_exists" | "enum_exists"
                ) {
                    // `if (class_exists(\Foo\Bar::class)) { ... }` — record \Foo\Bar as
                    // proven-to-exist in the true branch so that UndefinedClass is
                    // suppressed for all usages within the guarded block.
                    // Variable form: `if (class_exists($var)) { ... }` — narrow $var to
                    // class-string so it satisfies class-string-typed parameters.
                    // `interface_exists($var)` narrows to the more precise interface-string.
                    // `enum_exists($var)`/`trait_exists($var)` narrow like class_exists —
                    // no dedicated enum-string/trait-string atomic exists.
                    if is_true {
                        if let Some(arg_expr) = call.args.first() {
                            if let Some(fqcn) = extract_class_fqcn_from_expr(
                                &arg_expr.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                ctx.class_exists_guards.insert(fqcn);
                            } else if let Some(var_name) = extract_var_name(&arg_expr.value) {
                                let current = ctx.get_var(&var_name);
                                let narrowed = if bare.eq_ignore_ascii_case("interface_exists") {
                                    current.narrow_to_interface_string()
                                } else {
                                    current.narrow_to_class_string()
                                };
                                set_narrowed(ctx, &var_name, &current, narrowed, true);
                            } else if let Some((obj, prop)) = extract_prop_access(&arg_expr.value) {
                                let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                                if !current.is_mixed() {
                                    let narrowed = if bare.eq_ignore_ascii_case("interface_exists")
                                    {
                                        current.narrow_to_interface_string()
                                    } else {
                                        current.narrow_to_class_string()
                                    };
                                    apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, true);
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("defined") {
                    // `if (defined('NAME')) { ... NAME ... }` — record NAME as
                    // proven-defined in the true branch to suppress
                    // UndefinedConstant within the guarded block.
                    if is_true {
                        if let Some(arg) = call.args.first() {
                            if let ExprKind::String(name) = &arg.value.kind {
                                let name = name.as_ref().trim_start_matches('\\');
                                if !name.is_empty() {
                                    ctx.defined_guards.insert(std::sync::Arc::from(name));
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("function_exists") {
                    // `if (function_exists('fn')) { ... fn() ... }` — record fn
                    // as proven-to-exist in the true branch to suppress
                    // UndefinedFunction within the guarded block. Combined with
                    // negation + divergence (`if (!function_exists('fn')) throw;`)
                    // this also covers the early-exit pattern.
                    if is_true {
                        if let Some(arg) = call.args.first() {
                            if let ExprKind::String(name) = &arg.value.kind {
                                let name = name.as_ref().trim_start_matches('\\');
                                if !name.is_empty() {
                                    ctx.function_exists_guards
                                        .insert(std::sync::Arc::from(name));
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("extension_loaded") {
                    // `if (extension_loaded('ext')) { ... }` — record the extension
                    // name so that `UndefinedClass` is suppressed for any class used
                    // inside the guarded block (the caller verified the extension is
                    // present). The false-branch / early-exit pattern also works via
                    // the normal divergence+narrowing mechanism.
                    if is_true {
                        if let Some(arg) = call.args.first() {
                            if let ExprKind::String(ext) = &arg.value.kind {
                                if !ext.is_empty() {
                                    ctx.extension_loaded_guards
                                        .insert(std::sync::Arc::from(ext.as_ref()));
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("assert") {
                    // assert($condition) — narrow as if the condition is is_true
                    if let Some(arg_expr) = call.args.first() {
                        narrow_from_condition(&arg_expr.value, ctx, is_true, db, file);
                    }
                } else if bare.eq_ignore_ascii_case("method_exists")
                    || bare.eq_ignore_ascii_case("property_exists")
                {
                    // Narrow the first arg to TObject for simple variables (existing behaviour).
                    // Additionally record `(expr_key, method_name)` in method_exists_guards for
                    // property-access first args where variable narrowing can't reach — but only
                    // for method_exists(): properties and methods are independent namespaces in
                    // PHP, so property_exists($obj, 'foo') proves nothing about a method 'foo'.
                    if let Some(arg_expr) = call.args.first() {
                        if let Some(var_name) = extract_var_name(&arg_expr.value) {
                            narrow_from_type_fn(ctx, bare, &var_name, db, is_true);
                        } else if let Some((obj, prop)) = extract_prop_access(&arg_expr.value) {
                            narrow_prop_from_type_fn(ctx, bare, &obj, &prop, db, file, is_true);
                        } else if let Some((fqcn, prop)) =
                            extract_static_prop_access(&arg_expr.value, ctx, db, file)
                        {
                            narrow_static_prop_from_type_fn(ctx, bare, &fqcn, &prop, db, is_true);
                        }
                        if is_true && bare.eq_ignore_ascii_case("method_exists") {
                            if let Some(expr_key) =
                                extract_expr_guard_key(&arg_expr.value, db, file)
                            {
                                if let Some(method_arg) = call.args.get(1) {
                                    if let ExprKind::String(method_name) = &method_arg.value.kind {
                                        let method_lc = std::sync::Arc::from(
                                            crate::util::php_ident_lowercase(method_name).as_str(),
                                        );
                                        ctx.method_exists_guards.insert((expr_key, method_lc));
                                    }
                                }
                            }
                        }
                    }
                } else if matches!(
                    bare.to_ascii_lowercase().as_str(),
                    "array_key_exists" | "key_exists"
                ) {
                    // array_key_exists('k', $arr) in true-branch: prove the key
                    // exists in the array's sealed shape so that $arr['k'] does
                    // not trigger NonExistentArrayOffset afterwards.
                    // `key_exists()` is a built-in alias of `array_key_exists()`
                    // with identical semantics.
                    if let (Some(key_arg), Some(arr_arg)) = (call.args.first(), call.args.get(1)) {
                        let literal_key = match &key_arg.value.kind {
                            ExprKind::String(s) => Some(mir_types::atomic::ArrayKey::String(
                                std::sync::Arc::from(s.as_ref()),
                            )),
                            ExprKind::Int(i) => Some(mir_types::atomic::ArrayKey::Int(*i)),
                            // `$key = 'name'; array_key_exists($key, $arr)` — resolve a
                            // variable OR property-access key already narrowed to a
                            // single literal, same as an inline literal would be.
                            _ => {
                                let key_ty = if let Some(name) = extract_var_name(&key_arg.value) {
                                    Some(ctx.get_var(&name))
                                } else {
                                    extract_prop_access(&key_arg.value).map(|(obj, prop)| {
                                        resolve_prop_current_type(ctx, &obj, &prop, db, file)
                                    })
                                };
                                key_ty.and_then(|ty| match ty.types.as_slice() {
                                    [Atomic::TLiteralString(s)] => {
                                        Some(mir_types::atomic::ArrayKey::String(s.clone()))
                                    }
                                    [Atomic::TLiteralInt(i)] => {
                                        Some(mir_types::atomic::ArrayKey::Int(*i))
                                    }
                                    _ => None,
                                })
                            }
                        };
                        if let Some(key) = literal_key {
                            if is_true {
                                if let Some(var_name) = extract_var_name(&arr_arg.value) {
                                    let current = ctx.get_var(&var_name);
                                    let narrowed = add_key_to_sealed_shapes(&current, &key);
                                    if narrowed != current {
                                        ctx.set_var(&var_name, narrowed);
                                    }
                                } else if let Some((obj, prop)) =
                                    extract_prop_access(&arr_arg.value)
                                {
                                    narrow_prop_array_key_exists(ctx, &obj, &prop, &key, db, file);
                                } else if let Some((fqcn, prop)) =
                                    extract_static_prop_access(&arr_arg.value, ctx, db, file)
                                {
                                    narrow_static_prop_array_key_exists(
                                        ctx, &fqcn, &prop, &key, db,
                                    );
                                } else if let Some((base, path)) =
                                    collect_array_access_path(&arr_arg.value)
                                {
                                    // Nested container, e.g. array_key_exists('b', $arr['a']) —
                                    // walk down to the ['a'] shape and prove 'b' present there,
                                    // same as the single-level var/prop cases above.
                                    let current =
                                        resolve_shape_base_current_type(ctx, &base, db, file);
                                    if let Some(narrowed) =
                                        narrow_shape_path_key_exists(&current, &path, &key)
                                    {
                                        set_shape_base_narrowed(ctx, &base, current, narrowed);
                                    }
                                } else if let (
                                    mir_types::atomic::ArrayKey::String(iface_name),
                                    Some(target),
                                ) = (
                                    &key,
                                    extract_class_implements_or_parents_arg(&arr_arg.value),
                                ) {
                                    // array_key_exists('Iface', class_implements($x)) /
                                    // ('Ancestor', class_parents($x)) — same relationship
                                    // `$x instanceof Iface`/`instanceof Ancestor` proves.
                                    let fqcn = crate::db::resolve_name(db, file, iface_name);
                                    match &target {
                                        ScalarArgTarget::Var(var_name) => {
                                            let current = ctx.get_var(var_name);
                                            let narrowed = narrow_instanceof_preserving_subtypes(
                                                &current,
                                                &fqcn,
                                                db,
                                                &ctx.template_param_names,
                                            );
                                            set_narrowed(ctx, var_name, &current, narrowed, true);
                                        }
                                        ScalarArgTarget::Prop(obj, prop) => {
                                            narrow_prop_instanceof(
                                                ctx, obj, prop, &fqcn, db, file, true,
                                            );
                                        }
                                    }
                                }
                            } else {
                                // False branch: exclude shape members that
                                // guarantee the key's presence — see
                                // `remove_key_from_sealed_shapes`.
                                if let Some(var_name) = extract_var_name(&arr_arg.value) {
                                    let current = ctx.get_var(&var_name);
                                    let narrowed = remove_key_from_sealed_shapes(&current, &key);
                                    set_narrowed(ctx, &var_name, &current, narrowed, true);
                                } else if let Some((obj, prop)) =
                                    extract_prop_access(&arr_arg.value)
                                {
                                    let current =
                                        resolve_prop_current_type(ctx, &obj, &prop, db, file);
                                    if !current.is_mixed() {
                                        let narrowed =
                                            remove_key_from_sealed_shapes(&current, &key);
                                        apply_prop_narrowed(
                                            ctx, &obj, &prop, current, narrowed, true,
                                        );
                                    }
                                } else if let Some((fqcn, prop)) =
                                    extract_static_prop_access(&arr_arg.value, ctx, db, file)
                                {
                                    let current =
                                        resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                                    if !current.is_mixed() {
                                        let narrowed =
                                            remove_key_from_sealed_shapes(&current, &key);
                                        apply_prop_narrowed(
                                            ctx, &fqcn, &prop, current, narrowed, true,
                                        );
                                    }
                                } else if let Some((base, path)) =
                                    collect_array_access_path(&arr_arg.value)
                                {
                                    // Nested container, false branch, e.g.
                                    // array_key_exists('b', $arr['a']) proven
                                    // false — same as the single-level
                                    // var/prop cases above.
                                    let current =
                                        resolve_shape_base_current_type(ctx, &base, db, file);
                                    if let Some(narrowed) =
                                        narrow_shape_path_key_exists_false(&current, &path, &key)
                                    {
                                        set_shape_base_narrowed(ctx, &base, current, narrowed);
                                    }
                                } else if let (
                                    mir_types::atomic::ArrayKey::String(iface_name),
                                    Some(target),
                                ) = (
                                    &key,
                                    extract_class_implements_or_parents_arg(&arr_arg.value),
                                ) {
                                    // !array_key_exists('Iface', class_implements($x)) —
                                    // exclude Iface, same as `!($x instanceof Iface)`.
                                    let fqcn = crate::db::resolve_name(db, file, iface_name);
                                    match &target {
                                        ScalarArgTarget::Var(var_name) => {
                                            let current = ctx.get_var(var_name);
                                            let narrowed =
                                                filter_out_instanceof_match(&current, &fqcn, db);
                                            set_narrowed(ctx, var_name, &current, narrowed, true);
                                        }
                                        ScalarArgTarget::Prop(obj, prop) => {
                                            narrow_prop_instanceof(
                                                ctx, obj, prop, &fqcn, db, file, false,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if matches!(
                    bare.to_ascii_lowercase().as_str(),
                    "str_contains" | "str_starts_with" | "str_ends_with"
                ) {
                    // str_contains($haystack, 'x') true → $haystack is non-empty-string
                    // (when the needle is a non-empty literal — a non-empty needle can
                    // only be found in a non-empty haystack).
                    if is_true {
                        if let (Some(haystack_arg), Some(needle_arg)) =
                            (call.args.first(), call.args.get(1))
                        {
                            let needle_non_empty =
                                expr_is_nonempty_string_literal(&needle_arg.value, ctx, db, file);
                            if needle_non_empty {
                                match ScalarArgTarget::extract(&haystack_arg.value) {
                                    Some(ScalarArgTarget::Var(var_name)) => {
                                        let current = ctx.get_var(&var_name);
                                        if !current.is_mixed() {
                                            let narrowed = narrow_string_to_non_empty(&current);
                                            if narrowed != current {
                                                ctx.set_var(&var_name, narrowed);
                                            }
                                        }
                                    }
                                    Some(ScalarArgTarget::Prop(obj, prop)) => {
                                        let current =
                                            resolve_prop_current_type(ctx, &obj, &prop, db, file);
                                        if !current.is_mixed() {
                                            let narrowed = narrow_string_to_non_empty(&current);
                                            apply_prop_narrowed(
                                                ctx, &obj, &prop, current, narrowed, false,
                                            );
                                        }
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("in_array") {
                    // in_array($needle, ['a', 'b', 'c']) true →
                    // narrow $needle to 'a'|'b'|'c'. Only safe when either the
                    // 3rd (strict) argument is truthy, or $needle's current
                    // type and the haystack are both exclusively string atoms
                    // or both exclusively int atoms — for same-category
                    // scalars, loose (==) comparison agrees with strict
                    // (===). A mixed-category needle (e.g. int|string) can't
                    // be narrowed under loose comparison: the string "1"
                    // loosely equals the int 1, so a haystack of `[1, 2]`
                    // doesn't rule out $needle being the string "1".
                    let strict = call
                        .args
                        .get(2)
                        .map(|a| is_truthy_bool_literal(&a.value))
                        .unwrap_or(false);
                    if let (Some(needle_arg), Some(haystack_arg)) =
                        (call.args.first(), call.args.get(1))
                    {
                        if let Some(var_name) = extract_var_name(&needle_arg.value) {
                            if let Some(haystack_ty) =
                                extract_haystack_type(&haystack_arg.value, ctx)
                            {
                                let current = ctx.get_var(&var_name);
                                let loose_safe = strict
                                    || in_array_loose_narrowing_is_safe(&current, &haystack_ty);
                                if !current.is_mixed() && is_true && loose_safe {
                                    // intersect: keep only types that could match a haystack value
                                    let narrowed =
                                        narrow_to_haystack_values(&current, &haystack_ty);
                                    if !narrowed.is_empty() && narrowed != current {
                                        ctx.set_var(&var_name, narrowed);
                                    }
                                } else if !current.is_mixed() && !is_true && loose_safe {
                                    // False branch: safe only when the current type is a
                                    // finite literal union — remove the matched haystack values.
                                    let all_literals = !current.types.is_empty()
                                        && current.types.iter().all(|a| {
                                            matches!(
                                                a,
                                                Atomic::TLiteralString(_) | Atomic::TLiteralInt(_)
                                            )
                                        });
                                    if all_literals {
                                        let narrowed = current
                                            .filter(|a| !haystack_ty.types.iter().any(|h| h == a));
                                        if !narrowed.is_empty() && narrowed != current {
                                            ctx.set_var(&var_name, narrowed);
                                        }
                                    }
                                }
                            }
                        } else if let Some((obj, prop)) = extract_prop_access(&needle_arg.value) {
                            // Property-access counterpart of the plain-variable case
                            // above, e.g. `in_array($this->status, ['a', 'b'])`.
                            if let Some(haystack_ty) =
                                extract_haystack_type(&haystack_arg.value, ctx)
                            {
                                let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                                let loose_safe = strict
                                    || in_array_loose_narrowing_is_safe(&current, &haystack_ty);
                                if !current.is_mixed() && is_true && loose_safe {
                                    let narrowed =
                                        narrow_to_haystack_values(&current, &haystack_ty);
                                    if !narrowed.is_empty() {
                                        apply_prop_narrowed(
                                            ctx, &obj, &prop, current, narrowed, false,
                                        );
                                    }
                                } else if !current.is_mixed() && !is_true && loose_safe {
                                    let all_literals = !current.types.is_empty()
                                        && current.types.iter().all(|a| {
                                            matches!(
                                                a,
                                                Atomic::TLiteralString(_) | Atomic::TLiteralInt(_)
                                            )
                                        });
                                    if all_literals {
                                        let narrowed = current
                                            .filter(|a| !haystack_ty.types.iter().any(|h| h == a));
                                        if !narrowed.is_empty() {
                                            apply_prop_narrowed(
                                                ctx, &obj, &prop, current, narrowed, false,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("is_a") {
                    // is_a($obj, 'ClassName') → instanceof semantics (includes exact class).
                    // When $allow_string (3rd arg) is truthy, the first arg may be a class-string
                    // — preserve string/class-string atoms so the true branch stays reachable
                    // and no false diverge is set in the false branch.
                    if let (Some(obj_arg), Some(class_arg)) = (call.args.first(), call.args.get(1))
                    {
                        if let Some(var_name) = extract_var_name(&obj_arg.value) {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                let allow_string = call
                                    .args
                                    .get(2)
                                    .map(|a| is_truthy_bool_literal(&a.value))
                                    .unwrap_or(false);
                                let current = ctx.get_var(&var_name);
                                if allow_string {
                                    // When allow_string is true, string/class-string atoms are
                                    // valid is_a() true-branch values — preserve them so the
                                    // true branch stays reachable and type is not wrongly erased.
                                    let narrowed = if is_true {
                                        // Partition into string-like (kept only when consistent
                                        // with class_name) and object-like (narrowed via
                                        // instanceof) so `narrow_instanceof_preserving_subtypes`
                                        // fallback doesn't inject a spurious named-object atom
                                        // when the current type is purely string/class-string.
                                        let (mut result, obj_part) =
                                            partition_is_a_string_like(&current, &class_name, db);
                                        if !obj_part.is_empty() || current.is_mixed() {
                                            let obj_src = if obj_part.is_empty() {
                                                &current
                                            } else {
                                                &obj_part
                                            };
                                            let obj_narrowed =
                                                narrow_instanceof_preserving_subtypes(
                                                    obj_src,
                                                    &class_name,
                                                    db,
                                                    &ctx.template_param_names,
                                                );
                                            for atom in obj_narrowed.types.iter() {
                                                result.add_type(atom.clone());
                                            }
                                        }
                                        result
                                    } else {
                                        filter_out_is_a_string_match(&current, &class_name, db)
                                    };
                                    // Don't mark diverges when allow_string is set: a
                                    // class-string variable may still be a valid non-object
                                    // value that passes the test.
                                    set_narrowed(ctx, &var_name, &current, narrowed, false);
                                } else {
                                    let narrowed = if is_true {
                                        narrow_instanceof_preserving_subtypes(
                                            &current,
                                            &class_name,
                                            db,
                                            &ctx.template_param_names,
                                        )
                                    } else {
                                        filter_out_instanceof_match(&current, &class_name, db)
                                    };
                                    set_narrowed(ctx, &var_name, &current, narrowed, true);
                                }
                            }
                        } else if let Some((obj, prop)) = extract_prop_access(&obj_arg.value) {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                let allow_string = call
                                    .args
                                    .get(2)
                                    .map(|a| is_truthy_bool_literal(&a.value))
                                    .unwrap_or(false);
                                narrow_prop_is_a(
                                    ctx,
                                    &obj,
                                    &prop,
                                    &class_name,
                                    allow_string,
                                    db,
                                    file,
                                    is_true,
                                );
                                narrow_receiver_non_null_on_prop_match(ctx, &obj, is_true);
                            }
                        } else if let Some((fqcn, prop)) =
                            extract_static_prop_access(&obj_arg.value, ctx, db, file)
                        {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                let allow_string = call
                                    .args
                                    .get(2)
                                    .map(|a| is_truthy_bool_literal(&a.value))
                                    .unwrap_or(false);
                                narrow_static_prop_is_a(
                                    ctx,
                                    &fqcn,
                                    &prop,
                                    &class_name,
                                    allow_string,
                                    db,
                                    is_true,
                                );
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("is_subclass_of") {
                    // is_subclass_of($obj, 'ClassName') → strict-subclass check: the exact
                    // class itself is NOT a subclass of itself.
                    // True branch: keep only atoms that are known strict subclasses.
                    // False branch: no narrowing — is_subclass_of being false doesn't tell us
                    // the variable isn't Foo; Foo is not a subclass of itself, so Foo atoms
                    // must be kept (removing them would make a later `Foo` use diverge).
                    if let (Some(obj_arg), Some(class_arg)) = (call.args.first(), call.args.get(1))
                    {
                        if let Some(var_name) = extract_var_name(&obj_arg.value) {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                let current = ctx.get_var(&var_name);
                                if is_true {
                                    let narrowed = narrow_strict_subclass_of(
                                        &current,
                                        &class_name,
                                        db,
                                        &ctx.template_param_names,
                                    );
                                    // mark_diverges=false: the exact class being absent from
                                    // strict-subclass narrowing doesn't make the branch dead.
                                    set_narrowed(ctx, &var_name, &current, narrowed, false);
                                }
                                // False branch: leave current type unchanged.
                            }
                        } else if let Some((obj, prop)) = extract_prop_access(&obj_arg.value) {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                narrow_prop_is_subclass_of(
                                    ctx,
                                    &obj,
                                    &prop,
                                    &class_name,
                                    db,
                                    file,
                                    is_true,
                                );
                                narrow_receiver_non_null_on_prop_match(ctx, &obj, is_true);
                            }
                        } else if let Some((fqcn, prop)) =
                            extract_static_prop_access(&obj_arg.value, ctx, db, file)
                        {
                            if let Some(class_name) = extract_class_fqcn_from_expr(
                                &class_arg.value,
                                ctx.self_fqcn.as_deref(),
                                ctx.static_fqcn.as_deref(),
                                ctx.parent_fqcn.as_deref(),
                                db,
                                file,
                            ) {
                                narrow_static_prop_is_subclass_of(
                                    ctx,
                                    &fqcn,
                                    &prop,
                                    &class_name,
                                    db,
                                    is_true,
                                );
                            }
                        }
                    }
                } else if apply_docblock_assertions(call, ctx, is_true, db, file, fn_name) {
                    // User-defined assertion applied.
                } else if let Some(arg_expr) = call.args.first() {
                    if let Some(var_name) = extract_var_name(&arg_expr.value) {
                        narrow_from_type_fn(ctx, bare, &var_name, db, is_true);
                    } else if let Some((obj, prop)) = extract_prop_access(&arg_expr.value) {
                        narrow_prop_from_type_fn(ctx, bare, &obj, &prop, db, file, is_true);
                    } else if let Some((fqcn, prop)) =
                        extract_static_prop_access(&arg_expr.value, ctx, db, file)
                    {
                        narrow_static_prop_from_type_fn(ctx, bare, &fqcn, &prop, db, is_true);
                    }
                }
            }
        }

        // isset($x)
        ExprKind::Isset(vars) => {
            for var_expr in vars.iter() {
                if let Some(var_name) = extract_var_name(var_expr) {
                    if is_true {
                        // remove null; mark as definitely assigned
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                        std::sync::Arc::make_mut(&mut ctx.assigned_vars)
                            .insert(mir_types::Name::from(var_name.as_str()));
                    }
                } else if is_true {
                    // `isset($base[$k])` implies `$base` is a non-null, indexable
                    // value — remove null/false from the base (variable or
                    // property receiver) so a guarded access (`preg_split()`
                    // returns array|false) does not report PossiblyInvalidArrayAccess.
                    if let Some(target) = array_access_base_target(var_expr) {
                        narrow_container_non_null_non_false(ctx, &target, db, file);
                    }
                    // For a single-level `isset($arr['key'])` on a shape-typed
                    // base, also narrow that key's OWN value type: remove null
                    // and mark it no longer optional, so a later `$arr['key']`
                    // read inside the guard isn't reported as possibly-null (the
                    // isset check just proved the key is present and non-null).
                    narrow_isset_shape_key(var_expr, ctx, db, file);
                    // `isset($this->prop)` implies the property is non-null too
                    // — the property-receiver counterpart of the bare-variable
                    // case above, since `isset()` is false for both an unset
                    // and a null-valued property.
                    if let Some((obj_var, prop)) = extract_any_prop_access(var_expr) {
                        let current = resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
                        if !current.is_mixed() {
                            let narrowed = current.remove_null();
                            apply_prop_narrowed(ctx, &obj_var, &prop, current, narrowed, true);
                        }
                        // isset() is only true when the receiver was non-null too
                        // (a null receiver's ->/?-> access is itself unset/null).
                        narrow_receiver_non_null_on_prop_match(ctx, &obj_var, true);
                    }
                }
            }
        }

        // empty($x)
        ExprKind::Empty(var_expr) => {
            if let Some(var_name) = extract_var_name(var_expr) {
                let current = ctx.get_var(&var_name);
                let narrowed = if is_true {
                    // empty($x) is true: x is falsy
                    current.narrow_to_falsy()
                } else {
                    // empty($x) is false: x is truthy
                    current.narrow_to_truthy()
                };
                if !narrowed.is_empty() {
                    ctx.set_var(&var_name, narrowed);
                }
            } else {
                if !is_true {
                    // `!empty($base[$k])` implies `$base` is a non-null, indexable
                    // value, same as the `isset($base[$k])` case above.
                    if let Some(target) = array_access_base_target(var_expr) {
                        narrow_container_non_null_non_false(ctx, &target, db, file);
                    }
                }
                // For a single-level `empty($arr['key'])` on a shape-typed base,
                // also narrow that key's OWN value type by truthiness, mirroring
                // narrow_isset_shape_key.
                narrow_empty_shape_key(var_expr, ctx, is_true, db, file);
                // `empty($this->prop)` — property-receiver counterpart of the
                // bare-variable truthy/falsy case, mirroring the `if ($this->prop)`
                // arm below (narrow_prop_loose_bool). `empty()` inverts truthiness:
                // is_true means the property is falsy.
                if let Some((obj_var, prop)) = extract_any_prop_access(var_expr) {
                    narrow_prop_loose_bool(ctx, &obj_var, &prop, db, file, !is_true);
                    // `empty()` is false (the property is truthy) only when the
                    // receiver was non-null too — a null receiver's ->/?->
                    // access is itself falsy, so `empty()` would be true.
                    narrow_receiver_non_null_on_prop_match(ctx, &obj_var, !is_true);
                }
            }
        }

        // ($x = expr) / ($x ??= expr) used as a condition
        // The assignment has already been evaluated (ctx holds the post-assignment type).
        // Narrow the target variable based on the truthiness of the expression result.
        ExprKind::Assign(a) if matches!(a.op, AssignOp::Assign | AssignOp::Coalesce) => {
            if let Some(var_name) = extract_var_name(&a.target) {
                let current = ctx.get_var(&var_name);
                let mut narrowed = if is_true {
                    current.narrow_to_truthy()
                } else {
                    current.narrow_to_falsy()
                };
                // In the true-branch the assignment definitely executed, so
                // the variable is always defined here — clear possibly_undefined.
                if is_true {
                    narrowed.possibly_undefined = false;
                }
                if !narrowed.is_empty() {
                    ctx.set_var(&var_name, narrowed);
                } else if !current.is_empty() && !current.is_mixed() {
                    ctx.diverges = true;
                }
            } else if let Some((obj_var, prop)) = extract_prop_access(&a.target) {
                // `if ($this->prop = expr)` — property-access counterpart of
                // the plain-variable case above.
                let current = resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
                let mut narrowed = if is_true {
                    current.narrow_to_truthy()
                } else {
                    current.narrow_to_falsy()
                };
                if is_true {
                    narrowed.possibly_undefined = false;
                }
                apply_prop_narrowed(ctx, &obj_var, &prop, current, narrowed, true);
            }
        }

        // if ($x)  — truthy/falsy narrowing
        _ => {
            if let Some(var_name) = extract_var_name(expr) {
                let current = ctx.get_var(&var_name);
                let narrowed = if is_true {
                    current.narrow_to_truthy()
                } else {
                    current.narrow_to_falsy()
                };
                if !narrowed.is_empty() {
                    ctx.set_var(&var_name, narrowed);
                } else if !current.is_empty()
                    && !current.is_mixed()
                    && ctx.var_is_defined(&var_name)
                {
                    // The variable's type can never satisfy this truthiness
                    // constraint → this branch is statically unreachable.
                    // Possibly-undefined variables are exempt: an unset
                    // variable reads as null (falsy), so the branch stays
                    // reachable at runtime.
                    ctx.diverges = true;
                }
            } else if let Some((obj_var, prop)) = extract_any_prop_access(expr) {
                // `if ($this->prop)` — property-receiver counterpart of the
                // bare-variable truthy/falsy case above.
                narrow_prop_loose_bool(ctx, &obj_var, &prop, db, file, is_true);
                // Truthy also proves the receiver was non-null (a null
                // receiver's ->/?-> access is itself falsy).
                narrow_receiver_non_null_on_prop_match(ctx, &obj_var, is_true);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn apply_docblock_assertions(
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
    let expected_kind = if is_true {
        AssertionKind::AssertIfTrue
    } else {
        AssertionKind::AssertIfFalse
    };

    let assertions = &f.assertions;
    let params = &f.params;

    // An assertion type written in terms of the function's own `@template`s
    // (e.g. `@psalm-assert-if-true T $value` alongside `@param
    // class-string<T> $class`) must resolve T from this call's actual
    // arguments before narrowing — otherwise the variable narrows to the
    // bare, unresolved template atom instead of the concrete type.
    let template_bindings = if f.template_params.is_empty() {
        None
    } else {
        let arg_types: Vec<Type> = call
            .args
            .iter()
            .map(|arg| assertion_arg_type(&arg.value, ctx, db, file))
            .collect();
        let arg_names: Vec<Option<String>> = call
            .args
            .iter()
            .map(|arg| arg.name.as_ref().map(crate::parser::name_to_string_owned))
            .collect();
        Some(
            crate::generic::infer_template_bindings(
                db,
                &f.template_params,
                params,
                &arg_types,
                &arg_names,
            )
            .0,
        )
    };

    let mut applied = false;
    for assertion in assertions
        .iter()
        .filter(|a| a.kind == expected_kind || (is_true && a.kind == AssertionKind::Assert))
    {
        if let Some(index) = params.iter().position(|p| p.name == assertion.param) {
            // A variadic param's assertion applies to every trailing positional
            // arg it swallows (`assertVariadic(...$values)` asserted over each
            // of `assertVariadic($a, $b, $c)`), not just the first one —
            // `arg_for_param_index` only ever resolves a single positional arg.
            let variadic_args: Vec<&php_ast::owned::Arg>;
            let args_to_check: &[&php_ast::owned::Arg] = if params[index].is_variadic {
                variadic_args = call
                    .args
                    .iter()
                    .filter(|a| a.name.is_none())
                    .skip(index)
                    .collect();
                &variadic_args
            } else {
                variadic_args = arg_for_param_index(params, &call.args, index)
                    .into_iter()
                    .collect();
                &variadic_args
            };
            for arg in args_to_check {
                if let Some(var_name) = extract_var_name(&arg.value) {
                    let ty = match &template_bindings {
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
                } else if let Some((obj, prop)) = extract_prop_access(&arg.value) {
                    let ty = match &template_bindings {
                        Some(b) => assertion.ty.substitute_templates(b),
                        None => assertion.ty.clone(),
                    };
                    let ty = if assertion.negated {
                        let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                        negate_assertion_type(&current, &ty, db)
                    } else {
                        ty
                    };
                    ctx.set_prop_refined(&obj, &prop, ty);
                    applied = true;
                }
            }
        }
    }

    applied
}

/// Compute the narrowed type for a negated assertion (`@psalm-assert !Type
/// $x` — "$x is asserted NOT to be this type"): `current` minus `asserted`
/// for the shapes that can be precisely subtracted — `null`, `false`, and a
/// single named class/interface (via the same subclass-aware exclusion a
/// `!($x instanceof C)` guard already uses). Anything else is left
/// unchanged rather than risk excluding too much.
pub(crate) fn negate_assertion_type(current: &Type, asserted: &Type, db: &dyn MirDatabase) -> Type {
    if current.is_mixed_not_template() || asserted.types.len() != 1 {
        return current.clone();
    }
    match &asserted.types[0] {
        Atomic::TNull => current.remove_null(),
        Atomic::TFalse => current.remove_false(),
        Atomic::TNamedObject { fqcn, .. }
        | Atomic::TSelf { fqcn }
        | Atomic::TStaticObject { fqcn }
        | Atomic::TParent { fqcn } => filter_out_instanceof_match(current, fqcn, db),
        _ => current.clone(),
    }
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

/// Collect class names from `instanceof` checks on the SAME variable across
/// an arbitrary expression — recursing through `||`/`or` and parens. Returns
/// `false` as soon as something doesn't fit the shape (a non-instanceof leaf,
/// or an instanceof on a different variable), signaling the caller should not
/// treat the whole set as one OR-chain over a single variable.
#[allow(clippy::too_many_arguments)]
fn collect_instanceof(
    expr: &php_ast::owned::Expr,
    var_name: &mut Option<String>,
    class_names: &mut Vec<String>,
    db: &dyn MirDatabase,
    file: &str,
    self_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
) -> bool {
    match &expr.kind {
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            if let (Some(vn), Some(cn)) = (
                extract_var_name(&b.left),
                extract_class_name(&b.right, self_fqcn, parent_fqcn),
            ) {
                let resolved = crate::db::resolve_name(db, file, &cn);
                match var_name {
                    None => {
                        *var_name = Some(vn);
                        class_names.push(resolved);
                        true
                    }
                    Some(existing) if existing == &vn => {
                        class_names.push(resolved);
                        true
                    }
                    _ => false, // different variable — bail out
                }
            } else {
                false
            }
        }
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            collect_instanceof(
                &b.left,
                var_name,
                class_names,
                db,
                file,
                self_fqcn,
                parent_fqcn,
            ) && collect_instanceof(
                &b.right,
                var_name,
                class_names,
                db,
                file,
                self_fqcn,
                parent_fqcn,
            )
        }
        ExprKind::Parenthesized(inner) => collect_instanceof(
            inner,
            var_name,
            class_names,
            db,
            file,
            self_fqcn,
            parent_fqcn,
        ),
        _ => false,
    }
}

/// Narrow `$x` to the union of every `instanceof` class collected from
/// `conditions`, when EVERY condition is an `instanceof` (or OR-chain/parens
/// thereof) on the SAME variable — e.g. `$x instanceof A, $x instanceof B`
/// comma-separated `match(true)` arm conditions, or the two sides of an
/// `if ($x instanceof A || $x instanceof B)`.
///
/// Returns `true` when it fully narrowed (every condition fit the shape and
/// shared one variable) — the caller should then skip narrowing those
/// conditions again individually, since re-applying each `instanceof` in
/// sequence would AND-compose them and collapse the result to the last
/// disjunct (no value can be simultaneously exactly-A and exactly-B).
/// Returns `false` when the shape doesn't apply (mixed condition kinds, or
/// instanceof checks on different variables) — the caller should fall back
/// to narrowing each condition normally.
/// Returns the discovered variable name on success (the caller may want to
/// know which variable was narrowed, e.g. to re-apply the result on top of a
/// different context — see `analyze_switch_stmt`'s fallthrough handling).
pub(crate) fn narrow_instanceof_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<String> {
    if conditions.len() < 2 {
        return None;
    }
    let self_fqcn = ctx.self_fqcn.as_deref();
    let parent_fqcn = ctx.parent_fqcn.as_deref();

    let mut var_name: Option<String> = None;
    let mut class_names: Vec<String> = vec![];
    let all_ok = conditions.iter().all(|cond| {
        collect_instanceof(
            cond,
            &mut var_name,
            &mut class_names,
            db,
            file,
            self_fqcn,
            parent_fqcn,
        )
    });

    if !all_ok || class_names.len() < 2 {
        return None;
    }
    let vn = var_name?;

    let current = ctx.get_var(&vn);
    // Narrow to the union of all instanceof types, classifying each union
    // member against every disjunct at once (see narrow_or_instanceof_union's
    // doc comment for why this can't be done by narrowing per-class and
    // merging afterward).
    let narrowed =
        narrow_or_instanceof_union(&current, &class_names, db, &ctx.template_param_names);
    set_narrowed(ctx, &vn, &current, narrowed, true);
    Some(vn)
}

/// Property-access counterpart of `collect_instanceof`, for the OR-disjunct
/// `$this->prop instanceof A || $this->prop instanceof B` idiom that the
/// variable-only `collect_instanceof` can't reach (`extract_var_name` fails
/// on a property receiver). Kept as its own narrower helper — used only by
/// `narrow_or_instanceof_true` — rather than folding into
/// `collect_instanceof`/`narrow_instanceof_disjuncts`: that function's
/// `Option<String>` result is also consumed by switch/match fallthrough
/// narrowing (`stmt/control_flow.rs`, `expr/conditional.rs`) via plain
/// `ctx.get_var`/`set_var`, which has no property equivalent.
fn collect_prop_instanceof(
    expr: &php_ast::owned::Expr,
    receiver: &mut Option<(String, String)>,
    class_names: &mut Vec<String>,
    db: &dyn MirDatabase,
    file: &str,
    self_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
) -> bool {
    match &expr.kind {
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            if let (Some((obj, prop)), Some(cn)) = (
                extract_prop_access(&b.left),
                extract_class_name(&b.right, self_fqcn, parent_fqcn),
            ) {
                let resolved = crate::db::resolve_name(db, file, &cn);
                match receiver {
                    None => {
                        *receiver = Some((obj, prop));
                        class_names.push(resolved);
                        true
                    }
                    Some((existing_obj, existing_prop))
                        if *existing_obj == obj && *existing_prop == prop =>
                    {
                        class_names.push(resolved);
                        true
                    }
                    _ => false, // different receiver — bail out
                }
            } else {
                false
            }
        }
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            collect_prop_instanceof(
                &b.left,
                receiver,
                class_names,
                db,
                file,
                self_fqcn,
                parent_fqcn,
            ) && collect_prop_instanceof(
                &b.right,
                receiver,
                class_names,
                db,
                file,
                self_fqcn,
                parent_fqcn,
            )
        }
        ExprKind::Parenthesized(inner) => collect_prop_instanceof(
            inner,
            receiver,
            class_names,
            db,
            file,
            self_fqcn,
            parent_fqcn,
        ),
        _ => false,
    }
}

/// For `$this->prop instanceof A || $this->prop instanceof B` (true branch):
/// narrow the property to `A|B`, mirroring `narrow_instanceof_disjuncts` but
/// for a property-access receiver. Returns `true` if it matched and applied
/// narrowing. See `collect_prop_instanceof`'s doc comment for why this is a
/// separate function rather than an extension of the shared one.
fn narrow_prop_instanceof_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> bool {
    if conditions.len() < 2 {
        return false;
    }
    let self_fqcn = ctx.self_fqcn.as_deref();
    let parent_fqcn = ctx.parent_fqcn.as_deref();

    let mut receiver: Option<(String, String)> = None;
    let mut class_names: Vec<String> = vec![];
    let all_ok = conditions.iter().all(|cond| {
        collect_prop_instanceof(
            cond,
            &mut receiver,
            &mut class_names,
            db,
            file,
            self_fqcn,
            parent_fqcn,
        )
    });

    if !all_ok || class_names.len() < 2 {
        return false;
    }
    let Some((obj_var, prop)) = receiver else {
        return false;
    };

    let current = resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
    let narrowed =
        narrow_or_instanceof_union(&current, &class_names, db, &ctx.template_param_names);
    apply_prop_narrowed(ctx, &obj_var, &prop, current, narrowed, true);
    true
}

/// Recognized single-argument type-check functions whose truthy narrowing
/// `narrow_from_type_fn` implements. Matches its match arms (excluding
/// `method_exists`/`property_exists`, which take two arguments and are
/// unrelated to the single-variable-disjunct shape this supports).
const NARROWING_TYPE_FNS: &[&str] = &[
    "is_string",
    "is_int",
    "is_integer",
    "is_long",
    "is_float",
    "is_double",
    "is_real",
    "is_bool",
    "is_null",
    "is_array",
    "array_is_list",
    "is_object",
    "is_callable",
    "is_scalar",
    "is_iterable",
    "is_countable",
    "is_resource",
    "is_numeric",
    "ctype_alpha",
    "ctype_alnum",
    "ctype_digit",
    "ctype_lower",
    "ctype_upper",
    "ctype_punct",
    "ctype_space",
    "ctype_xdigit",
    "ctype_print",
    "ctype_graph",
    "ctype_cntrl",
];

/// Extract `(fn_name, var_name)` from a single-argument type-check call
/// (`is_int($x)`) recognized by [`NARROWING_TYPE_FNS`]. Returns `None` for
/// anything else — a different function, more than one argument, or an
/// argument that isn't a plain variable.
fn extract_type_fn_check(expr: &php_ast::owned::Expr) -> Option<(&str, String)> {
    let ExprKind::FunctionCall(call) = &expr.kind else {
        return None;
    };
    let ExprKind::Identifier(name) = &call.name.kind else {
        return None;
    };
    let bare = name.as_ref().trim_start_matches('\\');
    let canonical = NARROWING_TYPE_FNS
        .iter()
        .find(|f| f.eq_ignore_ascii_case(bare))?;
    if call.args.len() != 1 {
        return None;
    }
    let var_name = extract_var_name(&call.args[0].value)?;
    Some((canonical, var_name))
}

/// Property-access counterpart of `extract_type_fn_check`, for
/// `is_int($this->prop)` — returns `(fn_name, obj_var, prop)`. Kept separate
/// (rather than folding into `extract_type_fn_check`) for the same reason
/// `collect_prop_instanceof` is kept separate from `collect_instanceof`: the
/// var-only extractor's result feeds `narrow_type_fn_disjuncts`'s
/// `Option<String>`, which switch-fallthrough narrowing consumes via plain
/// `ctx.get_var`/`set_var` — a property receiver has no such representation.
fn extract_type_fn_check_prop(expr: &php_ast::owned::Expr) -> Option<(&str, String, String)> {
    let ExprKind::FunctionCall(call) = &expr.kind else {
        return None;
    };
    let ExprKind::Identifier(name) = &call.name.kind else {
        return None;
    };
    let bare = name.as_ref().trim_start_matches('\\');
    let canonical = NARROWING_TYPE_FNS
        .iter()
        .find(|f| f.eq_ignore_ascii_case(bare))?;
    if call.args.len() != 1 {
        return None;
    }
    let (obj, prop) = extract_prop_access(&call.args[0].value)?;
    Some((canonical, obj, prop))
}

/// Narrow `$x` to the union of every `is_TYPE($x)` truthy-narrowing
/// collected from `conditions`, when EVERY condition is a recognized
/// single-argument type-check call on the SAME variable — the scalar-type
/// counterpart to [`narrow_instanceof_disjuncts`], used for the same
/// `match(true)`/`switch(true)` fallthrough shape (`is_int($x)`,
/// `is_string($x)`, …) that instanceof-only narrowing doesn't cover.
///
/// Returns the narrowed variable name on success; `None` when the shape
/// doesn't apply (mixed condition kinds, or checks on different variables) —
/// the caller should fall back to narrowing each condition individually.
pub(crate) fn narrow_type_fn_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
) -> Option<String> {
    if conditions.len() < 2 {
        return None;
    }
    let mut var_name: Option<String> = None;
    let mut fn_names: Vec<&str> = Vec::with_capacity(conditions.len());
    for cond in conditions {
        let (fn_name, vn) = extract_type_fn_check(cond)?;
        match &var_name {
            None => var_name = Some(vn),
            Some(existing) if *existing == vn => {}
            _ => return None, // different variable — bail out
        }
        fn_names.push(fn_name);
    }
    let vn = var_name?;
    let original = ctx.get_var(&vn);
    let mut union_ty = Type::empty();
    for fn_name in &fn_names {
        let mut scratch = ctx.branch();
        scratch.set_var(&vn, original.clone());
        narrow_from_type_fn(&mut scratch, fn_name, &vn, db, true);
        union_ty.merge_with(&scratch.get_var(&vn));
    }
    if !union_ty.is_empty() {
        ctx.set_var(&vn, union_ty);
    }
    Some(vn)
}

/// Property-access counterpart of `narrow_type_fn_disjuncts`, for the
/// `match(true)`/`switch(true)` fallthrough shape applied to `$this->prop`
/// (e.g. `is_int($this->prop), is_string($this->prop)`). Returns `true` when
/// it matched and applied narrowing — returns the narrowed `(obj_var, prop)`
/// receiver on success, the property-side equivalent of the var-side
/// function's `Option<String>` (see `extract_type_fn_check_prop`'s doc
/// comment for why a plain variable name can't represent this).
pub(crate) fn narrow_prop_type_fn_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(String, String)> {
    if conditions.len() < 2 {
        return None;
    }
    let mut receiver: Option<(String, String)> = None;
    let mut fn_names: Vec<&str> = Vec::with_capacity(conditions.len());
    for cond in conditions {
        let (fn_name, obj, prop) = extract_type_fn_check_prop(cond)?;
        match &receiver {
            None => receiver = Some((obj, prop)),
            Some((existing_obj, existing_prop))
                if *existing_obj == obj && *existing_prop == prop => {}
            _ => return None, // different receiver — bail out
        }
        fn_names.push(fn_name);
    }
    let (obj_var, prop) = receiver?;

    let original = resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
    let mut union_ty = Type::empty();
    for fn_name in &fn_names {
        let mut scratch = ctx.branch();
        scratch.set_prop_refined(&obj_var, &prop, original.clone());
        narrow_prop_from_type_fn(&mut scratch, fn_name, &obj_var, &prop, db, file, true);
        union_ty.merge_with(&resolve_prop_current_type(
            &scratch, &obj_var, &prop, db, file,
        ));
    }
    if !union_ty.is_empty() {
        apply_prop_narrowed(ctx, &obj_var, &prop, original, union_ty, true);
    }
    Some((obj_var, prop))
}

/// Extract the single variable a leaf disjunct condition narrows — either a
/// direct `$x instanceof A` or a recognized `is_TYPE($x)` call — without
/// applying any narrowing. Used to check every condition in a disjunct list
/// targets the same variable before [`narrow_mixed_disjuncts`] mixes the two
/// kinds together. Recurses into nested `||`/parens (like [`collect_instanceof`])
/// so a 3-way-or-more chain mixing instanceof and is_TYPE() leaves — e.g. `$x
/// instanceof A || is_string($x) || $x instanceof B` — still resolves to a
/// shared variable name here; [`narrow_mixed_disjuncts`] then narrows each
/// top-level condition via `narrow_from_condition`, which re-dispatches into
/// this same machinery for any nested disjunct.
fn single_leaf_disjunct_var(expr: &php_ast::owned::Expr) -> Option<String> {
    let expr = peel_parens(expr);
    match &expr.kind {
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => extract_var_name(&b.left),
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            let l = single_leaf_disjunct_var(&b.left)?;
            let r = single_leaf_disjunct_var(&b.right)?;
            (l == r).then_some(l)
        }
        _ => extract_type_fn_check(expr).map(|(_, vn)| vn),
    }
}

/// Narrow `$x` to the union of every disjunct's narrowing when `conditions`
/// mixes `instanceof` and `is_TYPE()` checks on the SAME variable (e.g. `$x
/// instanceof A || is_string($x)`) — the case neither
/// [`narrow_instanceof_disjuncts`] (which requires every disjunct to be an
/// `instanceof`) nor [`narrow_type_fn_disjuncts`] (which requires every
/// disjunct to be a type-check call) can handle alone. Each disjunct is
/// narrowed independently from the original type in a scratch branch, then
/// the results are unioned — the same technique `narrow_type_fn_disjuncts`
/// already uses, generalized to a mix of condition kinds via the top-level
/// [`narrow_from_condition`] dispatcher instead of a single specialized
/// narrowing function.
pub(crate) fn narrow_mixed_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<String> {
    if conditions.len() < 2 {
        return None;
    }
    let mut var_name: Option<String> = None;
    for cond in conditions {
        let vn = single_leaf_disjunct_var(cond)?;
        match &var_name {
            None => var_name = Some(vn),
            Some(existing) if *existing == vn => {}
            _ => return None, // different variable — bail out
        }
    }
    let vn = var_name?;
    let original = ctx.get_var(&vn);
    let mut union_ty = Type::empty();
    for cond in conditions {
        let mut scratch = ctx.branch();
        scratch.set_var(&vn, original.clone());
        narrow_from_condition(cond, &mut scratch, true, db, file);
        union_ty.merge_with(&scratch.get_var(&vn));
    }
    if !union_ty.is_empty() {
        ctx.set_var(&vn, union_ty);
    }
    Some(vn)
}

/// Property-access counterpart of `single_leaf_disjunct_var`, for
/// `$this->prop instanceof A` / `is_TYPE($this->prop)` leaf disjuncts.
fn single_leaf_disjunct_prop(expr: &php_ast::owned::Expr) -> Option<(String, String)> {
    let expr = peel_parens(expr);
    match &expr.kind {
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => extract_prop_access(&b.left),
        _ => extract_type_fn_check_prop(expr).map(|(_, obj, prop)| (obj, prop)),
    }
}

/// Property-access counterpart of `narrow_mixed_disjuncts`, for a mixed
/// `instanceof`/`is_TYPE()` OR-chain on `$this->prop` (e.g. `$this->prop
/// instanceof Foo || is_string($this->prop)`) — the property side has a
/// dedicated pure-instanceof (`narrow_prop_instanceof_disjuncts`) and
/// pure-type-fn (`narrow_prop_type_fn_disjuncts`) counterpart already, but no
/// mixed-kind counterpart, unlike the plain-variable case.
pub(crate) fn narrow_mixed_prop_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(String, String)> {
    if conditions.len() < 2 {
        return None;
    }
    let mut receiver: Option<(String, String)> = None;
    for cond in conditions {
        let (obj, prop) = single_leaf_disjunct_prop(cond)?;
        match &receiver {
            None => receiver = Some((obj, prop)),
            Some((existing_obj, existing_prop))
                if *existing_obj == obj && *existing_prop == prop => {}
            _ => return None, // different receiver — bail out
        }
    }
    let (obj_var, prop) = receiver?;
    let original = resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
    let mut union_ty = Type::empty();
    for cond in conditions {
        let mut scratch = ctx.branch();
        scratch.set_prop_refined(&obj_var, &prop, original.clone());
        narrow_from_condition(cond, &mut scratch, true, db, file);
        union_ty.merge_with(&resolve_prop_current_type(
            &scratch, &obj_var, &prop, db, file,
        ));
    }
    if !union_ty.is_empty() {
        apply_prop_narrowed(ctx, &obj_var, &prop, original, union_ty, true);
    }
    Some((obj_var, prop))
}

/// For `$x instanceof A || $x instanceof B` (true branch): narrow $x to A|B.
/// Handles OR chains recursively, e.g. `$x instanceof A || $x instanceof B || $x instanceof C`.
/// Also handles the scalar-type-check counterpart (`is_int($x) || is_string($x)`)
/// via [`narrow_type_fn_disjuncts`], and a mix of the two (`$x instanceof A ||
/// is_string($x)`) via [`narrow_mixed_disjuncts`], when the pure-instanceof
/// shape doesn't apply.
fn narrow_or_instanceof_true(
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    if narrow_instanceof_disjuncts(&[left, right], ctx, db, file).is_none()
        && narrow_type_fn_disjuncts(&[left, right], ctx, db).is_none()
        && !narrow_prop_instanceof_disjuncts(&[left, right], ctx, db, file)
        && narrow_prop_type_fn_disjuncts(&[left, right], ctx, db, file).is_none()
        && narrow_mixed_disjuncts(&[left, right], ctx, db, file).is_none()
    {
        narrow_mixed_prop_disjuncts(&[left, right], ctx, db, file);
    }
}

/// Apply short-circuit narrowing for isset() in || expressions (true branch).
///
/// Handles the PHP idiom: `!isset($x) || use($x)`
///
/// When the || operator's RHS is evaluated:
/// - If LHS is `!isset($x)`, then isset($x) must be TRUE in RHS
///   (because short-circuit: RHS only executes when LHS is false)
///
/// The narrowing is scoped to RHS analysis only and is restored afterward.
/// This ensures the if-body context isn't incorrectly narrowed.
fn narrow_or_isset_true(
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    // Pattern: !isset($x) || RHS
    // When RHS is evaluated via short-circuit, !isset($x) is false, so isset($x) is true
    if let ExprKind::UnaryPrefix(u) = &left.kind {
        if u.op == UnaryPrefixOp::BooleanNot {
            if let ExprKind::Isset(vars) = &u.operand.kind {
                // `!isset($x) || RHS` is true either because `$x` isn't set (RHS never
                // runs, nothing is narrowed) or because `$x` is set AND RHS is true.
                // The merged true-branch state is the union of those two paths, and a
                // union with the "nothing narrowed" path always collapses back to the
                // pre-condition state — so *every* narrowing effect of evaluating RHS
                // (not just to the isset()-checked vars) must be undone afterward, or
                // an unrelated variable RHS happens to narrow (e.g. `$y instanceof Foo`
                // in `!isset($x) || $y instanceof Foo`) would incorrectly leak into the
                // if-body on the path where `$x` was simply never set.
                let saved_vars = ctx.vars.clone();
                let saved_assigned = ctx.assigned_vars.clone();
                let saved_possibly_assigned = ctx.possibly_assigned_vars.clone();
                let saved_prop_refined = ctx.prop_refined.clone();
                let saved_diverges = ctx.diverges;
                let saved_class_exists_guards = ctx.class_exists_guards.clone();
                let saved_defined_guards = ctx.defined_guards.clone();
                let saved_function_exists_guards = ctx.function_exists_guards.clone();
                let saved_method_exists_guards = ctx.method_exists_guards.clone();
                let saved_extension_loaded_guards = ctx.extension_loaded_guards.clone();

                // Apply isset narrowing: remove null and mark as definitely assigned,
                // so RHS's own narrowing logic can see $x as set while it's analyzed.
                for var_expr in vars.iter() {
                    if let Some(var_name) = extract_var_name(var_expr) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                        std::sync::Arc::make_mut(&mut ctx.assigned_vars)
                            .insert(mir_types::Name::from(var_name.as_str()));
                    }
                }

                // Evaluate RHS with narrowed context
                narrow_from_condition(right, ctx, true, db, file);

                // Discard every narrowing effect of the above — RHS's narrowing (of $x
                // or any other variable/property) only holds on the path where $x was
                // set, not on the merged true-branch as a whole. This includes
                // `diverges`: a contradiction found while analyzing RHS in isolation
                // (e.g. an unrelated `instanceof` that can never hold) does not make
                // the whole condition unreachable, since the "$x unset" path is still
                // live.
                ctx.vars = saved_vars;
                ctx.assigned_vars = saved_assigned;
                ctx.possibly_assigned_vars = saved_possibly_assigned;
                ctx.prop_refined = saved_prop_refined;
                ctx.diverges = saved_diverges;
                ctx.class_exists_guards = saved_class_exists_guards;
                ctx.defined_guards = saved_defined_guards;
                ctx.function_exists_guards = saved_function_exists_guards;
                ctx.method_exists_guards = saved_method_exists_guards;
                ctx.extension_loaded_guards = saved_extension_loaded_guards;
            }
        }
    }
}

/// When `class_name` is a (possibly indirect) subclass/subinterface of
/// `atom_fqcn` and the atom being narrowed carries concrete `type_params`
/// (e.g. `Box<int>`), project those onto `class_name`'s own template params
/// instead of discarding them: `$b instanceof IntBox` on a `Box<int>`
/// receiver should narrow to `IntBox<int>`, not a bare unparameterized
/// `IntBox` that leaves a later `IntBox` method's own `@return T` unresolved
/// and unrelated to `Box`'s binding.
///
/// Handles the two shapes real code uses to relate a subclass's own template
/// params to its ancestor's: an explicit `@extends`/`@implements
/// Ancestor<...>` clause whose args are bare references to the subclass's
/// own template param names (identity or renamed passthrough), and the
/// simpler case where the subclass declares no such clause at all but has
/// the same template arity as the ancestor, which real-world code (and this
/// analyzer's own `class_template_params`) treats as an implicit,
/// unchanged passthrough. Anything else (arity mismatch, no relationship
/// found) falls back to no type params, same as before this projection
/// existed.
fn project_type_params_onto_subclass(
    db: &dyn MirDatabase,
    atom_fqcn: &str,
    atom_type_params: &[Type],
    class_name: &str,
) -> std::sync::Arc<[Type]> {
    let Some(class_own_tps) = crate::db::class_template_params(db, class_name) else {
        return mir_types::union::empty_type_params();
    };
    if class_own_tps.is_empty() {
        return mir_types::union::empty_type_params();
    }
    let Some(atom_own_tps) = crate::db::class_template_params(db, atom_fqcn) else {
        return mir_types::union::empty_type_params();
    };
    let here = crate::db::Fqcn::from_str(db, class_name);
    let Some(class) = crate::db::find_class_like(db, here) else {
        return mir_types::union::empty_type_params();
    };

    let explicit_args: Option<&[Type]> = if class
        .parent()
        .is_some_and(|p| p.as_ref().eq_ignore_ascii_case(atom_fqcn))
    {
        Some(class.extends_type_args())
    } else {
        class
            .implements_type_args()
            .iter()
            .chain(class.interface_extends_type_args())
            .find(|(iface, _)| iface.as_ref().eq_ignore_ascii_case(atom_fqcn))
            .map(|(_, args)| args.as_slice())
    };

    let mut result = vec![Type::mixed(); class_own_tps.len()];
    let mut any_bound = false;

    if let Some(args) = explicit_args.filter(|a| !a.is_empty()) {
        for (idx, given_ty) in atom_type_params.iter().enumerate() {
            let Some(arg_expr) = args.get(idx) else {
                continue;
            };
            let Some(bare_name) = bare_named_type(arg_expr) else {
                continue;
            };
            if let Some(pos) = class_own_tps
                .iter()
                .position(|tp| tp.name.as_str() == bare_name)
            {
                result[pos] = given_ty.clone();
                any_bound = true;
            }
        }
    } else if class_own_tps.len() == atom_own_tps.len() {
        result = atom_type_params.to_vec();
        any_bound = !result.is_empty();
    }

    if any_bound {
        mir_types::union::vec_to_type_params(result)
    } else {
        mir_types::union::empty_type_params()
    }
}

/// A `Type` consisting of exactly one bare, unqualified named-type atom
/// (e.g. a docblock's `@extends Box<U>` argument referencing a template
/// param by name) — as opposed to a real, concrete class reference or a
/// compound type. Returns that name, if so.
fn bare_named_type(ty: &Type) -> Option<&str> {
    if ty.types.len() != 1 {
        return None;
    }
    match &ty.types[0] {
        Atomic::TNamedObject { fqcn, type_params }
            if type_params.is_empty() && !fqcn.contains('\\') =>
        {
            Some(fqcn.as_ref())
        }
        // The collector stores `@extends Box<U>` args template-aware, so a
        // template-param reference arrives as a proper TTemplateParam atom.
        Atomic::TTemplateParam { name, .. } => Some(name.as_ref()),
        _ => None,
    }
}

fn narrow_instanceof_preserving_subtypes(
    current: &Type,
    class_name: &str,
    db: &dyn MirDatabase,
    template_param_names: &rustc_hash::FxHashSet<mir_types::Name>,
) -> Type {
    let narrowed_ty = Atomic::TNamedObject {
        fqcn: class_name.into(),
        type_params: mir_types::union::empty_type_params(),
    };

    if current.is_empty() || current.is_mixed_not_template() {
        return Type::single(narrowed_ty);
    }

    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;

    for atomic in &current.types {
        match atomic {
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if named_object_matches_instanceof(fqcn, class_name, db) =>
            {
                result.add_type(atomic.clone());
            }
            // Handle template parameters: if a bare unqualified name matches a template param,
            // intersect it with the checked class rather than replacing it — the value is
            // still guaranteed to be a T (e.g. for a later `@return T`), just now also
            // known to be an instance of `class_name`.
            Atomic::TNamedObject { fqcn, type_params }
                if type_params.is_empty()
                    && !fqcn.contains('\\')
                    && template_param_names.contains(fqcn) =>
            {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            // Handle TTemplateParam: intersect it with the instanceof check class instead
            // of discarding the template binding (see comment above).
            Atomic::TTemplateParam { .. } => {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            Atomic::TObject | Atomic::TMixed => result.add_type(narrowed_ty.clone()),
            // `$x instanceof C` on an `A&B`-typed value adds C to the
            // intersection rather than replacing it — the value is still
            // guaranteed to be an A and a B, so dropping them here would
            // falsely reject valid uses of the original intersection.
            Atomic::TIntersection { parts } => {
                let already_covered = parts.iter().any(|p| {
                    p.types.iter().any(|a| {
                        matches!(a, Atomic::TNamedObject { fqcn, .. }
                            if named_object_matches_instanceof(fqcn, class_name, db))
                    })
                });
                if already_covered {
                    result.add_type(atomic.clone());
                } else {
                    let mut new_parts: Vec<Type> = parts.iter().cloned().collect();
                    new_parts.push(Type::single(narrowed_ty.clone()));
                    result.add_type(Atomic::TIntersection {
                        parts: std::sync::Arc::from(new_parts),
                    });
                }
            }
            // `class_name` is a (possibly indirect) subtype of the atom's own class
            // AND the atom carries concrete type params (e.g. `Box<int>`
            // narrowed by `instanceof IntBox`) — project them onto
            // `class_name`'s own template params rather than discarding them.
            Atomic::TNamedObject { fqcn, type_params }
                if !type_params.is_empty()
                    && named_object_matches_instanceof(class_name, fqcn, db) =>
            {
                let projected =
                    project_type_params_onto_subclass(db, fqcn, type_params, class_name);
                result.add_type(Atomic::TNamedObject {
                    fqcn: class_name.into(),
                    type_params: projected,
                });
            }
            // `class_name` is a (possibly indirect) subtype of the atom's own class
            // — e.g. atom is the `Foo` interface and class_name is `A implements
            // Foo` — so the instanceof check's result subsumes and is strictly
            // more specific than what's already known; replace outright rather
            // than forming a redundant `Foo&A` intersection.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if named_object_matches_instanceof(class_name, fqcn, db) =>
            {
                result.add_type(narrowed_ty.clone());
            }
            // A named object unrelated to `class_name` by inheritance in either
            // direction (e.g. two interfaces neither of which extends the other,
            // as in `$x instanceof A && $x instanceof B`) must not be silently
            // discarded — the instanceof check proved the value ALSO satisfies
            // class_name. Form an intersection when that's actually possible
            // (at least one side is an interface, so a single object can
            // implement both); otherwise the atom's own class and class_name are
            // both concrete classes, which PHP's single inheritance makes
            // mutually exclusive, so the atom is provably impossible here and is
            // correctly dropped.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if classes_can_coexist(fqcn, class_name, db) =>
            {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            // A `Closure(...): R`-typed atom (its own dedicated atomic, not a
            // TNamedObject) genuinely IS an instance of `Closure` at runtime —
            // keep it as-is rather than falling through to the catch-all drop,
            // which would make `$x instanceof Closure` on a `Closure(): T`-typed
            // value look provably impossible.
            Atomic::TClosure { .. } if class_name.eq_ignore_ascii_case("Closure") => {
                result.add_type(atomic.clone());
            }
            _ => {}
        }
    }

    // Unlike the early-return above (truly unconstrained `mixed`/empty `current`),
    // reaching here with an empty `result` means `current` had at least one real
    // atom and NONE of them survived narrowing — every atom was proven
    // incompatible with `class_name` (e.g. two unrelated `final` classes).
    // Propagate the emptiness instead of resetting to a bare `narrowed_ty`, so
    // the caller's `mark_diverges` can correctly flag the branch as unreachable
    // instead of silently treating a provably-impossible instanceof as if
    // nothing were known about the value.
    result
}

/// Like [`narrow_instanceof_preserving_subtypes`], but for an OR-chain of
/// `instanceof` checks against several classes at once (`$x instanceof A ||
/// $x instanceof B`) — narrowing per-class-then-merging (as opposed to
/// per-atom-across-all-classes) double-counts `TIntersection` union members
/// unrelated to any single disjunct: `(A&B)|C|D` narrowed by `instanceof C ||
/// instanceof D` would otherwise produce two separate `A&B&C`/`A&B&D`
/// members instead of one `A&B&(C|D)` member, bloating the displayed type
/// and hiding it from later, more precise checks.
fn narrow_or_instanceof_union(
    current: &Type,
    class_names: &[String],
    db: &dyn MirDatabase,
    template_param_names: &rustc_hash::FxHashSet<mir_types::Name>,
) -> Type {
    let class_atom = |cn: &str| Atomic::TNamedObject {
        fqcn: cn.into(),
        type_params: mir_types::union::empty_type_params(),
    };

    if current.is_empty() || current.is_mixed_not_template() {
        let mut out = Type::empty();
        for cn in class_names {
            out.add_type(class_atom(cn));
        }
        return out;
    }

    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;

    for atomic in &current.types {
        match atomic {
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if class_names
                    .iter()
                    .any(|cn| named_object_matches_instanceof(fqcn, cn, db)) =>
            {
                result.add_type(atomic.clone());
            }
            // As in narrow_instanceof_preserving_subtypes, keep the template atom by
            // intersecting it with the union of checked classes rather than replacing it.
            Atomic::TNamedObject { fqcn, type_params }
                if type_params.is_empty()
                    && !fqcn.contains('\\')
                    && template_param_names.contains(fqcn) =>
            {
                let mut classes = Type::empty();
                for cn in class_names {
                    classes.add_type(class_atom(cn));
                }
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![Type::single(atomic.clone()), classes]),
                });
            }
            Atomic::TTemplateParam { .. } => {
                let mut classes = Type::empty();
                for cn in class_names {
                    classes.add_type(class_atom(cn));
                }
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![Type::single(atomic.clone()), classes]),
                });
            }
            Atomic::TObject | Atomic::TMixed => {
                for cn in class_names {
                    result.add_type(class_atom(cn));
                }
            }
            // As in narrow_instanceof_preserving_subtypes, a Closure(...): R atom
            // genuinely IS an instance of Closure at runtime — keep it when one
            // of the OR-chain's classes is Closure, instead of falling through
            // to the catch-all drop.
            Atomic::TClosure { .. }
                if class_names
                    .iter()
                    .any(|cn| cn.eq_ignore_ascii_case("Closure")) =>
            {
                result.add_type(atomic.clone());
            }
            Atomic::TIntersection { parts } => {
                let mut remaining = Type::empty();
                for cn in class_names {
                    let already_covered = parts.iter().any(|p| {
                        p.types.iter().any(|a| {
                            matches!(a, Atomic::TNamedObject { fqcn, .. }
                                if named_object_matches_instanceof(fqcn, cn, db))
                        })
                    });
                    if !already_covered {
                        remaining.add_type(class_atom(cn));
                    }
                }
                if remaining.is_empty() {
                    result.add_type(atomic.clone());
                } else {
                    let mut new_parts: Vec<Type> = parts.iter().cloned().collect();
                    new_parts.push(remaining);
                    result.add_type(Atomic::TIntersection {
                        parts: std::sync::Arc::from(new_parts),
                    });
                }
            }
            // Some disjunct(s) are a (possibly indirect) subtype of the atom's own
            // class AND the atom carries concrete type params — project them
            // onto each subsuming disjunct's own template params rather than
            // discarding them, mirroring narrow_instanceof_preserving_subtypes.
            Atomic::TNamedObject { fqcn, type_params }
                if !type_params.is_empty()
                    && class_names
                        .iter()
                        .any(|cn| named_object_matches_instanceof(cn, fqcn, db)) =>
            {
                for cn in class_names {
                    if named_object_matches_instanceof(cn, fqcn, db) {
                        let projected =
                            project_type_params_onto_subclass(db, fqcn, type_params, cn);
                        result.add_type(Atomic::TNamedObject {
                            fqcn: cn.as_str().into(),
                            type_params: projected,
                        });
                    }
                }
            }
            // Some disjunct(s) are a (possibly indirect) subtype of the atom's own
            // class — e.g. atom is `Foo` and one label checks `instanceof A` where
            // `A implements Foo` — so the instanceof result subsumes and is
            // strictly more specific; narrow to just the subsuming disjunct(s)
            // rather than forming a redundant `Foo&A` intersection.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if class_names
                    .iter()
                    .any(|cn| named_object_matches_instanceof(cn, fqcn, db)) =>
            {
                for cn in class_names {
                    if named_object_matches_instanceof(cn, fqcn, db) {
                        result.add_type(class_atom(cn));
                    }
                }
            }
            // A named object matching none of the disjuncts by inheritance in
            // either direction must not be silently discarded — the
            // (already-true) instanceof check proved the value ALSO satisfies
            // one of class_names. Intersect with only the disjuncts that could
            // actually coexist with this atom (at least one side an interface);
            // a disjunct that's a concrete class unrelated to this atom's own
            // concrete class is impossible under PHP's single inheritance and is
            // dropped instead. Mirrors the equivalent fix in
            // narrow_instanceof_preserving_subtypes for the single-class case.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn } => {
                let mut classes = Type::empty();
                for cn in class_names {
                    if classes_can_coexist(fqcn, cn, db) {
                        classes.add_type(class_atom(cn));
                    }
                }
                if !classes.is_empty() {
                    result.add_type(Atomic::TIntersection {
                        parts: std::sync::Arc::from(vec![Type::single(atomic.clone()), classes]),
                    });
                }
            }
            _ => {}
        }
    }

    // Unlike the early-return above (truly unconstrained `mixed`/empty
    // `current`), reaching here with an empty `result` means `current` had
    // at least one real atom and NONE of them survived narrowing against any
    // disjunct — every atom was proven incompatible with every disjunct
    // (e.g. unrelated `final` classes). Propagate the emptiness instead of
    // resetting to the disjuncts' bare union, mirroring
    // `narrow_instanceof_preserving_subtypes`, so the caller can correctly
    // flag the branch as unreachable instead of silently widening a
    // provably-impossible `instanceof` chain to `A|B`.
    result
}

/// Whether a value could simultaneously be (a subtype of) both `a` and `b` —
/// true when either is an interface (a class can implement any number of
/// interfaces), false when both are concrete classes, which PHP's single
/// inheritance makes mutually exclusive unless one already extends the other
/// (checked separately by the caller via `named_object_matches_instanceof`).
fn classes_can_coexist(a: &str, b: &str, db: &dyn MirDatabase) -> bool {
    crate::db::class_kind(db, a).is_some_and(|k| k.is_interface)
        || crate::db::class_kind(db, b).is_some_and(|k| k.is_interface)
}

fn filter_out_instanceof_match(current: &Type, class_name: &str, db: &dyn MirDatabase) -> Type {
    current.filter(|t| match t {
        Atomic::TNamedObject { fqcn, .. }
        | Atomic::TSelf { fqcn }
        | Atomic::TStaticObject { fqcn }
        | Atomic::TParent { fqcn } => !named_object_matches_instanceof(fqcn, class_name, db),
        // A Closure(...): R atom genuinely IS an instance of Closure at
        // runtime, so it's excluded by the false branch of `instanceof Closure`
        // just like a TNamedObject would be.
        Atomic::TClosure { .. } => !class_name.eq_ignore_ascii_case("Closure"),
        // A&B is provably excluded by `!($x instanceof C)` when EITHER part
        // alone would satisfy it — a value that's simultaneously an A and a B
        // is also a C the moment either A or B extends/implements C, so the
        // whole intersection can't survive the negation, not just its own
        // (nonexistent) direct name.
        Atomic::TIntersection { parts } => !parts.iter().any(|part| {
            part.types.iter().any(|inner| match inner {
                Atomic::TNamedObject { fqcn, .. }
                | Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } => named_object_matches_instanceof(fqcn, class_name, db),
                _ => false,
            })
        }),
        _ => true,
    })
}

fn named_object_matches_instanceof(fqcn: &str, class_name: &str, db: &dyn MirDatabase) -> bool {
    fqcn == class_name || crate::db::extends_or_implements(db, fqcn, class_name)
}

/// Partition `current`'s atoms for the `allow_string: true` true-branch of
/// `is_a($x, $class_name, true)`. A `class-string<C>` atom is dropped only
/// when `C` is provably unrelated to `class_name` in both directions AND
/// the two can't coexist on a single class (mirrors the object-atom
/// coexistence check in `narrow_instanceof_preserving_subtypes` above,
/// via `classes_can_coexist`) — a class-string naming an interface, or a
/// concrete class unrelated to `class_name` where `class_name` itself
/// names an interface, could still describe a subtype that also satisfies
/// `class_name`, so it isn't provably excluded. Any other string atom is
/// kept as-is (it might name `class_name` at runtime; there's nothing more
/// precise to narrow it to). The second element of the tuple is every
/// non-string atom, handed back separately so the caller can narrow it via
/// `instanceof` semantics.
fn partition_is_a_string_like(
    current: &Type,
    class_name: &str,
    db: &dyn MirDatabase,
) -> (Type, Type) {
    let mut string_part = Type::empty();
    string_part.possibly_undefined = current.possibly_undefined;
    string_part.from_docblock = current.from_docblock;
    let mut obj_part = Type::empty();
    for atom in &current.types {
        if let Atomic::TClassString(Some(name)) = atom {
            if named_object_matches_instanceof(name, class_name, db)
                || classes_can_coexist(name, class_name, db)
            {
                string_part.add_type(atom.clone());
            }
        } else if atom.is_string() {
            string_part.add_type(atom.clone());
        } else {
            obj_part.add_type(atom.clone());
        }
    }
    (string_part, obj_part)
}

/// `filter_out_instanceof_match`, extended for the `allow_string: true`
/// false-branch of `is_a()`: a `class-string<C>` atom provably matching
/// `class_name` is also excluded (mirrors the object-atom exclusion above —
/// `is_a()` being false rules out that specific class-string just as surely
/// as it rules out that specific object class).
fn filter_out_is_a_string_match(current: &Type, class_name: &str, db: &dyn MirDatabase) -> Type {
    filter_out_instanceof_match(current, class_name, db).filter(|t| {
        !matches!(t, Atomic::TClassString(Some(name)) if named_object_matches_instanceof(name, class_name, db))
    })
}

/// Narrow `current` for the true branch of `is_subclass_of($obj, 'ClassName')`.
///
/// Unlike `instanceof` / `is_a`, `is_subclass_of` requires a *strict* subclass:
/// the exact class itself is excluded. Atoms that are only the named class (not a
/// descendant) are dropped. Mixed/TObject are narrowed to the named class as the
/// best approximation (a value satisfying `is_subclass_of` must be some subclass,
/// and the named class is the tightest bound we can express).
fn narrow_strict_subclass_of(
    current: &Type,
    class_name: &str,
    db: &dyn MirDatabase,
    template_param_names: &rustc_hash::FxHashSet<mir_types::Name>,
) -> Type {
    let narrowed_ty = Atomic::TNamedObject {
        fqcn: class_name.into(),
        type_params: mir_types::union::empty_type_params(),
    };

    if current.is_empty() || current.is_mixed_not_template() {
        return Type::single(narrowed_ty);
    }

    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;

    for atomic in &current.types {
        match atomic {
            // Strict subclass: keep only atoms that extend/implement without being the class itself.
            Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn }
                if crate::db::extends_or_implements(db, fqcn.as_ref(), class_name)
                    && fqcn.as_ref() != class_name =>
            {
                result.add_type(atomic.clone());
            }
            // Template parameter — intersect with the named class rather than replacing it,
            // so the value is still known to be a T as well as a strict subclass of it.
            Atomic::TNamedObject { fqcn, type_params }
                if type_params.is_empty()
                    && !fqcn.contains('\\')
                    && template_param_names.contains(fqcn) =>
            {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            Atomic::TTemplateParam { .. } => {
                result.add_type(Atomic::TIntersection {
                    parts: std::sync::Arc::from(vec![
                        Type::single(atomic.clone()),
                        Type::single(narrowed_ty.clone()),
                    ]),
                });
            }
            Atomic::TObject | Atomic::TMixed => result.add_type(narrowed_ty.clone()),
            // `is_subclass_of($x, class_name)` on an `A&B`-typed value adds
            // class_name to the intersection rather than discarding it —
            // mirrors narrow_instanceof_preserving_subtypes's TIntersection
            // handling above.
            Atomic::TIntersection { parts } => {
                let already_covered = parts.iter().any(|p| {
                    p.types.iter().any(|a| {
                        matches!(a, Atomic::TNamedObject { fqcn, .. }
                            if crate::db::extends_or_implements(db, fqcn.as_ref(), class_name)
                                && fqcn.as_ref() != class_name)
                    })
                });
                if already_covered {
                    result.add_type(atomic.clone());
                } else {
                    let mut new_parts: Vec<Type> = parts.iter().cloned().collect();
                    new_parts.push(Type::single(narrowed_ty.clone()));
                    result.add_type(Atomic::TIntersection {
                        parts: std::sync::Arc::from(new_parts),
                    });
                }
            }
            _ => {}
        }
    }

    result
    // Note: no fallback to Type::single(narrowed_ty) when result is empty — if the
    // current type contains no known subclasses of the named class, the narrowing
    // returns empty and the caller should NOT mark diverges (is_subclass_of may still
    // be false at runtime for the exact class, which is valid).
}

/// Returns true if `expr` is the boolean literal `true`.
fn is_truthy_bool_literal(expr: &php_ast::owned::Expr) -> bool {
    matches!(expr.kind, php_ast::owned::ExprKind::Bool(true))
}

/// Narrow from a call compared against the `false` literal — the idiomatic
/// way to interpret `strpos()`/`array_search()`'s `int|string|false` result,
/// since a loose truthy check misfires on a match at offset/key 0. `is_false`
/// is whether `expr === false` holds in this branch (so `!is_false` means
/// the call proved a match — a substring was found, or the needle is
/// present in the haystack).
/// True when `expr` is a non-empty string literal, or a variable/property
/// already narrowed to one — shared by `str_contains()`/`str_starts_with()`/
/// `str_ends_with()` and the `strpos()`-family false-comparable narrowing,
/// both of which only narrow their haystack when the needle is provably
/// non-empty (an empty needle is trivially "found" at offset 0).
fn expr_is_nonempty_string_literal(
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
            None => false,
        },
    }
}

fn narrow_from_false_comparable_call(
    expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    is_false: bool,
) {
    let ExprKind::FunctionCall(call) = &expr.kind else {
        return;
    };
    let fn_name_opt: Option<&str> = match &call.name.kind {
        ExprKind::Identifier(name) => Some(name.as_ref()),
        _ => None,
    };
    let Some(fn_name) = fn_name_opt else {
        return;
    };
    let bare = fn_name.trim_start_matches('\\');
    if matches!(
        bare.to_ascii_lowercase().as_str(),
        "strpos"
            | "stripos"
            | "strrpos"
            | "strripos"
            | "mb_strpos"
            | "mb_stripos"
            | "mb_strrpos"
            | "mb_strripos"
    ) {
        // Found (result != false) proves the haystack is non-empty, mirroring
        // str_contains()'s true-branch narrowing — only sound for a non-empty
        // literal needle (an empty needle is "found" at offset 0 vacuously).
        if !is_false {
            if let (Some(haystack_arg), Some(needle_arg)) = (call.args.first(), call.args.get(1)) {
                let needle_non_empty =
                    expr_is_nonempty_string_literal(&needle_arg.value, ctx, db, file);
                if needle_non_empty {
                    match ScalarArgTarget::extract(&haystack_arg.value) {
                        Some(ScalarArgTarget::Var(var_name)) => {
                            let current = ctx.get_var(&var_name);
                            if !current.is_mixed() {
                                let narrowed = narrow_string_to_non_empty(&current);
                                if narrowed != current {
                                    ctx.set_var(&var_name, narrowed);
                                }
                            }
                        }
                        Some(ScalarArgTarget::Prop(obj, prop)) => {
                            let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                            if !current.is_mixed() {
                                let narrowed = narrow_string_to_non_empty(&current);
                                apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false);
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    } else if bare.eq_ignore_ascii_case("array_search") {
        // array_search($needle, $haystack) !== false / === false — same
        // haystack-literal narrowing as in_array(), keyed off the same
        // strict/loose-safety rule (see in_array_loose_narrowing_is_safe).
        let strict = call
            .args
            .get(2)
            .map(|a| is_truthy_bool_literal(&a.value))
            .unwrap_or(false);
        if let (Some(needle_arg), Some(haystack_arg)) = (call.args.first(), call.args.get(1)) {
            if let Some(target) = ScalarArgTarget::extract(&needle_arg.value) {
                if let Some(haystack_ty) = extract_haystack_type(&haystack_arg.value, ctx) {
                    let current = match &target {
                        ScalarArgTarget::Var(name) => ctx.get_var(name),
                        ScalarArgTarget::Prop(obj, prop) => {
                            resolve_prop_current_type(ctx, obj, prop, db, file)
                        }
                    };
                    let loose_safe =
                        strict || in_array_loose_narrowing_is_safe(&current, &haystack_ty);
                    if !current.is_mixed() && loose_safe {
                        let narrowed = if !is_false {
                            // Found: keep only atoms that could match a haystack value.
                            let narrowed = narrow_to_haystack_values(&current, &haystack_ty);
                            (!narrowed.is_empty() && narrowed != current).then_some(narrowed)
                        } else {
                            // Not found: safe only when current is a finite literal
                            // union — remove the matched haystack values.
                            let all_literals = !current.types.is_empty()
                                && current.types.iter().all(|a| {
                                    matches!(a, Atomic::TLiteralString(_) | Atomic::TLiteralInt(_))
                                });
                            all_literals
                                .then(|| {
                                    current.filter(|a| !haystack_ty.types.iter().any(|h| h == a))
                                })
                                .filter(|narrowed| !narrowed.is_empty() && *narrowed != current)
                        };
                        if let Some(narrowed) = narrowed {
                            match target {
                                ScalarArgTarget::Var(name) => ctx.set_var(&name, narrowed),
                                ScalarArgTarget::Prop(obj, prop) => {
                                    apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, false)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Apply a pre-computed narrowed type to a variable.
///
/// If `mark_diverges` is true and the narrowed type is empty (the current type
/// can never satisfy the constraint), the branch is marked unreachable.
fn set_narrowed(
    ctx: &mut FlowState,
    name: &str,
    current: &Type,
    narrowed: Type,
    mark_diverges: bool,
) {
    if !narrowed.is_empty() {
        ctx.set_var(name, narrowed);
    } else if mark_diverges && !current.is_empty() && !current.is_mixed() {
        ctx.diverges = true;
    }
}

/// Resolve the current type of `$obj_var->prop`: an existing flow-state
/// refinement if one is already tracked, else the declared type looked up
/// through the object variable's own type (including `self`/`static`, and
/// falling back to `self_fqcn` for `$this`).
pub(crate) fn resolve_prop_current_type(
    ctx: &FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
) -> Type {
    if let Some(refined) = ctx.get_prop_refined(obj_var, prop) {
        return refined.clone();
    }
    // Resolve through the object variable's type
    let obj_ty = ctx.get_var(obj_var);
    let mut prop_ty = mir_types::Type::mixed();
    'outer: for atomic in &obj_ty.types {
        if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
            let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
            // Try to find the property in the class chain
            if let Some((_, p_def)) = crate::db::find_property_in_chain(db, here, prop) {
                if let Some(ty) = p_def.ty.as_deref() {
                    prop_ty = ty.clone();
                    break 'outer;
                }
            }
        } else if let mir_types::Atomic::TSelf { fqcn }
        | mir_types::Atomic::TStaticObject { fqcn }
        | mir_types::Atomic::TParent { fqcn } = atomic
        {
            let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
            if let Some((_, p_def)) = crate::db::find_property_in_chain(db, here, prop) {
                if let Some(ty) = p_def.ty.as_deref() {
                    prop_ty = ty.clone();
                    break 'outer;
                }
            }
        }
    }
    // Also try self_fqcn if obj_var is "this"
    if prop_ty.is_mixed() && obj_var == "this" {
        if let Some(fqcn) = ctx.self_fqcn.as_ref() {
            let resolved = crate::db::resolve_name(db, file, fqcn.as_ref());
            let here = crate::db::Fqcn::from_str(db, &resolved);
            if let Some((_, p_def)) = crate::db::find_property_in_chain(db, here, prop) {
                if let Some(ty) = p_def.ty.as_deref() {
                    prop_ty = ty.clone();
                }
            }
        }
    }
    prop_ty
}

/// Narrow a property access `$obj->prop` by a null check.
/// PHP 8 reads a plain `->` access on a null receiver as a warning, not a
/// fatal error, evaluating to `null` just like `?->` would — so when the
/// receiver's type actually admits null, `$obj->prop` carries the exact same
/// null-source ambiguity as the nullsafe form and is narrowed identically
/// (see `narrow_nullsafe_prop_null`). When the receiver's type provably
/// excludes null, that ambiguity doesn't exist and a property-null
/// contradiction is a genuine contradiction, so divergence is still marked.
fn narrow_prop_null(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
) {
    if !ctx.get_var(obj_var).is_nullable() {
        narrow_prop_null_with_divergence(ctx, obj_var, prop, db, file, is_null, true);
        return;
    }
    narrow_nullsafe_prop_null(ctx, obj_var, prop, db, file, is_null);
}

fn narrow_prop_null_with_divergence(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
    mark_diverges: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);

    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// Narrow a nullsafe property access (`$obj?->prop`) by a null check. Also
/// used by the plain (`->`) form via `narrow_prop_null` — both read as
/// `null` when EITHER the receiver is null OR the property's own value is
/// null, so:
/// - the `is_null=true` direction must never mark divergence: proving the
///   property's own declared type excludes null doesn't rule out the
///   receiver-null source.
/// - the `is_null=false` direction additionally proves the receiver itself
///   is non-null, since a null receiver could only ever produce `null` here.
fn narrow_nullsafe_prop_null(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
) {
    narrow_prop_null_with_divergence(ctx, obj_var, prop, db, file, is_null, !is_null);
    if !is_null {
        narrow_var_null(ctx, obj_var, false);
    }
}

/// Narrow a static property access `self::$prop`/`Class::$prop` by a null
/// check. `prop_refined` is keyed by FQCN here instead of a receiver
/// variable name — a FQCN string can never collide with a real PHP variable.
/// Resolve the current type of `self::$prop`/`static::$prop`/`Class::$prop`:
/// an existing flow-state refinement if one is already tracked, else the
/// declared type looked up through the class hierarchy. Static-property
/// counterpart of `resolve_prop_current_type`.
fn resolve_static_prop_current_type(
    ctx: &FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
) -> Type {
    if let Some(refined) = ctx.get_prop_refined(fqcn, prop) {
        return refined.clone();
    }
    let here = crate::db::Fqcn::from_str(db, fqcn);
    crate::db::find_property_in_chain(db, here, prop)
        .and_then(|(_, p)| p.ty.as_deref().cloned())
        .unwrap_or_else(mir_types::Type::mixed)
}

fn narrow_static_prop_null(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    is_null: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);

    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
}

/// Loose-null counterpart of `narrow_static_prop_null`, for
/// `self::$prop == null` / `!= null` (and `static::$prop`/`Class::$prop`).
/// Loose semantics differ from strict: `== null` is also true for any other
/// falsy value (not just `null` itself), so the true direction only narrows
/// to falsy — same reasoning as `narrow_var_loose_null`/`narrow_prop_loose_null`.
/// The false direction stays exactly as sound as the strict form.
fn narrow_static_prop_loose_null(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    is_null: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_falsy()
    } else {
        current.remove_null()
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
}

/// Static-property counterpart of `narrow_prop_literal_string`, for
/// `self::$prop === 'literal'` / `static::$prop === 'literal'` /
/// `Class::$prop === 'literal'`.
fn narrow_static_prop_literal_string(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    value: &str,
    is_value: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let narrowed = literal_string_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_literal_int`, for
/// `self::$prop === 42` / `static::$prop === 42` / `Class::$prop === 42`.
fn narrow_static_prop_literal_int(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    value: i64,
    is_value: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let narrowed = literal_int_narrow_type(&current, value, is_value);
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_bool`, for
/// `self::$prop === true` / `static::$prop === false` / `Class::$prop === true`.
fn narrow_static_prop_bool(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    value: bool,
    is_value: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let narrowed = bool_narrow_type(&current, value, is_value);
    // mark_diverges=false: matches narrow_var_bool/narrow_prop_bool's rationale
    // — a separate contradiction pass already owns flagging an always-true/false compare.
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
}

/// Narrow a static property's type when `self::$prop instanceof ClassName` /
/// `static::$prop instanceof ClassName` is proven true or false.
fn narrow_static_prop_instanceof(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    class_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    let current = if let Some(refined) = ctx.get_prop_refined(fqcn, prop) {
        refined.clone()
    } else {
        let here = crate::db::Fqcn::from_str(db, fqcn);
        crate::db::find_property_in_chain(db, here, prop)
            .and_then(|(_, p)| p.ty.as_deref().cloned())
            .unwrap_or_else(mir_types::Type::mixed)
    };

    if current.is_mixed_not_template() {
        return;
    }
    let narrowed = if is_true {
        narrow_instanceof_preserving_subtypes(&current, class_name, db, &ctx.template_param_names)
    } else {
        filter_out_instanceof_match(&current, class_name, db)
    };
    if !narrowed.is_empty() {
        if narrowed != current {
            ctx.set_prop_refined(fqcn, prop, narrowed);
        }
    } else if !current.is_empty() && !current.is_mixed() {
        ctx.diverges = true;
    }
}

/// Applies a narrowed property type computed from `current`, mirroring
/// `set_narrowed`'s variable-side semantics for the property-refinement store.
fn apply_prop_narrowed(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    current: Type,
    narrowed: Type,
    mark_diverges: bool,
) {
    if !narrowed.is_empty() {
        if narrowed != current {
            ctx.set_prop_refined(obj_var, prop, narrowed);
        }
    } else if mark_diverges && !current.is_empty() && !current.is_mixed() {
        ctx.diverges = true;
    }
}

/// Property-access counterpart of the `$arr === []`/`$arr !== []` (and loose
/// `==`/`!=`) var-based array-emptiness narrowing above, for `$this->prop`.
/// `mark_diverges=false` matches the var-side behavior, which also leaves an
/// empty narrowing result untouched instead of flagging a contradiction.
fn narrow_prop_array_empty(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_empty: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    let narrowed = if is_empty {
        current.narrow_to_empty_collection()
    } else {
        current.narrow_to_non_empty_collection()
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

/// Static-property counterpart of `narrow_prop_array_empty`, for
/// `self::$prop === []`/`!==`/`==`/`!=` (and `static::$prop`/`Class::$prop`).
fn narrow_static_prop_array_empty(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    is_empty: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    let narrowed = if is_empty {
        current.narrow_to_empty_collection()
    } else {
        current.narrow_to_non_empty_collection()
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
}

fn narrow_prop_instanceof(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    class_name: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed_not_template() {
        return;
    }
    let narrowed = if is_true {
        narrow_instanceof_preserving_subtypes(&current, class_name, db, &ctx.template_param_names)
    } else {
        filter_out_instanceof_match(&current, class_name, db)
    };
    // `!($obj->prop instanceof X)` is also true whenever $obj itself is null
    // (`null instanceof X` is always false), so a nullable receiver means an
    // empty false-branch narrowing isn't a real contradiction — same
    // reasoning as `narrow_prop_null`'s nullable-receiver gate. The true
    // branch is unaffected: `narrow_instanceof_preserving_subtypes` never
    // returns empty.
    let mark_diverges = is_true || !ctx.get_var(obj_var).is_nullable();
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
}

/// Static-property counterpart of `narrow_prop_is_a`, for
/// `is_a(self::$prop, X::class, ...)` (and `static::$prop`/`Class::$prop`).
fn narrow_static_prop_is_a(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    class_name: &str,
    allow_string: bool,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed_not_template() {
        return;
    }
    if allow_string {
        let narrowed = if is_true {
            let (mut result, obj_part) = partition_is_a_string_like(&current, class_name, db);
            if !obj_part.is_empty() || current.is_mixed() {
                let obj_src = if obj_part.is_empty() {
                    &current
                } else {
                    &obj_part
                };
                let obj_narrowed = narrow_instanceof_preserving_subtypes(
                    obj_src,
                    class_name,
                    db,
                    &ctx.template_param_names,
                );
                for atom in obj_narrowed.types.iter() {
                    result.add_type(atom.clone());
                }
            }
            result
        } else {
            filter_out_is_a_string_match(&current, class_name, db)
        };
        // Same rationale as the variable case: don't mark diverges when
        // allow_string is set, since a class-string value may still pass.
        apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
    } else {
        let narrowed = if is_true {
            narrow_instanceof_preserving_subtypes(
                &current,
                class_name,
                db,
                &ctx.template_param_names,
            )
        } else {
            filter_out_instanceof_match(&current, class_name, db)
        };
        apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
    }
}

/// `is_a($obj->prop, ClassName::class)` / `is_a($obj->prop, ClassName::class, true)`
/// narrowing — same semantics as the variable-based `is_a` branch in
/// `narrow_from_condition`, applied to a property-access receiver instead.
#[allow(clippy::too_many_arguments)]
fn narrow_prop_is_a(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    class_name: &str,
    allow_string: bool,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed_not_template() {
        return;
    }
    if allow_string {
        let narrowed = if is_true {
            let (mut result, obj_part) = partition_is_a_string_like(&current, class_name, db);
            if !obj_part.is_empty() || current.is_mixed() {
                let obj_src = if obj_part.is_empty() {
                    &current
                } else {
                    &obj_part
                };
                let obj_narrowed = narrow_instanceof_preserving_subtypes(
                    obj_src,
                    class_name,
                    db,
                    &ctx.template_param_names,
                );
                for atom in obj_narrowed.types.iter() {
                    result.add_type(atom.clone());
                }
            }
            result
        } else {
            filter_out_is_a_string_match(&current, class_name, db)
        };
        // Same rationale as the variable case: don't mark diverges when
        // allow_string is set, since a class-string value may still pass.
        if !narrowed.is_empty() && narrowed != current {
            ctx.set_prop_refined(obj_var, prop, narrowed);
        }
    } else {
        let narrowed = if is_true {
            narrow_instanceof_preserving_subtypes(
                &current,
                class_name,
                db,
                &ctx.template_param_names,
            )
        } else {
            filter_out_instanceof_match(&current, class_name, db)
        };
        // Same nullable-receiver gate as `narrow_prop_instanceof`'s false
        // branch: `is_a($obj->prop, X)` false is also true whenever $obj
        // itself is null.
        let mark_diverges = is_true || !ctx.get_var(obj_var).is_nullable();
        apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
    }
}

/// Static-property counterpart of `narrow_prop_is_subclass_of`, for
/// `is_subclass_of(self::$prop, ClassName::class)` (and `static::$prop`/
/// `Class::$prop`) — same strict-subclass-only semantics.
fn narrow_static_prop_is_subclass_of(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    class_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    if !is_true {
        return;
    }
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed_not_template() {
        return;
    }
    let narrowed = narrow_strict_subclass_of(&current, class_name, db, &ctx.template_param_names);
    // mark_diverges=false: the exact class being absent from strict-subclass
    // narrowing doesn't make the branch dead, mirroring the var/prop siblings.
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false);
}

/// `is_subclass_of($obj->prop, ClassName::class)` narrowing — same semantics
/// as the variable-based branch (strict-subclass only; the false branch never
/// narrows since a non-subclass could still be the exact class itself).
fn narrow_prop_is_subclass_of(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    class_name: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    if !is_true {
        return;
    }
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed_not_template() {
        return;
    }
    let narrowed = narrow_strict_subclass_of(&current, class_name, db, &ctx.template_param_names);
    if !narrowed.is_empty() && narrowed != current {
        ctx.set_prop_refined(obj_var, prop, narrowed);
    }
}

/// Static-property counterpart of `narrow_prop_array_key_exists`, for
/// `array_key_exists('k', self::$prop)` (and `static::$prop`/`Class::$prop`).
/// Mirrors the var/prop siblings' true-branch convention: just apply the
/// narrowed shape, no divergence marking.
fn narrow_static_prop_array_key_exists(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    key: &mir_types::atomic::ArrayKey,
    db: &dyn MirDatabase,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let narrowed = add_key_to_sealed_shapes(&current, key);
    if narrowed != current {
        ctx.set_prop_refined(fqcn, prop, narrowed);
    }
}

/// Narrow a property's type when `array_key_exists('k', $this->prop)` is proven true.
fn narrow_prop_array_key_exists(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    key: &mir_types::atomic::ArrayKey,
    db: &dyn MirDatabase,
    file: &str,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    let narrowed = add_key_to_sealed_shapes(&current, key);
    if narrowed != current {
        ctx.set_prop_refined(obj_var, prop, narrowed);
    }
}

/// For each `TKeyedArray` in `ty` that does not already contain `key`: if
/// it's open, add `key` as non-optional `mixed` (an open shape might
/// genuinely carry it at runtime).
///
/// If it's sealed (`is_open == false`) AND `ty` is a real union of more than
/// one shape, exclude that member entirely instead — among a known finite
/// set of shape *alternatives*, one lacking the key can never satisfy
/// `array_key_exists()`, so keeping it let an impossible arm survive into
/// the true branch and widen later reads of that key to `mixed`.
///
/// A single (non-union) sealed shape lacking the key still falls back to
/// adding it as `mixed`, same as an open shape: a lone `@var array{a: T}`
/// docblock is a hint, not proof the underlying array can hold no other
/// key, so treating `array_key_exists` on an undeclared key as definitely
/// impossible would be a real false positive on ordinary runtime arrays.
fn add_key_to_sealed_shapes(
    ty: &mir_types::Type,
    key: &mir_types::atomic::ArrayKey,
) -> mir_types::Type {
    use mir_types::atomic::{ArrayKey, KeyedProperty};
    let is_real_union = ty.types.len() > 1;
    let mut changed = false;
    let mut result = mir_types::Type::empty();
    for a in &ty.types {
        if let Atomic::TKeyedArray {
            properties,
            is_open,
            is_list,
        } = a
        {
            if !properties.contains_key(key) {
                changed = true;
                if *is_open || !is_real_union {
                    let mut new_props = properties.clone();
                    // A newly-proven key only keeps the shape a list if it
                    // continues the sequence (next contiguous int index) —
                    // a string key, or any non-contiguous int, proves this
                    // can no longer be `array_is_list()`-true.
                    let stays_list = *is_list
                        && matches!(key, ArrayKey::Int(n) if *n == properties.len() as i64);
                    new_props.insert(
                        key.clone(),
                        KeyedProperty {
                            ty: mir_types::Type::mixed(),
                            optional: false,
                        },
                    );
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: stays_list,
                    });
                }
                continue;
            }
            // The key is already declared but optional — array_key_exists()
            // proves it's actually present, so clear the optional flag. It
            // does NOT prove the value is non-null (unlike isset()): PHP's
            // array_key_exists('k', ['k' => null]) is true, so the value
            // type must be left untouched.
            if let Some(prop) = properties.get(key) {
                if prop.optional {
                    changed = true;
                    let mut new_props = properties.clone();
                    if let Some(new_prop) = new_props.get_mut(key) {
                        new_prop.optional = false;
                    }
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                    continue;
                }
            }
        }
        result.add_type(a.clone());
    }
    if !changed {
        return ty.clone();
    }
    // Every union member turned out to be an impossible closed shape — keep
    // the original type rather than narrowing to an empty union.
    if result.types.is_empty() {
        return ty.clone();
    }
    result.from_docblock = ty.from_docblock;
    result
}

/// False-branch counterpart of `add_key_to_sealed_shapes`, for
/// `!array_key_exists($key, $arr)`: among a real union (`ty.types.len() > 1`)
/// of shape *alternatives*, excludes any `TKeyedArray` member that declares
/// `key` as present and non-optional — such a member guarantees the key
/// exists, so it can never satisfy the key's absence, the same reasoning
/// `add_key_to_sealed_shapes` already applies in the opposite direction to
/// members lacking the key. A *lone* (non-union) shape is left untouched
/// even when it declares the key mandatory: same "hint, not proof" caution
/// as the true-branch helper — a single docblock shape isn't necessarily
/// exhaustive proof about one specific real array's actual contents.
/// Optional or undeclared keys are also left untouched: both are already
/// consistent with the key's absence.
fn remove_key_from_sealed_shapes(
    ty: &mir_types::Type,
    key: &mir_types::atomic::ArrayKey,
) -> mir_types::Type {
    if ty.types.len() <= 1 {
        return ty.clone();
    }
    let mut changed = false;
    let mut result = mir_types::Type::empty();
    for a in &ty.types {
        if let Atomic::TKeyedArray { properties, .. } = a {
            if let Some(prop) = properties.get(key) {
                if !prop.optional {
                    changed = true;
                    continue;
                }
            }
        }
        result.add_type(a.clone());
    }
    if !changed {
        return ty.clone();
    }
    // Every union member turned out to guarantee the key's presence — keep
    // the original type rather than narrowing to an empty union, mirroring
    // `add_key_to_sealed_shapes`'s same fallback in the opposite direction.
    if result.types.is_empty() {
        return ty.clone();
    }
    result.from_docblock = ty.from_docblock;
    result
}

/// Extract a signed integer literal from an expression, handling negation.
fn extract_int_literal(expr: &php_ast::owned::Expr) -> Option<i64> {
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
fn flip_comparison_op(op: BinaryOp) -> BinaryOp {
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

fn narrow_var_int_comparison(ctx: &mut FlowState, name: &str, op: BinaryOp, n: i64, is_true: bool) {
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
fn narrow_prop_int_comparison(
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
    narrow_receiver_non_null_on_prop_match(
        ctx,
        obj_var,
        int_comparison_excludes_null(op, n, is_true),
    );
}

/// Static-property counterpart of `narrow_prop_int_comparison`, for
/// `self::$prop < N` (or `static::$prop`/`Class::$prop`). Unlike the
/// instance-property case, a static property has no separate receiver
/// variable whose nullability could also satisfy the comparison —
/// `self::`/`static::` is never itself null — so mark_diverges only
/// depends on `is_closed_precise`, matching the plain-variable case.
fn narrow_static_prop_int_comparison(
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

/// Narrow all `TString` atoms to `TNonEmptyString`, preserving other atoms.
/// Used when a condition proves the string is non-empty.
fn narrow_string_to_non_empty(ty: &Type) -> Type {
    let mut result = Type::empty();
    result.from_docblock = ty.from_docblock;
    for t in &ty.types {
        match t {
            Atomic::TString => result.add_type(Atomic::TNonEmptyString),
            _ => result.add_type(t.clone()),
        }
    }
    result
}

/// Drop the `non-empty-string` variant when a length check proves the string
/// is exactly empty (mirrors `Type::narrow_to_empty_collection` for arrays).
fn narrow_string_to_empty(ty: &Type) -> Type {
    ty.filter(|t| !matches!(t, Atomic::TNonEmptyString))
}

fn narrow_var_null(ctx: &mut FlowState, name: &str, is_null: bool) {
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
fn narrow_var_loose_null(ctx: &mut FlowState, name: &str, is_null: bool) {
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
fn narrow_prop_loose_null(
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

/// After proving `$obj->prop` equals a definite non-null literal value
/// (`proved_match`), the receiver itself must also be non-null: PHP 8 reads
/// `$obj->prop` on a null `$obj` as a warning, still evaluating to `null`
/// (same ambiguity as `narrow_nullsafe_prop_null`).
fn narrow_receiver_non_null_on_prop_match(ctx: &mut FlowState, obj_var: &str, proved_match: bool) {
    if proved_match {
        narrow_var_null(ctx, obj_var, false);
    }
}

/// Narrow `name` to truthy (`want_truthy`) or falsy, for the loose
/// `$x == true`/`$x == false` idiom — distinct from `narrow_var_bool`, which
/// handles the strict `$x === true`/`$x === false` identity check.
fn narrow_var_loose_bool(ctx: &mut FlowState, name: &str, want_truthy: bool) {
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
fn narrow_prop_loose_bool(
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

fn narrow_var_bool(ctx: &mut FlowState, name: &str, value: bool, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = bool_narrow_type(&current, value, is_value);
    set_narrowed(ctx, name, &current, narrowed, false);
}

/// Property-access counterpart of `narrow_var_bool`, for
/// `$this->prop === true`/`false` (or any `$obj->prop` receiver).
fn narrow_prop_bool(
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
fn bool_narrow_type(current: &Type, value: bool, is_value: bool) -> Type {
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
            Atomic::TMixed => true,
            _ => !is_value, // non-bool atoms: keep only when narrowing away
        };
        if keep {
            narrowed.add_type(t.clone());
        }
    }
    narrowed
}

fn narrow_from_type_fn(
    ctx: &mut FlowState,
    fn_name: &str,
    var_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    let current = ctx.get_var(var_name);
    let Some(narrowed) = type_fn_narrowed(&current, fn_name, db, is_true) else {
        return;
    };
    set_narrowed(ctx, var_name, &current, narrowed, true);
}

/// Property-access counterpart of `narrow_from_type_fn`, for
/// `is_string($this->prop)`, `is_array($this->prop)`, `array_is_list($this->prop)`,
/// `ctype_digit($this->prop)`, `method_exists($this->prop, ...)`, etc. — the
/// whole `is_*`/`ctype_*`/type-check family previously only ever narrowed a
/// plain-variable receiver, unlike the analogous `instanceof`/null/literal-match
/// arms elsewhere in this file, which all have a property-access fallback.
fn narrow_prop_from_type_fn(
    ctx: &mut FlowState,
    fn_name: &str,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    let Some(narrowed) = type_fn_narrowed(&current, fn_name, db, is_true) else {
        return;
    };
    // A nullable receiver makes `$obj->prop` itself evaluate to `null` (PHP 8
    // warning, not fatal), which is an extra value the property's own declared
    // type doesn't account for. That extra `null` only satisfies `is_null()`'s
    // true branch — every other `is_*`/`ctype_*` check returns false on null,
    // so it's their false branch that gains a reachable-despite-empty case
    // (same reasoning as `narrow_prop_instanceof`'s false-branch gate).
    let is_null_check = crate::util::php_ident_lowercase(fn_name) == "is_null";
    let receiver_nullable = ctx.get_var(obj_var).is_nullable();
    // Proves the property's own value isn't null (`is_true != is_null_check`,
    // same condition as mark_diverges above) — which, on a nullable receiver,
    // also proves the receiver itself wasn't null (see the comment above).
    let proved_prop_non_null = is_true != is_null_check;
    let mark_diverges = !receiver_nullable || proved_prop_non_null;
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, mark_diverges);
    narrow_receiver_non_null_on_prop_match(ctx, obj_var, proved_prop_non_null);
}

/// Static-property counterpart of `narrow_prop_from_type_fn`, for
/// `is_string(self::$prop)`, `ctype_digit(static::$prop)`, etc. Unlike the
/// instance-property case, a static property has no separate "receiver
/// variable" whose own nullability could add an extra unaccounted-for
/// `null` — `self::`/`static::` is never itself null — so mark_diverges is
/// unconditional, matching the plain-variable case.
fn narrow_static_prop_from_type_fn(
    ctx: &mut FlowState,
    fn_name: &str,
    fqcn: &str,
    prop: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) {
    let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
    if current.is_mixed() {
        return;
    }
    let Some(narrowed) = type_fn_narrowed(&current, fn_name, db, is_true) else {
        return;
    };
    apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
}

/// Core `is_*`/`ctype_*`/`array_is_list`/`method_exists`/`property_exists`
/// narrowing logic, shared between the variable-receiver
/// (`narrow_from_type_fn`) and property-receiver (`narrow_prop_from_type_fn`)
/// entry points. Returns `None` for an unrecognized function name — the
/// caller should leave the type untouched.
fn type_fn_narrowed(
    current: &Type,
    fn_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) -> Option<Type> {
    Some(match crate::util::php_ident_lowercase(fn_name).as_str() {
        "is_string" => {
            if is_true {
                current.narrow_to_string()
            } else {
                current.filter(|t| !t.is_string())
            }
        }
        "is_int" | "is_integer" | "is_long" => {
            if is_true {
                current.narrow_to_int()
            } else {
                current.filter(|t| !t.is_int())
            }
        }
        "is_float" | "is_double" | "is_real" => {
            if is_true {
                current.narrow_to_float()
            } else {
                current.filter(|t| {
                    !matches!(
                        t,
                        Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
                    )
                })
            }
        }
        "is_bool" => {
            if is_true {
                current.narrow_to_bool()
            } else {
                current.filter(|t| !matches!(t, Atomic::TBool | Atomic::TTrue | Atomic::TFalse))
            }
        }
        "is_null" => {
            if is_true {
                current.narrow_to_null()
            } else {
                current.remove_null()
            }
        }
        "is_array" => {
            if is_true {
                current.narrow_to_array()
            } else {
                current.filter(|t| !t.is_array())
            }
        }
        "array_is_list" => {
            if is_true {
                current.narrow_to_list()
            } else {
                current.filter(|t| {
                    !matches!(
                        t,
                        Atomic::TList { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TKeyedArray { is_list: true, .. }
                    )
                })
            }
        }
        "is_object" => {
            if is_true {
                current.narrow_to_object()
            } else {
                current.filter(|t| !t.is_object())
            }
        }
        "is_callable" => {
            if is_true {
                current.narrow_to_callable()
            } else {
                current.filter(|t| !t.is_callable())
            }
        }
        "is_scalar" => {
            if is_true {
                current.narrow_to_scalar()
            } else {
                current.filter(|t| {
                    !t.is_string()
                        && !t.is_int()
                        && !matches!(
                            t,
                            Atomic::TFloat
                                | Atomic::TIntegralFloat
                                | Atomic::TLiteralFloat(..)
                                | Atomic::TBool
                                | Atomic::TTrue
                                | Atomic::TFalse
                                | Atomic::TScalar
                                | Atomic::TNumeric
                        )
                })
            }
        }
        "is_iterable" => {
            if is_true {
                current.narrow_to_iterable()
            } else {
                // Beyond the array atom (always excludable), a named-object atom
                // is only excludable when it's `final` (no subclass could add
                // `implements Traversable` later) AND its own hierarchy provably
                // doesn't already extend/implement Traversable — same
                // final-class soundness gate `narrow_var_to_specific_class` uses
                // for its false branch. A non-final or unresolvable class stays,
                // same conservatism as before this class-hierarchy check existed.
                current
                    .filter(|t| !atom_excluded_from_is_iterable_or_countable(t, "Traversable", db))
            }
        }
        "is_countable" => {
            if is_true {
                current.narrow_to_countable()
            } else {
                current.filter(|t| !atom_excluded_from_is_iterable_or_countable(t, "Countable", db))
            }
        }
        "is_resource" => {
            if is_true {
                current.narrow_to_resource()
            } else {
                // Exclude nothing (no resource type exists); return unchanged
                current.clone()
            }
        }
        "is_numeric" => {
            if is_true {
                // In the truthy branch: keep numeric types and string types that
                // *could* be numeric. TString / TNonEmptyString narrow to TNumericString
                // (a string proven to be numeric-valued). All int and float variants are
                // always numeric. TMixed is kept as-is.
                let mut narrowed_parts = Type::empty();
                for t in &current.types {
                    match t {
                        // All int and float variants are unconditionally numeric.
                        Atomic::TInt
                        | Atomic::TIntRange { .. }
                        | Atomic::TPositiveInt
                        | Atomic::TNonNegativeInt
                        | Atomic::TNegativeInt
                        | Atomic::TLiteralInt(_)
                        | Atomic::TFloat
                        | Atomic::TIntegralFloat
                        | Atomic::TLiteralFloat(..)
                        | Atomic::TNumeric
                        | Atomic::TNumericString => {
                            narrowed_parts.add_type(t.clone());
                        }
                        // A generic string could be numeric; narrow to numeric-string.
                        Atomic::TString | Atomic::TNonEmptyString => {
                            narrowed_parts.add_type(Atomic::TNumericString);
                        }
                        // A literal string is numeric only if it parses as a number.
                        Atomic::TLiteralString(s) if is_numeric_string(s) => {
                            narrowed_parts.add_type(t.clone());
                        }
                        // mixed/scalar could be anything; a truthy is_numeric()
                        // proves it's specifically int|float|numeric-string, so
                        // replace it rather than leaving it as mixed/scalar (matching
                        // how is_string/is_int/etc. narrow these two atoms).
                        Atomic::TScalar | Atomic::TMixed => {
                            narrowed_parts.add_type(Atomic::TInt);
                            narrowed_parts.add_type(Atomic::TFloat);
                            narrowed_parts.add_type(Atomic::TNumericString);
                        }
                        _ => {} // non-numeric types are excluded
                    }
                }
                narrowed_parts
            } else {
                current.filter(|t| {
                    !matches!(
                        t,
                        Atomic::TInt
                            | Atomic::TIntRange { .. }
                            | Atomic::TPositiveInt
                            | Atomic::TNonNegativeInt
                            | Atomic::TNegativeInt
                            | Atomic::TFloat
                            | Atomic::TIntegralFloat
                            | Atomic::TNumeric
                            | Atomic::TNumericString
                            | Atomic::TLiteralInt(_)
                            | Atomic::TLiteralFloat(..)
                    ) && !matches!(t, Atomic::TLiteralString(s) if is_numeric_string(s))
                })
            }
        }
        // ctype_*() returns false on an empty string for every variant, so a
        // truthy result proves the string argument is non-empty. It says
        // nothing when the argument isn't a string (e.g. ctype_digit(65) is
        // true because 65 is ASCII 'A', unrelated to decimal digits), so
        // only TString atoms are touched — everything else passes through.
        "ctype_alpha" | "ctype_alnum" | "ctype_digit" | "ctype_lower" | "ctype_upper"
        | "ctype_punct" | "ctype_space" | "ctype_xdigit" | "ctype_print" | "ctype_graph"
        | "ctype_cntrl" => {
            if is_true {
                narrow_string_to_non_empty(current)
            } else {
                current.clone()
            }
        }
        // method_exists($obj, 'method') / property_exists($obj, 'prop') — both accept
        // object|string, so on true keep object atoms as-is (preserving the specific
        // class instead of collapsing it to bare TObject) and keep string/class-string
        // atoms as-is; TMixed is replaced with bare TObject (a usable placeholder — the
        // string alternative isn't worth widening a plain mixed into a union for).
        // TScalar excludes object entirely though, so it can only narrow to TString,
        // never TObject (bool|int|float|string can never be an object instance).
        "method_exists" | "property_exists" => {
            if is_true {
                let mut result = Type::empty();
                result.from_docblock = current.from_docblock;
                for t in &current.types {
                    if t.is_object() || t.is_string() {
                        result.add_type(t.clone());
                    } else if matches!(t, Atomic::TMixed) {
                        result.add_type(Atomic::TObject);
                    } else if matches!(t, Atomic::TScalar) {
                        result.add_type(Atomic::TString);
                    }
                }
                if result.is_empty() {
                    current.clone()
                } else {
                    result
                }
            } else {
                current.clone()
            }
        }
        _ => return None,
    })
}

/// Whether `t` is provably excluded from `is_iterable()`/`is_countable()`'s
/// false branch: always true for the `array` atom (the one type both
/// functions are unconditionally true for), and true for a named-object atom
/// that already provably extends/implements `$interface` — such an atom can
/// never make `is_iterable()`/`is_countable()` false, so it can't survive
/// into the false branch. A `final` class that provably does NOT
/// extend/implement `$interface` is the opposite case (guaranteed to make
/// the check false) and must be kept, not excluded. A non-final or
/// unresolvable class is left alone (kept), matching this function's
/// conservative behavior before this class-hierarchy check existed
/// (stripping a non-final class here risks falsely excluding a legitimately
/// Countable/Traversable subclass).
fn atom_excluded_from_is_iterable_or_countable(
    t: &Atomic,
    interface: &str,
    db: &dyn MirDatabase,
) -> bool {
    if t.is_array() {
        return true;
    }
    if let Atomic::TNamedObject { fqcn, .. } = t {
        return crate::db::extends_or_implements(db, fqcn, interface);
    }
    false
}

fn narrow_var_literal_string(ctx: &mut FlowState, name: &str, value: &str, is_value: bool) {
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
fn narrow_prop_literal_string(
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

fn literal_string_narrow_type(current: &Type, value: &str, is_value: bool) -> Type {
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

fn narrow_var_literal_int(ctx: &mut FlowState, name: &str, value: i64, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = literal_int_narrow_type(&current, value, is_value);
    // For closed-precise types (bounded ranges, named int subtypes, literal unions),
    // an empty result means the exclusion is a genuine contradiction — mark divergence.
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    set_narrowed(ctx, name, &current, narrowed, mark_diverges);
}

/// Property-access counterpart of `narrow_var_literal_int`, for
/// `$this->prop === 42` (or any `$obj->prop` receiver).
fn narrow_prop_literal_int(
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

fn literal_int_narrow_type(current: &Type, value: i64, is_value: bool) -> Type {
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

/// If `ty` contains an atomic referring to the WHOLE enum `enum_fqcn` (a
/// plain `TNamedObject` — e.g. a `Status $s` parameter that was never
/// narrowed to individual cases), replace that atomic with a union of
/// `TLiteralEnumCase` for every case the enum declares. Atoms that already
/// are per-case literals, or refer to something else entirely, pass through
/// unchanged. Falls back to `ty` unchanged if the enum can't be resolved or
/// nothing needed expanding.
fn expand_enum_to_cases(db: &dyn MirDatabase, ty: &Type, enum_fqcn: &str) -> Type {
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

fn narrow_var_to_literal_enum_case(
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
fn narrow_prop_to_literal_enum_case(
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
fn narrow_static_prop_to_literal_enum_case(
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
fn narrow_static_prop_to_class_string(
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
fn narrow_var_to_class_string(
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
fn narrow_prop_to_class_string(
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
fn narrow_var_to_specific_class(
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
            type_params: mir_types::union::empty_type_params(),
        })
    } else {
        current.filter(|t| match t {
            Atomic::TNamedObject { fqcn: obj_fqcn, .. } => {
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
fn narrow_prop_to_specific_class(
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
            type_params: mir_types::union::empty_type_params(),
        })
    } else {
        current.filter(|t| match t {
            Atomic::TNamedObject { fqcn: obj_fqcn, .. } => {
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

/// Extract a fully-qualified class name from the first argument of
/// `class_exists()` / `interface_exists()` / `trait_exists()`.
///
/// Recognised forms:
/// - `\Foo\Bar::class` or `Foo\Bar::class` — resolved via `crate::db::resolve_name`
/// - `'Foo\Bar'` or `'Foo\\Bar'` — string literals
pub(crate) fn extract_class_fqcn_from_expr(
    expr: &php_ast::owned::Expr,
    self_fqcn: Option<&str>,
    static_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<str>> {
    let expr = peel_parens(expr);
    match &expr.kind {
        // \Foo\Bar::class  or  Foo\Bar::class  (also self::class/static::class/parent::class)
        ExprKind::ClassConstAccess(cca) => {
            if let ExprKind::Identifier(id) = &cca.class.kind {
                let member = match &cca.member.kind {
                    ExprKind::Identifier(s) => s.as_ref(),
                    _ => return None,
                };
                if member.eq_ignore_ascii_case("class") {
                    match id.to_ascii_lowercase().as_str() {
                        "self" | "static" => {
                            let fqcn = if id.eq_ignore_ascii_case("static") {
                                static_fqcn.or(self_fqcn)
                            } else {
                                self_fqcn
                            };
                            return fqcn.map(std::sync::Arc::from);
                        }
                        "parent" => return parent_fqcn.map(std::sync::Arc::from),
                        _ => {
                            let resolved = crate::db::resolve_name(db, file, id.as_ref());
                            return Some(std::sync::Arc::from(resolved.as_str()));
                        }
                    }
                }
            }
            None
        }
        // 'Foo\Bar'  or  'Foo\\Bar'  or  'Foo'
        ExprKind::String(s) => {
            let name = s.as_ref().trim_start_matches('\\');
            if !name.is_empty() {
                Some(std::sync::Arc::from(name))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Extract `(obj_var, prop_name)` from a simple `$var->prop` expression.
pub(crate) fn extract_prop_access(expr: &php_ast::owned::Expr) -> Option<(String, String)> {
    match &expr.kind {
        ExprKind::PropertyAccess(pa) => {
            let obj = extract_var_name(&pa.object)?;
            let prop = match &pa.property.kind {
                ExprKind::Identifier(s) => s.as_ref().to_string(),
                _ => return None,
            };
            Some((obj, prop))
        }
        ExprKind::Parenthesized(inner) => extract_prop_access(inner),
        _ => None,
    }
}

/// Like `extract_prop_access`, but only matches the nullsafe (`?->`) form.
/// Kept as a separate matcher purely to distinguish the two operators in the
/// AST — both are narrowed by the same logic (`narrow_nullsafe_prop_null`),
/// since a plain `->` on a null receiver also evaluates to `null` in PHP 8
/// (a warning, not a fatal error), same as `?->`'s short-circuit.
fn extract_nullsafe_prop_access(expr: &php_ast::owned::Expr) -> Option<(String, String)> {
    match &expr.kind {
        ExprKind::NullsafePropertyAccess(pa) => {
            let obj = extract_var_name(&pa.object)?;
            let prop = match &pa.property.kind {
                ExprKind::Identifier(s) => s.as_ref().to_string(),
                _ => return None,
            };
            Some((obj, prop))
        }
        ExprKind::Parenthesized(inner) => extract_nullsafe_prop_access(inner),
        _ => None,
    }
}

/// `extract_prop_access`, but also accepts the nullsafe (`?->`) form —
/// for arms that don't need to distinguish the two operators (a plain
/// `->` on a null receiver also evaluates to null in PHP 8, same as
/// `?->`'s short-circuit, so most literal-comparison narrowing arms
/// should treat both identically).
fn extract_any_prop_access(expr: &php_ast::owned::Expr) -> Option<(String, String)> {
    extract_nullsafe_prop_access(expr).or_else(|| extract_prop_access(expr))
}

/// Extract `(fqcn, prop_name)` from a `self::$prop` / `static::$prop` /
/// `parent::$prop` / `ClassName::$prop` expression, resolving relative
/// keywords through the current `FlowState`.
fn extract_static_prop_access(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    match &expr.kind {
        ExprKind::StaticPropertyAccess(spa) => {
            let id = match &spa.class.kind {
                ExprKind::Identifier(id) => id,
                _ => return None,
            };
            let resolved = crate::db::resolve_name(db, file, id.as_ref());
            let fqcn = match resolved.as_str() {
                "self" | "static" => ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone())?,
                "parent" => ctx.parent_fqcn.clone()?,
                s => std::sync::Arc::from(s),
            };
            let prop = match &spa.member.kind {
                ExprKind::Variable(name) | ExprKind::Identifier(name) => {
                    name.trim_start_matches('$').to_string()
                }
                _ => return None,
            };
            Some((fqcn, prop))
        }
        ExprKind::Parenthesized(inner) => extract_static_prop_access(inner, ctx, db, file),
        _ => None,
    }
}

fn extract_var_name(expr: &php_ast::owned::Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => extract_var_name(inner),
        // is_null($x = expr) — narrow the assigned variable, not the RHS
        ExprKind::Assign(a) if matches!(a.op, AssignOp::Assign) => extract_var_name(&a.target),
        _ => None,
    }
}

/// Extract a compact key for simple expressions used as the first arg of
/// `method_exists`/`property_exists`. Supports `$var` → `"var"`,
/// `$var->prop` → `"var->prop"` (depth-1 only), and `Foo::class` → the
/// resolved FQCN prefixed `"cls:"` (disjoint from the variable-key
/// namespace, so a variable named e.g. `Foo` can never collide with a
/// class-name guard). Returns `None` for anything more complex so we don't
/// risk false-positive suppression.
pub(crate) fn extract_expr_guard_key(
    expr: &php_ast::owned::Expr,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<str>> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(std::sync::Arc::from(name.trim_start_matches('$'))),
        ExprKind::Parenthesized(inner) => extract_expr_guard_key(inner, db, file),
        ExprKind::PropertyAccess(pa) => {
            let base = extract_var_name(&pa.object)?;
            let prop = match &pa.property.kind {
                ExprKind::Identifier(s) => s.as_ref(),
                ExprKind::Variable(s) => s.trim_start_matches('$'),
                _ => return None,
            };
            Some(std::sync::Arc::from(format!("{base}->{prop}").as_str()))
        }
        ExprKind::ClassConstAccess(cca) => {
            let ExprKind::Identifier(member) = &cca.member.kind else {
                return None;
            };
            if !member.eq_ignore_ascii_case("class") {
                return None;
            }
            let ExprKind::Identifier(class_name) = &cca.class.kind else {
                return None;
            };
            let resolved = crate::db::resolve_name(db, file, class_name.as_ref());
            Some(std::sync::Arc::from(format!("cls:{resolved}").as_str()))
        }
        _ => None,
    }
}

/// The base (variable or property receiver) of a (possibly nested)
/// array-access expression: `$a[1][2]` → `Var("a")`, `$this->data[1]` →
/// `Prop("this", "data")`. Unlike `collect_array_access_path`, doesn't
/// require every key along the way to be a literal — stripping null/false
/// from the container itself doesn't depend on the key being statically
/// known.
fn array_access_base_target(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    match &expr.kind {
        ExprKind::ArrayAccess(aa) => array_access_base_target(&aa.array),
        ExprKind::Parenthesized(inner) => array_access_base_target(inner),
        _ => ScalarArgTarget::extract(expr),
    }
}

/// Remove `null`/`false` from an `isset($base[...])`/`!empty($base[...])`
/// container, whichever receiver shape `base` is — the property-receiver
/// counterpart of the plain-variable case, since `->` access on a nullable
/// property is just as valid an `isset()`/`empty()` target as a variable.
fn narrow_container_non_null_non_false(
    ctx: &mut FlowState,
    target: &ScalarArgTarget,
    db: &dyn MirDatabase,
    file: &str,
) {
    match target {
        ScalarArgTarget::Var(name) => {
            let current = ctx.get_var(name);
            ctx.set_var(name, current.remove_null().remove_false());
        }
        ScalarArgTarget::Prop(obj, prop) => {
            let current = resolve_prop_current_type(ctx, obj, prop, db, file);
            if !current.is_mixed() {
                let narrowed = current.remove_null().remove_false();
                apply_prop_narrowed(ctx, obj, prop, current, narrowed, true);
            }
        }
    }
}

/// For `isset($base['a']['b']...)` where `$base` is (partly) a known shape,
/// narrow every level of the access path: remove `null` from each key's
/// value type and mark it no longer optional, recursing into the key's own
/// value type for the next path segment. `isset($a['b']['c'])` proves both
/// `$a['b']` and `$a['b']['c']` present, not just the outermost key.
fn narrow_isset_shape_key(
    var_expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    let Some((base, path)) = collect_array_access_path(var_expr) else {
        return;
    };
    let current = resolve_shape_base_current_type(ctx, &base, db, file);
    if let Some(narrowed) = narrow_shape_path(&current, &path) {
        set_shape_base_narrowed(ctx, &base, current, narrowed);
    }
}

/// Collect `(base, [key1, key2, ...])` from a chain of literal-keyed
/// `ArrayAccess` nodes, outermost-to-innermost (`$a['x']['y']` -> `(Var("a"),
/// [x, y])`, `$this->data['x']` -> `(Prop("this", "data"), [x])`). Returns
/// `None` as soon as a non-literal key or non-var/prop root is found — those
/// cases are left unnarrowed.
fn collect_array_access_path(
    expr: &php_ast::owned::Expr,
) -> Option<(ScalarArgTarget, Vec<mir_types::atomic::ArrayKey>)> {
    let ExprKind::ArrayAccess(aa) = &expr.kind else {
        return None;
    };
    let idx = aa.index.as_ref()?;
    let key = match &idx.kind {
        ExprKind::String(s) => {
            mir_types::atomic::ArrayKey::String(std::sync::Arc::from(s.as_ref()))
        }
        ExprKind::Int(i) => mir_types::atomic::ArrayKey::Int(*i),
        _ => return None,
    };
    if let Some(base) = ScalarArgTarget::extract(&aa.array) {
        Some((base, vec![key]))
    } else {
        let (base, mut path) = collect_array_access_path(&aa.array)?;
        path.push(key);
        Some((base, path))
    }
}

/// Read the current type of a `collect_array_access_path` base, whichever
/// receiver shape it is.
fn resolve_shape_base_current_type(
    ctx: &mut FlowState,
    base: &ScalarArgTarget,
    db: &dyn MirDatabase,
    file: &str,
) -> Type {
    match base {
        ScalarArgTarget::Var(name) => ctx.get_var(name),
        ScalarArgTarget::Prop(obj, prop) => resolve_prop_current_type(ctx, obj, prop, db, file),
    }
}

/// Apply a narrowed type back to a `collect_array_access_path` base.
fn set_shape_base_narrowed(
    ctx: &mut FlowState,
    base: &ScalarArgTarget,
    current: Type,
    narrowed: Type,
) {
    match base {
        ScalarArgTarget::Var(name) => ctx.set_var(name, narrowed),
        ScalarArgTarget::Prop(obj, prop) => {
            apply_prop_narrowed(ctx, obj, prop, current, narrowed, false)
        }
    }
}

/// Narrow `ty` along a shape-key access path proven present by `isset()`:
/// clears `optional`/`null` at `path[0]`, then recurses into that key's own
/// value type for `path[1..]`. Returns `None` when nothing changed (e.g. no
/// union member is a `TKeyedArray` carrying the key at all).
fn narrow_shape_path(ty: &Type, path: &[mir_types::atomic::ArrayKey]) -> Option<Type> {
    let (key, rest) = path.split_first()?;
    let mut changed = false;
    let mut result = Type::empty();
    for atomic in &ty.types {
        match atomic {
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                if properties.contains_key(key) {
                    let mut new_props = properties.clone();
                    if let Some(prop) = new_props.get_mut(key) {
                        let mut narrowed_ty = prop.ty.remove_null();
                        if !rest.is_empty() {
                            if let Some(deeper) = narrow_shape_path(&narrowed_ty, rest) {
                                narrowed_ty = deeper;
                            }
                        }
                        if !narrowed_ty.is_empty() {
                            prop.ty = narrowed_ty;
                        }
                        prop.optional = false;
                    }
                    changed = true;
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                } else if *is_open {
                    // An open shape might still carry the key at runtime —
                    // keep it (unnarrowed) rather than dropping it.
                    result.add_type(atomic.clone());
                } else {
                    // A closed shape without this key can never satisfy
                    // isset() — this union member is impossible in the true
                    // branch, so exclude it instead of leaving it to be
                    // treated as if the key existed.
                    changed = true;
                }
            }
            _ => result.add_type(atomic.clone()),
        }
    }
    // If every union member turned out to be an impossible closed shape, keep
    // the original type rather than narrowing to an empty union — proving
    // the branch itself unreachable is a separate concern from key narrowing.
    if changed && !result.types.is_empty() {
        Some(result)
    } else {
        None
    }
}

/// For `array_key_exists('key', $base['a']['b']...)` where the array
/// argument is itself a nested shape-key access: walk down `path` to the
/// container shape, then apply `array_key_exists`'s own key-presence
/// semantics (`add_key_to_sealed_shapes`) there — parallel to
/// `narrow_shape_path`, but the leaf operation proves a *given* key present
/// in the container rather than the last path segment itself.
fn narrow_shape_path_key_exists(
    ty: &Type,
    path: &[mir_types::atomic::ArrayKey],
    key: &mir_types::atomic::ArrayKey,
) -> Option<Type> {
    let Some((head, rest)) = path.split_first() else {
        let narrowed = add_key_to_sealed_shapes(ty, key);
        return if narrowed != *ty {
            Some(narrowed)
        } else {
            None
        };
    };
    let mut changed = false;
    let mut result = Type::empty();
    for atomic in &ty.types {
        match atomic {
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                if properties.contains_key(head) {
                    let mut new_props = properties.clone();
                    if let Some(prop) = new_props.get_mut(head) {
                        if let Some(deeper) = narrow_shape_path_key_exists(&prop.ty, rest, key) {
                            prop.ty = deeper;
                            changed = true;
                        }
                        // Reaching here at all proves `head` is a real array
                        // (array_key_exists's second argument), so it's no
                        // longer optional — regardless of whether the deeper
                        // key-presence narrowing itself changed anything.
                        if prop.optional {
                            prop.optional = false;
                            changed = true;
                        }
                    }
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                } else {
                    result.add_type(atomic.clone());
                }
            }
            _ => result.add_type(atomic.clone()),
        }
    }
    if changed {
        Some(result)
    } else {
        None
    }
}

/// False-branch counterpart of `narrow_shape_path_key_exists`: walk down
/// `path` to the container shape, then exclude union members that guarantee
/// `key`'s presence there (`remove_key_from_sealed_shapes`) — same
/// leaf-operation swap `remove_key_from_sealed_shapes` is to
/// `add_key_to_sealed_shapes` for the single-level (non-nested) false branch.
fn narrow_shape_path_key_exists_false(
    ty: &Type,
    path: &[mir_types::atomic::ArrayKey],
    key: &mir_types::atomic::ArrayKey,
) -> Option<Type> {
    let Some((head, rest)) = path.split_first() else {
        let narrowed = remove_key_from_sealed_shapes(ty, key);
        return if narrowed != *ty {
            Some(narrowed)
        } else {
            None
        };
    };
    let mut changed = false;
    let mut result = Type::empty();
    for atomic in &ty.types {
        match atomic {
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                if properties.contains_key(head) {
                    let mut new_props = properties.clone();
                    if let Some(prop) = new_props.get_mut(head) {
                        if let Some(deeper) =
                            narrow_shape_path_key_exists_false(&prop.ty, rest, key)
                        {
                            prop.ty = deeper;
                            changed = true;
                        }
                        // Same reasoning as the true-branch twin above:
                        // reaching here at all proves `head` is a real array,
                        // regardless of the false-branch key result.
                        if prop.optional {
                            prop.optional = false;
                            changed = true;
                        }
                    }
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                } else {
                    result.add_type(atomic.clone());
                }
            }
            _ => result.add_type(atomic.clone()),
        }
    }
    if changed {
        Some(result)
    } else {
        None
    }
}

/// For `empty($base['a']['b']...)` where `$base` is (partly) a known shape,
/// narrow that key's own property by truthiness — mirroring
/// `narrow_isset_shape_key`, but with `empty()`'s truthy/falsy semantics
/// instead of `isset()`'s presence/null semantics.
///
/// Nested paths (`$base['a']['b']`) are only narrowed for `!empty(...)`:
/// that direction proves presence at every level plus truthiness of the
/// final value, exactly like `narrow_not_empty_shape_path` computes. Plain
/// `empty(...)` being true doesn't pin down which level was missing/falsy,
/// so nested paths are left unnarrowed there (single-level `empty()` still
/// narrows as before).
fn narrow_empty_shape_key(
    var_expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    let Some((base, path)) = collect_array_access_path(var_expr) else {
        return;
    };
    if path.len() > 1 {
        if !is_true {
            let current = resolve_shape_base_current_type(ctx, &base, db, file);
            if let Some(narrowed) = narrow_not_empty_shape_path(&current, &path) {
                set_shape_base_narrowed(ctx, &base, current, narrowed);
            }
        }
        return;
    }
    let key = path
        .into_iter()
        .next()
        .expect("path.len() == 1 checked above");

    let current = resolve_shape_base_current_type(ctx, &base, db, file);
    let mut changed = false;
    let mut result = Type::empty();
    for atomic in &current.types {
        match atomic {
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                if properties.contains_key(&key) {
                    let mut new_props = properties.clone();
                    if let Some(prop) = new_props.get_mut(&key) {
                        if is_true {
                            // empty($base['key']) true: the key's value (if any) is
                            // falsy. The key may also be entirely absent (also
                            // falsy), so `optional` is left untouched.
                            let narrowed_ty = prop.ty.narrow_to_falsy();
                            if !narrowed_ty.is_empty() {
                                prop.ty = narrowed_ty;
                            }
                        } else {
                            // !empty($base['key']): the key is present and truthy.
                            let narrowed_ty = prop.ty.narrow_to_truthy();
                            if !narrowed_ty.is_empty() {
                                prop.ty = narrowed_ty;
                            }
                            prop.optional = false;
                        }
                    }
                    changed = true;
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                } else if *is_open || is_true {
                    // An open shape might still carry the key at runtime; and a
                    // closed shape genuinely missing the key is exactly the
                    // (falsy, offset-doesn't-exist) case `empty() === true`
                    // covers — either way, keep this arm unnarrowed.
                    result.add_type(atomic.clone());
                } else {
                    // A closed shape without this key can never satisfy
                    // `!empty(...)` — exclude this union member.
                    changed = true;
                }
            }
            _ => result.add_type(atomic.clone()),
        }
    }
    if changed && !result.types.is_empty() {
        set_shape_base_narrowed(ctx, &base, current, result);
    }
}

/// For `!empty($base['a']['b']...)`, narrow every level of the access path:
/// each level but the last is proven present (same as `narrow_shape_path`'s
/// `isset()` semantics), and the innermost key is additionally narrowed to
/// truthy. Mirrors `narrow_shape_path`'s recursion/exclusion structure.
fn narrow_not_empty_shape_path(ty: &Type, path: &[mir_types::atomic::ArrayKey]) -> Option<Type> {
    let (key, rest) = path.split_first()?;
    let is_last = rest.is_empty();
    let mut changed = false;
    let mut result = Type::empty();
    for atomic in &ty.types {
        match atomic {
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                if properties.contains_key(key) {
                    let mut new_props = properties.clone();
                    if let Some(prop) = new_props.get_mut(key) {
                        if is_last {
                            let narrowed_ty = prop.ty.narrow_to_truthy();
                            if !narrowed_ty.is_empty() {
                                prop.ty = narrowed_ty;
                            }
                        } else {
                            let mut narrowed_ty = prop.ty.remove_null();
                            if let Some(deeper) = narrow_not_empty_shape_path(&narrowed_ty, rest) {
                                narrowed_ty = deeper;
                            }
                            if !narrowed_ty.is_empty() {
                                prop.ty = narrowed_ty;
                            }
                        }
                        prop.optional = false;
                    }
                    changed = true;
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                } else if *is_open {
                    result.add_type(atomic.clone());
                } else {
                    // A closed shape without this key can never satisfy
                    // `!empty(...)` at every level — exclude this union member.
                    changed = true;
                }
            }
            _ => result.add_type(atomic.clone()),
        }
    }
    if changed && !result.types.is_empty() {
        Some(result)
    } else {
        None
    }
}

fn extract_null_coalesce(expr: &php_ast::owned::Expr) -> Option<&php_ast::owned::NullCoalesceExpr> {
    match &expr.kind {
        ExprKind::NullCoalesce(nc) => Some(nc),
        ExprKind::Parenthesized(inner) => extract_null_coalesce(inner),
        _ => None,
    }
}

fn same_literal(a: &php_ast::owned::Expr, b: &php_ast::owned::Expr) -> bool {
    let a = peel_parens(a);
    let b = peel_parens(b);
    match (&a.kind, &b.kind) {
        (ExprKind::Null, ExprKind::Null) => true,
        (ExprKind::Bool(a), ExprKind::Bool(b)) => a == b,
        (ExprKind::Int(a), ExprKind::Int(b)) => a == b,
        (ExprKind::String(a), ExprKind::String(b)) => a == b,
        _ => false,
    }
}

fn peel_parens(expr: &php_ast::owned::Expr) -> &php_ast::owned::Expr {
    match &expr.kind {
        ExprKind::Parenthesized(inner) => peel_parens(inner),
        _ => expr,
    }
}

/// `self`/`static`/`parent` are bare keywords, not real class names —
/// `db::resolve_name` deliberately returns them unresolved (it has no class
/// context), so they must be resolved to a concrete FQCN here, where the
/// surrounding class context (`self_fqcn`/`parent_fqcn`) is available.
/// `static` resolves to `self_fqcn` too: it's late-static-binding, so the
/// exact runtime class is unknown, but `self_fqcn` is its precise lower
/// bound — the same approximation `Atomic::TStaticObject` uses elsewhere.
fn extract_class_name(
    expr: &php_ast::owned::Expr,
    self_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => match name.to_ascii_lowercase().as_str() {
            "self" | "static" => self_fqcn.map(|s| s.to_string()),
            "parent" => parent_fqcn.map(|s| s.to_string()),
            _ => Some(name.to_string()),
        },
        ExprKind::Variable(name) if name.trim_start_matches('$') == "this" => {
            self_fqcn.map(|s| s.to_string())
        }
        ExprKind::Variable(_) => None, // dynamic class — can't narrow
        _ => None,
    }
}

fn extract_enum_case(
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

fn extract_class_const_fqcn(
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

/// Promote variables that were assigned as side effects of evaluating `expr`.
///
/// Called when we know `expr` was definitely evaluated (e.g., from the true-branch
/// of `&&` or the false-branch of `||`). Promotes variables that are in
/// `possibly_assigned_vars` up to `assigned_vars` if they appear as assignment
/// targets inside `expr`.
///
/// Conservative for internal short-circuit operators: only recurses into the
/// guaranteed-evaluated side (LHS) of nested `&&`/`||` sub-expressions, since
/// we cannot know whether the RHS of those was reached.
fn promote_assignment_effects(
    expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) {
    match &expr.kind {
        ExprKind::Assign(a) => {
            if let Some(var_name) = extract_var_name(&a.target) {
                let sym = mir_types::Name::from(var_name.as_str());
                if ctx.possibly_assigned_vars.contains(&sym) {
                    let ty = ctx.get_var(&var_name);
                    ctx.set_var(&var_name, ty);
                    std::sync::Arc::make_mut(&mut ctx.possibly_assigned_vars).remove(&sym);
                }
            }
            promote_assignment_effects(&a.value, ctx, db, file);
        }
        ExprKind::UnaryPrefix(u) => {
            promote_assignment_effects(&u.operand, ctx, db, file);
        }
        ExprKind::FunctionCall(call) => {
            // Promote variables that were assigned via by-ref parameters
            if let ExprKind::Identifier(fn_name) = &call.name.kind {
                let resolved = crate::db::resolve_name(db, file, fn_name.as_ref());
                let here = crate::db::Fqcn::from_str(db, &resolved);
                if let Some(func) = crate::db::find_function(db, here) {
                    for (i, param) in func.params.iter().enumerate() {
                        if param.is_byref {
                            let arg = call.args.get(i);
                            if let Some(arg) = arg {
                                if let ExprKind::Variable(name) = &arg.value.kind {
                                    let var_name = name.as_ref().trim_start_matches('$');
                                    let sym = mir_types::Name::from(var_name);
                                    if ctx.possibly_assigned_vars.contains(&sym) {
                                        let ty = ctx.get_var(var_name);
                                        ctx.set_var(var_name, ty);
                                        std::sync::Arc::make_mut(&mut ctx.possibly_assigned_vars)
                                            .remove(&sym);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            for arg in call.args.iter() {
                promote_assignment_effects(&arg.value, ctx, db, file);
            }
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            promote_assignment_effects(&mc.object, ctx, db, file);
            for arg in mc.args.iter() {
                promote_assignment_effects(&arg.value, ctx, db, file);
            }
        }
        ExprKind::StaticMethodCall(smc) => {
            for arg in smc.args.iter() {
                promote_assignment_effects(&arg.value, ctx, db, file);
            }
        }
        // For nested &&: LHS is always evaluated; RHS might short-circuit — only recurse LHS.
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanAnd || b.op == BinaryOp::LogicalAnd => {
            promote_assignment_effects(&b.left, ctx, db, file);
        }
        // For nested ||: LHS is always evaluated; RHS might short-circuit — only recurse LHS.
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            promote_assignment_effects(&b.left, ctx, db, file);
        }
        // For all other binary operators (===, !==, instanceof, +, etc.) both sides are evaluated.
        ExprKind::Binary(b) => {
            promote_assignment_effects(&b.left, ctx, db, file);
            promote_assignment_effects(&b.right, ctx, db, file);
        }
        ExprKind::Parenthesized(inner) => {
            promote_assignment_effects(inner, ctx, db, file);
        }
        // Array access: both base and index are evaluated; assignments inside either matter.
        ExprKind::ArrayAccess(aa) => {
            promote_assignment_effects(&aa.array, ctx, db, file);
            if let Some(idx) = &aa.index {
                promote_assignment_effects(idx, ctx, db, file);
            }
        }
        _ => {}
    }
}

fn extract_get_class_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("get_class")
            {
                if let Some(arg) = call.args.first() {
                    return ScalarArgTarget::extract(&arg.value);
                }
            }
        }
    }
    None
}

/// A `gettype()`/`get_debug_type()` argument, resolved to either a plain
/// variable or a `$obj->prop` property access — lets a single literal-mapping
/// function (`narrow_from_gettype_literal`/`narrow_from_get_debug_type_literal`)
/// dispatch to the right narrowing entry point (`narrow_from_type_fn` vs
/// `narrow_prop_from_type_fn`/`narrow_prop_to_specific_class`) for either
/// receiver shape.
enum ScalarArgTarget {
    Var(String),
    Prop(String, String),
}

impl ScalarArgTarget {
    fn extract(expr: &php_ast::owned::Expr) -> Option<Self> {
        if let Some(name) = extract_var_name(expr) {
            return Some(ScalarArgTarget::Var(name));
        }
        extract_prop_access(expr).map(|(obj, prop)| ScalarArgTarget::Prop(obj, prop))
    }
}

fn extract_gettype_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("gettype")
            {
                if let Some(arg) = call.args.first() {
                    return ScalarArgTarget::extract(&arg.value);
                }
            }
        }
    }
    None
}

fn extract_get_debug_type_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("get_debug_type")
            {
                if let Some(arg) = call.args.first() {
                    return ScalarArgTarget::extract(&arg.value);
                }
            }
        }
    }
    None
}

fn extract_get_parent_class_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("get_parent_class")
            {
                if let Some(arg) = call.args.first() {
                    return ScalarArgTarget::extract(&arg.value);
                }
            }
        }
    }
    None
}

/// Extract the receiver from `class_implements($x)`/`class_parents($x)` —
/// both return an array keyed (and valued) by interface/ancestor-class name,
/// so `array_key_exists('IfaceOrAncestor', class_implements($x))` proves `$x`
/// an instance of that interface/ancestor, the same relationship
/// `$x instanceof IfaceOrAncestor` proves.
fn extract_class_implements_or_parents_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            let bare = name.trim_start_matches('\\');
            if bare.eq_ignore_ascii_case("class_implements")
                || bare.eq_ignore_ascii_case("class_parents")
            {
                if let Some(arg) = call.args.first() {
                    return ScalarArgTarget::extract(&arg.value);
                }
            }
        }
    }
    None
}

/// Narrow `$x`/`$this->prop` from `get_parent_class(...) === 'ClassName'` (or
/// `=== ClassName::class`): the receiver's class's immediate parent being
/// exactly `fqcn` proves the receiver is a strict subclass instance of
/// `fqcn` — the same relationship `is_subclass_of($x, ClassName::class)`
/// proves, so this reuses that narrowing (`narrow_strict_subclass_of`/
/// `narrow_prop_is_subclass_of`) rather than duplicating it. Like
/// `is_subclass_of`, the false branch narrows nothing: a parent name other
/// than `fqcn` doesn't rule out the receiver being exactly `fqcn` itself.
fn narrow_from_get_parent_class_literal(
    ctx: &mut FlowState,
    target: &ScalarArgTarget,
    fqcn: &str,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    match target {
        ScalarArgTarget::Var(var_name) => {
            if is_true {
                let current = ctx.get_var(var_name);
                let narrowed =
                    narrow_strict_subclass_of(&current, fqcn, db, &ctx.template_param_names);
                set_narrowed(ctx, var_name, &current, narrowed, false);
            }
        }
        ScalarArgTarget::Prop(obj, prop) => {
            narrow_prop_is_subclass_of(ctx, obj, prop, fqcn, db, file, is_true);
            narrow_receiver_non_null_on_prop_match(ctx, obj, is_true);
        }
    }
}

/// Extract the receiver (variable or property access) from `$obj::class` /
/// `$this->obj::class` — PHP 8's `get_class($obj)` equivalent, parsed as a
/// `ClassConstAccess` whose class side is an expression rather than a
/// static class-name identifier.
fn extract_dynamic_class_const_var(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::ClassConstAccess(cca) = &expr.kind {
        if matches!(&cca.member.kind, ExprKind::Identifier(n) if n.as_ref() == "class") {
            return ScalarArgTarget::extract(&cca.class);
        }
    }
    None
}

/// Narrow `$x`/`$this->prop` from `gettype(...) === 'literal'`, mapping
/// `gettype()`'s fixed set of return strings to the equivalent `is_TYPE()`
/// narrowing on whichever receiver shape `target` resolved to.
fn narrow_from_gettype_literal(
    ctx: &mut FlowState,
    target: &ScalarArgTarget,
    literal: &str,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    let type_fn = match literal {
        "boolean" => "is_bool",
        "integer" => "is_int",
        "double" => "is_float",
        "string" => "is_string",
        "array" => "is_array",
        "object" => "is_object",
        "NULL" => "is_null",
        "resource" | "resource (closed)" => "is_resource",
        _ => return,
    };
    match target {
        ScalarArgTarget::Var(var_name) => narrow_from_type_fn(ctx, type_fn, var_name, db, is_true),
        ScalarArgTarget::Prop(obj, prop) => {
            narrow_prop_from_type_fn(ctx, type_fn, obj, prop, db, file, is_true)
        }
    }
}

/// Narrow `$x`/`$this->prop` from `get_debug_type(...) === 'literal'`. Unlike
/// `gettype()`, `get_debug_type()`'s scalar names are lowercase and don't
/// cover `object` (it returns the actual class name instead), so anything
/// outside its fixed scalar set is treated as an exact class name — same
/// semantics as `get_class($x) === 'literal'` above.
fn narrow_from_get_debug_type_literal(
    ctx: &mut FlowState,
    target: &ScalarArgTarget,
    literal: &str,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    let type_fn = match literal {
        "null" => Some("is_null"),
        "bool" => Some("is_bool"),
        "int" => Some("is_int"),
        "float" => Some("is_float"),
        "string" => Some("is_string"),
        "array" => Some("is_array"),
        "resource" | "resource (closed)" => Some("is_resource"),
        _ => None,
    };
    if let Some(type_fn) = type_fn {
        match target {
            ScalarArgTarget::Var(var_name) => {
                narrow_from_type_fn(ctx, type_fn, var_name, db, is_true)
            }
            ScalarArgTarget::Prop(obj, prop) => {
                narrow_prop_from_type_fn(ctx, type_fn, obj, prop, db, file, is_true)
            }
        }
    } else {
        let fqcn = crate::db::resolve_name(db, file, literal);
        match target {
            ScalarArgTarget::Var(var_name) => {
                narrow_var_to_specific_class(ctx, var_name, &fqcn, is_true, db)
            }
            ScalarArgTarget::Prop(obj, prop) => {
                narrow_prop_to_specific_class(ctx, obj, prop, &fqcn, is_true, db, file);
                narrow_receiver_non_null_on_prop_match(ctx, obj, is_true);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Extension methods on Type used only in narrowing
// ---------------------------------------------------------------------------

trait UnionNarrowExt {
    fn filter<F: Fn(&Atomic) -> bool>(&self, f: F) -> Type;
}

impl UnionNarrowExt for Type {
    fn filter<F: Fn(&Atomic) -> bool>(&self, f: F) -> Type {
        let mut result = Type::empty();
        result.possibly_undefined = self.possibly_undefined;
        result.from_docblock = self.from_docblock;
        for atomic in &self.types {
            if f(atomic) {
                result.types.push(atomic.clone());
            }
        }
        result
    }
}

pub(crate) fn is_numeric_string(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    // Rust's f64 parser accepts "inf"/"nan"/"infinity" (case-insensitively,
    // optionally signed) as valid floats, but PHP's is_numeric() does not —
    // reject non-finite results so e.g. "NAN" isn't treated as numeric.
    t.parse::<i64>().is_ok() || t.parse::<f64>().is_ok_and(f64::is_finite)
}

/// Extract the variable/property target from `count($var)` / `sizeof($var)` /
/// `iterator_count($var)` — all three return an int length and narrow
/// identically.
fn extract_count_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("count")
            || bare.eq_ignore_ascii_case("sizeof")
            || bare.eq_ignore_ascii_case("iterator_count")
        {
            if let Some(arg) = call.args.first() {
                return ScalarArgTarget::extract(&arg.value);
            }
        }
    }
    None
}

/// Extract the variable/property target from `array_key_first($var)` /
/// `array_key_last($var)`.
fn extract_array_key_first_or_last_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("array_key_first")
            || bare.eq_ignore_ascii_case("array_key_last")
        {
            if let Some(arg) = call.args.first() {
                return ScalarArgTarget::extract(&arg.value);
            }
        }
    }
    None
}

/// `array_key_first($arr) !== null` / `array_key_last($arr) !== null` — a common
/// non-empty-array idiom, equivalent to `count($arr) > 0`. Both functions return
/// `null` only when the array is empty, so `!== null` proves it's non-empty and
/// `=== null` proves it's empty.
fn narrow_array_key_first_or_last_null(ctx: &mut FlowState, arr_var: &str, is_null: bool) {
    let current = ctx.get_var(arr_var);
    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_empty_collection()
    } else {
        current.narrow_to_non_empty_collection()
    };
    // `narrow_to_empty_collection` can filter every atom away when `current` is
    // already known to be exclusively non-empty (a provably-dead branch); leave
    // the type as-is rather than collapsing the variable to an empty union.
    if !narrowed.is_empty() && narrowed != current {
        ctx.set_var(arr_var, narrowed);
    }
}

/// Property-access counterpart of `narrow_array_key_first_or_last_null`.
fn narrow_prop_array_key_first_or_last_null(
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
        current.narrow_to_empty_collection()
    } else {
        current.narrow_to_non_empty_collection()
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

/// Extract the variable/property target from `strlen($var)` / `mb_strlen($var, ...)`.
fn extract_strlen_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("strlen") || bare.eq_ignore_ascii_case("mb_strlen") {
            if let Some(arg) = call.args.first() {
                return ScalarArgTarget::extract(&arg.value);
            }
        }
    }
    None
}

/// `count($arr) op N` / `strlen($str) op N` for the equality operators
/// (`===`, `!==`, `==`, `!=`) — the `<`/`<=`/`>`/`>=` forms are normalized
/// and handled inline where those operators are matched; equality is
/// symmetric so, unlike that relational-operator normalization, no operator
/// flip is needed when the call is on the right-hand side.
fn narrow_count_or_strlen_equality(
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    op: BinaryOp,
    is_true: bool,
) {
    let (count_expr, count_lit) = if extract_count_arg(left).is_some() {
        (left, right)
    } else {
        (right, left)
    };
    if let (Some(target), Some(n)) = (
        extract_count_arg(count_expr),
        extract_int_literal(count_lit),
    ) {
        match target {
            ScalarArgTarget::Var(arr_var) => {
                narrow_array_count_comparison(ctx, &arr_var, op, n, is_true)
            }
            ScalarArgTarget::Prop(obj, prop) => {
                narrow_prop_array_count_comparison(ctx, &obj, &prop, db, file, op, n, is_true)
            }
        }
        return;
    }
    let (strlen_expr, strlen_lit) = if extract_strlen_arg(left).is_some() {
        (left, right)
    } else {
        (right, left)
    };
    if let (Some(target), Some(n)) = (
        extract_strlen_arg(strlen_expr),
        extract_int_literal(strlen_lit),
    ) {
        match target {
            ScalarArgTarget::Var(str_var) => {
                narrow_string_strlen_comparison(ctx, &str_var, op, n, is_true)
            }
            ScalarArgTarget::Prop(obj, prop) => {
                narrow_prop_string_strlen_comparison(ctx, &obj, &prop, db, file, op, n, is_true)
            }
        }
    }
}

/// Whether `count()`/`strlen() op n` being `is_true` proves the underlying
/// collection/string is non-empty (`Some(true)`) or empty (`Some(false)`);
/// `None` if the comparison proves neither (count/strlen is always >= 0, so
/// e.g. `count($x) < 5` proves nothing about emptiness either way).
fn count_or_strlen_emptiness(op: BinaryOp, n: i64, is_true: bool) -> Option<bool> {
    match (op, is_true) {
        (BinaryOp::Greater, true) if n >= 0 => Some(true), // len > 0 (or > n>=0)
        (BinaryOp::GreaterOrEqual, true) if n >= 1 => Some(true), // len >= 1
        (BinaryOp::Less, false) if n >= 1 => Some(true),   // NOT (len < 1)
        (BinaryOp::LessOrEqual, false) if n >= 0 => Some(true), // NOT (len <= 0)
        // len === N / == N, true, for a positive N: an exact positive length is non-empty.
        (BinaryOp::Identical | BinaryOp::Equal, true) if n >= 1 => Some(true),
        // len === 0 / == 0, false: length is proven not zero.
        (BinaryOp::Identical | BinaryOp::Equal, false) if n == 0 => Some(true),
        // len !== 0 / != 0, true: length is proven not zero.
        (BinaryOp::NotIdentical | BinaryOp::NotEqual, true) if n == 0 => Some(true),
        // len !== N / != N, false, for a positive N: length equals that N exactly.
        (BinaryOp::NotIdentical | BinaryOp::NotEqual, false) if n >= 1 => Some(true),
        // Mirror image: len < n / len <= n / NOT(len >= n) / NOT(len > n) with
        // n small enough that, combined with len >= 0, length must be exactly 0.
        (BinaryOp::Less, true) if n <= 1 => Some(false),
        (BinaryOp::LessOrEqual, true) if n <= 0 => Some(false),
        (BinaryOp::GreaterOrEqual, false) if n <= 1 => Some(false),
        (BinaryOp::Greater, false) if n <= 0 => Some(false),
        // len === 0 / == 0, true: length is proven exactly zero.
        (BinaryOp::Identical | BinaryOp::Equal, true) if n == 0 => Some(false),
        // len !== N / != N, false, for N == 0: length equals zero exactly.
        (BinaryOp::NotIdentical | BinaryOp::NotEqual, false) if n == 0 => Some(false),
        _ => None,
    }
}

/// Narrow an array variable based on `count($arr) op n` being `is_true`.
/// Promotes `array` / `list` to their non-empty variants when the comparison
/// proves the count is >= 1, or drops the non-empty variants when it proves
/// the count is exactly 0.
fn narrow_array_count_comparison(
    ctx: &mut FlowState,
    arr_var: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some(non_empty) = count_or_strlen_emptiness(op, n, is_true) else {
        return;
    };
    let current = ctx.get_var(arr_var);
    if current.is_mixed() {
        return;
    }
    let narrowed = if non_empty {
        current.narrow_to_non_empty_collection()
    } else {
        current.narrow_to_empty_collection()
    };
    // `narrow_to_empty_collection` can filter every atom away when `current` is
    // already known to be exclusively non-empty (a provably-dead branch); leave
    // the type as-is rather than collapsing the variable to an empty union.
    if !narrowed.is_empty() && narrowed != current {
        ctx.set_var(arr_var, narrowed);
    }
}

/// Narrow a string variable based on `strlen($str) op n` being `is_true`.
/// Promotes `string` to `non-empty-string` when the comparison proves length
/// >= 1, or drops `non-empty-string` when it proves length is exactly 0.
fn narrow_string_strlen_comparison(
    ctx: &mut FlowState,
    str_var: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some(non_empty) = count_or_strlen_emptiness(op, n, is_true) else {
        return;
    };
    let current = ctx.get_var(str_var);
    if current.is_mixed() {
        return;
    }
    let narrowed = if non_empty {
        narrow_string_to_non_empty(&current)
    } else {
        narrow_string_to_empty(&current)
    };
    // Same rationale as the array case above: don't collapse to an empty union.
    if !narrowed.is_empty() && narrowed != current {
        ctx.set_var(str_var, narrowed);
    }
}

/// Property-access counterpart of `narrow_array_count_comparison`.
#[allow(clippy::too_many_arguments)]
fn narrow_prop_array_count_comparison(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some(non_empty) = count_or_strlen_emptiness(op, n, is_true) else {
        return;
    };
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    let narrowed = if non_empty {
        current.narrow_to_non_empty_collection()
    } else {
        current.narrow_to_empty_collection()
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

/// Property-access counterpart of `narrow_string_strlen_comparison`.
#[allow(clippy::too_many_arguments)]
fn narrow_prop_string_strlen_comparison(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let Some(non_empty) = count_or_strlen_emptiness(op, n, is_true) else {
        return;
    };
    let current = resolve_prop_current_type(ctx, obj_var, prop, db, file);
    if current.is_mixed() {
        return;
    }
    let narrowed = if non_empty {
        narrow_string_to_non_empty(&current)
    } else {
        narrow_string_to_empty(&current)
    };
    apply_prop_narrowed(ctx, obj_var, prop, current, narrowed, false);
}

/// Extract a union Type from an `in_array` haystack argument.
/// Supports:
/// - Literal arrays: `['a', 'b', 1]` → union of `TLiteralString` / `TLiteralInt`
/// - Variables: look up from ctx and collect the TLiteralString/TLiteralInt values
///   inside the TKeyedArray's properties.
fn extract_haystack_type(expr: &php_ast::owned::Expr, ctx: &FlowState) -> Option<Type> {
    match &expr.kind {
        ExprKind::Array(elements) => {
            let mut ty = Type::empty();
            for item in elements.iter() {
                match &item.value.kind {
                    ExprKind::String(s) => {
                        ty.add_type(Atomic::TLiteralString(std::sync::Arc::from(s.as_ref())))
                    }
                    ExprKind::Int(n) => ty.add_type(Atomic::TLiteralInt(*n)),
                    _ => return None, // non-literal element — bail out
                }
            }
            if ty.is_empty() {
                None
            } else {
                Some(ty)
            }
        }
        ExprKind::Variable(name) => {
            let var_name = name.trim_start_matches('$');
            let var_ty = ctx.get_var(var_name);
            if var_ty.is_mixed() || var_ty.is_empty() {
                return None;
            }
            let mut ty = Type::empty();
            for atomic in &var_ty.types {
                match atomic {
                    Atomic::TKeyedArray { properties, .. } => {
                        for prop in properties.values() {
                            match &prop.ty.types[..] {
                                [Atomic::TLiteralString(_)] | [Atomic::TLiteralInt(_)] => {
                                    for a in &prop.ty.types {
                                        ty.add_type(a.clone());
                                    }
                                }
                                _ => return None, // non-literal value
                            }
                        }
                    }
                    _ => return None,
                }
            }
            if ty.is_empty() {
                None
            } else {
                Some(ty)
            }
        }
        ExprKind::Parenthesized(inner) => extract_haystack_type(inner, ctx),
        _ => None,
    }
}

/// Narrow `current` to only the atomic types that overlap with `haystack` literals.
/// For each literal atom in `haystack` (TLiteralString / TLiteralInt): keep it in
/// the output if `current` could hold that value — i.e., the literal is a subtype
/// of at least one atom in `current`.
fn narrow_to_haystack_values(current: &Type, haystack: &Type) -> Type {
    let mut out = Type::empty();
    for hay_atom in &haystack.types {
        let lit_ty = Type::single(hay_atom.clone());
        if lit_ty.is_subtype_structural(current) {
            out.add_type(hay_atom.clone());
        }
    }
    out
}

/// Whether narrowing `current` by an `in_array()`/`!in_array()` check against
/// `haystack` is sound without a strict (third-argument) comparison. Loose
/// (`==`) comparison agrees with strict (`===`) comparison whenever both
/// sides are exclusively strings, or exclusively ints — cross-category
/// comparisons (e.g. a string needle against an int haystack) can match via
/// PHP's loose-equality coercion rules in ways a same-category narrowing
/// would incorrectly rule out.
fn in_array_loose_narrowing_is_safe(current: &Type, haystack: &Type) -> bool {
    fn all(ty: &Type, pred: fn(&Atomic) -> bool) -> bool {
        !ty.types.is_empty() && ty.types.iter().all(pred)
    }
    (all(current, Atomic::is_int) && all(haystack, Atomic::is_int))
        || (all(current, Atomic::is_string) && all(haystack, Atomic::is_string))
}

#[cfg(test)]
mod tests {
    use super::is_numeric_string;

    #[test]
    fn numeric_strings_are_recognized() {
        assert!(is_numeric_string("42"));
        assert!(is_numeric_string("-42"));
        assert!(is_numeric_string("+42"));
        assert!(is_numeric_string("3.14"));
        assert!(is_numeric_string(".5"));
        assert!(is_numeric_string("1e10"));
        assert!(is_numeric_string("  123  "));
    }

    #[test]
    fn non_numeric_strings_are_rejected() {
        assert!(!is_numeric_string(""));
        assert!(!is_numeric_string("   "));
        assert!(!is_numeric_string("hello"));
        assert!(!is_numeric_string("0x1A"));
        assert!(!is_numeric_string("12abc"));
    }

    #[test]
    fn nan_and_infinity_keywords_are_not_numeric() {
        // PHP's is_numeric() rejects these, unlike Rust's f64 parser.
        assert!(!is_numeric_string("NAN"));
        assert!(!is_numeric_string("nan"));
        assert!(!is_numeric_string("INF"));
        assert!(!is_numeric_string("-INF"));
        assert!(!is_numeric_string("Infinity"));
        assert!(!is_numeric_string("-Infinity"));
    }
}
