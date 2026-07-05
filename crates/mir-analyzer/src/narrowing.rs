/// Type narrowing — refines variable types based on conditional expressions.
///
/// Given a condition expression and a branch direction (true/false), this
/// module updates the `FlowState` to narrow variable types accordingly.
use php_ast::ast::{AssignOp, BinaryOp, UnaryPrefixOp};
use php_ast::owned::ExprKind;

use mir_codebase::storage::AssertionKind;
use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

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

            // `($x ?? FALLBACK) === FALLBACK` — on the false branch, $x was defined
            // Must be checked before literal comparisons because `b.right` matching a literal
            // would otherwise consume the arm before we check for NullCoalesce on `b.left`.
            if let Some(nc) = extract_null_coalesce(&b.left) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.right) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                }
            } else if let Some(nc) = extract_null_coalesce(&b.right) {
                if let Some(var_name) = extract_var_name(&nc.left) {
                    if !effective_true && same_literal(&nc.right, &b.left) {
                        let current = ctx.get_var(&var_name);
                        ctx.set_var(&var_name, current.remove_null());
                    }
                }
            }
            // `$x === null`
            else if matches!(b.right.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.left) {
                    narrow_var_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_prop_access(&b.left) {
                    narrow_prop_null(ctx, &obj, &prop, db, file, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_null(ctx, &name, effective_true);
                } else if let Some((obj, prop)) = extract_prop_access(&b.right) {
                    narrow_prop_null(ctx, &obj, &prop, db, file, effective_true);
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
            // `true === $x` / `false === $x` — symmetric; extract_var_name looks through
            // assignment exprs, so this also handles `false === ($x = expr)`.
            else if matches!(b.left.kind, ExprKind::Bool(true)) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_bool(ctx, &name, true, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Bool(false)) {
                if let Some(name) = extract_var_name(&b.right) {
                    narrow_var_bool(ctx, &name, false, effective_true);
                }
            }
            // `get_class($x) === 'ClassName'` — check before literal strings so it takes precedence
            else if let ExprKind::String(class_name_str) = &b.right.kind {
                if let Some(obj_var_name) = extract_get_class_arg(&b.left) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
                    narrow_var_to_specific_class(ctx, &obj_var_name, &fqcn, effective_true);
                } else if let Some(name) = extract_var_name(&b.left) {
                    // `$x === 'literal'`
                    narrow_var_literal_string(ctx, &name, class_name_str, effective_true);
                }
            } else if let ExprKind::String(class_name_str) = &b.left.kind {
                if let Some(obj_var_name) = extract_get_class_arg(&b.right) {
                    let fqcn = crate::db::resolve_name(db, file, class_name_str.as_ref());
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
            // `$arr === []` — false-branch (i.e. `$arr !== []`) narrows $arr to non-empty.
            else if let ExprKind::Array(elems) = &b.right.kind {
                if elems.is_empty() {
                    if let Some(var_name) = extract_var_name(&b.left) {
                        if !effective_true {
                            let current = ctx.get_var(&var_name);
                            let narrowed = current.narrow_to_non_empty_collection();
                            if !narrowed.is_empty() && narrowed != current {
                                ctx.set_var(&var_name, narrowed);
                            }
                        }
                    }
                }
            } else if let ExprKind::Array(elems) = &b.left.kind {
                if elems.is_empty() {
                    if let Some(var_name) = extract_var_name(&b.right) {
                        if !effective_true {
                            let current = ctx.get_var(&var_name);
                            let narrowed = current.narrow_to_non_empty_collection();
                            if !narrowed.is_empty() && narrowed != current {
                                ctx.set_var(&var_name, narrowed);
                            }
                        }
                    }
                }
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
            // Normalize: variable on left, integer literal on right.
            // If the literal is on the left (`5 > $x`), swap and flip the operator.
            let (var_expr, cmp_op, lit_expr) = if extract_var_name(&b.left).is_some() {
                (&b.left, b.op, &b.right)
            } else {
                (&b.right, flip_comparison_op(b.op), &b.left)
            };

            if let (Some(var_name), Some(n)) =
                (extract_var_name(var_expr), extract_int_literal(lit_expr))
            {
                narrow_var_int_comparison(ctx, &var_name, cmp_op, n, is_true);
            }
            // count($arr) op N  /  N op count($arr) — normalize so count call is on left.
            let (count_expr, count_cmp_op, count_lit) = if extract_count_of_var(&b.left).is_some() {
                (&b.left, b.op, &b.right)
            } else {
                (&b.right, flip_comparison_op(b.op), &b.left)
            };
            if let (Some(arr_var), Some(n)) = (
                extract_count_of_var(count_expr),
                extract_int_literal(count_lit),
            ) {
                narrow_array_count_comparison(ctx, &arr_var, count_cmp_op, n, is_true);
            }
            // strlen($str) op N  /  N op strlen($str) — same normalization.
            let (strlen_expr, strlen_cmp_op, strlen_lit) =
                if extract_strlen_of_var(&b.left).is_some() {
                    (&b.left, b.op, &b.right)
                } else {
                    (&b.right, flip_comparison_op(b.op), &b.left)
                };
            if let (Some(str_var), Some(n)) = (
                extract_strlen_of_var(strlen_expr),
                extract_int_literal(strlen_lit),
            ) {
                narrow_string_strlen_comparison(ctx, &str_var, strlen_cmp_op, n, is_true);
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

        // $x instanceof ClassName  /  $this->prop instanceof ClassName
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            if let Some(var_name) = extract_var_name(&b.left) {
                if let Some(raw_name) = extract_class_name(&b.right, ctx.self_fqcn.as_deref()) {
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
            } else if let Some((obj, prop)) = extract_prop_access(&b.left) {
                if let Some(raw_name) = extract_class_name(&b.right, ctx.self_fqcn.as_deref()) {
                    let class_name = crate::db::resolve_name(db, file, &raw_name);
                    narrow_prop_instanceof(ctx, &obj, &prop, &class_name, db, file, is_true);
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
                let bare = fn_name.trim_start_matches('\\');
                if matches!(bare, "class_exists" | "interface_exists" | "trait_exists") {
                    // `if (class_exists(\Foo\Bar::class)) { ... }` — record \Foo\Bar as
                    // proven-to-exist in the true branch so that UndefinedClass is
                    // suppressed for all usages within the guarded block.
                    // Variable form: `if (class_exists($var)) { ... }` — narrow $var to
                    // class-string so it satisfies class-string-typed parameters.
                    // `interface_exists($var)` narrows to the more precise interface-string.
                    if is_true {
                        if let Some(arg_expr) = call.args.first() {
                            if let Some(fqcn) =
                                extract_class_fqcn_from_expr(&arg_expr.value, db, file)
                            {
                                ctx.class_exists_guards.insert(fqcn);
                            } else if let Some(var_name) = extract_var_name(&arg_expr.value) {
                                let current = ctx.get_var(&var_name);
                                let narrowed = if bare == "interface_exists" {
                                    current.narrow_to_interface_string()
                                } else {
                                    current.narrow_to_class_string()
                                };
                                set_narrowed(ctx, &var_name, &current, narrowed, true);
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
                } else if fn_name.eq_ignore_ascii_case("assert") {
                    // assert($condition) — narrow as if the condition is is_true
                    if let Some(arg_expr) = call.args.first() {
                        narrow_from_condition(&arg_expr.value, ctx, is_true, db, file);
                    }
                } else if fn_name.eq_ignore_ascii_case("method_exists")
                    || fn_name.eq_ignore_ascii_case("property_exists")
                {
                    // Narrow the first arg to TObject for simple variables (existing behaviour).
                    // Additionally record `(expr_key, method_name)` in method_exists_guards for
                    // property-access first args where variable narrowing can't reach.
                    if let Some(arg_expr) = call.args.first() {
                        if let Some(var_name) = extract_var_name(&arg_expr.value) {
                            narrow_from_type_fn(ctx, fn_name, &var_name, is_true);
                        }
                        if is_true {
                            if let Some(expr_key) = extract_expr_guard_key(&arg_expr.value) {
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
                } else if bare.eq_ignore_ascii_case("array_key_exists") && is_true {
                    // array_key_exists('k', $arr) in true-branch: prove the key
                    // exists in the array's sealed shape so that $arr['k'] does
                    // not trigger NonExistentArrayOffset afterwards.
                    if let (Some(key_arg), Some(arr_arg)) = (call.args.first(), call.args.get(1)) {
                        let literal_key = match &key_arg.value.kind {
                            ExprKind::String(s) => Some(mir_types::atomic::ArrayKey::String(
                                std::sync::Arc::from(s.as_ref()),
                            )),
                            ExprKind::Int(i) => Some(mir_types::atomic::ArrayKey::Int(*i)),
                            _ => None,
                        };
                        if let Some(key) = literal_key {
                            if let Some(var_name) = extract_var_name(&arr_arg.value) {
                                let current = ctx.get_var(&var_name);
                                let narrowed = add_key_to_sealed_shapes(&current, &key);
                                if narrowed != current {
                                    ctx.set_var(&var_name, narrowed);
                                }
                            } else if let Some((obj, prop)) = extract_prop_access(&arr_arg.value) {
                                narrow_prop_array_key_exists(ctx, &obj, &prop, &key, db, file);
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
                            let needle_non_empty = match &needle_arg.value.kind {
                                ExprKind::String(s) => !s.is_empty(),
                                _ => false,
                            };
                            if needle_non_empty {
                                if let Some(var_name) = extract_var_name(&haystack_arg.value) {
                                    let current = ctx.get_var(&var_name);
                                    if !current.is_mixed() {
                                        let narrowed = narrow_string_to_non_empty(&current);
                                        if narrowed != current {
                                            ctx.set_var(&var_name, narrowed);
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if bare.eq_ignore_ascii_case("in_array") {
                    // in_array($needle, ['a', 'b', 'c']) true →
                    // narrow $needle to 'a'|'b'|'c'.
                    if let (Some(needle_arg), Some(haystack_arg)) =
                        (call.args.first(), call.args.get(1))
                    {
                        if let Some(var_name) = extract_var_name(&needle_arg.value) {
                            if let Some(haystack_ty) =
                                extract_haystack_type(&haystack_arg.value, ctx)
                            {
                                let current = ctx.get_var(&var_name);
                                if !current.is_mixed() && is_true {
                                    // intersect: keep only types that could match a haystack value
                                    let narrowed =
                                        narrow_to_haystack_values(&current, &haystack_ty);
                                    if !narrowed.is_empty() && narrowed != current {
                                        ctx.set_var(&var_name, narrowed);
                                    }
                                } else if !current.is_mixed() && !is_true {
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
                            if let Some(class_name) =
                                extract_class_fqcn_from_expr(&class_arg.value, db, file)
                            {
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
                                        // Partition into string-like (kept as-is) and object-like
                                        // (narrowed via instanceof) so `narrow_instanceof_preserving_subtypes`
                                        // fallback doesn't inject a spurious named-object atom when
                                        // the current type is purely string/class-string.
                                        let mut result = Type::empty();
                                        result.possibly_undefined = current.possibly_undefined;
                                        result.from_docblock = current.from_docblock;
                                        let mut obj_part = Type::empty();
                                        for atom in &current.types {
                                            if atom.is_string()
                                                || matches!(atom, Atomic::TClassString(_))
                                            {
                                                result.add_type(atom.clone());
                                            } else {
                                                obj_part.add_type(atom.clone());
                                            }
                                        }
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
                                        filter_out_instanceof_match(&current, &class_name, db)
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
                            if let Some(class_name) =
                                extract_class_fqcn_from_expr(&class_arg.value, db, file)
                            {
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
                        }
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
                            .insert(mir_types::Name::from(var_name.as_str()));
                    }
                } else if is_true {
                    // `isset($base[$k])` implies `$base` is a non-null, indexable
                    // value — remove null/false from the base variable so a
                    // guarded access (`preg_split()` returns array|false) does
                    // not report PossiblyInvalidArrayAccess.
                    if let Some(base) = array_access_base_var(var_expr) {
                        let current = ctx.get_var(&base);
                        ctx.set_var(&base, current.remove_null().remove_false());
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
    ctx: &mut FlowState,
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
                let mut narrowed = Type::empty();
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
    ctx: &mut FlowState,
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
                        extract_var_name(var_expr).map(|name| {
                            let was_assigned = ctx.var_is_defined(&name);
                            (name.clone(), ctx.get_var(&name), was_assigned)
                        })
                    })
                    .collect();

                // Apply isset narrowing: remove null and mark as definitely assigned
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

                // Restore original variable states for if-body context
                for (var_name, original_type, was_assigned) in original_vars {
                    let sym = mir_types::Name::from(var_name.as_str());
                    std::sync::Arc::make_mut(&mut ctx.vars)
                        .insert(sym, mir_codebase::storage::wrap_var_type(original_type));
                    if !was_assigned {
                        std::sync::Arc::make_mut(&mut ctx.assigned_vars).remove(&sym);
                    }
                }
            }
        }
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

    if current.is_empty() || current.is_mixed() {
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
            _ => {}
        }
    }

    if result.is_empty() {
        Type::single(narrowed_ty)
    } else {
        result
    }
}

fn filter_out_instanceof_match(current: &Type, class_name: &str, db: &dyn MirDatabase) -> Type {
    current.filter(|t| match t {
        Atomic::TNamedObject { fqcn, .. }
        | Atomic::TSelf { fqcn }
        | Atomic::TStaticObject { fqcn }
        | Atomic::TParent { fqcn } => !named_object_matches_instanceof(fqcn, class_name, db),
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

    if current.is_empty() || current.is_mixed() {
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
            // Template parameter — narrow to the named class as the bound.
            Atomic::TNamedObject { fqcn, type_params }
                if type_params.is_empty()
                    && !fqcn.contains('\\')
                    && template_param_names.contains(fqcn) =>
            {
                result.add_type(narrowed_ty.clone());
            }
            Atomic::TTemplateParam { .. } => {
                result.add_type(narrowed_ty.clone());
            }
            Atomic::TObject | Atomic::TMixed => result.add_type(narrowed_ty.clone()),
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

/// Narrow a property access `$obj->prop` by a null check.
/// Looks up the declared property type through the database and stores the
/// narrowed result in `ctx.prop_refined`.
fn narrow_prop_null(
    ctx: &mut FlowState,
    obj_var: &str,
    prop: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
) {
    // Get the current type: use an existing refinement if present, else look up
    // the declared type through the object variable's type.
    let current = if let Some(refined) = ctx.get_prop_refined(obj_var, prop) {
        refined.clone()
    } else {
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
            | mir_types::Atomic::TStaticObject { fqcn } = atomic
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
    };

    if current.is_mixed() {
        return;
    }
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    if narrowed != current {
        ctx.set_prop_refined(obj_var, prop, narrowed);
    }
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
    let current = if let Some(refined) = ctx.get_prop_refined(obj_var, prop) {
        refined.clone()
    } else {
        let obj_ty = ctx.get_var(obj_var);
        let mut prop_ty = mir_types::Type::mixed();
        'outer: for atomic in &obj_ty.types {
            if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                if let Some((_, p_def)) = crate::db::find_property_in_chain(db, here, prop) {
                    if let Some(ty) = p_def.ty.as_deref() {
                        prop_ty = ty.clone();
                        break 'outer;
                    }
                }
            } else if let mir_types::Atomic::TSelf { fqcn }
            | mir_types::Atomic::TStaticObject { fqcn } = atomic
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
    };

    if current.is_mixed() {
        return;
    }
    let narrowed = if is_true {
        narrow_instanceof_preserving_subtypes(&current, class_name, db, &ctx.template_param_names)
    } else {
        filter_out_instanceof_match(&current, class_name, db)
    };
    if narrowed != current {
        ctx.set_prop_refined(obj_var, prop, narrowed);
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
    let current = if let Some(refined) = ctx.get_prop_refined(obj_var, prop) {
        refined.clone()
    } else {
        let obj_ty = ctx.get_var(obj_var);
        let mut prop_ty = mir_types::Type::mixed();
        'outer: for atomic in &obj_ty.types {
            if let mir_types::Atomic::TNamedObject { fqcn, .. }
            | mir_types::Atomic::TSelf { fqcn }
            | mir_types::Atomic::TStaticObject { fqcn } = atomic
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
    };
    if current.is_mixed() {
        return;
    }
    let narrowed = add_key_to_sealed_shapes(&current, key);
    if narrowed != current {
        ctx.set_prop_refined(obj_var, prop, narrowed);
    }
}

/// For each `TKeyedArray` in `ty` that is sealed (`is_open == false`) and does not
/// already contain `key`, return a version with `key` added as non-optional `mixed`.
fn add_key_to_sealed_shapes(
    ty: &mir_types::Type,
    key: &mir_types::atomic::ArrayKey,
) -> mir_types::Type {
    use mir_types::atomic::KeyedProperty;
    let new_types: Vec<Atomic> = ty
        .types
        .iter()
        .map(|a| {
            if let Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } = a
            {
                if !is_open && !properties.contains_key(key) {
                    let mut new_props = properties.clone();
                    new_props.insert(
                        key.clone(),
                        KeyedProperty {
                            ty: mir_types::Type::mixed(),
                            optional: false,
                        },
                    );
                    return Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    };
                }
            }
            a.clone()
        })
        .collect();
    let mut result = mir_types::Type::from_vec(new_types);
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
fn narrow_var_int_comparison(ctx: &mut FlowState, name: &str, op: BinaryOp, n: i64, is_true: bool) {
    // Determine the range constraint when the condition holds.
    // Negation (`!is_true`) flips the constraint (e.g. NOT `< N` becomes `>= N`).
    let (min, max): (Option<i64>, Option<i64>) = match (op, is_true) {
        (BinaryOp::Less, true) | (BinaryOp::GreaterOrEqual, false) => (None, n.checked_sub(1)),
        (BinaryOp::LessOrEqual, true) | (BinaryOp::Greater, false) => (None, Some(n)),
        (BinaryOp::Greater, true) | (BinaryOp::LessOrEqual, false) => (n.checked_add(1), None),
        (BinaryOp::GreaterOrEqual, true) | (BinaryOp::Less, false) => (Some(n), None),
        _ => return,
    };
    let current = ctx.get_var(name);
    let narrowed = narrow_type_to_int_range(&current, min, max);
    // Mark the branch unreachable only when the current type is "closed precise"
    // (a bounded int range, named int subtype, or literal union) — these only arise
    // from docblocks/inference, so an empty intersection is a real contradiction.
    // A plain `int` narrowed to an empty range is just conservative widening, not a bug.
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    set_narrowed(ctx, name, &current, narrowed, mark_diverges);
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
                result.add_type(Atomic::TIntRange { min, max });
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

fn narrow_var_null(ctx: &mut FlowState, name: &str, is_null: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

fn narrow_var_bool(ctx: &mut FlowState, name: &str, value: bool, is_value: bool) {
    let current = ctx.get_var(name);
    // `TBool` (PHP `bool`) must be split into TTrue/TFalse rather than kept wholesale.
    // e.g. `$x: bool; if ($x === false)` → true-branch should be `false`, not `bool`.
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
    set_narrowed(ctx, name, &current, narrowed, false);
}

fn narrow_from_type_fn(ctx: &mut FlowState, fn_name: &str, var_name: &str, is_true: bool) {
    let current = ctx.get_var(var_name);
    let narrowed = match crate::util::php_ident_lowercase(fn_name).as_str() {
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
        "array_is_list" => {
            if is_true {
                current.narrow_to_list()
            } else {
                current
                    .filter(|t| !matches!(t, Atomic::TList { .. } | Atomic::TNonEmptyList { .. }))
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
                        Atomic::TScalar | Atomic::TMixed => {
                            narrowed_parts.add_type(t.clone());
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
        // method_exists($obj, 'method') — if true, narrow to TObject (suppresses
        // UndefinedMethod; the concrete type is unresolvable without knowing the method arg)
        "method_exists" | "property_exists" => {
            if is_true {
                Type::single(Atomic::TObject)
            } else {
                current.clone()
            }
        }
        _ => return,
    };
    set_narrowed(ctx, var_name, &current, narrowed, true);
}

fn narrow_var_literal_string(ctx: &mut FlowState, name: &str, value: &str, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_value {
        let lit: std::sync::Arc<str> = std::sync::Arc::from(value);
        let mut result = Type::empty();
        result.from_docblock = current.from_docblock;
        for t in &current.types {
            match t {
                Atomic::TLiteralString(s) if s.as_ref() == value => {
                    result.add_type(t.clone());
                }
                // Generic/wide string types: keep as-is (we can't narrow further without
                // knowing the literal's place in a union)
                Atomic::TString | Atomic::TScalar | Atomic::TMixed => {
                    result.add_type(t.clone());
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
    };
    set_narrowed(ctx, name, &current, narrowed, false);
}

fn narrow_var_literal_int(ctx: &mut FlowState, name: &str, value: i64, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_value {
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
                Atomic::TInt | Atomic::TScalar | Atomic::TNumeric | Atomic::TMixed => {
                    result.add_type(t.clone());
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
                (value.checked_add(1), max)
            } else if max == Some(value) {
                (min, value.checked_sub(1))
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
    };
    // For closed-precise types (bounded ranges, named int subtypes, literal unions),
    // an empty result means the exclusion is a genuine contradiction — mark divergence.
    let mark_diverges = crate::contradiction::is_closed_precise(&current);
    set_narrowed(ctx, name, &current, narrowed, mark_diverges);
}

fn narrow_var_to_literal_enum_case(
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
        // For !== comparison with enum case, remove that specific case from the union.
        current.filter(|t| {
            !matches!(t, Atomic::TLiteralEnumCase { enum_fqcn: fqcn, case_name: c }
                if fqcn.as_ref() == enum_fqcn && c.as_ref() == case_name)
        })
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

fn narrow_var_to_class_string(ctx: &mut FlowState, name: &str, fqcn: &str, is_class: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_class {
        Type::single(Atomic::TClassString(Some(mir_types::Name::from(fqcn))))
    } else {
        current.filter(|t| !matches!(t, Atomic::TClassString(Some(f)) if f.as_ref() == fqcn))
    };
    set_narrowed(ctx, name, &current, narrowed, true);
}

fn narrow_var_to_specific_class(ctx: &mut FlowState, name: &str, fqcn: &str, is_exact_class: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_exact_class {
        Type::single(Atomic::TNamedObject {
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

/// Extract a fully-qualified class name from the first argument of
/// `class_exists()` / `interface_exists()` / `trait_exists()`.
///
/// Recognised forms:
/// - `\Foo\Bar::class` or `Foo\Bar::class` — resolved via `crate::db::resolve_name`
/// - `'Foo\Bar'` or `'Foo\\Bar'` — string literals
pub(crate) fn extract_class_fqcn_from_expr(
    expr: &php_ast::owned::Expr,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<str>> {
    let expr = peel_parens(expr);
    match &expr.kind {
        // \Foo\Bar::class  or  Foo\Bar::class
        ExprKind::ClassConstAccess(cca) => {
            if let ExprKind::Identifier(id) = &cca.class.kind {
                let member = match &cca.member.kind {
                    ExprKind::Identifier(s) => s.as_ref(),
                    _ => return None,
                };
                if member.eq_ignore_ascii_case("class") {
                    let resolved = crate::db::resolve_name(db, file, id.as_ref());
                    if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                        return Some(std::sync::Arc::from(resolved.as_str()));
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
fn extract_prop_access(expr: &php_ast::owned::Expr) -> Option<(String, String)> {
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
/// `method_exists`/`property_exists`. Supports `$var` → `"var"` and
/// `$var->prop` → `"var->prop"` (depth-1 only). Returns `None` for anything
/// more complex so we don't risk false-positive suppression.
pub(crate) fn extract_expr_guard_key(expr: &php_ast::owned::Expr) -> Option<std::sync::Arc<str>> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(std::sync::Arc::from(name.trim_start_matches('$'))),
        ExprKind::Parenthesized(inner) => extract_expr_guard_key(inner),
        ExprKind::PropertyAccess(pa) => {
            let base = extract_var_name(&pa.object)?;
            let prop = match &pa.property.kind {
                ExprKind::Identifier(s) => s.as_ref(),
                ExprKind::Variable(s) => s.trim_start_matches('$'),
                _ => return None,
            };
            Some(std::sync::Arc::from(format!("{base}->{prop}").as_str()))
        }
        _ => None,
    }
}

/// The base variable name of a (possibly nested) array-access expression:
/// `$a[1][2]` → `a`. Returns `None` if the base is not a plain variable.
fn array_access_base_var(expr: &php_ast::owned::Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::ArrayAccess(aa) => array_access_base_var(&aa.array),
        ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => array_access_base_var(inner),
        _ => None,
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
            let enum_fqcn = crate::db::resolve_name(db, file, &enum_short_name);
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

fn is_numeric_string(s: &str) -> bool {
    let t = s.trim();
    !t.is_empty() && (t.parse::<i64>().is_ok() || t.parse::<f64>().is_ok())
}

/// Extract the variable name from `count($var)` / `sizeof($var)`.
fn extract_count_of_var(expr: &php_ast::owned::Expr) -> Option<String> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("count") || bare.eq_ignore_ascii_case("sizeof") {
            if let Some(arg) = call.args.first() {
                return extract_var_name(&arg.value);
            }
        }
    }
    None
}

/// Extract the variable name from `strlen($var)` / `mb_strlen($var, ...)`.
fn extract_strlen_of_var(expr: &php_ast::owned::Expr) -> Option<String> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        let name = match &call.name.kind {
            ExprKind::Identifier(n) => n.as_ref(),
            _ => return None,
        };
        let bare = name.trim_start_matches('\\');
        if bare.eq_ignore_ascii_case("strlen") || bare.eq_ignore_ascii_case("mb_strlen") {
            if let Some(arg) = call.args.first() {
                return extract_var_name(&arg.value);
            }
        }
    }
    None
}

/// Narrow an array variable based on `count($arr) op n` being `is_true`.
/// Promotes `array` / `list` to their non-empty variants when the comparison
/// proves the count is >= 1.
fn narrow_array_count_comparison(
    ctx: &mut FlowState,
    arr_var: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    // Determine whether the comparison proves count >= 1 (i.e., non-empty).
    let non_empty = match (op, is_true) {
        (BinaryOp::Greater, true) if n >= 0 => true, // count > 0 (or > n>=0)
        (BinaryOp::GreaterOrEqual, true) if n >= 1 => true, // count >= 1
        (BinaryOp::Less, false) if n >= 1 => true,   // NOT (count < 1)
        (BinaryOp::LessOrEqual, false) if n >= 0 => true, // NOT (count <= 0)
        _ => false,
    };
    if !non_empty {
        return;
    }
    let current = ctx.get_var(arr_var);
    if current.is_mixed() {
        return;
    }
    let narrowed = current.narrow_to_non_empty_collection();
    if narrowed != current {
        ctx.set_var(arr_var, narrowed);
    }
}

/// Narrow a string variable based on `strlen($str) op n` being `is_true`.
/// Promotes `string` to `non-empty-string` when the comparison proves length >= 1.
fn narrow_string_strlen_comparison(
    ctx: &mut FlowState,
    str_var: &str,
    op: BinaryOp,
    n: i64,
    is_true: bool,
) {
    let non_empty = match (op, is_true) {
        (BinaryOp::Greater, true) if n >= 0 => true,
        (BinaryOp::GreaterOrEqual, true) if n >= 1 => true,
        (BinaryOp::Less, false) if n >= 1 => true,
        (BinaryOp::LessOrEqual, false) if n >= 0 => true,
        _ => false,
    };
    if !non_empty {
        return;
    }
    let current = ctx.get_var(str_var);
    if current.is_mixed() {
        return;
    }
    let narrowed = narrow_string_to_non_empty(&current);
    if narrowed != current {
        ctx.set_var(str_var, narrowed);
    }
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
