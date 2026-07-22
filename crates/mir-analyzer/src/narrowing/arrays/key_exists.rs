//! `array_key_exists()`/`key_exists()` narrowing for property and
//! static-property receivers, plus the sealed-shape key-presence helpers
//! shared with the variable case and with `shapes`'s nested-path variants.
use mir_types::Atomic;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::super::core::{resolve_prop_current_type, resolve_static_prop_current_type};

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
    let narrowed = add_key_to_sealed_shapes(&current, key);
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
