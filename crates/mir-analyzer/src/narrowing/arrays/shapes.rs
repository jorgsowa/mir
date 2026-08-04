//! The shape-narrowing core: tracks `$var['key']`/`$obj->prop['key']`-style
//! array-shape access paths for `isset()`/`empty()`/`array_key_exists()`, plus
//! the `$arr === []`/`!== []` array-emptiness narrowing.
use php_ast::owned::ExprKind;

use mir_types::{Atomic, Type};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::super::core::{
    apply_prop_narrowed, extract_any_prop_access, extract_static_prop_access, extract_var_name,
    narrow_receiver_non_null_on_prop_match, resolve_prop_current_type,
    resolve_static_prop_current_type, ScalarArgTarget,
};
use super::super::instanceof_core::{
    filter_out_instanceof_match, narrow_instanceof_preserving_subtypes,
};
use super::super::type_fn::type_fn_narrowed;
use super::key_exists::{add_key_to_sealed_shapes, remove_key_from_sealed_shapes};

/// Property-access counterpart of the `$arr === []`/`$arr !== []` (and loose
/// `==`/`!=`) var-based array-emptiness narrowing above, for `$this->prop`.
/// `mark_diverges=false` matches the var-side behavior, which also leaves an
/// empty narrowing result untouched instead of flagging a contradiction.
pub(crate) fn narrow_prop_array_empty(
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
pub(crate) fn narrow_static_prop_array_empty(
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

/// The base of an `isset()`/`empty()`/`array_key_exists()` array-shape
/// access, widened with a static-property variant alongside the plain
/// `ScalarArgTarget` shapes — kept as its own local type rather than a new
/// `ScalarArgTarget::Static` variant, since that enum is matched
/// exhaustively at ~25 unrelated call sites across this file (each already
/// has its own call-site-local static-prop arm, per this file's established
/// convention — see the `ScalarArgTarget` doc comment).
pub(crate) enum ShapeBase {
    Var(String),
    Prop(String, String),
    Static(std::sync::Arc<str>, String),
}

impl ShapeBase {
    fn extract(
        expr: &php_ast::owned::Expr,
        ctx: &FlowState,
        db: &dyn MirDatabase,
        file: &str,
    ) -> Option<Self> {
        match ScalarArgTarget::extract(expr) {
            Some(ScalarArgTarget::Var(name)) => Some(ShapeBase::Var(name)),
            Some(ScalarArgTarget::Prop(obj, prop)) => Some(ShapeBase::Prop(obj, prop)),
            None => extract_static_prop_access(expr, ctx, db, file)
                .map(|(fqcn, prop)| ShapeBase::Static(fqcn, prop)),
        }
    }
}

/// The base (variable, property, or static-property receiver) of a
/// (possibly nested) array-access expression: `$a[1][2]` → `Var("a")`,
/// `$this->data[1]` → `Prop("this", "data")`, `self::$data[1]` →
/// `Static(fqcn, "data")`. Unlike `collect_array_access_path`, doesn't
/// require every key along the way to be a literal — stripping null/false
/// from the container itself doesn't depend on the key being statically
/// known.
pub(crate) fn array_access_base_target(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<ShapeBase> {
    match &expr.kind {
        ExprKind::ArrayAccess(aa) => array_access_base_target(&aa.array, ctx, db, file),
        ExprKind::Parenthesized(inner) => array_access_base_target(inner, ctx, db, file),
        _ => ShapeBase::extract(expr, ctx, db, file),
    }
}

/// Remove `null`/`false` from an `isset($base[...])`/`!empty($base[...])`
/// container, whichever receiver shape `base` is — the property/static-
/// property counterpart of the plain-variable case, since `->`/`::` access
/// on a nullable property is just as valid an `isset()`/`empty()` target as
/// a variable.
pub(crate) fn narrow_container_non_null_non_false(
    ctx: &mut FlowState,
    target: &ShapeBase,
    db: &dyn MirDatabase,
    file: &str,
) {
    match target {
        ShapeBase::Var(name) => {
            let current = ctx.get_var(name);
            ctx.set_var(name, current.remove_null().remove_false());
        }
        ShapeBase::Prop(obj, prop) => {
            let current = resolve_prop_current_type(ctx, obj, prop, db, file);
            if !current.is_mixed() {
                let narrowed = current.remove_null().remove_false();
                apply_prop_narrowed(ctx, obj, prop, current, narrowed, true);
            }
        }
        ShapeBase::Static(fqcn, prop) => {
            let current = resolve_static_prop_current_type(ctx, fqcn, prop, db);
            if !current.is_mixed() {
                let narrowed = current.remove_null().remove_false();
                apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, true);
            }
        }
    }
}

/// For `isset($base['a']['b']...)` where `$base` is (partly) a known shape,
/// narrow every level of the access path: remove `null` from each key's
/// value type and mark it no longer optional, recursing into the key's own
/// value type for the next path segment. `isset($a['b']['c'])` proves both
/// `$a['b']` and `$a['b']['c']` present, not just the outermost key.
pub(crate) fn narrow_isset_shape_key(
    var_expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    let Some((base, path)) = collect_array_access_path(var_expr, ctx, db, file) else {
        return;
    };
    let current = resolve_shape_base_current_type(ctx, &base, db, file);
    if let Some(narrowed) = narrow_shape_path(&current, &path) {
        set_shape_base_narrowed(ctx, &base, current, narrowed);
    }
}

/// `assert($base['a']['b'] !== null)` / `if ($base['a']['b'] === null) {
/// throw/return; }` — a null check on a literal-keyed array-offset access,
/// same access-path shape `narrow_isset_shape_key` covers but narrowing the
/// key's OWN value type (to/from null) rather than proving mere presence.
/// A no-op when `var_expr` isn't a literal-keyed array-access chain at all
/// (`collect_array_access_path` returns `None`), so callers can invoke this
/// unconditionally as a dispatch-chain fallback.
pub(crate) fn narrow_offset_null_by_path(
    var_expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    is_null: bool,
) {
    let Some((base, path)) = collect_array_access_path(var_expr, ctx, db, file) else {
        return;
    };
    let current = resolve_shape_base_current_type(ctx, &base, db, file);
    if let Some(narrowed) = narrow_shape_path_null(&current, &path, is_null) {
        set_shape_base_narrowed(ctx, &base, current, narrowed);
    }
}

/// Collect `(base, [key1, key2, ...])` from a chain of literal-keyed
/// `ArrayAccess` nodes, outermost-to-innermost (`$a['x']['y']` -> `(Var("a"),
/// [x, y])`, `$this->data['x']` -> `(Prop("this", "data"), [x])`,
/// `self::$data['x']` -> `(Static(fqcn, "data"), [x])`). Returns `None` as
/// soon as a non-literal key or non-var/prop/static-prop root is found —
/// those cases are left unnarrowed.
pub(crate) fn collect_array_access_path(
    expr: &php_ast::owned::Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(ShapeBase, Vec<mir_types::atomic::ArrayKey>)> {
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
    if let Some(base) = ShapeBase::extract(&aa.array, ctx, db, file) {
        Some((base, vec![key]))
    } else {
        let (base, mut path) = collect_array_access_path(&aa.array, ctx, db, file)?;
        path.push(key);
        Some((base, path))
    }
}

/// Read the current type of a `collect_array_access_path` base, whichever
/// receiver shape it is.
pub(crate) fn resolve_shape_base_current_type(
    ctx: &mut FlowState,
    base: &ShapeBase,
    db: &dyn MirDatabase,
    file: &str,
) -> Type {
    match base {
        ShapeBase::Var(name) => ctx.get_var(name),
        ShapeBase::Prop(obj, prop) => resolve_prop_current_type(ctx, obj, prop, db, file),
        ShapeBase::Static(fqcn, prop) => resolve_static_prop_current_type(ctx, fqcn, prop, db),
    }
}

/// Apply a narrowed type back to a `collect_array_access_path` base.
pub(crate) fn set_shape_base_narrowed(
    ctx: &mut FlowState,
    base: &ShapeBase,
    current: Type,
    narrowed: Type,
) {
    match base {
        ShapeBase::Var(name) => ctx.set_var(name, narrowed),
        ShapeBase::Prop(obj, prop) => apply_prop_narrowed(ctx, obj, prop, current, narrowed, false),
        ShapeBase::Static(fqcn, prop) => {
            apply_prop_narrowed(ctx, fqcn, prop, current, narrowed, false)
        }
    }
}

/// Force `path`'s value type to `asserted` in every `TKeyedArray` union
/// member — the assign counterpart of `narrow_shape_path` (which only
/// refines a key ALREADY proven present): used by an assertion-tag target
/// on a specific array key (`@psalm-assert-if-true string $arr['key']`),
/// which must set the key's type outright rather than merely narrow an
/// existing one. Adds the key if the shape doesn't have it yet (open or
/// not), same as an ordinary literal-keyed array write would. A non-shape
/// union member (`array`, `TArray`, a non-array atom) is left untouched —
/// there's no shape structure to attach a single key's type to without
/// first synthesizing one, out of scope for this targeted fix.
pub(crate) fn set_shape_path(
    ty: &Type,
    path: &[mir_types::atomic::ArrayKey],
    asserted: &Type,
) -> Type {
    let Some((key, rest)) = path.split_first() else {
        return asserted.clone();
    };
    let mut touched = false;
    let mut result = Type::empty();
    for atomic in &ty.types {
        match atomic {
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                touched = true;
                let mut new_props = properties.clone();
                let value_ty = if rest.is_empty() {
                    asserted.clone()
                } else {
                    let existing = new_props
                        .get(key)
                        .map(|p| p.ty.clone())
                        .unwrap_or_else(Type::mixed);
                    set_shape_path(&existing, rest, asserted)
                };
                new_props.insert(
                    key.clone(),
                    mir_types::atomic::KeyedProperty {
                        ty: value_ty,
                        optional: false,
                    },
                );
                result.add_type(Atomic::TKeyedArray {
                    properties: new_props,
                    is_open: *is_open,
                    is_list: *is_list,
                });
            }
            other => result.add_type(other.clone()),
        }
    }
    if touched {
        result
    } else {
        ty.clone()
    }
}

/// Read the current type at a shape-key access path, merging across every
/// `TKeyedArray` union member that carries the key — used to negate an
/// assertion-tag target against the key's own current value, mirroring how
/// a plain (whole-parameter) assertion's negation reads the receiver's
/// current value via `resolve_prop_current_type`/`ctx.get_var`. Falls back
/// to `mixed` when no union member carries the key at all, the same
/// conservative default a fresh, never-narrowed variable would have.
pub(crate) fn get_shape_path_type(ty: &Type, path: &[mir_types::atomic::ArrayKey]) -> Type {
    let Some((key, rest)) = path.split_first() else {
        return ty.clone();
    };
    let mut result = Type::empty();
    let mut found = false;
    for atomic in &ty.types {
        if let Atomic::TKeyedArray { properties, .. } = atomic {
            if let Some(prop) = properties.get(key) {
                found = true;
                if rest.is_empty() {
                    result.merge_with(&prop.ty);
                } else {
                    result.merge_with(&get_shape_path_type(&prop.ty, rest));
                }
            }
        }
    }
    if found {
        result
    } else {
        Type::mixed()
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

/// Narrow `ty` along a shape-key access path by a null check on the key's OWN
/// value (`M24`): unlike `narrow_shape_path` (which proves mere presence),
/// this narrows `path`'s leaf key to/from null. Proving non-null also proves
/// presence (an absent optional key reads as null via PHP's undefined-index-
/// then-null semantics, same reasoning as `narrow_shape_path`'s
/// `optional = false`), so a closed shape lacking the key entirely is
/// excluded on the non-null side — it could only ever read as null.
fn narrow_shape_path_null(
    ty: &Type,
    path: &[mir_types::atomic::ArrayKey],
    is_null: bool,
) -> Option<Type> {
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
                let Some(prop) = properties.get(key) else {
                    if !is_null && !*is_open {
                        // Closed shape without this key always reads as
                        // null — proving non-null is impossible for this
                        // union member.
                        changed = true;
                    } else {
                        result.add_type(atomic.clone());
                    }
                    continue;
                };
                if rest.is_empty() {
                    let narrowed_ty = if is_null {
                        prop.ty.narrow_to_null()
                    } else {
                        prop.ty.remove_null()
                    };
                    if narrowed_ty.is_empty() {
                        // Contradiction for this union member (e.g. proving
                        // null on a key whose declared type excludes it) —
                        // exclude it rather than keep a stale wider type.
                        changed = true;
                        continue;
                    }
                    let mut new_props = properties.clone();
                    if let Some(p) = new_props.get_mut(key) {
                        p.ty = narrowed_ty;
                        if !is_null {
                            p.optional = false;
                        }
                    }
                    changed = true;
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                } else if let Some(deeper) = narrow_shape_path_null(&prop.ty, rest, is_null) {
                    let mut new_props = properties.clone();
                    if let Some(p) = new_props.get_mut(key) {
                        p.ty = deeper;
                    }
                    changed = true;
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
    if changed && !result.types.is_empty() {
        Some(result)
    } else {
        None
    }
}

/// `$base['a']['b'] instanceof ClassName` — the `instanceof` counterpart of
/// `narrow_offset_null_by_path` (M24), extending G11 (type-guard narrowing
/// never applying to array-offset expressions) to `instanceof`. Same
/// literal-keyed-shape-only scope as M24: a generic `array<K,V>`/`list<T>`
/// base has no per-key storage to narrow, so it's left untouched — same
/// limitation `narrow_shape_path`/`narrow_shape_path_null` already have. A
/// no-op when `var_expr` isn't a literal-keyed array-access chain at all.
pub(crate) fn narrow_offset_instanceof_by_path(
    var_expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    class_name: &str,
    is_true: bool,
) {
    let Some((base, path)) = collect_array_access_path(var_expr, ctx, db, file) else {
        return;
    };
    let current = resolve_shape_base_current_type(ctx, &base, db, file);
    if let Some(narrowed) = narrow_shape_path_instanceof(
        &current,
        &path,
        class_name,
        db,
        &ctx.template_param_names,
        is_true,
    ) {
        set_shape_base_narrowed(ctx, &base, current, narrowed);
    }
}

/// Narrow `ty` along a shape-key access path by an `instanceof` check on the
/// key's OWN value — same recursion/exclusion shape as `narrow_shape_path_null`,
/// with the leaf swapped for `narrow_instanceof_preserving_subtypes`/
/// `filter_out_instanceof_match`. A closed shape lacking the key entirely
/// always reads as `null` (never an instance of anything), so it's excluded
/// on the `is_true` side — the true-branch analogue of `narrow_shape_path_null`
/// excluding a closed shape missing the key on its non-null side.
fn narrow_shape_path_instanceof(
    ty: &Type,
    path: &[mir_types::atomic::ArrayKey],
    class_name: &str,
    db: &dyn MirDatabase,
    template_param_names: &rustc_hash::FxHashSet<mir_types::Name>,
    is_true: bool,
) -> Option<Type> {
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
                let Some(prop) = properties.get(key) else {
                    if is_true && !*is_open {
                        // Closed shape without this key always reads null —
                        // `instanceof` is always false for it, so this union
                        // member can't satisfy the true branch.
                        changed = true;
                    } else {
                        result.add_type(atomic.clone());
                    }
                    continue;
                };
                if rest.is_empty() {
                    let narrowed_ty = if is_true {
                        narrow_instanceof_preserving_subtypes(
                            &prop.ty,
                            class_name,
                            db,
                            template_param_names,
                        )
                    } else {
                        filter_out_instanceof_match(&prop.ty, class_name, db)
                    };
                    if narrowed_ty.is_empty() {
                        changed = true;
                        continue;
                    }
                    let mut new_props = properties.clone();
                    if let Some(p) = new_props.get_mut(key) {
                        p.ty = narrowed_ty;
                        if is_true {
                            p.optional = false;
                        }
                    }
                    changed = true;
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                } else if let Some(deeper) = narrow_shape_path_instanceof(
                    &prop.ty,
                    rest,
                    class_name,
                    db,
                    template_param_names,
                    is_true,
                ) {
                    let mut new_props = properties.clone();
                    if let Some(p) = new_props.get_mut(key) {
                        p.ty = deeper;
                    }
                    changed = true;
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
    if changed && !result.types.is_empty() {
        Some(result)
    } else {
        None
    }
}

/// `is_string($base['a']['b'])`/`is_int(...)`/`is_null(...)`/etc. — the
/// `is_*`/`ctype_*` type-check counterpart of `narrow_offset_instanceof_by_path`,
/// reusing the same `type_fn_narrowed` leaf the plain-variable/property arms
/// already share. Same literal-keyed-shape-only scope.
pub(crate) fn narrow_offset_type_fn_by_path(
    var_expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    fn_name: &str,
    db: &dyn MirDatabase,
    file: &str,
    is_true: bool,
) {
    let Some((base, path)) = collect_array_access_path(var_expr, ctx, db, file) else {
        return;
    };
    let current = resolve_shape_base_current_type(ctx, &base, db, file);
    if let Some(narrowed) = narrow_shape_path_type_fn(&current, &path, fn_name, db, is_true) {
        set_shape_base_narrowed(ctx, &base, current, narrowed);
    }
}

/// Narrow `ty` along a shape-key access path by an `is_*`/`ctype_*` type
/// check on the key's OWN value. Mirrors `narrow_shape_path_instanceof`'s
/// recursion/exclusion shape; a closed shape lacking the key always reads as
/// `null`, so it can only satisfy `is_null()`'s true branch (or any other
/// check's false branch) — the same `is_true != is_null_check` reasoning
/// `narrow_prop_from_type_fn` already uses for property receivers.
fn narrow_shape_path_type_fn(
    ty: &Type,
    path: &[mir_types::atomic::ArrayKey],
    fn_name: &str,
    db: &dyn MirDatabase,
    is_true: bool,
) -> Option<Type> {
    let (key, rest) = path.split_first()?;
    let is_null_check = crate::util::php_ident_lowercase(fn_name) == "is_null";
    let proves_non_null = is_true != is_null_check;
    let mut changed = false;
    let mut result = Type::empty();
    for atomic in &ty.types {
        match atomic {
            Atomic::TKeyedArray {
                properties,
                is_open,
                is_list,
            } => {
                let Some(prop) = properties.get(key) else {
                    if proves_non_null && !*is_open {
                        // Closed shape without this key always reads null —
                        // impossible to satisfy a check that proves non-null.
                        changed = true;
                    } else {
                        result.add_type(atomic.clone());
                    }
                    continue;
                };
                if rest.is_empty() {
                    let Some(narrowed_ty) = type_fn_narrowed(&prop.ty, fn_name, db, is_true) else {
                        result.add_type(atomic.clone());
                        continue;
                    };
                    if narrowed_ty.is_empty() {
                        changed = true;
                        continue;
                    }
                    let mut new_props = properties.clone();
                    if let Some(p) = new_props.get_mut(key) {
                        p.ty = narrowed_ty;
                        if proves_non_null {
                            p.optional = false;
                        }
                    }
                    changed = true;
                    result.add_type(Atomic::TKeyedArray {
                        properties: new_props,
                        is_open: *is_open,
                        is_list: *is_list,
                    });
                } else if let Some(deeper) =
                    narrow_shape_path_type_fn(&prop.ty, rest, fn_name, db, is_true)
                {
                    let mut new_props = properties.clone();
                    if let Some(p) = new_props.get_mut(key) {
                        p.ty = deeper;
                    }
                    changed = true;
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
    if changed && !result.types.is_empty() {
        Some(result)
    } else {
        None
    }
}

/// False-branch counterpart of `narrow_isset_shape_key`, for
/// `!isset($base['key'])`. Weaker than `array_key_exists`'s false branch
/// (`narrow_shape_path_key_exists_false`/`remove_key_from_sealed_shapes`):
/// `!isset(...)` is true when the key is either absent OR present-but-null,
/// so the only sound exclusion is a union member where the key is present,
/// not optional, and its declared type doesn't include `null` — that member
/// would have made `isset()` true, contradicting the false branch.
///
/// Scoped to single-level access only (`path.len() == 1`): a nested false
/// branch doesn't pin down which level failed, mirroring
/// `narrow_empty_shape_key`'s identical scoping decision for `empty()`.
pub(crate) fn narrow_isset_shape_key_false(
    var_expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    let Some((base, path)) = collect_array_access_path(var_expr, ctx, db, file) else {
        return;
    };
    if path.len() != 1 {
        return;
    }
    let key = &path[0];
    let current = resolve_shape_base_current_type(ctx, &base, db, file);
    let mut changed = false;
    let mut result = Type::empty();
    for atomic in &current.types {
        if let Atomic::TKeyedArray { properties, .. } = atomic {
            if let Some(prop) = properties.get(key) {
                let definitely_present_non_null =
                    !prop.optional && !prop.ty.types.iter().any(|a| matches!(a, Atomic::TNull));
                if definitely_present_non_null {
                    // isset() would necessarily be true for this member —
                    // impossible under the false branch, so exclude it.
                    changed = true;
                    continue;
                }
            }
        }
        result.add_type(atomic.clone());
    }
    if changed && !result.types.is_empty() {
        set_shape_base_narrowed(ctx, &base, current, result);
    }
}

/// For `array_key_exists('key', $base['a']['b']...)` where the array
/// argument is itself a nested shape-key access: walk down `path` to the
/// container shape, then apply `array_key_exists`'s own key-presence
/// semantics (`add_key_to_sealed_shapes`) there — parallel to
/// `narrow_shape_path`, but the leaf operation proves a *given* key present
/// in the container rather than the last path segment itself.
pub(crate) fn narrow_shape_path_key_exists(
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
pub(crate) fn narrow_shape_path_key_exists_false(
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

/// Condition-matching glue for `narrow_from_condition`'s `===`/`!==` and
/// `==`/`!=` arms: recognizes `$arr === []` / `$arr !== []` (and the loose
/// `==`/`!=` equivalents, equally sound since loose array equality requires
/// identical key/value pairs) for a var, prop, or static-prop receiver on
/// either side of the comparison, dispatching to `narrow_prop_array_empty`/
/// `narrow_static_prop_array_empty` above (the plain-variable case is
/// narrowed inline, mirroring those two). Returns whether an empty-array-
/// literal side was found at all — regardless of whether narrowing actually
/// changed anything — since callers rely on this to short-circuit their own
/// `else if` chain exactly as the inlined form did.
pub(crate) fn narrow_array_emptiness_condition(
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
    left: &php_ast::owned::Expr,
    right: &php_ast::owned::Expr,
    effective_true: bool,
) -> bool {
    if let ExprKind::Array(elems) = &right.kind {
        if elems.is_empty() {
            if let Some(var_name) = extract_var_name(left) {
                let current = ctx.get_var(&var_name);
                let narrowed = if effective_true {
                    current.narrow_to_empty_collection()
                } else {
                    current.narrow_to_non_empty_collection()
                };
                if !narrowed.is_empty() && narrowed != current {
                    ctx.set_var(&var_name, narrowed);
                }
            } else if let Some((obj, prop)) = extract_any_prop_access(left) {
                narrow_prop_array_empty(ctx, &obj, &prop, db, file, effective_true);
                narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
            } else if let Some((fqcn, prop)) = extract_static_prop_access(left, ctx, db, file) {
                narrow_static_prop_array_empty(ctx, &fqcn, &prop, db, effective_true);
            }
        }
        true
    } else if let ExprKind::Array(elems) = &left.kind {
        if elems.is_empty() {
            if let Some(var_name) = extract_var_name(right) {
                let current = ctx.get_var(&var_name);
                let narrowed = if effective_true {
                    current.narrow_to_empty_collection()
                } else {
                    current.narrow_to_non_empty_collection()
                };
                if !narrowed.is_empty() && narrowed != current {
                    ctx.set_var(&var_name, narrowed);
                }
            } else if let Some((obj, prop)) = extract_any_prop_access(right) {
                narrow_prop_array_empty(ctx, &obj, &prop, db, file, effective_true);
                narrow_receiver_non_null_on_prop_match(ctx, &obj, effective_true);
            } else if let Some((fqcn, prop)) = extract_static_prop_access(right, ctx, db, file) {
                narrow_static_prop_array_empty(ctx, &fqcn, &prop, db, effective_true);
            }
        }
        true
    } else {
        false
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
pub(crate) fn narrow_empty_shape_key(
    var_expr: &php_ast::owned::Expr,
    ctx: &mut FlowState,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    let Some((base, path)) = collect_array_access_path(var_expr, ctx, db, file) else {
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
