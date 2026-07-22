//! Shared kernel for the narrowing submodules: expression-shape extractors,
//! property-refinement helpers, and small cross-cutting utilities used by
//! multiple narrowing arms.
use php_ast::ast::{AssignOp, BinaryOp};
use php_ast::owned::ExprKind;

use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::literals::{extract_int_literal, narrow_var_null};
use super::strings::{
    extract_strlen_arg, extract_strlen_static_prop_arg, narrow_prop_string_strlen_comparison,
    narrow_static_prop_string_strlen_comparison, narrow_string_strlen_comparison,
};
use super::{
    extract_count_arg, extract_count_static_prop_arg, narrow_array_count_comparison,
    narrow_prop_array_count_comparison, narrow_static_prop_array_count_comparison,
};

/// Apply a pre-computed narrowed type to a variable.
///
/// If `mark_diverges` is true and the narrowed type is empty (the current type
/// can never satisfy the constraint), the branch is marked unreachable.
pub(super) fn set_narrowed(
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

/// Narrow a static property access `self::$prop`/`Class::$prop` by a null
/// check. `prop_refined` is keyed by FQCN here instead of a receiver
/// variable name — a FQCN string can never collide with a real PHP variable.
/// Resolve the current type of `self::$prop`/`static::$prop`/`Class::$prop`:
/// an existing flow-state refinement if one is already tracked, else the
/// declared type looked up through the class hierarchy. Static-property
/// counterpart of `resolve_prop_current_type`.
pub(crate) fn resolve_static_prop_current_type(
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

/// Applies a narrowed property type computed from `current`, mirroring
/// `set_narrowed`'s variable-side semantics for the property-refinement store.
pub(crate) fn apply_prop_narrowed(
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

/// After proving `$obj->prop` equals a definite non-null literal value
/// (`proved_match`), the receiver itself must also be non-null: PHP 8 reads
/// `$obj->prop` on a null `$obj` as a warning, still evaluating to `null`
/// (same ambiguity as `narrow_nullsafe_prop_null`).
pub(crate) fn narrow_receiver_non_null_on_prop_match(
    ctx: &mut FlowState,
    obj_var: &str,
    proved_match: bool,
) {
    if proved_match {
        narrow_var_null(ctx, obj_var, false);
    }
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
pub(super) fn extract_nullsafe_prop_access(
    expr: &php_ast::owned::Expr,
) -> Option<(String, String)> {
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
pub(super) fn extract_any_prop_access(expr: &php_ast::owned::Expr) -> Option<(String, String)> {
    extract_nullsafe_prop_access(expr).or_else(|| extract_prop_access(expr))
}

/// Extract `(fqcn, prop_name)` from a `self::$prop` / `static::$prop` /
/// `parent::$prop` / `ClassName::$prop` expression, resolving relative
/// keywords through the current `FlowState`.
pub(crate) fn extract_static_prop_access(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    extract_static_prop_access_parts(
        expr,
        db,
        file,
        ctx.self_fqcn.as_deref(),
        ctx.static_fqcn.as_deref(),
        ctx.parent_fqcn.as_deref(),
    )
}

/// Same resolution logic as `extract_static_prop_access`, parameterized on
/// the three relative-keyword FQCNs directly instead of a full `FlowState`
/// borrow — used by the OR-disjunct collector helpers (`collect_static_prop_instanceof`,
/// `extract_type_fn_check_static_prop`), which precompute these once before
/// looping so they don't need to hold a `FlowState` reference alongside a
/// later `&mut FlowState` use in the same function.
pub(super) fn extract_static_prop_access_parts(
    expr: &php_ast::owned::Expr,
    db: &dyn MirDatabase,
    file: &str,
    self_fqcn: Option<&str>,
    static_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
) -> Option<(std::sync::Arc<str>, String)> {
    match &expr.kind {
        ExprKind::StaticPropertyAccess(spa) => {
            let id = match &spa.class.kind {
                ExprKind::Identifier(id) => id,
                _ => return None,
            };
            let resolved = crate::db::resolve_name(db, file, id.as_ref());
            let fqcn = match resolved.as_str() {
                "self" | "static" => std::sync::Arc::from(self_fqcn.or(static_fqcn)?),
                "parent" => std::sync::Arc::from(parent_fqcn?),
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
        ExprKind::Parenthesized(inner) => {
            extract_static_prop_access_parts(inner, db, file, self_fqcn, static_fqcn, parent_fqcn)
        }
        _ => None,
    }
}

pub(super) fn extract_var_name(expr: &php_ast::owned::Expr) -> Option<String> {
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
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<str>> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(std::sync::Arc::from(name.trim_start_matches('$'))),
        ExprKind::Parenthesized(inner) => extract_expr_guard_key(inner, ctx, db, file),
        ExprKind::PropertyAccess(pa) => {
            let base = extract_var_name(&pa.object)?;
            let prop = match &pa.property.kind {
                ExprKind::Identifier(s) => s.as_ref(),
                ExprKind::Variable(s) => s.trim_start_matches('$'),
                _ => return None,
            };
            Some(std::sync::Arc::from(format!("{base}->{prop}").as_str()))
        }
        ExprKind::StaticPropertyAccess(_) => {
            let (fqcn, prop) = extract_static_prop_access(expr, ctx, db, file)?;
            Some(std::sync::Arc::from(
                format!("static:{fqcn}::{prop}").as_str(),
            ))
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

/// The subject of a `match`/`switch` statement, whichever receiver shape it
/// is — used to narrow each arm's context by intersecting the subject's
/// type with that arm's condition type, same as a plain variable subject
/// already did (a property/static-property subject is just as valid a
/// narrowing target).
pub(crate) enum MatchSubject {
    Var(String),
    Prop(String, String),
    Static(std::sync::Arc<str>, String),
}

impl MatchSubject {
    pub(crate) fn extract(
        expr: &php_ast::owned::Expr,
        ctx: &FlowState,
        db: &dyn MirDatabase,
        file: &str,
    ) -> Option<Self> {
        if let Some(name) = extract_var_name(expr) {
            return Some(MatchSubject::Var(name));
        }
        if let Some((obj, prop)) = extract_prop_access(expr) {
            return Some(MatchSubject::Prop(obj, prop));
        }
        extract_static_prop_access(expr, ctx, db, file)
            .map(|(fqcn, prop)| MatchSubject::Static(fqcn, prop))
    }
}

pub(super) fn extract_null_coalesce(
    expr: &php_ast::owned::Expr,
) -> Option<&php_ast::owned::NullCoalesceExpr> {
    match &expr.kind {
        ExprKind::NullCoalesce(nc) => Some(nc),
        ExprKind::Parenthesized(inner) => extract_null_coalesce(inner),
        _ => None,
    }
}

pub(super) fn same_literal(a: &php_ast::owned::Expr, b: &php_ast::owned::Expr) -> bool {
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

pub(super) fn peel_parens(expr: &php_ast::owned::Expr) -> &php_ast::owned::Expr {
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
pub(super) fn extract_class_name(
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
pub(super) fn promote_assignment_effects(
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

/// A `gettype()`/`get_debug_type()` argument, resolved to either a plain
/// variable or a `$obj->prop` property access — lets a single literal-mapping
/// function (`narrow_from_gettype_literal`/`narrow_from_get_debug_type_literal`)
/// dispatch to the right narrowing entry point (`narrow_from_type_fn` vs
/// `narrow_prop_from_type_fn`/`narrow_prop_to_specific_class`) for either
/// receiver shape.
pub(super) enum ScalarArgTarget {
    Var(String),
    Prop(String, String),
}

impl ScalarArgTarget {
    pub(super) fn extract(expr: &php_ast::owned::Expr) -> Option<Self> {
        if let Some(name) = extract_var_name(expr) {
            return Some(ScalarArgTarget::Var(name));
        }
        // `extract_any_prop_access` so a nullsafe receiver (`$x?->prop`) is
        // recognized too — a plain `->` on a null receiver evaluates to null
        // in PHP 8 same as `?->`, so every consumer here already narrows both
        // identically once the target is extracted.
        extract_any_prop_access(expr).map(|(obj, prop)| ScalarArgTarget::Prop(obj, prop))
    }
}

// ---------------------------------------------------------------------------
// Extension methods on Type used only in narrowing
// ---------------------------------------------------------------------------

pub(super) trait UnionNarrowExt {
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

/// `count($arr) op N` / `strlen($str) op N` for the equality operators
/// (`===`, `!==`, `==`, `!=`) — the `<`/`<=`/`>`/`>=` forms are normalized
/// and handled inline where those operators are matched; equality is
/// symmetric so, unlike that relational-operator normalization, no operator
/// flip is needed when the call is on the right-hand side.
pub(super) fn narrow_count_or_strlen_equality(
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    op: BinaryOp,
    is_true: bool,
) {
    let count_on_left = extract_count_arg(left).is_some()
        || extract_count_static_prop_arg(left, ctx, db, file).is_some();
    let (count_expr, count_lit) = if count_on_left {
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
    } else if let (Some((fqcn, prop)), Some(n)) = (
        extract_count_static_prop_arg(count_expr, ctx, db, file),
        extract_int_literal(count_lit),
    ) {
        narrow_static_prop_array_count_comparison(ctx, &fqcn, &prop, db, op, n, is_true);
        return;
    }
    let strlen_on_left = extract_strlen_arg(left).is_some()
        || extract_strlen_static_prop_arg(left, ctx, db, file).is_some();
    let (strlen_expr, strlen_lit) = if strlen_on_left {
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
    } else if let (Some((fqcn, prop)), Some(n)) = (
        extract_strlen_static_prop_arg(strlen_expr, ctx, db, file),
        extract_int_literal(strlen_lit),
    ) {
        narrow_static_prop_string_strlen_comparison(ctx, &fqcn, &prop, db, op, n, is_true);
    }
}

/// Whether `count()`/`strlen() op n` being `is_true` proves the underlying
/// collection/string is non-empty (`Some(true)`) or empty (`Some(false)`);
/// `None` if the comparison proves neither (count/strlen is always >= 0, so
/// e.g. `count($x) < 5` proves nothing about emptiness either way).
pub(super) fn count_or_strlen_emptiness(op: BinaryOp, n: i64, is_true: bool) -> Option<bool> {
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
