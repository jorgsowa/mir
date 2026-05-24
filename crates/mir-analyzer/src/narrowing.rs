/// Type narrowing — refines variable types based on conditional expressions.
///
/// Given a condition expression and a branch direction (true/false), this
/// module updates the `Context` to narrow variable types accordingly.
use php_ast::ast::{AssignOp, BinaryOp, UnaryPrefixOp};
use php_ast::owned::ExprKind;

use mir_codebase::storage::AssertionKind;
use mir_types::{Atomic, Union};

use crate::context::Context;
use crate::db::MirDatabase;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Narrow the types in `ctx` as if `expr` evaluates to `is_true`.
pub fn narrow_from_condition(
    expr: &php_ast::owned::Expr,
    ctx: &mut Context,
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
            }
        }

        // $a || $b  →  if false: narrow both; if true: try to narrow same-var instanceof union
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            if !is_true {
                narrow_from_condition(&b.left, ctx, false, db, file);
                narrow_from_condition(&b.right, ctx, false, db, file);
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

            // `$x === null`
            if matches!(b.right.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_null(ctx, &name, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_null(ctx, &name, effective_true);
                }
            }
            // `$x === true` / `$x === false`
            else if matches!(b.right.kind, ExprKind::Bool(true)) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_bool(ctx, &name, true, effective_true);
                }
            } else if matches!(b.right.kind, ExprKind::Bool(false)) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_bool(ctx, &name, false, effective_true);
                }
            }
            // `get_class($x) === 'ClassName'` — check before literal strings so it takes precedence
            else if let ExprKind::String(class_name_str) = &b.right.kind {
                if let Some(obj_var_name) = extract_get_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name_via_db(db, file, class_name_str.as_ref());
                    narrow_var_to_specific_class(ctx, &obj_var_name, &fqcn, effective_true);
                } else if let Some(name) = extract_var_name(&b.left) {
                    // `$x === 'literal'`
                    narrow_var_literal_string(ctx, &name, class_name_str, effective_true);
                }
            } else if let ExprKind::String(class_name_str) = &b.left.kind {
                if let Some(obj_var_name) = extract_get_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name_via_db(db, file, class_name_str.as_ref());
                    narrow_var_to_specific_class(ctx, &obj_var_name, &fqcn, effective_true);
                } else if let Some(name) = extract_var_name(&b.right) {
                    // `$x === 'literal'`
                    narrow_var_literal_string(ctx, &name, class_name_str, effective_true);
                }
            }
            // `$x === 42`
            else if let ExprKind::Int(n) = &b.right.kind {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_literal_int(ctx, &name, *n, effective_true);
                }
            } else if let ExprKind::Int(n) = &b.left.kind {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_literal_int(ctx, &name, *n, effective_true);
                }
            }
            // `$x === EnumName::CaseName`
            else if let ExprKind::StaticPropertyAccess(_) = &b.right.kind {
                if let Some(var_name) = extract_var_name(&b.left) {
                    if let Some((enum_fqcn, case_name)) =
                        extract_enum_case(&b.right, ctx.self_fqcn.as_deref(), db, file)
                    {
                        narrow_var_to_literal_enum_case(
                            ctx,
                            &var_name,
                            &enum_fqcn,
                            &case_name,
                            effective_true,
                        );
                    }
                }
            } else if let ExprKind::StaticPropertyAccess(_) = &b.left.kind {
                if let Some(var_name) = extract_var_name(&b.right) {
                    if let Some((enum_fqcn, case_name)) =
                        extract_enum_case(&b.left, ctx.self_fqcn.as_deref(), db, file)
                    {
                        narrow_var_to_literal_enum_case(
                            ctx,
                            &var_name,
                            &enum_fqcn,
                            &case_name,
                            effective_true,
                        );
                    }
                }
            }
            // `$x === SomeClass::class`
            else if let ExprKind::ClassConstAccess(cca) = &b.right.kind {
                if let Some(var_name) = extract_var_name(&b.left) {
                    if let Some(fqcn) =
                        extract_class_const_fqcn(cca, ctx.self_fqcn.as_deref(), db, file)
                    {
                        narrow_var_to_class_string(ctx, &var_name, &fqcn, effective_true);
                    }
                }
            } else if let ExprKind::ClassConstAccess(cca) = &b.left.kind {
                if let Some(var_name) = extract_var_name(&b.right) {
                    if let Some(fqcn) =
                        extract_class_const_fqcn(cca, ctx.self_fqcn.as_deref(), db, file)
                    {
                        narrow_var_to_class_string(ctx, &var_name, &fqcn, effective_true);
                    }
                }
            }
        }

        // $x == null  (loose equality)
        ExprKind::Binary(b) if b.op == BinaryOp::Equal || b.op == BinaryOp::NotEqual => {
            let is_equal = b.op == BinaryOp::Equal;
            let effective_true = if is_equal { is_true } else { !is_true };
            if matches!(b.right.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_null(ctx, &name, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_null(ctx, &name, effective_true);
                }
            }
        }

        // $x instanceof ClassName
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            if let Some(var_name) = extract_var_name(&b.left) {
                if let Some(raw_name) = extract_class_name(&b.right, ctx.self_fqcn.as_deref()) {
                    // Resolve the short name to its FQCN using file imports
                    let class_name = crate::db::resolve_name_via_db(db, file, &raw_name);
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
            }
        }

        // is_string($x), is_int($x), is_null($x), is_array($x), etc.
        // Also handles assert($x instanceof Y) — narrows like a bare condition.
        ExprKind::FunctionCall(call) => {
            let fn_name_opt: Option<&str> = match &call.name.kind {
                ExprKind::Identifier(name) => Some(name.as_ref()),
                ExprKind::Variable(name) => Some(name.as_ref()),
                _ => None,
            };
            if let Some(fn_name) = fn_name_opt {
                if fn_name.eq_ignore_ascii_case("assert") {
                    // assert($condition) — narrow as if the condition is is_true
                    if let Some(arg_expr) = call.args.first() {
                        narrow_from_condition(&arg_expr.value, ctx, is_true, db, file);
                    }
                } else if apply_docblock_assertions(call, ctx, is_true, db, file, fn_name) {
                    // User-defined assertion applied.
                } else if let Some(arg_expr) = call.args.first() {
                    if let Some(var_name) = extract_var_name(&arg_expr.value) {
                        narrow_from_type_fn(ctx, fn_name, &var_name, is_true);
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
                            .insert(mir_types::Symbol::from(var_name.as_str()));
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
            }
        }

        // ($x = expr) / ($x ??= expr) used as a condition
        // The assignment has already been evaluated (ctx holds the post-assignment type).
        // Narrow the target variable based on the truthiness of the expression result.
        ExprKind::Assign(a) if matches!(a.op, AssignOp::Assign | AssignOp::Coalesce) => {
            if let Some(var_name) = extract_var_name(&a.target) {
                let current = ctx.get_var(&var_name);
                let narrowed = if is_true {
                    current.narrow_to_truthy()
                } else {
                    current.narrow_to_falsy()
                };
                if !narrowed.is_empty() {
                    ctx.set_var(&var_name, narrowed);
                } else if !current.is_empty() && !current.is_mixed() {
                    ctx.diverges = true;
                }
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
                } else if !current.is_empty() && !current.is_mixed() {
                    // The variable's type can never satisfy this truthiness
                    // constraint → this branch is statically unreachable.
                    ctx.diverges = true;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn apply_docblock_assertions(
    call: &php_ast::owned::FunctionCallExpr,
    ctx: &mut Context,
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
        let qualified = crate::db::resolve_name_via_db(db, file, &fn_name);
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

    let mut applied = false;
    for assertion in assertions
        .iter()
        .filter(|a| a.kind == expected_kind || (is_true && a.kind == AssertionKind::Assert))
    {
        if let Some(index) = params.iter().position(|p| p.name == assertion.param) {
            if let Some(arg) = call.args.get(index) {
                if let Some(var_name) = extract_var_name(&arg.value) {
                    ctx.set_var(&var_name, assertion.ty.clone());
                    applied = true;
                }
            }
        }
    }

    applied
}

/// For `$x instanceof A || $x instanceof B` (true branch): narrow $x to A|B.
/// Handles OR chains recursively, e.g. `$x instanceof A || $x instanceof B || $x instanceof C`.
fn narrow_or_instanceof_true(
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    ctx: &mut Context,
    db: &dyn MirDatabase,
    file: &str,
) {
    let self_fqcn = ctx.self_fqcn.as_deref();

    // Collect all class names from instanceof checks on the same variable.
    let mut var_name: Option<String> = None;
    let mut class_names: Vec<String> = vec![];

    fn collect_instanceof(
        expr: &php_ast::owned::Expr,
        var_name: &mut Option<String>,
        class_names: &mut Vec<String>,
        db: &dyn MirDatabase,
        file: &str,
        self_fqcn: Option<&str>,
    ) -> bool {
        match &expr.kind {
            ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
                if let (Some(vn), Some(cn)) = (
                    extract_var_name(&b.left),
                    extract_class_name(&b.right, self_fqcn),
                ) {
                    let resolved = crate::db::resolve_name_via_db(db, file, &cn);
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
                collect_instanceof(&b.left, var_name, class_names, db, file, self_fqcn)
                    && collect_instanceof(&b.right, var_name, class_names, db, file, self_fqcn)
            }
            ExprKind::Parenthesized(inner) => {
                collect_instanceof(inner, var_name, class_names, db, file, self_fqcn)
            }
            _ => false,
        }
    }

    // Wrap left and right into a fake OR so we can reuse the collector
    let left_ok = collect_instanceof(left, &mut var_name, &mut class_names, db, file, self_fqcn);
    let right_ok = collect_instanceof(right, &mut var_name, &mut class_names, db, file, self_fqcn);

    if left_ok && right_ok {
        if let Some(vn) = var_name {
            if !class_names.is_empty() {
                let current = ctx.get_var(&vn);
                // Narrow to the union of all instanceof types: take union of narrow_instanceof results
                let mut narrowed = Union::empty();
                for cn in &class_names {
                    let n = narrow_instanceof_preserving_subtypes(
                        &current,
                        cn,
                        db,
                        &ctx.template_param_names,
                    );
                    narrowed.merge_with(&n);
                }
                // Fall back to current if narrowed is empty (e.g. mixed)
                let result = if narrowed.is_empty() {
                    current.clone()
                } else {
                    narrowed
                };
                if !result.is_empty() {
                    ctx.set_var(&vn, result);
                }
            }
        }
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
    ctx: &mut Context,
    db: &dyn MirDatabase,
    file: &str,
) {
    // Pattern: !isset($x) || RHS
    // When RHS is evaluated via short-circuit, !isset($x) is false, so isset($x) is true
    if let ExprKind::UnaryPrefix(u) = &left.kind {
        if u.op == UnaryPrefixOp::BooleanNot {
            if let ExprKind::Isset(vars) = &u.operand.kind {
                // Save original variable states so narrowing only affects RHS analysis
                let original_vars: Vec<_> = vars
                    .iter()
                    .filter_map(|var_expr| {
                        extract_var_name(var_expr).map(|name| (name.clone(), ctx.get_var(&name)))
                    })
                    .collect();

                // Apply isset narrowing: remove null and mark as definitely assigned
                for var_expr in vars.iter() {
                    if let Some(var_name) = extract_var_name(var_expr) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                        std::sync::Arc::make_mut(&mut ctx.assigned_vars)
                            .insert(mir_types::Symbol::from(var_name.as_str()));
                    }
                }

                // Evaluate RHS with narrowed context
                narrow_from_condition(right, ctx, true, db, file);

                // Restore original variable states for if-body context
                for (var_name, original_type) in original_vars {
                    ctx.set_var(&var_name, original_type);
                }
            }
        }
    }
}

fn narrow_instanceof_preserving_subtypes(
    current: &Union,
    class_name: &str,
    db: &dyn MirDatabase,
    template_param_names: &rustc_hash::FxHashSet<mir_types::Symbol>,
) -> Union {
    let narrowed_ty = Atomic::TNamedObject {
        fqcn: class_name.into(),
        type_params: mir_types::union::empty_type_params(),
    };

    if current.is_empty() || current.is_mixed() {
        return Union::single(narrowed_ty);
    }

    let mut result = Union::empty();
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
            // treat it as matching any typeof and keep it in the result (it represents the narrowed bound)
            Atomic::TNamedObject { fqcn, type_params }
                if type_params.is_empty()
                    && !fqcn.contains('\\')
                    && template_param_names.contains(fqcn) =>
            {
                // Keep the template parameter in the result — it will be constrained by the instanceof check
                result.add_type(narrowed_ty.clone());
            }
            // Handle TTemplateParam: narrow it to the instanceof check class
            Atomic::TTemplateParam { .. } => {
                result.add_type(narrowed_ty.clone());
            }
            Atomic::TObject | Atomic::TMixed => result.add_type(narrowed_ty.clone()),
            _ => {}
        }
    }

    if result.is_empty() {
        Union::single(narrowed_ty)
    } else {
        result
    }
}

fn filter_out_instanceof_match(current: &Union, class_name: &str, db: &dyn MirDatabase) -> Union {
    current.filter(|t| match t {
        Atomic::TNamedObject { fqcn, .. }
        | Atomic::TSelf { fqcn }
        | Atomic::TStaticObject { fqcn }
        | Atomic::TParent { fqcn } => !named_object_matches_instanceof(fqcn, class_name, db),
        _ => true,
    })
}

fn named_object_matches_instanceof(fqcn: &str, class_name: &str, db: &dyn MirDatabase) -> bool {
    fqcn == class_name || crate::db::extends_or_implements_via_db(db, fqcn, class_name)
}

/// Apply a pre-computed narrowed type to a variable.
///
/// If `mark_diverges` is true and the narrowed type is empty (the current type
/// can never satisfy the constraint), the branch is marked unreachable.
fn set_narrowed(
    ctx: &mut Context,
    name: &str,
    current: &Union,
    narrowed: Union,
    mark_diverges: bool,
) {
    if !narrowed.is_empty() {
        ctx.set_var(name, narrowed);
    } else if mark_diverges && !current.is_empty() && !current.is_mixed() {
        ctx.diverges = true;
    }
}

fn narrow_var_null(ctx: &mut Context, name: &str, is_null: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

fn narrow_var_bool(ctx: &mut Context, name: &str, value: bool, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_value {
        if value {
            current.filter(|t| matches!(t, Atomic::TTrue | Atomic::TBool | Atomic::TMixed))
        } else {
            current.filter(|t| matches!(t, Atomic::TFalse | Atomic::TBool | Atomic::TMixed))
        }
    } else if value {
        current.filter(|t| !matches!(t, Atomic::TTrue))
    } else {
        current.filter(|t| !matches!(t, Atomic::TFalse))
    };
    set_narrowed(ctx, name, &current, narrowed, false);
}

fn narrow_from_type_fn(ctx: &mut Context, fn_name: &str, var_name: &str, is_true: bool) {
    let current = ctx.get_var(var_name);
    let narrowed = match fn_name.to_lowercase().as_str() {
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
                current.filter(|t| !matches!(t, Atomic::TFloat | Atomic::TLiteralFloat(..)))
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
                    !matches!(
                        t,
                        Atomic::TString
                            | Atomic::TLiteralString(..)
                            | Atomic::TNumericString
                            | Atomic::TInt
                            | Atomic::TLiteralInt(..)
                            | Atomic::TFloat
                            | Atomic::TLiteralFloat(..)
                            | Atomic::TBool
                            | Atomic::TTrue
                            | Atomic::TFalse
                            | Atomic::TScalar
                    )
                })
            }
        }
        "is_iterable" => {
            if is_true {
                current.narrow_to_iterable()
            } else {
                current.filter(|t| !t.is_array() && !t.is_object())
            }
        }
        "is_countable" => {
            if is_true {
                current.narrow_to_countable()
            } else {
                current.filter(|t| !t.is_array() && !t.is_object())
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
                current.filter(|t| {
                    matches!(
                        t,
                        Atomic::TInt
                            | Atomic::TFloat
                            | Atomic::TNumeric
                            | Atomic::TNumericString
                            | Atomic::TLiteralInt(_)
                            | Atomic::TMixed
                    )
                })
            } else {
                current.filter(|t| {
                    !matches!(
                        t,
                        Atomic::TInt
                            | Atomic::TFloat
                            | Atomic::TNumeric
                            | Atomic::TNumericString
                            | Atomic::TLiteralInt(_)
                    )
                })
            }
        }
        // method_exists($obj, 'method') — if true, narrow to TObject (suppresses
        // UndefinedMethod; the concrete type is unresolvable without knowing the method arg)
        "method_exists" | "property_exists" => {
            if is_true {
                Union::single(Atomic::TObject)
            } else {
                current.clone()
            }
        }
        _ => return,
    };
    set_narrowed(ctx, var_name, &current, narrowed, true);
}

