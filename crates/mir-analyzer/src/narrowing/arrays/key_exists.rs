//! `array_key_exists()`/`key_exists()` narrowing for property and
//! static-property receivers, plus the sealed-shape key-presence helpers
//! shared with the variable case and with `shapes`'s nested-path variants.
use php_ast::owned::{ExprKind, FunctionCallExpr};

use mir_types::Atomic;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::super::class_introspection::{
    extract_class_implements_or_parents_arg, extract_class_implements_or_parents_static_prop_arg,
};
use super::super::core::{
    apply_prop_narrowed, extract_any_prop_access, extract_static_prop_access, extract_var_name,
    narrow_receiver_non_null_on_prop_match, resolve_prop_current_type,
    resolve_static_prop_current_type, set_narrowed, ScalarArgTarget,
};
use super::super::instanceof_core::{
    filter_out_instanceof_match, narrow_instanceof_preserving_subtypes, narrow_prop_instanceof,
    narrow_prop_is_subclass_of, narrow_static_prop_instanceof, narrow_static_prop_is_subclass_of,
    narrow_strict_subclass_of,
};
use super::shapes::{
    collect_array_access_path, narrow_shape_path_key_exists, narrow_shape_path_key_exists_false,
    resolve_shape_base_current_type, set_shape_base_narrowed,
};

/// Static-property counterpart of `narrow_prop_array_key_exists`, for
/// `array_key_exists('k', self::$prop)` (and `static::$prop`/`Class::$prop`).
/// Mirrors the var/prop siblings' true-branch convention: just apply the
/// narrowed shape, no divergence marking.
pub(crate) fn narrow_static_prop_array_key_exists(
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
    // array_key_exists() throws TypeError on a null 2nd arg, so reaching
    // this point already proves the property itself wasn't null.
    let non_null = current.remove_null();
    let narrowed = add_key_to_sealed_shapes(&non_null, key);
    if narrowed != current {
        ctx.set_prop_refined(fqcn, prop, narrowed);
    }
}

