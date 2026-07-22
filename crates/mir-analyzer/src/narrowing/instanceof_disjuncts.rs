//! OR-disjunct machinery powering `match(true)`/`switch(true)`/`||`
//! fallthrough narrowing: collects `instanceof`/`is_TYPE()`/`is_a()`/
//! `is_subclass_of()` leaves across a condition list (or an OR-chain) that
//! all target the same variable/property/static-property receiver, and
//! narrows it to the union of every leaf's narrowing.
use php_ast::ast::{BinaryOp, UnaryPrefixOp};
use php_ast::owned::ExprKind;

use mir_types::Type;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{
    apply_prop_narrowed, extract_class_name, extract_prop_access, extract_static_prop_access_parts,
    extract_var_name, narrow_receiver_non_null_on_prop_match, peel_parens,
    resolve_prop_current_type, resolve_static_prop_current_type, set_narrowed,
};
use super::instanceof_core::narrow_or_instanceof_union;
use super::narrow_from_condition;
use super::type_fn::{
    narrow_from_type_fn, narrow_prop_from_type_fn, narrow_static_prop_from_type_fn,
};

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
pub(crate) fn narrow_prop_instanceof_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(String, String)> {
    if conditions.len() < 2 {
        return None;
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
        return None;
    }
    let (obj_var, prop) = receiver?;

    let current = resolve_prop_current_type(ctx, &obj_var, &prop, db, file);
    let narrowed =
        narrow_or_instanceof_union(&current, &class_names, db, &ctx.template_param_names);
    apply_prop_narrowed(ctx, &obj_var, &prop, current, narrowed, true);
    // Every disjunct is an instanceof check, and `null instanceof X` is
    // always false, so proving any one of them proves the receiver wasn't
    // null.
    narrow_receiver_non_null_on_prop_match(ctx, &obj_var, true);
    Some((obj_var, prop))
}

/// Static-property counterpart of `collect_prop_instanceof`, for
/// `self::$prop instanceof A || self::$prop instanceof B` (also
/// `static::$prop`/`Class::$prop`). A static property has no separate
/// "receiver variable" — the map key is the resolved FQCN itself — so unlike
/// the instance-property collector this needs `static_fqcn` too, to resolve
/// a bare `self`/`static`/`parent` keyword the same way
/// `extract_static_prop_access` does.
#[allow(clippy::too_many_arguments)]
fn collect_static_prop_instanceof(
    expr: &php_ast::owned::Expr,
    receiver: &mut Option<(std::sync::Arc<str>, String)>,
    class_names: &mut Vec<String>,
    db: &dyn MirDatabase,
    file: &str,
    self_fqcn: Option<&str>,
    static_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
) -> bool {
    match &expr.kind {
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            if let (Some((fqcn, prop)), Some(cn)) = (
                extract_static_prop_access_parts(
                    &b.left,
                    db,
                    file,
                    self_fqcn,
                    static_fqcn,
                    parent_fqcn,
                ),
                extract_class_name(&b.right, self_fqcn, parent_fqcn),
            ) {
                let resolved = crate::db::resolve_name(db, file, &cn);
                match receiver {
                    None => {
                        *receiver = Some((fqcn, prop));
                        class_names.push(resolved);
                        true
                    }
                    Some((existing_fqcn, existing_prop))
                        if *existing_fqcn == fqcn && *existing_prop == prop =>
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
            collect_static_prop_instanceof(
                &b.left,
                receiver,
                class_names,
                db,
                file,
                self_fqcn,
                static_fqcn,
                parent_fqcn,
            ) && collect_static_prop_instanceof(
                &b.right,
                receiver,
                class_names,
                db,
                file,
                self_fqcn,
                static_fqcn,
                parent_fqcn,
            )
        }
        ExprKind::Parenthesized(inner) => collect_static_prop_instanceof(
            inner,
            receiver,
            class_names,
            db,
            file,
            self_fqcn,
            static_fqcn,
            parent_fqcn,
        ),
        _ => false,
    }
}

/// Static-property counterpart of `narrow_prop_instanceof_disjuncts`, for
/// `self::$prop instanceof A || self::$prop instanceof B` (true branch, also
/// `static::$prop`/`Class::$prop`). See `collect_static_prop_instanceof`'s
/// doc comment for why this needs a third, static-prop-specific leg rather
/// than reusing the instance-property collector.
pub(crate) fn narrow_static_prop_instanceof_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if conditions.len() < 2 {
        return None;
    }
    let self_fqcn = ctx.self_fqcn.as_deref();
    let static_fqcn = ctx.static_fqcn.as_deref();
    let parent_fqcn = ctx.parent_fqcn.as_deref();

    let mut receiver: Option<(std::sync::Arc<str>, String)> = None;
    let mut class_names: Vec<String> = vec![];
    let all_ok = conditions.iter().all(|cond| {
        collect_static_prop_instanceof(
            cond,
            &mut receiver,
            &mut class_names,
            db,
            file,
            self_fqcn,
            static_fqcn,
            parent_fqcn,
        )
    });

    if !all_ok || class_names.len() < 2 {
        return None;
    }
    let (fqcn, prop) = receiver?;

    let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
    let narrowed =
        narrow_or_instanceof_union(&current, &class_names, db, &ctx.template_param_names);
    apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, true);
    Some((fqcn, prop))
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
        // An `is_null($this->prop)`-true disjunct doesn't prove the
        // receiver non-null (a null receiver's ->prop read is itself null,
        // satisfying is_null()) — every other recognized leaf kind does.
        if !fn_names.iter().any(|f| f.eq_ignore_ascii_case("is_null")) {
            narrow_receiver_non_null_on_prop_match(ctx, &obj_var, true);
        }
    }
    Some((obj_var, prop))
}