fn narrow_var_literal_string(ctx: &mut Context, name: &str, value: &str, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_value {
        current.filter(|t| match t {
            Atomic::TLiteralString(s) => s.as_ref() == value,
            Atomic::TString | Atomic::TScalar | Atomic::TMixed => true,
            _ => false,
        })
    } else {
        current.filter(|t| !matches!(t, Atomic::TLiteralString(s) if s.as_ref() == value))
    };
    set_narrowed(ctx, name, &current, narrowed, false);
}

fn narrow_var_literal_int(ctx: &mut Context, name: &str, value: i64, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_value {
        current.filter(|t| match t {
            Atomic::TLiteralInt(n) => *n == value,
            Atomic::TInt | Atomic::TScalar | Atomic::TNumeric | Atomic::TMixed => true,
            _ => false,
        })
    } else {
        current.filter(|t| !matches!(t, Atomic::TLiteralInt(n) if *n == value))
    };
    set_narrowed(ctx, name, &current, narrowed, false);
}

fn narrow_var_to_literal_enum_case(
    ctx: &mut Context,
    name: &str,
    enum_fqcn: &str,
    case_name: &str,
    is_case: bool,
) {
    let current = ctx.get_var(name);
    let narrowed = if is_case {
        Union::single(Atomic::TLiteralEnumCase {
            enum_fqcn: enum_fqcn.into(),
            case_name: case_name.into(),
        })
    } else {
        // For !== comparison with enum case, remove that specific case from the union.
        current.filter(|t| {
            !matches!(t, Atomic::TLiteralEnumCase { enum_fqcn: fqcn, case_name: c }
                if fqcn.as_ref() == enum_fqcn && c.as_ref() == case_name)
        })
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

fn narrow_var_to_class_string(ctx: &mut Context, name: &str, fqcn: &str, is_class: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_class {
        Union::single(Atomic::TClassString(Some(mir_types::Symbol::from(fqcn))))
    } else {
        current.filter(|t| !matches!(t, Atomic::TClassString(Some(f)) if f.as_ref() == fqcn))
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

fn narrow_var_to_specific_class(ctx: &mut Context, name: &str, fqcn: &str, is_exact_class: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_exact_class {
        Union::single(Atomic::TNamedObject {
            fqcn: fqcn.into(),
            type_params: mir_types::union::empty_type_params(),
        })
    } else {
        current.filter(|t| match t {
            Atomic::TNamedObject { fqcn: obj_fqcn, .. } => obj_fqcn.as_ref() != fqcn,
            _ => true,
        })
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

fn extract_var_name(expr: &php_ast::owned::Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => extract_var_name(inner),
        _ => None,
    }
}

fn extract_class_name(expr: &php_ast::owned::Expr, self_fqcn: Option<&str>) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.to_string()),
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
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(String, String)> {
    if let ExprKind::StaticPropertyAccess(spa) = &expr.kind {
        if let Some(enum_short_name) = extract_class_name(&spa.class, self_fqcn) {
            let enum_fqcn = crate::db::resolve_name_via_db(db, file, &enum_short_name);
            if let ExprKind::Identifier(case_name) = &spa.member.kind {
                return Some((enum_fqcn, case_name.to_string()));
            }
        }
    }
    None
}

fn extract_class_const_fqcn(
    cca: &php_ast::owned::StaticAccessExpr,
    self_fqcn: Option<&str>,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<String> {
    let is_class = matches!(&cca.member.kind, ExprKind::Identifier(n) if n.as_ref() == "class");
    if !is_class {
        return None;
    }
    let short = extract_class_name(&cca.class, self_fqcn)?;
    Some(crate::db::resolve_name_via_db(db, file, &short))
}

fn extract_get_class_arg(expr: &php_ast::owned::Expr) -> Option<String> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name.eq_ignore_ascii_case("get_class") {
                if let Some(arg) = call.args.first() {
                    return extract_var_name(&arg.value);
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Extension methods on Union used only in narrowing
// ---------------------------------------------------------------------------

trait UnionNarrowExt {
    fn filter<F: Fn(&Atomic) -> bool>(&self, f: F) -> Union;
}

impl UnionNarrowExt for Union {
    fn filter<F: Fn(&Atomic) -> bool>(&self, f: F) -> Union {
        let mut result = Union::empty();
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
