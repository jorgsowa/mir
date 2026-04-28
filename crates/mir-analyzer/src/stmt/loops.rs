use indexmap::IndexMap;
use mir_types::{ArrayKey, Atomic, Union};

// ---------------------------------------------------------------------------
// Loop widening helpers
// ---------------------------------------------------------------------------

/// Returns true when every variable present in `prev` has the same type in
/// `next`, indicating the fixed-point has been reached.
pub(super) fn vars_stabilized(
    prev: &IndexMap<String, Union>,
    next: &IndexMap<String, Union>,
) -> bool {
    if prev.len() != next.len() {
        return false;
    }
    prev.iter()
        .all(|(k, v)| next.get(k).map(|u| u == v).unwrap_or(false))
}

/// For any variable whose type changed relative to `pre_vars`, widen to
/// `mixed`.  Called after MAX_ITERS to avoid non-termination.
pub(super) fn widen_unstable(
    pre_vars: &IndexMap<String, Union>,
    current_vars: &mut IndexMap<String, Union>,
) {
    for (name, ty) in current_vars.iter_mut() {
        if pre_vars.get(name).map(|p| p != ty).unwrap_or(true) && !ty.is_mixed() {
            *ty = Union::mixed();
        }
    }
}

// ---------------------------------------------------------------------------
// foreach key/value type inference
// ---------------------------------------------------------------------------

pub(super) fn infer_foreach_types(arr_ty: &Union) -> (Union, Union) {
    if arr_ty.is_mixed() {
        return (Union::mixed(), Union::mixed());
    }
    for atomic in &arr_ty.types {
        match atomic {
            Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
                return (*key.clone(), *value.clone());
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                return (Union::single(Atomic::TInt), *value.clone());
            }
            Atomic::TKeyedArray { properties, .. } => {
                let mut keys = Union::empty();
                let mut values = Union::empty();
                for (k, prop) in properties {
                    let key_atomic = match k {
                        ArrayKey::String(s) => Atomic::TLiteralString(s.clone()),
                        ArrayKey::Int(i) => Atomic::TLiteralInt(*i),
                    };
                    keys = Union::merge(&keys, &Union::single(key_atomic));
                    values = Union::merge(&values, &prop.ty);
                }
                // Empty keyed array (e.g. `$arr = []` before push) — treat both as
                // mixed to avoid propagating Union::empty() as a variable type.
                let keys = if keys.is_empty() {
                    Union::mixed()
                } else {
                    keys
                };
                let values = if values.is_empty() {
                    Union::mixed()
                } else {
                    values
                };
                return (keys, values);
            }
            Atomic::TString => {
                return (Union::single(Atomic::TInt), Union::single(Atomic::TString));
            }
            _ => {}
        }
    }
    (Union::mixed(), Union::mixed())
}