/// Static-property counterpart of `extract_type_fn_check_prop`, for
/// `is_int(self::$prop)` (also `static::$prop`/`Class::$prop`) — returns
/// `(fn_name, fqcn, prop)`.
fn extract_type_fn_check_static_prop(
    expr: &php_ast::owned::Expr,
    db: &dyn MirDatabase,
    file: &str,
    self_fqcn: Option<&str>,
    static_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
) -> Option<(&'static str, std::sync::Arc<str>, String)> {
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
    let (fqcn, prop) = extract_static_prop_access_parts(
        &call.args[0].value,
        db,
        file,
        self_fqcn,
        static_fqcn,
        parent_fqcn,
    )?;
    Some((canonical, fqcn, prop))
}

/// Static-property counterpart of `narrow_prop_type_fn_disjuncts`, for the
/// `match(true)`/`switch(true)` fallthrough shape applied to `self::$prop`
/// (e.g. `is_int(self::$prop), is_string(self::$prop)`).
pub(crate) fn narrow_static_prop_type_fn_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if conditions.len() < 2 {
        return None;
    }
    let self_fqcn = ctx.self_fqcn.as_deref();
    let static_fqcn = ctx.static_fqcn.as_deref();
    let parent_fqcn = ctx.parent_fqcn.as_deref();

    let mut receiver: Option<(std::sync::Arc<str>, String)> = None;
    let mut fn_names: Vec<&str> = Vec::with_capacity(conditions.len());
    for cond in conditions {
        let (fn_name, fqcn, prop) =
            extract_type_fn_check_static_prop(cond, db, file, self_fqcn, static_fqcn, parent_fqcn)?;
        match &receiver {
            None => receiver = Some((fqcn, prop)),
            Some((existing_fqcn, existing_prop))
                if *existing_fqcn == fqcn && *existing_prop == prop => {}
            _ => return None, // different receiver — bail out
        }
        fn_names.push(fn_name);
    }
    let (fqcn, prop) = receiver?;

    let original = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
    let mut union_ty = Type::empty();
    for fn_name in &fn_names {
        let mut scratch = ctx.branch();
        scratch.set_prop_refined(&fqcn, &prop, original.clone());
        narrow_static_prop_from_type_fn(&mut scratch, fn_name, &fqcn, &prop, db, true);
        union_ty.merge_with(&resolve_static_prop_current_type(
            &scratch, &fqcn, &prop, db,
        ));
    }
    if !union_ty.is_empty() {
        apply_prop_narrowed(ctx, &fqcn, &prop, original, union_ty, true);
    }
    Some((fqcn, prop))
}

/// Whether `expr` is an `is_a(...)`/`is_subclass_of(...)` call, returning its
/// first (receiver) argument expression. `narrow_from_condition` already
/// fully narrows both functions for every receiver shape (var/prop/
/// static-prop) — this only lets the `single_leaf_disjunct_*` family
/// recognize them as a single-receiver leaf instead of bailing out of the
/// whole OR-disjunct union machinery, which previously fell back to a
/// sequential AND-compose that collapses the result to the last disjunct
/// (the same bug class `collect_instanceof`'s doc comment describes).
fn is_a_or_subclass_of_call_receiver(expr: &php_ast::owned::Expr) -> Option<&php_ast::owned::Expr> {
    let ExprKind::FunctionCall(call) = &expr.kind else {
        return None;
    };
    let ExprKind::Identifier(name) = &call.name.kind else {
        return None;
    };
    let bare = name.as_ref().trim_start_matches('\\');
    if !(bare.eq_ignore_ascii_case("is_a") || bare.eq_ignore_ascii_case("is_subclass_of")) {
        return None;
    }
    Some(&call.args.first()?.value)
}

