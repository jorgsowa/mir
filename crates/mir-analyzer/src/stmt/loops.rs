use crate::db::MirDatabase;
use mir_types::{ArrayKey, Atomic, Name, Type};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Loop execution guarantees
// ---------------------------------------------------------------------------

/// Returns true if a foreach loop over `arr_ty` is guaranteed to execute at least once.
/// A loop is guaranteed to execute if the array is known to be non-empty.
pub(super) fn loop_guaranteed_to_execute(arr_ty: &Type) -> bool {
    for atomic in &arr_ty.types {
        match atomic {
            // Non-empty array types guarantee at least one iteration
            Atomic::TNonEmptyArray { .. } | Atomic::TNonEmptyList { .. } => return true,
            // Keyed arrays with known properties are non-empty if closed and not empty
            Atomic::TKeyedArray {
                properties,
                is_open: false,
                ..
            } if !properties.is_empty() => return true,
            _ => {}
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Loop widening helpers
// ---------------------------------------------------------------------------

/// Returns true when every variable present in `prev` has the same type in
/// `next`, indicating the fixed-point has been reached.
pub(super) fn vars_stabilized(
    prev: &rustc_hash::FxHashMap<Name, Arc<Type>>,
    next: &rustc_hash::FxHashMap<Name, Arc<Type>>,
) -> bool {
    if prev.len() != next.len() {
        return false;
    }
    prev.iter().all(|(k, v)| {
        next.get(k)
            .map(|u| Arc::ptr_eq(u, v) || **u == **v)
            .unwrap_or(false)
    })
}

/// For any variable whose type changed relative to `pre_vars`, widen to
/// the union of both types.  Called after MAX_ITERS to avoid non-termination.
///
/// If `loop_guaranteed` is true (loop is guaranteed to execute at least once),
/// variables that are new in the loop (only in current, not in pre) won't be
/// merged with null/undefined, since the loop will definitely assign them.
pub(super) fn widen_unstable(
    pre_vars: &rustc_hash::FxHashMap<Name, Arc<Type>>,
    current_vars: &mut rustc_hash::FxHashMap<Name, Arc<Type>>,
    loop_guaranteed: bool,
) {
    for (name, ty) in current_vars.iter_mut() {
        if let Some(pre_ty) = pre_vars.get(name) {
            if !Arc::ptr_eq(ty, pre_ty) && **ty != **pre_ty {
                let mut merged = (**ty).clone();
                merged.merge_with(pre_ty);
                *ty = mir_codebase::storage::wrap_var_type(merged);
            }
        } else if loop_guaranteed {
            // Variable is new in loop and loop is guaranteed to execute.
            // Don't merge with pre-type (which would be null/undefined).
            // The variable type is just its assigned value.
        } else {
            // Loop might not execute; variable might be undefined.
            // Leave as-is since it's already set in the entry context.
        }
    }
}

// ---------------------------------------------------------------------------
// foreach key/value type inference
// ---------------------------------------------------------------------------

pub(crate) fn infer_foreach_types(arr_ty: &Type) -> (Type, Type) {
    if arr_ty.is_mixed() {
        return (Type::mixed(), Type::mixed());
    }
    for atomic in &arr_ty.types {
        match atomic {
            Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
                return (*key.clone(), *value.clone());
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                return (Type::single(Atomic::TInt), *value.clone());
            }
            Atomic::TKeyedArray { properties, .. } => {
                let mut keys = Type::empty();
                let mut values = Type::empty();
                for (k, prop) in properties {
                    let key_atomic = match k {
                        ArrayKey::String(s) => Atomic::TLiteralString(s.clone()),
                        ArrayKey::Int(i) => Atomic::TLiteralInt(*i),
                    };
                    keys.merge_with(&Type::single(key_atomic));
                    values.merge_with(&prop.ty);
                }
                // Empty keyed array (e.g. `$arr = []` before push) — treat both as
                // mixed to avoid propagating Type::empty() as a variable type.
                let keys = if keys.is_empty() { Type::mixed() } else { keys };
                let values = if values.is_empty() {
                    Type::mixed()
                } else {
                    values
                };
                return (keys, values);
            }
            Atomic::TString => {
                return (Type::single(Atomic::TInt), Type::single(Atomic::TString));
            }
            _ => {}
        }
    }
    (Type::mixed(), Type::mixed())
}

/// Like [`infer_foreach_types`], but also resolves `foreach` over an object —
/// a `Generator`, or a user-defined class implementing `Iterator` /
/// `IteratorAggregate` — into its key/value item types, instead of always
/// falling back to `mixed`.
pub(crate) fn infer_foreach_types_with_db(db: &dyn MirDatabase, arr_ty: &Type) -> (Type, Type) {
    infer_foreach_types_with_db_depth(db, arr_ty, 4)
}

fn infer_foreach_types_with_db_depth(
    db: &dyn MirDatabase,
    arr_ty: &Type,
    depth: u8,
) -> (Type, Type) {
    if depth == 0 || arr_ty.is_mixed() {
        return (Type::mixed(), Type::mixed());
    }
    for atomic in &arr_ty.types {
        if let Atomic::TNamedObject { fqcn, type_params } = atomic {
            if let Some(kv) = resolve_iterator_item_types(db, fqcn, type_params, depth) {
                return kv;
            }
        }
    }
    infer_foreach_types(arr_ty)
}

/// `Generator<TKey, TValue, TSend, TReturn>` (per the stdlib generic stub) —
/// a bare `Generator` or the one-arg `Generator<TValue>` shorthand fall back
/// to a mixed/inferred key.
fn generator_item_types(type_params: &[Type]) -> (Type, Type) {
    match type_params {
        [] => (Type::mixed(), Type::mixed()),
        [value] => (Type::mixed(), value.clone()),
        [key, value, ..] => (key.clone(), value.clone()),
    }
}

/// Resolve the key/value item types `foreach` produces for an instance of
/// `fqcn<type_params>`. Returns `None` when `fqcn` isn't `Generator` and
/// doesn't implement `Iterator`/`IteratorAggregate` (or the info needed to
/// resolve it further just isn't available) — the caller then falls back to
/// treating the object as non-iterable-typed (`mixed`/`mixed`).
fn resolve_iterator_item_types(
    db: &dyn MirDatabase,
    fqcn: &Name,
    type_params: &[Type],
    depth: u8,
) -> Option<(Type, Type)> {
    let bare = fqcn.as_ref().trim_start_matches('\\');
    if bare.eq_ignore_ascii_case("Generator") {
        return Some(generator_item_types(type_params));
    }
    // The receiver's static type may itself be one of the built-in iteration
    // interfaces used generically — e.g. `@param Iterator<int, string> $x` —
    // rather than a concrete class implementing it. There's no `current()`/
    // `getIterator()` to chase in that case; the annotation's own type args
    // (if supplied) directly are the key/value types.
    if (bare.eq_ignore_ascii_case("Iterator")
        || bare.eq_ignore_ascii_case("IteratorAggregate")
        || bare.eq_ignore_ascii_case("Traversable"))
        && !type_params.is_empty()
    {
        return Some(generator_item_types(type_params));
    }

    let class = crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, bare))?;
    let class_tps = crate::db::class_template_params(db, bare).unwrap_or_default();
    let bindings = crate::generic::build_class_bindings(&class_tps, type_params);

    // Prefer an explicit `@implements Iterator<TKey, TValue>` (or
    // `IteratorAggregate<TKey, TValue>`) annotation: it directly states the
    // item types without needing to chase `current()`/`getIterator()`.
    let annotated = class
        .implements_type_args()
        .iter()
        .find_map(|(iface, args)| {
            let iface_bare = iface.trim_start_matches('\\');
            (iface_bare.eq_ignore_ascii_case("Iterator")
                || iface_bare.eq_ignore_ascii_case("IteratorAggregate"))
            .then_some(args)
        });
    if let Some(args) = annotated {
        if args.len() >= 2 {
            let key = args[0].substitute_templates(&bindings);
            let value = args[1].substitute_templates(&bindings);
            return Some((key, value));
        }
    }

    let implements = |name: &str| {
        class
            .interfaces()
            .iter()
            .any(|i| i.trim_start_matches('\\').eq_ignore_ascii_case(name))
    };
    let method_return_ty = |method: &str| -> Option<Type> {
        let (_, def) =
            crate::db::find_method_in_chain(db, crate::db::Fqcn::from_str(db, bare), method)?;
        let ty = def.return_type.as_deref().cloned()?;
        Some(ty.substitute_templates(&bindings))
    };

    if implements("IteratorAggregate") {
        let ret_ty = method_return_ty("getiterator")?;
        return Some(infer_foreach_types_with_db_depth(db, &ret_ty, depth - 1));
    }
    if implements("Iterator") {
        let value = method_return_ty("current").unwrap_or_else(Type::mixed);
        let key = method_return_ty("key").unwrap_or_else(Type::mixed);
        return Some((key, value));
    }
    None
}
