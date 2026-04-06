/// Type narrowing — refines variable types based on conditional expressions.
///
/// Given a condition expression and a branch direction (true/false), this
/// module updates the `Context` to narrow variable types accordingly.
use php_ast::ast::{BinaryOp, ExprKind, UnaryPrefixOp};

use mir_codebase::Codebase;
use mir_types::{Atomic, Union};

use crate::context::Context;

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Narrow the types in `ctx` as if `expr` evaluates to `is_true`.
pub fn narrow_from_condition<'arena, 'src>(
    expr: &php_ast::ast::Expr<'arena, 'src>,
    ctx: &mut Context,
    is_true: bool,
    codebase: &Codebase,
    file: &str,
) {
    match &expr.kind {
        // Parenthesized — unwrap and narrow the inner expression
        ExprKind::Parenthesized(inner) => {
            narrow_from_condition(inner, ctx, is_true, codebase, file);
        }

        // !expr  →  narrow as if expr is !is_true
        ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::BooleanNot => {
            narrow_from_condition(u.operand, ctx, !is_true, codebase, file);
        }

        // $a && $b  →  if true: narrow both; if false: no constraint
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanAnd || b.op == BinaryOp::LogicalAnd => {
            if is_true {
                narrow_from_condition(b.left, ctx, true, codebase, file);
                narrow_from_condition(b.right, ctx, true, codebase, file);
            }
        }

        // $a || $b  →  if false: narrow both; if true: try to narrow same-var instanceof union
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            if !is_true {
                narrow_from_condition(b.left, ctx, false, codebase, file);
                narrow_from_condition(b.right, ctx, false, codebase, file);
            } else {
                // For `$x instanceof A || $x instanceof B` in true-branch: narrow $x to A|B
                narrow_or_instanceof_true(b.left, b.right, ctx, codebase, file);
            }
        }

        // $x === null / $x !== null
        ExprKind::Binary(b) if b.op == BinaryOp::Identical || b.op == BinaryOp::NotIdentical => {
            let is_identical = b.op == BinaryOp::Identical;
            let effective_true = if is_identical { is_true } else { !is_true };

            // `$x === null`
            if matches!(b.right.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(b.left) {
                    narrow_var_null(ctx, &name, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(b.right) {
                    narrow_var_null(ctx, &name, effective_true);
                }
            }
            // `$x === true` / `$x === false`
            else if matches!(b.right.kind, ExprKind::Bool(true)) {
                if let Some(name) = extract_var_name(b.left) {
                    narrow_var_bool(ctx, &name, true, effective_true);
                }
            } else if matches!(b.right.kind, ExprKind::Bool(false)) {
                if let Some(name) = extract_var_name(b.left) {
                    narrow_var_bool(ctx, &name, false, effective_true);
                }
            }
            // `$x === 'literal'`
            else if let ExprKind::String(s) = &b.right.kind {
                if let Some(name) = extract_var_name(b.left) {
                    narrow_var_literal_string(ctx, &name, s, effective_true);
                }
            } else if let ExprKind::String(s) = &b.left.kind {
                if let Some(name) = extract_var_name(b.right) {
                    narrow_var_literal_string(ctx, &name, s, effective_true);
                }
            }
            // `$x === 42`
            else if let ExprKind::Int(n) = &b.right.kind {
                if let Some(name) = extract_var_name(b.left) {
                    narrow_var_literal_int(ctx, &name, *n, effective_true);
                }
            } else if let ExprKind::Int(n) = &b.left.kind {
                if let Some(name) = extract_var_name(b.right) {
                    narrow_var_literal_int(ctx, &name, *n, effective_true);
                }
            }
        }

        // $x == null  (loose equality)
        ExprKind::Binary(b) if b.op == BinaryOp::Equal || b.op == BinaryOp::NotEqual => {
            let is_equal = b.op == BinaryOp::Equal;
            let effective_true = if is_equal { is_true } else { !is_true };
            if matches!(b.right.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(b.left) {
                    narrow_var_null(ctx, &name, effective_true);
                }
            } else if matches!(b.left.kind, ExprKind::Null) {
                if let Some(name) = extract_var_name(b.right) {
                    narrow_var_null(ctx, &name, effective_true);
                }
            }
        }

        // $x instanceof ClassName
        // Also handles `!$x instanceof ClassName` which the parser produces as
        // `(!$x) instanceof ClassName` due to PHP operator precedence. The developer
        // intent is always `!($x instanceof ClassName)`, so we flip is_true.
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            // Unwrap `(!$x)` on the left side — treat as negated instanceof
            let (lhs, extra_negation) = match &b.left.kind {
                ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::BooleanNot => (u.operand, true),
                ExprKind::Parenthesized(inner) => match &inner.kind {
                    ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::BooleanNot => {
                        (u.operand, true)
                    }
                    _ => (b.left, false),
                },
                _ => (b.left, false),
            };
            let effective_is_true = if extra_negation { !is_true } else { is_true };
            if let Some(var_name) = extract_var_name(lhs) {
                if let Some(raw_name) = extract_class_name(b.right) {
                    // Resolve the short name to its FQCN using file imports
                    let class_name = codebase.resolve_class_name(file, &raw_name);
                    let current = ctx.get_var(&var_name);
                    let narrowed = if effective_is_true {
                        current.narrow_instanceof(&class_name)
                    } else {
                        // remove that specific named object type
                        current.filter_out_named_object(&class_name)
                    };
                    ctx.set_var(&var_name, narrowed);
                }
            }
        }

        // is_string($x), is_int($x), is_null($x), is_array($x), etc.
        // Also handles assert($x instanceof Y) — narrows like a bare condition.
        ExprKind::FunctionCall(call) => {
            let fn_name_opt: Option<&str> = match &call.name.kind {
                ExprKind::Identifier(name) => Some(name),
                ExprKind::Variable(name) => Some(name.as_ref()),
                _ => None,
            };
            if let Some(fn_name) = fn_name_opt {
                if fn_name.eq_ignore_ascii_case("assert") {
                    // assert($condition) — narrow as if the condition is is_true
                    if let Some(arg_expr) = call.args.first() {
                        narrow_from_condition(&arg_expr.value, ctx, is_true, codebase, file);
                    }
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
                        ctx.assigned_vars.insert(var_name);
                    }
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

/// For `$x instanceof A || $x instanceof B` (true branch): narrow $x to A|B.
/// Handles OR chains recursively, e.g. `$x instanceof A || $x instanceof B || $x instanceof C`.
fn narrow_or_instanceof_true<'arena, 'src>(
    left: &php_ast::ast::Expr<'arena, 'src>,
    right: &php_ast::ast::Expr<'arena, 'src>,
    ctx: &mut Context,
    codebase: &Codebase,
    file: &str,
) {
    // Collect all class names from instanceof checks on the same variable.
    let mut var_name: Option<String> = None;
    let mut class_names: Vec<String> = vec![];

    fn collect_instanceof<'a, 's>(
        expr: &php_ast::ast::Expr<'a, 's>,
        var_name: &mut Option<String>,
        class_names: &mut Vec<String>,
        codebase: &Codebase,
        file: &str,
    ) -> bool {
        match &expr.kind {
            ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
                if let (Some(vn), Some(cn)) =
                    (extract_var_name(b.left), extract_class_name(b.right))
                {
                    let resolved = codebase.resolve_class_name(file, &cn);
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
                collect_instanceof(b.left, var_name, class_names, codebase, file)
                    && collect_instanceof(b.right, var_name, class_names, codebase, file)
            }
            ExprKind::Parenthesized(inner) => {
                collect_instanceof(inner, var_name, class_names, codebase, file)
            }
            _ => false,
        }
    }

    // Wrap left and right into a fake OR so we can reuse the collector
    let left_ok = collect_instanceof(left, &mut var_name, &mut class_names, codebase, file);
    let right_ok = collect_instanceof(right, &mut var_name, &mut class_names, codebase, file);

    if left_ok && right_ok {
        if let Some(vn) = var_name {
            if !class_names.is_empty() {
                let current = ctx.get_var(&vn);
                // Narrow to the union of all instanceof types: take union of narrow_instanceof results
                let mut narrowed = Union::empty();
                for cn in &class_names {
                    let n = current.narrow_instanceof(cn);
                    narrowed = Union::merge(&narrowed, &n);
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

fn narrow_var_null(ctx: &mut Context, name: &str, is_null: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_null {
        current.narrow_to_null()
    } else {
        current.remove_null()
    };
    if !narrowed.is_empty() {
        ctx.set_var(name, narrowed);
    } else if !current.is_empty() && !current.is_mixed() {
        // The type cannot satisfy this nullness constraint → dead branch.
        ctx.diverges = true;
    }
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
    if !narrowed.is_empty() {
        ctx.set_var(name, narrowed);
    }
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
    if !narrowed.is_empty() {
        ctx.set_var(var_name, narrowed);
    } else if !current.is_empty() && !current.is_mixed() {
        // The type cannot satisfy this type-function constraint → dead branch.
        ctx.diverges = true;
    }
}

fn narrow_var_literal_string(ctx: &mut Context, name: &str, value: &str, is_value: bool) {
    let current = ctx.get_var(name);
    let narrowed = if is_value {
        // Keep the specific literal, plus catch-all types that could contain it
        current.filter(|t| match t {
            Atomic::TLiteralString(s) => s.as_ref() == value,
            Atomic::TString | Atomic::TScalar | Atomic::TMixed => true,
            _ => false,
        })
    } else {
        // Remove only this specific literal; leave TString/TMixed intact
        current.filter(|t| !matches!(t, Atomic::TLiteralString(s) if s.as_ref() == value))
    };
    if !narrowed.is_empty() {
        ctx.set_var(name, narrowed);
    }
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
    if !narrowed.is_empty() {
        ctx.set_var(name, narrowed);
    }
}

fn extract_var_name<'a, 'arena, 'src>(
    expr: &'a php_ast::ast::Expr<'arena, 'src>,
) -> Option<String> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.as_ref().trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => extract_var_name(inner),
        _ => None,
    }
}

fn extract_class_name<'arena, 'src>(expr: &php_ast::ast::Expr<'arena, 'src>) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(name) => Some(name.to_string()),
        ExprKind::Variable(_name) => None, // dynamic class — can't narrow
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Extension methods on Union used only in narrowing
// ---------------------------------------------------------------------------

trait UnionNarrowExt {
    fn filter<F: Fn(&Atomic) -> bool>(&self, f: F) -> Union;
    fn filter_out_named_object(&self, fqcn: &str) -> Union;
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

    fn filter_out_named_object(&self, fqcn: &str) -> Union {
        self.filter(|t| match t {
            Atomic::TNamedObject { fqcn: f, .. } => f.as_ref() != fqcn,
            _ => true,
        })
    }
}
