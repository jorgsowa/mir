//! `gettype()`/`get_debug_type()`/`get_parent_class()`/`class_implements()`/
//! `class_parents()`/`$obj::class` narrowing, for variable, property, and
//! static-property receivers.
use php_ast::owned::ExprKind;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{
    extract_static_prop_access, narrow_receiver_non_null_on_prop_match, set_narrowed,
    ScalarArgTarget,
};
use super::enum_class::{
    narrow_prop_to_specific_class, narrow_static_prop_to_specific_class,
    narrow_var_to_specific_class,
};
use super::instanceof_core::{narrow_prop_is_subclass_of, narrow_strict_subclass_of};
use super::type_fn::{
    narrow_from_type_fn, narrow_prop_from_type_fn, narrow_static_prop_from_type_fn,
};

pub(super) fn extract_get_class_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
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

/// Static-property counterpart of `extract_get_class_arg`, mirroring
/// `extract_get_debug_type_static_prop_arg` (see its doc comment for why
/// this is a separate, call-site-local extractor rather than a
/// `ScalarArgTarget` variant).
pub(super) fn extract_get_class_static_prop_arg(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("get_class")
            {
                if let Some(arg) = call.args.first() {
                    return extract_static_prop_access(&arg.value, ctx, db, file);
                }
            }
        }
    }
    None
}

pub(super) fn extract_gettype_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
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

pub(super) fn extract_get_debug_type_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
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

/// Static-property counterpart of `extract_gettype_arg` — `ScalarArgTarget`
/// only extracts Var/Prop targets (see the ROADMAP's `ScalarArgTarget`
/// static-property gap), so `gettype(self::$prop)` needs its own,
/// call-site-local extraction via `extract_static_prop_access`, mirroring
/// how `is_a()`/`is_subclass_of()` already special-case it as a third
/// branch alongside `ScalarArgTarget` rather than widening the enum.
pub(super) fn extract_gettype_static_prop_arg(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("gettype")
            {
                if let Some(arg) = call.args.first() {
                    return extract_static_prop_access(&arg.value, ctx, db, file);
                }
            }
        }
    }
    None
}

/// Static-property counterpart of `extract_get_debug_type_arg`.
pub(super) fn extract_get_debug_type_static_prop_arg(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("get_debug_type")
            {
                if let Some(arg) = call.args.first() {
                    return extract_static_prop_access(&arg.value, ctx, db, file);
                }
            }
        }
    }
    None
}

pub(super) fn extract_get_parent_class_arg(expr: &php_ast::owned::Expr) -> Option<ScalarArgTarget> {
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

/// Static-property counterpart of `extract_get_parent_class_arg` —
/// `ScalarArgTarget` has no static-property variant (tracked as S19), so
/// extract it call-site-locally instead, mirroring
/// `extract_get_debug_type_static_prop_arg`.
pub(super) fn extract_get_parent_class_static_prop_arg(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            if name
                .trim_start_matches('\\')
                .eq_ignore_ascii_case("get_parent_class")
            {
                if let Some(arg) = call.args.first() {
                    return extract_static_prop_access(&arg.value, ctx, db, file);
                }
            }
        }
    }
    None
}

/// Extract the receiver from `class_implements($x)`/`class_parents($x)`,
/// along with which of the two builtins matched (`true` for
/// `class_parents`) — both return an array keyed (and valued) by
/// interface/ancestor-class name, but the relationship each proves is NOT
/// the same: `class_implements()` matches `instanceof` semantics (a class
/// satisfies its own implemented interfaces), while `class_parents()`
/// excludes the receiver's own exact class (it's a STRICT-ancestor list,
/// like `is_subclass_of()`) — callers must dispatch on the returned flag
/// rather than treating both as instanceof-style.
pub(super) fn extract_class_implements_or_parents_arg(
    expr: &php_ast::owned::Expr,
) -> Option<(ScalarArgTarget, bool)> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            let bare = name.trim_start_matches('\\');
            let is_parents = bare.eq_ignore_ascii_case("class_parents");
            if is_parents || bare.eq_ignore_ascii_case("class_implements") {
                if let Some(arg) = call.args.first() {
                    return ScalarArgTarget::extract(&arg.value).map(|t| (t, is_parents));
                }
            }
        }
    }
    None
}