/// Extract the single variable a leaf disjunct condition narrows — either a
/// direct `$x instanceof A` or a recognized `is_TYPE($x)`/`is_a($x, ...)`/
/// `is_subclass_of($x, ...)` call — without applying any narrowing. Used to
/// check every condition in a disjunct list targets the same variable before
/// [`narrow_mixed_disjuncts`] mixes the two kinds together. Recurses into
/// nested `||`/parens (like [`collect_instanceof`]) so a 3-way-or-more chain
/// mixing instanceof and is_TYPE() leaves — e.g. `$x instanceof A ||
/// is_string($x) || $x instanceof B` — still resolves to a shared variable
/// name here; [`narrow_mixed_disjuncts`] then narrows each top-level
/// condition via `narrow_from_condition`, which re-dispatches into this same
/// machinery for any nested disjunct.
fn single_leaf_disjunct_var(expr: &php_ast::owned::Expr) -> Option<String> {
    let expr = peel_parens(expr);
    match &expr.kind {
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => extract_var_name(&b.left),
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            let l = single_leaf_disjunct_var(&b.left)?;
            let r = single_leaf_disjunct_var(&b.right)?;
            (l == r).then_some(l)
        }
        _ => extract_type_fn_check(expr)
            .map(|(_, vn)| vn)
            .or_else(|| extract_var_name(is_a_or_subclass_of_call_receiver(expr)?)),
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
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            let l = single_leaf_disjunct_prop(&b.left)?;
            let r = single_leaf_disjunct_prop(&b.right)?;
            (l == r).then_some(l)
        }
        _ => extract_type_fn_check_prop(expr)
            .map(|(_, obj, prop)| (obj, prop))
            .or_else(|| extract_prop_access(is_a_or_subclass_of_call_receiver(expr)?)),
    }
}

/// Whether any leaf disjunct in `expr` is `is_null($this->prop)` — such a
/// disjunct doesn't prove the receiver non-null (a null receiver's ->prop
/// read is itself null, satisfying is_null()), unlike every other leaf kind
/// [`single_leaf_disjunct_prop`] recognizes. Mirrors its recursion through
/// nested `||`/parens.
fn disjunct_contains_is_null_prop_leaf(expr: &php_ast::owned::Expr) -> bool {
    let expr = peel_parens(expr);
    match &expr.kind {
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            disjunct_contains_is_null_prop_leaf(&b.left)
                || disjunct_contains_is_null_prop_leaf(&b.right)
        }
        _ => extract_type_fn_check_prop(expr)
            .is_some_and(|(fn_name, ..)| fn_name.eq_ignore_ascii_case("is_null")),
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
        if !conditions
            .iter()
            .any(|c| disjunct_contains_is_null_prop_leaf(c))
        {
            narrow_receiver_non_null_on_prop_match(ctx, &obj_var, true);
        }
    }
    Some((obj_var, prop))
}

/// Static-property counterpart of `single_leaf_disjunct_prop`, for
/// `self::$prop instanceof A` / `is_TYPE(self::$prop)` leaf disjuncts.
#[allow(clippy::too_many_arguments)]
fn single_leaf_disjunct_static_prop(
    expr: &php_ast::owned::Expr,
    db: &dyn MirDatabase,
    file: &str,
    self_fqcn: Option<&str>,
    static_fqcn: Option<&str>,
    parent_fqcn: Option<&str>,
) -> Option<(std::sync::Arc<str>, String)> {
    let expr = peel_parens(expr);
    match &expr.kind {
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => {
            extract_static_prop_access_parts(&b.left, db, file, self_fqcn, static_fqcn, parent_fqcn)
        }
        ExprKind::Binary(b) if b.op == BinaryOp::BooleanOr || b.op == BinaryOp::LogicalOr => {
            let l = single_leaf_disjunct_static_prop(
                &b.left,
                db,
                file,
                self_fqcn,
                static_fqcn,
                parent_fqcn,
            )?;
            let r = single_leaf_disjunct_static_prop(
                &b.right,
                db,
                file,
                self_fqcn,
                static_fqcn,
                parent_fqcn,
            )?;
            (l == r).then_some(l)
        }
        _ => extract_type_fn_check_static_prop(expr, db, file, self_fqcn, static_fqcn, parent_fqcn)
            .map(|(_, fqcn, prop)| (fqcn, prop))
            .or_else(|| {
                extract_static_prop_access_parts(
                    is_a_or_subclass_of_call_receiver(expr)?,
                    db,
                    file,
                    self_fqcn,
                    static_fqcn,
                    parent_fqcn,
                )
            }),
    }
}

