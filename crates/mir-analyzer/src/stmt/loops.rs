use mir_types::{ArrayKey, Atomic, Symbol, Union};

// ---------------------------------------------------------------------------
// Loop execution guarantees
// ---------------------------------------------------------------------------

/// Returns true if a foreach loop over `arr_ty` is guaranteed to execute at least once.
/// A loop is guaranteed to execute if the array is known to be non-empty.
pub(super) fn loop_guaranteed_to_execute(arr_ty: &Union) -> bool {
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
    prev: &im::HashMap<Symbol, Union>,
    next: &im::HashMap<Symbol, Union>,
) -> bool {
    if prev.len() != next.len() {
        return false;
    }
    prev.iter()
        .all(|(k, v)| next.get(k).map(|u| u == v).unwrap_or(false))
}

/// For any variable whose type changed relative to `pre_vars`, widen to
/// the union of both types.  Called after MAX_ITERS to avoid non-termination.
///
/// If `loop_guaranteed` is true (loop is guaranteed to execute at least once),
/// variables that are new in the loop (only in current, not in pre) won't be
/// merged with null/undefined, since the loop will definitely assign them.
pub(super) fn widen_unstable(
    pre_vars: &im::HashMap<Symbol, Union>,
    current_vars: &mut im::HashMap<Symbol, Union>,
    loop_guaranteed: bool,
) {
    for (name, ty) in current_vars.iter_mut() {
        if let Some(pre_ty) = pre_vars.get(name) {
            if pre_ty != ty {
                ty.merge_with(pre_ty);
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
                    keys.merge_with(&Union::single(key_atomic));
                    values.merge_with(&prop.ty);
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