/// Static-property counterpart of `extract_class_implements_or_parents_arg` —
/// `ScalarArgTarget` has no static-property variant (tracked as S19), so a
/// `class_implements(self::$prop)`/`class_parents(self::$prop)` receiver is
/// extracted call-site-locally instead, mirroring
/// `extract_get_class_static_prop_arg`. Also reports which builtin matched,
/// same as the var/prop counterpart above.
pub(super) fn extract_class_implements_or_parents_static_prop_arg(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<((std::sync::Arc<str>, String), bool)> {
    if let ExprKind::FunctionCall(call) = &expr.kind {
        if let ExprKind::Identifier(name) = &call.name.kind {
            let bare = name.trim_start_matches('\\');
            let is_parents = bare.eq_ignore_ascii_case("class_parents");
            if is_parents || bare.eq_ignore_ascii_case("class_implements") {
                if let Some(arg) = call.args.first() {
                    return extract_static_prop_access(&arg.value, ctx, db, file)
                        .map(|t| (t, is_parents));
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
pub(super) fn narrow_from_get_parent_class_literal(
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
            // get_parent_class(null) throws a TypeError, so reaching either
            // comparison result at all proves the receiver was non-null.
            narrow_receiver_non_null_on_prop_match(ctx, obj, true);
        }
    }
}

/// Extract the receiver (variable or property access) from `$obj::class` /
/// `$this->obj::class` — PHP 8's `get_class($obj)` equivalent, parsed as a
/// `ClassConstAccess` whose class side is an expression rather than a
/// static class-name identifier.
pub(super) fn extract_dynamic_class_const_var(
    expr: &php_ast::owned::Expr,
) -> Option<ScalarArgTarget> {
    if let ExprKind::ClassConstAccess(cca) = &expr.kind {
        if matches!(&cca.member.kind, ExprKind::Identifier(n) if n.as_ref() == "class") {
            return ScalarArgTarget::extract(&cca.class);
        }
    }
    None
}

/// Static-property counterpart of `extract_dynamic_class_const_var`, for
/// `self::$prop::class` / `Foo::$prop::class` — mirrors
/// `extract_get_class_static_prop_arg`.
pub(super) fn extract_dynamic_class_const_static_prop_var(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    if let ExprKind::ClassConstAccess(cca) = &expr.kind {
        if matches!(&cca.member.kind, ExprKind::Identifier(n) if n.as_ref() == "class") {
            return extract_static_prop_access(&cca.class, ctx, db, file);
        }
    }
    None
}

/// Narrow `$x`/`$this->prop` from `gettype(...) === 'literal'`, mapping
/// `gettype()`'s fixed set of return strings to the equivalent `is_TYPE()`
/// narrowing on whichever receiver shape `target` resolved to.
pub(super) fn narrow_from_gettype_literal(
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
pub(super) fn narrow_from_get_debug_type_literal(
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

/// Static-property counterpart of `narrow_from_gettype_literal`, for
/// `gettype(self::$prop) === 'literal'`.
pub(super) fn narrow_static_prop_from_gettype_literal(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
    literal: &str,
    is_true: bool,
    db: &dyn MirDatabase,
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
    narrow_static_prop_from_type_fn(ctx, type_fn, fqcn, prop, db, is_true);
}

/// Static-property counterpart of `narrow_from_get_debug_type_literal`, for
/// `get_debug_type(self::$prop) === 'literal'`.
pub(super) fn narrow_static_prop_from_get_debug_type_literal(
    ctx: &mut FlowState,
    fqcn: &str,
    prop: &str,
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
        narrow_static_prop_from_type_fn(ctx, type_fn, fqcn, prop, db, is_true);
    } else {
        let resolved = crate::db::resolve_name(db, file, literal);
        narrow_static_prop_to_specific_class(ctx, fqcn, prop, &resolved, is_true, db);
    }
}