/// Static-property counterpart of `narrow_mixed_prop_disjuncts`, for a mixed
/// `instanceof`/`is_TYPE()` OR-chain on `self::$prop` (also
/// `static::$prop`/`Class::$prop`). Unlike the instance-property version,
/// there's no separate receiver variable whose non-nullness a proved match
/// could additionally establish (`self::`/`static::` is never itself null),
/// so this never calls `narrow_receiver_non_null_on_prop_match`.
pub(crate) fn narrow_mixed_static_prop_disjuncts(
    conditions: &[&php_ast::owned::Expr],
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if conditions.len() < 2 {
        return None;
    }
    let self_fqcn = ctx.self_fqcn.as_deref();
    let static_fqcn = ctx.static_fqcn.as_deref();
    let parent_fqcn = ctx.parent_fqcn.as_deref();

    let mut receiver: Option<(std::sync::Arc<str>, String)> = None;
    for cond in conditions {
        let (fqcn, prop) =
            single_leaf_disjunct_static_prop(cond, db, file, self_fqcn, static_fqcn, parent_fqcn)?;
        match &receiver {
            None => receiver = Some((fqcn, prop)),
            Some((existing_fqcn, existing_prop))
                if *existing_fqcn == fqcn && *existing_prop == prop => {}
            _ => return None, // different receiver — bail out
        }
    }
    let (fqcn, prop) = receiver?;
    let original = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
    let mut union_ty = Type::empty();
    for cond in conditions {
        let mut scratch = ctx.branch();
        scratch.set_prop_refined(&fqcn, &prop, original.clone());
        narrow_from_condition(cond, &mut scratch, true, db, file);
        union_ty.merge_with(&resolve_static_prop_current_type(
            &scratch, &fqcn, &prop, db,
        ));
    }
    if !union_ty.is_empty() {
        apply_prop_narrowed(ctx, &fqcn, &prop, original, union_ty, true);
    }
    Some((fqcn, prop))
}

/// For `$x instanceof A || $x instanceof B` (true branch): narrow $x to A|B.
/// Handles OR chains recursively, e.g. `$x instanceof A || $x instanceof B || $x instanceof C`.
/// Also handles the scalar-type-check counterpart (`is_int($x) || is_string($x)`)
/// via [`narrow_type_fn_disjuncts`], and a mix of the two (`$x instanceof A ||
/// is_string($x)`) via [`narrow_mixed_disjuncts`], when the pure-instanceof
/// shape doesn't apply.
pub(super) fn narrow_or_instanceof_true(
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    if narrow_instanceof_disjuncts(&[left, right], ctx, db, file).is_none()
        && narrow_type_fn_disjuncts(&[left, right], ctx, db).is_none()
        && narrow_prop_instanceof_disjuncts(&[left, right], ctx, db, file).is_none()
        && narrow_prop_type_fn_disjuncts(&[left, right], ctx, db, file).is_none()
        && narrow_static_prop_instanceof_disjuncts(&[left, right], ctx, db, file).is_none()
        && narrow_static_prop_type_fn_disjuncts(&[left, right], ctx, db, file).is_none()
        && narrow_mixed_disjuncts(&[left, right], ctx, db, file).is_none()
        && narrow_mixed_prop_disjuncts(&[left, right], ctx, db, file).is_none()
    {
        narrow_mixed_static_prop_disjuncts(&[left, right], ctx, db, file);
    }
}

/// Apply short-circuit narrowing for isset() in || expressions (true branch).
///
/// Handles the PHP idiom: `!isset($x) || use($x)`
///
/// `!isset($x) || RHS` being true means either `$x` isn't set (RHS never
/// runs) or `$x` is set AND RHS is true — the merged true-branch state is
/// the union of those two paths, not "nothing narrowed" (a plain union
/// with an unnarrowed path does NOT generally collapse back to the
/// pre-condition state, e.g. `$x` itself is null on one path and
/// non-null-and-`instanceof`-narrowed on the other, which merges to a
/// strictly narrower type than the original).
pub(super) fn narrow_or_isset_true(
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    // Pattern: !isset($x) || RHS
    if let ExprKind::UnaryPrefix(u) = &left.kind {
        if u.op == UnaryPrefixOp::BooleanNot {
            if let ExprKind::Isset(_) = &u.operand.kind {
                let pre = ctx.branch();

                // Path A: $x (or whichever operands) is/are not set; RHS never runs.
                let mut not_set_branch = ctx.branch();
                narrow_from_condition(left, &mut not_set_branch, true, db, file);

                // Path B: $x is set, and RHS was true.
                let mut set_branch = ctx.branch();
                narrow_from_condition(left, &mut set_branch, false, db, file);
                if !set_branch.diverges {
                    narrow_from_condition(right, &mut set_branch, true, db, file);
                }

                *ctx = FlowState::merge_branches(&pre, set_branch, Some(not_set_branch));
            }
        }
    }
}