/// Narrow a property's type when `array_key_exists('k', $this->prop)` is proven true.
pub(crate) fn narrow_prop_array_key_exists(
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
    // array_key_exists() throws TypeError on a null 2nd arg, so reaching
    // this point already proves the property itself wasn't null.
    let non_null = current.remove_null();
    let narrowed = add_key_to_sealed_shapes(&non_null, key);
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
pub(crate) fn add_key_to_sealed_shapes(
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
pub(crate) fn remove_key_from_sealed_shapes(
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

/// Condition-matching glue for `narrow_from_condition`'s `FunctionCall` arm:
/// handles `array_key_exists('k', $arr)` / `key_exists('k', $arr)` (and the
/// `class_implements()`/`class_parents()`-haystack special cases), dispatching
/// to the var/prop/static-prop/nested-shape narrowing helpers above. Callers
/// are expected to have already checked that the function name is
/// `array_key_exists`/`key_exists` before calling this.
pub(crate) fn narrow_array_key_exists_condition(
    ctx: &mut FlowState,
    call: &FunctionCallExpr,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    // array_key_exists('k', $arr) in true-branch: prove the key
    // exists in the array's sealed shape so that $arr['k'] does
    // not trigger NonExistentArrayOffset afterwards.
    // `key_exists()` is a built-in alias of `array_key_exists()`
    // with identical semantics.
    if let (Some(key_arg), Some(arr_arg)) = (call.args.first(), call.args.get(1)) {
        let literal_key = match &key_arg.value.kind {
            ExprKind::String(s) => Some(mir_types::atomic::ArrayKey::String(std::sync::Arc::from(
                s.as_ref(),
            ))),
            ExprKind::Int(i) => Some(mir_types::atomic::ArrayKey::Int(*i)),
            // `$key = 'name'; array_key_exists($key, $arr)` — resolve a
            // variable, property-access, or static-property key already
            // narrowed to a single literal, same as an inline literal
            // would be.
            _ => {
                let key_ty = if let Some(name) = extract_var_name(&key_arg.value) {
                    Some(ctx.get_var(&name))
                } else if let Some((obj, prop)) = extract_any_prop_access(&key_arg.value) {
                    Some(resolve_prop_current_type(ctx, &obj, &prop, db, file))
                } else {
                    extract_static_prop_access(&key_arg.value, ctx, db, file)
                        .map(|(fqcn, prop)| resolve_static_prop_current_type(ctx, &fqcn, &prop, db))
                };
                key_ty.and_then(|ty| match ty.types.as_slice() {
                    [Atomic::TLiteralString(s)] => {
                        Some(mir_types::atomic::ArrayKey::String(s.clone()))
                    }
                    [Atomic::TLiteralInt(i)] => Some(mir_types::atomic::ArrayKey::Int(*i)),
                    _ => None,
                })
            }
        };
        if let Some(key) = literal_key {
            if is_true {
                if let Some(var_name) = extract_var_name(&arr_arg.value) {
                    let current = ctx.get_var(&var_name);
                    // array_key_exists() throws TypeError on a null 2nd arg,
                    // so reaching the true branch already proves $arr wasn't
                    // null.
                    let non_null = current.remove_null();
                    let narrowed = add_key_to_sealed_shapes(&non_null, &key);
                    if narrowed != current {
                        ctx.set_var(&var_name, narrowed);
                    }
                } else if let Some((obj, prop)) = extract_any_prop_access(&arr_arg.value) {
                    narrow_prop_array_key_exists(ctx, &obj, &prop, &key, db, file);
                    // array_key_exists() throws TypeError on a null 2nd
                    // arg, so reaching the true branch already proves
                    // $obj->prop (and thus $obj) was non-null.
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&arr_arg.value, ctx, db, file)
                {
                    narrow_static_prop_array_key_exists(ctx, &fqcn, &prop, &key, db);
                } else if let Some((base, path)) =
                    collect_array_access_path(&arr_arg.value, ctx, db, file)
                {
                    // Nested container, e.g. array_key_exists('b', $arr['a']) —
                    // walk down to the ['a'] shape and prove 'b' present there,
                    // same as the single-level var/prop cases above.
                    let current = resolve_shape_base_current_type(ctx, &base, db, file);
                    if let Some(narrowed) = narrow_shape_path_key_exists(&current, &path, &key) {
                        set_shape_base_narrowed(ctx, &base, current, narrowed);
                    }
                } else if let (
                    mir_types::atomic::ArrayKey::String(iface_name),
                    Some((target, is_parents)),
                ) = (
                    &key,
                    extract_class_implements_or_parents_arg(&arr_arg.value),
                ) {
                    // array_key_exists('Iface', class_implements($x)) —
                    // same relationship `$x instanceof Iface` proves.
                    // array_key_exists('Ancestor', class_parents($x)) is
                    // STRICTER: class_parents() excludes $x's own exact
                    // class, the same relationship `is_subclass_of($x,
                    // Ancestor)` proves — reuse that narrowing instead.
                    let fqcn = crate::db::resolve_name(db, file, iface_name);
                    match &target {
                        ScalarArgTarget::Var(var_name) => {
                            let current = ctx.get_var(var_name);
                            let narrowed = if is_parents {
                                narrow_strict_subclass_of(
                                    &current,
                                    &fqcn,
                                    db,
                                    &ctx.template_param_names,
                                )
                            } else {
                                narrow_instanceof_preserving_subtypes(
                                    &current,
                                    &fqcn,
                                    db,
                                    &ctx.template_param_names,
                                )
                            };
                            set_narrowed(ctx, var_name, &current, narrowed, true);
                        }
                        ScalarArgTarget::Prop(obj, prop) => {
                            if is_parents {
                                narrow_prop_is_subclass_of(ctx, obj, prop, &fqcn, db, file, true);
                            } else {
                                narrow_prop_instanceof(ctx, obj, prop, &fqcn, db, file, true);
                            }
                            narrow_receiver_non_null_on_prop_match(ctx, obj, true);
                        }
                    }
                } else if let (
                    mir_types::atomic::ArrayKey::String(iface_name),
                    Some(((static_fqcn, prop), is_parents)),
                ) = (
                    &key,
                    extract_class_implements_or_parents_static_prop_arg(
                        &arr_arg.value,
                        ctx,
                        db,
                        file,
                    ),
                ) {
                    // array_key_exists('Iface', class_implements(self::$prop)) —
                    // static-property counterpart of the var/prop arm above.
                    let fqcn = crate::db::resolve_name(db, file, iface_name);
                    if is_parents {
                        narrow_static_prop_is_subclass_of(
                            ctx,
                            &static_fqcn,
                            &prop,
                            &fqcn,
                            db,
                            true,
                        );
                    } else {
                        narrow_static_prop_instanceof(ctx, &static_fqcn, &prop, &fqcn, db, true);
                    }
                }
            } else {
                // False branch: exclude shape members that
                // guarantee the key's presence — see
                // `remove_key_from_sealed_shapes`.
                if let Some(var_name) = extract_var_name(&arr_arg.value) {
                    let current = ctx.get_var(&var_name);
                    // array_key_exists() throws TypeError on a null 2nd arg,
                    // so reaching the false branch also proves $arr wasn't
                    // null.
                    let non_null = current.remove_null();
                    let narrowed = remove_key_from_sealed_shapes(&non_null, &key);
                    set_narrowed(ctx, &var_name, &current, narrowed, true);
                } else if let Some((obj, prop)) = extract_any_prop_access(&arr_arg.value) {
                    let current = resolve_prop_current_type(ctx, &obj, &prop, db, file);
                    if !current.is_mixed() {
                        let non_null = current.remove_null();
                        let narrowed = remove_key_from_sealed_shapes(&non_null, &key);
                        apply_prop_narrowed(ctx, &obj, &prop, current, narrowed, true);
                    }
                    // array_key_exists() throws TypeError on a null 2nd
                    // arg, so reaching the false branch also proves
                    // $obj->prop (and thus $obj) was non-null.
                    narrow_receiver_non_null_on_prop_match(ctx, &obj, true);
                } else if let Some((fqcn, prop)) =
                    extract_static_prop_access(&arr_arg.value, ctx, db, file)
                {
                    let current = resolve_static_prop_current_type(ctx, &fqcn, &prop, db);
                    if !current.is_mixed() {
                        let non_null = current.remove_null();
                        let narrowed = remove_key_from_sealed_shapes(&non_null, &key);
                        apply_prop_narrowed(ctx, &fqcn, &prop, current, narrowed, true);
                    }
                } else if let Some((base, path)) =
                    collect_array_access_path(&arr_arg.value, ctx, db, file)
                {
                    // Nested container, false branch, e.g.
                    // array_key_exists('b', $arr['a']) proven
                    // false — same as the single-level
                    // var/prop cases above.
                    let current = resolve_shape_base_current_type(ctx, &base, db, file);
                    if let Some(narrowed) =
                        narrow_shape_path_key_exists_false(&current, &path, &key)
                    {
                        set_shape_base_narrowed(ctx, &base, current, narrowed);
                    }
                } else if let (
                    mir_types::atomic::ArrayKey::String(iface_name),
                    Some((target, is_parents)),
                ) = (
                    &key,
                    extract_class_implements_or_parents_arg(&arr_arg.value),
                ) {
                    // !array_key_exists('Iface', class_implements($x)) —
                    // exclude Iface, same as `!($x instanceof Iface)`.
                    // class_parents(), by contrast, never narrows on the
                    // false branch — mirrors `is_subclass_of()`'s own
                    // convention: a parent-name mismatch doesn't rule out
                    // the receiver being exactly that class.
                    if !is_parents {
                        let fqcn = crate::db::resolve_name(db, file, iface_name);
                        match &target {
                            ScalarArgTarget::Var(var_name) => {
                                let current = ctx.get_var(var_name);
                                let narrowed = filter_out_instanceof_match(&current, &fqcn, db);
                                set_narrowed(ctx, var_name, &current, narrowed, true);
                            }
                            ScalarArgTarget::Prop(obj, prop) => {
                                narrow_prop_instanceof(ctx, obj, prop, &fqcn, db, file, false);
                            }
                        }
                    }
                } else if let (
                    mir_types::atomic::ArrayKey::String(iface_name),
                    Some(((static_fqcn, prop), is_parents)),
                ) = (
                    &key,
                    extract_class_implements_or_parents_static_prop_arg(
                        &arr_arg.value,
                        ctx,
                        db,
                        file,
                    ),
                ) {
                    // !array_key_exists('Iface', class_implements(self::$prop)) —
                    // static-property counterpart of the var/prop arm above.
                    if !is_parents {
                        let fqcn = crate::db::resolve_name(db, file, iface_name);
                        narrow_static_prop_instanceof(ctx, &static_fqcn, &prop, &fqcn, db, false);
                    }
                }
            }
        }
    }
}
