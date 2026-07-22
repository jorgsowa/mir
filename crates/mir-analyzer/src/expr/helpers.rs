use mir_types::{ArrayKey, Atomic, Name, Type};
use php_ast::ast::BinaryOp;
use php_ast::owned::{Expr, ExprKind};
use rustc_hash::FxHashSet;

use crate::subtype::is_subtype;

/// PHP canonicalizes a numeric string array key (e.g. `"0"`, `"42"`, `"-5"`)
/// to an int key at runtime — `$arr['0']` and `$arr[0]` are the same slot.
/// Returns the canonical int when `s` is such a string; `None` means `s`
/// stays a string key (e.g. `"01"`, `"+1"`, `"-0"`, `"1.0"`, `""`).
pub fn canonical_int_array_key(s: &str) -> Option<i64> {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let (neg, digits) = if bytes[0] == b'-' {
        (true, &bytes[1..])
    } else {
        (false, bytes)
    };
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    // No leading zero unless the value is exactly "0"; PHP also treats "-0"
    // as a non-canonical string key.
    if digits.len() > 1 && digits[0] == b'0' {
        return None;
    }
    if neg && digits == b"0" {
        return None;
    }
    std::str::from_utf8(digits)
        .ok()?
        .parse::<i64>()
        .ok()
        .map(|v| if neg { -v } else { v })
}

/// Resolve an index expression to a literal array key, applying PHP's own
/// array-key casting rules: numeric strings canonicalize to int (`"0"` → `0`,
/// same as [`canonical_int_array_key`]), bools cast to `0`/`1`, floats
/// truncate toward zero, and `null` casts to `""`. Returns `None` for a
/// dynamic (non-literal) index.
pub fn literal_array_key_of_kind(kind: &ExprKind) -> Option<ArrayKey> {
    match kind {
        ExprKind::String(s) => Some(match canonical_int_array_key(s) {
            Some(i) => ArrayKey::Int(i),
            None => ArrayKey::String(std::sync::Arc::from(s.as_ref())),
        }),
        ExprKind::Int(i) => Some(ArrayKey::Int(*i)),
        ExprKind::Bool(b) => Some(ArrayKey::Int(if *b { 1 } else { 0 })),
        ExprKind::Float(f) => Some(ArrayKey::Int(*f as i64)),
        ExprKind::Null => Some(ArrayKey::String(std::sync::Arc::from(""))),
        _ => None,
    }
}

/// Coerce a general index-expression type to PHP's canonical array-key
/// representation: bools cast to `0`/`1`, floats truncate toward zero, `null`
/// casts to `""`, and a numeric string canonicalizes to int — mirroring
/// [`literal_array_key_of_kind`] for callers that already have a `Type`
/// (constant-folded expressions, a generic fallback path) rather than the
/// raw index `ExprKind`. Atoms that aren't a legal-but-uncanonical key type
/// pass through unchanged.
pub fn coerce_array_key_type(ty: &Type) -> Type {
    let mut changed = false;
    let mut out = Type::empty();
    for a in &ty.types {
        match a {
            Atomic::TTrue => {
                changed = true;
                out.add_type(Atomic::TLiteralInt(1));
            }
            Atomic::TFalse => {
                changed = true;
                out.add_type(Atomic::TLiteralInt(0));
            }
            Atomic::TBool => {
                changed = true;
                out.add_type(Atomic::TInt);
            }
            Atomic::TNull => {
                changed = true;
                out.add_type(Atomic::TLiteralString(std::sync::Arc::from("")));
            }
            Atomic::TFloat | Atomic::TIntegralFloat => {
                changed = true;
                out.add_type(Atomic::TInt);
            }
            Atomic::TLiteralFloat(hi, lo) => {
                changed = true;
                let bits = ((*hi as u64) << 32) | (*lo as u32 as u64);
                out.add_type(Atomic::TLiteralInt(f64::from_bits(bits) as i64));
            }
            Atomic::TLiteralString(s) => match canonical_int_array_key(s) {
                Some(i) => {
                    changed = true;
                    out.add_type(Atomic::TLiteralInt(i));
                }
                None => out.add_type(a.clone()),
            },
            _ => out.add_type(a.clone()),
        }
    }
    if !changed {
        return ty.clone();
    }
    out.possibly_undefined = ty.possibly_undefined;
    out.from_docblock = ty.from_docblock;
    out
}

/// Update a nested shape write (`$arr['a']['b'] = $v`) by walking into the
/// matching per-key property at each level instead of widening the whole
/// outer shape into a generic array. `path` is ordered innermost-first (the
/// key directly on `current`, then progressively outer keys) and `leaf_value`
/// is the value being assigned at the final (outermost) key.
///
/// Returns `None` when the shape at any level doesn't cleanly resolve (an
/// unknown key, a non-uniform union, a non-shape atom, …) so the caller can
/// fall back to the existing generic accumulator.
pub fn set_nested_keyed_value(
    current: &Type,
    path: &[ArrayKey],
    leaf_value: &Type,
) -> Option<Type> {
    let (key, rest) = path.split_first()?;
    if current.types.is_empty() {
        return None;
    }
    let all_shapes_have_key = current.types.iter().all(
        |a| matches!(a, Atomic::TKeyedArray { properties, .. } if properties.contains_key(key)),
    );
    if !all_shapes_have_key {
        return None;
    }
    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    for atomic in &current.types {
        let Atomic::TKeyedArray {
            properties,
            is_open,
            is_list,
        } = atomic
        else {
            unreachable!("filtered to TKeyedArray above")
        };
        let mut new_properties = properties.clone();
        let existing = properties.get(key).expect("checked by all_shapes_have_key");
        let new_inner = if rest.is_empty() {
            leaf_value.clone()
        } else {
            set_nested_keyed_value(&existing.ty, rest, leaf_value)?
        };
        new_properties.insert(
            key.clone(),
            mir_types::atomic::KeyedProperty {
                ty: new_inner,
                optional: false,
            },
        );
        result.add_type(Atomic::TKeyedArray {
            properties: new_properties,
            is_open: *is_open,
            is_list: *is_list,
        });
    }
    Some(result)
}

/// Remove `key` from every `TKeyedArray` atomic in `ty`'s union that has it,
/// leaving all other atoms and properties unchanged. Used for
/// `unset($arr['key'])`, which genuinely removes the key from the array
/// (regardless of whether the shape is open or closed) rather than merely
/// marking it optional.
pub fn remove_key_from_shapes(ty: &Type, key: &ArrayKey) -> Type {
    let mut changed = false;
    let mut result = Type::empty();
    for a in &ty.types {
        if let Atomic::TKeyedArray {
            properties,
            is_open,
            is_list,
        } = a
        {
            if properties.contains_key(key) {
                changed = true;
                let mut new_props = properties.clone();
                new_props.shift_remove(key);
                result.add_type(Atomic::TKeyedArray {
                    properties: new_props,
                    is_open: *is_open,
                    is_list: *is_list,
                });
                continue;
            }
        }
        result.add_type(a.clone());
    }
    if !changed {
        return ty.clone();
    }
    result.from_docblock = ty.from_docblock;
    result
}

/// Whether a literal array key is definitely present (with a known,
/// non-optional, non-null type) or definitely absent across an array type's
/// whole union — used by `??=` to tell whether its right-hand side can ever
/// actually run.
pub enum DefiniteKeyState {
    Absent,
    Present(Type),
}

/// Resolve [`DefiniteKeyState`] for `key` on `current`. Returns `None` when
/// neither state can be proven for the WHOLE union — a non-shape atom, an
/// open shape (an undeclared key might still match), an optional property
/// (may or may not be set), or a union where one branch has the key and
/// another doesn't — so the caller should fall back to treating it as
/// "maybe set".
pub fn definite_key_state(current: &Type, key: &ArrayKey) -> Option<DefiniteKeyState> {
    if current.types.is_empty() {
        return None;
    }
    let mut any_absent = false;
    let mut any_present = false;
    let mut present_ty: Option<Type> = None;
    for a in &current.types {
        let Atomic::TKeyedArray {
            properties,
            is_open,
            ..
        } = a
        else {
            return None;
        };
        match properties.get(key) {
            // Non-optional AND provably non-null: `??=` can never run, so the
            // key's own type is the final answer. A non-optional property
            // whose type still admits `null` is genuinely uncertain — the
            // stored value could be null at runtime, in which case `??=`
            // *would* run — so that falls through to the catch-all `None`.
            Some(prop) if !prop.optional && !prop.ty.is_nullable() => {
                any_present = true;
                fold_into(&mut present_ty, prop.ty.clone());
            }
            Some(_) => return None,
            None if !*is_open => any_absent = true,
            None => return None,
        }
    }
    match (any_absent, any_present) {
        (true, false) => Some(DefiniteKeyState::Absent),
        (false, true) => present_ty.map(DefiniteKeyState::Present),
        _ => None,
    }
}

/// Cap on how many properties a `TKeyedArray` shape can accumulate from
/// straight-line literal writes before a further write generalizes the whole
/// atom to a plain `array<K, V>`/`list<T>`. Keeps property maps (and their
/// per-write clones) small, and bounds how large a printed shape can get.
const MAX_SHAPE_KEYS: usize = 8;

/// Try to extend every `TKeyedArray` atom in `current` with a brand-new
/// `key: new_value` property in place, instead of collapsing the shape to a
/// generic array. `None` means the caller should fall back to the generic
/// accumulator: some atom isn't a shape, already has `key`, or is already at
/// [`MAX_SHAPE_KEYS`].
fn try_insert_new_shape_key(current: &Type, key: &ArrayKey, new_value: &Type) -> Option<Type> {
    if current.types.is_empty() {
        return None;
    }
    let all_growable = current.types.iter().all(|a| {
        matches!(a, Atomic::TKeyedArray { properties, .. }
            if !properties.contains_key(key) && properties.len() < MAX_SHAPE_KEYS)
    });
    if !all_growable {
        return None;
    }
    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    for atomic in &current.types {
        let Atomic::TKeyedArray {
            properties,
            is_open,
            is_list,
        } = atomic
        else {
            unreachable!("filtered to growable TKeyedArray above")
        };
        // A list stays a list only if the new key continues the 0, 1, 2, …
        // sequence; any other key (a string, or an int that skips ahead)
        // makes it a plain keyed shape from here on.
        let next_is_list =
            *is_list && matches!(key, ArrayKey::Int(i) if *i == properties.len() as i64);
        let mut new_properties = properties.clone();
        new_properties.insert(
            key.clone(),
            mir_types::atomic::KeyedProperty {
                ty: new_value.clone(),
                optional: false,
            },
        );
        result.add_type(Atomic::TKeyedArray {
            properties: new_properties,
            is_open: *is_open,
            is_list: next_is_list,
        });
    }
    Some(result)
}

/// Like [`try_insert_new_shape_key`], but for push notation (`$arr[] = v`):
/// the new key is always the next sequential integer index, so this only
/// applies to atoms that are still list-shaped (an assoc shape can't be
/// pushed onto without knowing what key PHP would assign it).
fn try_push_new_shape_key(current: &Type, new_value: &Type) -> Option<Type> {
    if current.types.is_empty() {
        return None;
    }
    let all_growable = current.types.iter().all(|a| {
        matches!(a, Atomic::TKeyedArray { properties, is_list, .. }
            if *is_list && properties.len() < MAX_SHAPE_KEYS)
    });
    if !all_growable {
        return None;
    }
    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    for atomic in &current.types {
        let Atomic::TKeyedArray {
            properties,
            is_open,
            ..
        } = atomic
        else {
            unreachable!("filtered to growable TKeyedArray above")
        };
        let mut new_properties = properties.clone();
        new_properties.insert(
            ArrayKey::Int(properties.len() as i64),
            mir_types::atomic::KeyedProperty {
                ty: new_value.clone(),
                optional: false,
            },
        );
        result.add_type(Atomic::TKeyedArray {
            properties: new_properties,
            is_open: *is_open,
            is_list: true,
        });
    }
    Some(result)
}

/// Pull the key/value type out of `declared`'s array-like atom, if any. Used
/// so a generalization fallback never ends up narrower than the variable's
/// own declared type — otherwise a handful of same-typed literal writes seen
/// so far could generalize to e.g. `array<string, int>` even though the
/// declared type promises `array<string, int|string>`.
fn declared_array_key_value(declared: Option<&Type>) -> Option<(Type, Type)> {
    declared?.types.iter().find_map(|a| match a {
        Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
            Some(((**key).clone(), (**value).clone()))
        }
        Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
            Some((Type::single(Atomic::TInt), (**value).clone()))
        }
        _ => None,
    })
}

pub fn widen_array_with_value_and_key(
    current: &Type,
    new_value: &Type,
    new_key: &Type,
    literal_key: Option<&mir_types::ArrayKey>,
    inside_loop: bool,
    declared_ceiling: Option<&Type>,
) -> Type {
    // Overwriting an EXISTING literal key on a shape (`$arr['a'] = 2;` where
    // 'a' is already a known property) updates just that one property,
    // leaving every other key's type untouched — routing this through the
    // generic accumulator below would collapse the whole shape into a wide
    // `array<K, V>` union even though no other key was affected by the write.
    if let Some(key) = literal_key {
        let all_shapes_have_key = !current.types.is_empty()
            && current.types.iter().all(|a| match a {
                Atomic::TKeyedArray { properties, .. } => properties.contains_key(key),
                _ => false,
            });
        if all_shapes_have_key {
            let mut result = Type::empty();
            result.possibly_undefined = current.possibly_undefined;
            result.from_docblock = current.from_docblock;
            for atomic in &current.types {
                let Atomic::TKeyedArray {
                    properties,
                    is_open,
                    is_list,
                } = atomic
                else {
                    unreachable!("filtered to TKeyedArray above")
                };
                let mut new_properties = properties.clone();
                // The key is now definitely assigned on this path, regardless
                // of whether it was previously optional.
                new_properties.insert(
                    key.clone(),
                    mir_types::atomic::KeyedProperty {
                        ty: new_value.clone(),
                        optional: false,
                    },
                );
                result.add_type(Atomic::TKeyedArray {
                    properties: new_properties,
                    is_open: *is_open,
                    is_list: *is_list,
                });
            }
            return result;
        }

        // A brand-new key on a shape: grow it in place rather than
        // generalizing, as long as we're not inside a loop (where the shape
        // would otherwise grow a fresh property every fixed-point pass and
        // never converge).
        if !inside_loop {
            if let Some(grown) = try_insert_new_shape_key(current, key, new_value) {
                return grown;
            }
        }
    }

    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    let mut found_array = false;
    // Merge ALL array-like variants from current into a single accumulated TArray/TList.
    // Without this, each TArray variant in a growing union independently emits a new TArray,
    // causing unbounded union growth across salsa fixpoint iterations (infinite recursion).
    let mut acc_key: Option<Type> = None;
    let mut acc_value: Option<Type> = None;
    let mut acc_list: Option<Type> = None;
    for atomic in &current.types {
        match atomic {
            Atomic::TKeyedArray { properties, .. } => {
                let mut all_values = new_value.clone();
                let mut all_keys = new_key.clone();
                for prop in properties.values() {
                    all_values.merge_with(&prop.ty);
                }
                for k in properties.keys() {
                    let key_atomic = match k {
                        mir_types::ArrayKey::String(s) => Atomic::TLiteralString(s.clone()),
                        mir_types::ArrayKey::Int(i) => Atomic::TLiteralInt(*i),
                    };
                    all_keys.merge_with(&Type::single(key_atomic));
                }
                fold_into(&mut acc_key, all_keys);
                fold_into(&mut acc_value, all_values);
                found_array = true;
            }
            Atomic::TArray { key, value } => {
                fold_into(&mut acc_key, Type::merge(key, new_key));
                fold_into(&mut acc_value, Type::merge(value, new_value));
                found_array = true;
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                fold_into(&mut acc_list, Type::merge(value, new_value));
                found_array = true;
            }
            Atomic::TNonEmptyArray { key, value } => {
                fold_into(&mut acc_key, Type::merge(key, new_key));
                fold_into(&mut acc_value, Type::merge(value, new_value));
                found_array = true;
            }
            Atomic::TMixed => {
                return Type::mixed();
            }
            other => {
                result.add_type(other.clone());
            }
        }
    }
    if let (Some(mut key), Some(mut value)) = (acc_key, acc_value) {
        if let Some((declared_key, declared_value)) = declared_array_key_value(declared_ceiling) {
            key.merge_with(&declared_key);
            value.merge_with(&declared_value);
        }
        result.add_type(Atomic::TArray {
            key: Box::new(key),
            value: Box::new(value),
        });
    }
    if let Some(v) = acc_list {
        result.add_type(Atomic::TList { value: Box::new(v) });
    }
    if !found_array {
        return current.clone();
    }
    result
}

/// Widen an existing array-like type by appending `new_value` via push notation (`[]`).
/// Produces `TList { merged_value }` in the general case, regardless of the
/// current key type, because push notation in PHP assigns the next integer
/// index — except when the current type is a still-growable list shape and
/// we're not inside a loop, in which case the shape simply gains one more
/// property (see [`try_push_new_shape_key`]).
pub fn widen_array_as_list(
    current: &Type,
    new_value: &Type,
    inside_loop: bool,
    declared_ceiling: Option<&Type>,
) -> Type {
    if !inside_loop {
        if let Some(grown) = try_push_new_shape_key(current, new_value) {
            return grown;
        }
    }

    let mut result = Type::empty();
    result.possibly_undefined = current.possibly_undefined;
    result.from_docblock = current.from_docblock;
    let mut acc: Option<Type> = Some(new_value.clone());
    let mut found_array = false;
    for atomic in &current.types {
        match atomic {
            Atomic::TKeyedArray { properties, .. } => {
                for prop in properties.values() {
                    fold_into(&mut acc, prop.ty.clone());
                }
                found_array = true;
            }
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => {
                fold_into(&mut acc, *value.clone());
                found_array = true;
            }
            Atomic::TMixed => return Type::mixed(),
            other => result.add_type(other.clone()),
        }
    }
    if !found_array {
        return current.clone();
    }
    if let Some(mut v) = acc {
        if let Some((_, declared_value)) = declared_array_key_value(declared_ceiling) {
            v.merge_with(&declared_value);
        }
        result.add_type(Atomic::TList { value: Box::new(v) });
    }
    result
}

fn fold_into(acc: &mut Option<Type>, new: Type) {
    match acc {
        None => *acc = Some(new),
        Some(existing) => existing.merge_with(&new),
    }
}

/// The inclusive integer bounds of `ty` when it is an integer-only type, as
/// `(min, max)` where `None` means unbounded on that side. Returns `None` when
/// any member is not an integer (so the caller falls back to scalar inference).
/// Literals are exact bounds; a general `int` is unbounded both ways.
fn int_bounds(ty: &Type) -> Option<(Option<i64>, Option<i64>)> {
    if ty.types.is_empty() {
        return None;
    }
    let mut min: Option<i64> = Some(i64::MAX);
    let mut max: Option<i64> = Some(i64::MIN);
    for a in &ty.types {
        let (lo, hi) = match a {
            Atomic::TLiteralInt(n) => (Some(*n), Some(*n)),
            Atomic::TIntRange { min, max } => (*min, *max),
            // Named int subtypes carry implicit bounds: use them so arithmetic
            // like `positive-int + 1` yields `int<2, max>` rather than bare `int`.
            Atomic::TPositiveInt => (Some(1), None),
            Atomic::TNonNegativeInt => (Some(0), None),
            Atomic::TNegativeInt => (None, Some(-1)),
            Atomic::TInt => (None, None),
            _ => return None,
        };
        // Widen the accumulated bounds to cover this member (union semantics).
        min = match (min, lo) {
            (Some(m), Some(l)) => Some(m.min(l)),
            _ => None,
        };
        max = match (max, hi) {
            (Some(m), Some(h)) => Some(m.max(h)),
            _ => None,
        };
    }
    Some((min, max))
}

/// Whether `ty` carries an explicit integer range or a named int subtype with
/// known implicit bounds (positive-int, non-negative-int, negative-int).
fn contains_int_range(ty: &Type) -> bool {
    ty.types.iter().any(|a| {
        matches!(
            a,
            Atomic::TIntRange { .. }
                | Atomic::TPositiveInt
                | Atomic::TNonNegativeInt
                | Atomic::TNegativeInt
        )
    })
}

/// Range-aware integer arithmetic for `+` and `-`: when at least one operand is
/// an integer range (e.g. a `count()` result), propagate faithful bounds so
/// `count($a) + 1` is `int<1, max>` and `count($a) - 1` is `int<-1, max>`.
/// Returns `None` for anything else (including literal-only arithmetic, left to
/// [`infer_arithmetic`] so it is not perturbed).
fn as_single_literal_int(ty: &Type) -> Option<i64> {
    if ty.types.len() == 1 {
        if let Atomic::TLiteralInt(n) = &ty.types[0] {
            return Some(*n);
        }
    }
    None
}

pub fn infer_int_range_arithmetic(left: &Type, right: &Type, op: BinaryOp) -> Option<Type> {
    // Fast path: both operands are known literal ints — fold at analysis time.
    if let (Some(l), Some(r)) = (as_single_literal_int(left), as_single_literal_int(right)) {
        let result = match op {
            BinaryOp::Add => l.checked_add(r),
            BinaryOp::Sub => l.checked_sub(r),
            BinaryOp::Mul => l.checked_mul(r),
            // Integer division only when divisor is nonzero and result is exact.
            BinaryOp::Div if r != 0 && l % r == 0 => Some(l / r),
            BinaryOp::Mod if r != 0 => Some(l % r),
            _ => None,
        };
        if let Some(n) = result {
            return Some(Type::single(Atomic::TLiteralInt(n)));
        }
        // Non-exact literal integer division (e.g. 5 / 2 = 2.5) → float.
        if op == BinaryOp::Div && r != 0 {
            return Some(Type::single(Atomic::TFloat));
        }
    }

    // Only engage when a genuine range is in play; plain int/literal operands
    // keep the existing scalar inference.
    if !contains_int_range(left) && !contains_int_range(right) {
        return None;
    }
    let (lmin, lmax) = int_bounds(left)?;
    let (rmin, rmax) = int_bounds(right)?;
    let add = |a: Option<i64>, b: Option<i64>| match (a, b) {
        (Some(a), Some(b)) => a.checked_add(b),
        _ => None,
    };
    let sub = |a: Option<i64>, b: Option<i64>| match (a, b) {
        (Some(a), Some(b)) => a.checked_sub(b),
        _ => None,
    };
    let mul_opt = |a: Option<i64>, b: Option<i64>| match (a, b) {
        (Some(a), Some(b)) => a.checked_mul(b),
        _ => None,
    };
    let (min, max) = match op {
        BinaryOp::Add => (add(lmin, rmin), add(lmax, rmax)),
        // [lmin,lmax] - [rmin,rmax] = [lmin - rmax, lmax - rmin]
        BinaryOp::Sub => (sub(lmin, rmax), sub(lmax, rmin)),
        // Multiplication: only handle the case where both operands are non-negative,
        // which is the common case (`count * stride`, `width * height`, etc.).
        // lmin/rmin must be Some(>=0) — None means unbounded below, i.e., can be negative.
        // For mixed-sign operands the four-corner product is complex; defer to infer_arithmetic.
        BinaryOp::Mul if lmin.is_some_and(|m| m >= 0) && rmin.is_some_and(|m| m >= 0) => {
            (mul_opt(lmin, rmin), mul_opt(lmax, rmax))
        }
        // Modulo: result range depends only on the divisor.
        // For a known positive divisor K: result ∈ [0, K-1] when dividend ≥ 0,
        // or [-(K-1), K-1] when dividend may be negative (PHP truncates toward zero).
        BinaryOp::Mod if rmin == rmax && rmin.is_some_and(|r| r > 0) => {
            let divisor = rmin.unwrap();
            if lmin.is_some_and(|m| m >= 0) {
                (Some(0), Some(divisor - 1))
            } else {
                (Some(-(divisor - 1)), Some(divisor - 1))
            }
        }
        _ => return None,
    };
    Some(Type::single(Atomic::TIntRange { min, max }))
}

/// Bool and null coerce to int (0/1 and 0 respectively) in PHP arithmetic;
/// they never produce float. This predicate is used in `infer_arithmetic` to
/// extend the "returns int" condition beyond pure-int operands.
fn coerces_to_int_in_arithmetic(t: &Atomic) -> bool {
    t.is_int()
        || matches!(
            t,
            Atomic::TBool | Atomic::TTrue | Atomic::TFalse | Atomic::TNull
        )
}

pub fn infer_arithmetic(left: &Type, right: &Type) -> Type {
    if left.is_mixed() || right.is_mixed() {
        return Type::mixed();
    }

    let left_is_array = left.contains(|t| {
        matches!(
            t,
            Atomic::TArray { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
        )
    });
    let right_is_array = right.contains(|t| {
        matches!(
            t,
            Atomic::TArray { .. }
                | Atomic::TNonEmptyArray { .. }
                | Atomic::TList { .. }
                | Atomic::TNonEmptyList { .. }
                | Atomic::TKeyedArray { .. }
        )
    });
    if left_is_array || right_is_array {
        let merged_left = if left_is_array {
            left.clone()
        } else {
            Type::single(Atomic::TArray {
                key: Box::new(Type::single(Atomic::TMixed)),
                value: Box::new(Type::mixed()),
            })
        };
        return merged_left;
    }

    let left_is_float = left.contains(|t| {
        matches!(
            t,
            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
        )
    });
    let right_is_float = right.contains(|t| {
        matches!(
            t,
            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
        )
    });
    if left_is_float || right_is_float {
        Type::single(Atomic::TFloat)
    } else if left.contains(coerces_to_int_in_arithmetic)
        && right.contains(coerces_to_int_in_arithmetic)
    {
        Type::single(Atomic::TInt)
    } else {
        let mut u = Type::empty();
        u.add_type(Atomic::TInt);
        u.add_type(Atomic::TFloat);
        u
    }
}

/// Type of the `/` operator. Unlike `+`/`-`/`*`, `int / int` yields `int|float` in PHP
/// because division may produce a fractional result (e.g. `5 / 2 = 2.5`).
pub fn infer_div(left: &Type, right: &Type) -> Type {
    if left.is_mixed() || right.is_mixed() {
        return Type::mixed();
    }
    let left_is_float = left.contains(|t| {
        matches!(
            t,
            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
        )
    });
    let right_is_float = right.contains(|t| {
        matches!(
            t,
            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..)
        )
    });
    if left_is_float || right_is_float {
        return Type::single(Atomic::TFloat);
    }
    let mut u = Type::empty();
    u.add_type(Atomic::TInt);
    u.add_type(Atomic::TFloat);
    u
}

/// Returns true when all atoms of `ty` produce a non-empty string in PHP's string cast.
///
/// Used by the concat (`.`) operator and `.=` assignment to determine whether the
/// result of a concatenation is guaranteed non-empty.
pub fn is_non_empty_when_concat(ty: &Type) -> bool {
    !ty.types.is_empty()
        && ty.types.iter().all(|a| match a {
            Atomic::TNonEmptyString
            | Atomic::TNumericString
            | Atomic::TCallableString
            | Atomic::TClassString(_)
            | Atomic::TInterfaceString(_)
            | Atomic::TEnumString
            | Atomic::TTraitString => true,
            Atomic::TLiteralString(s) => !s.is_empty(),
            // Any integer — including 0 — casts to a non-empty string ("0", "1", "-1", …)
            Atomic::TLiteralInt(_)
            | Atomic::TInt
            | Atomic::TPositiveInt
            | Atomic::TNegativeInt
            | Atomic::TNonNegativeInt
            | Atomic::TIntRange { .. } => true,
            // Any float casts to a non-empty string ("0", "1.5", …)
            Atomic::TFloat | Atomic::TIntegralFloat | Atomic::TLiteralFloat(..) => true,
            // true → "1"; false → "" so TBool and TFalse are excluded
            Atomic::TTrue => true,
            _ => false,
        })
}

/// Extract the string representation of a single scalar literal for concat folding.
/// Returns `None` for unions or non-literal types.
pub fn as_concat_str(ty: &Type) -> Option<String> {
    if ty.types.len() != 1 {
        return None;
    }
    match &ty.types[0] {
        Atomic::TLiteralString(s) => Some(s.as_ref().to_string()),
        Atomic::TLiteralInt(n) => Some(n.to_string()),
        Atomic::TTrue => Some("1".to_string()),
        Atomic::TFalse => Some(String::new()),
        _ => None,
    }
}

pub fn extract_simple_var(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
        ExprKind::Parenthesized(inner) => extract_simple_var(inner),
        _ => None,
    }
}

pub(crate) fn ast_params_to_fn_params_resolved(
    params: &[php_ast::owned::Param],
    self_fqcn: Option<&str>,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Vec<mir_codebase::DeclaredParam> {
    params
        .iter()
        .map(|p| {
            let name_str = p.name.as_deref().unwrap_or("").trim_start_matches('$');
            let ty = p
                .type_hint
                .as_ref()
                .map(|h| crate::parser::type_from_hint_owned(h, self_fqcn))
                .map(|u| resolve_named_objects_in_union(u, db, file));
            mir_codebase::DeclaredParam {
                name: Name::new(name_str),
                ty: mir_codebase::wrap_param_type(ty),
                out_ty: None,
                has_default: p.default.is_some(),
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: p.default.is_some() || p.variadic,
            }
        })
        .collect()
}

/// Merge `@param` docblock types into already-resolved closure/arrow-function params,
/// matching by parameter name.
///
/// Docblock types win over native hints — the same precedence used for top-level
/// function/method declarations — except when the native hint is a concrete scalar
/// whose family is entirely absent from the docblock type (e.g. `@param int $x` on a
/// `bool $x` hint), in which case the native hint is the runtime truth and wins.
pub(crate) fn apply_doc_param_types(
    params: &mut [mir_codebase::DeclaredParam],
    ast_params: &[php_ast::owned::Param],
    doc_params: &[(String, Type)],
    db: &dyn crate::db::MirDatabase,
    file: &str,
) {
    if doc_params.is_empty() {
        return;
    }
    for (param, ast_param) in params.iter_mut().zip(ast_params.iter()) {
        let name = ast_param
            .name
            .as_deref()
            .unwrap_or("")
            .trim_start_matches('$');
        let Some((_, doc_ty)) = doc_params.iter().find(|(n, _)| n == name) else {
            continue;
        };
        let mut doc_ty = resolve_named_objects_in_union(doc_ty.clone(), db, file);
        if let Some(native_ty) = param.ty.as_deref() {
            if crate::collector::native_hint_wins_over_docblock_scalar(native_ty, &doc_ty) {
                continue;
            }
            // Partial conflict: strip atoms foreign to the native hint's
            // scalar family instead of storing the raw docblock union.
            doc_ty = crate::collector::resolve_docblock_scalar_conflict(native_ty, doc_ty);
        }
        doc_ty.from_docblock = true;
        param.ty = mir_codebase::wrap_param_type(Some(doc_ty));
    }
}

pub(crate) fn resolve_named_objects_in_union(
    union: Type,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Type {
    let from_docblock = union.from_docblock;
    let possibly_undefined = union.possibly_undefined;
    let types: Vec<Atomic> = union
        .types
        .into_iter()
        .map(|a| resolve_named_objects_in_atomic(a, db, file))
        .collect();
    let mut result = Type::from_vec(types);
    result.from_docblock = from_docblock;
    result.possibly_undefined = possibly_undefined;
    result
}

/// Recurse into a type-argument list, array/list element+key type, or
/// intersection member — not just a top-level `TNamedObject` — so a
/// `use`-imported short name nested inside `Wrap<list<ShortName>>`,
/// `array<int, ShortName>`, or `ShortName&Other` is resolved the same as one
/// written at the top level.
fn resolve_named_objects_in_atomic(
    atomic: Atomic,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Atomic {
    match atomic {
        Atomic::TNamedObject { fqcn, type_params } => {
            let resolved = crate::db::resolve_name(db, file, fqcn.as_ref());
            let type_params = type_params
                .iter()
                .cloned()
                .map(|tp| resolve_named_objects_in_union(tp, db, file))
                .collect();
            Atomic::TNamedObject {
                fqcn: resolved.into(),
                type_params,
            }
        }
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(resolve_named_objects_in_union(*key, db, file)),
            value: Box::new(resolve_named_objects_in_union(*value, db, file)),
        },
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(resolve_named_objects_in_union(*value, db, file)),
        },
        Atomic::TIntersection { parts } => Atomic::TIntersection {
            parts: parts
                .iter()
                .cloned()
                .map(|p| resolve_named_objects_in_union(p, db, file))
                .collect(),
        },
        other => other,
    }
}

pub(crate) fn extract_string_from_expr(expr: &Expr) -> Option<String> {
    match &expr.kind {
        ExprKind::Identifier(s) => Some(s.trim_start_matches('$').to_string()),
        ExprKind::Variable(_) => None,
        ExprKind::String(s) => Some(s.to_string()),
        _ => None,
    }
}

/// For a literal `switch` case / `match` arm condition, return a
/// `(dedup_key, display)` pair. The key is type-tagged so that distinct
/// literal kinds never collide (e.g. the int `0` and the string `"0"`),
/// keeping duplicate detection free of PHP's loose-comparison surprises.
///
/// Returns `None` for any non-literal (variables, calls, negation, floats,
/// …) so dynamic conditions are never flagged — duplicate detection stays at
/// zero false positives.
fn literal_condition_key(expr: &Expr) -> Option<(String, String)> {
    match &expr.kind {
        ExprKind::Int(n) => Some((format!("int:{n}"), n.to_string())),
        ExprKind::String(s) => Some((format!("str:{s}"), format!("\"{s}\""))),
        ExprKind::Bool(b) => Some((format!("bool:{b}"), b.to_string())),
        ExprKind::Null => Some(("null".to_string(), "null".to_string())),
        _ => None,
    }
}

/// Given `switch`/`match` condition expressions in source order, return the
/// `(span, display)` of each literal whose value repeats an earlier one — the
/// duplicate branch can never be reached. Non-literal conditions are ignored,
/// so dynamic arms are never flagged.
pub fn duplicate_literal_conditions<'e>(
    conditions: impl Iterator<Item = &'e Expr>,
) -> Vec<(php_ast::Span, String)> {
    let mut seen = FxHashSet::default();
    let mut duplicates = Vec::new();
    for cond in conditions {
        if let Some((key, display)) = literal_condition_key(cond) {
            if !seen.insert(key) {
                duplicates.push((cond.span, display));
            }
        }
    }
    duplicates
}

/// Returns true if `ty` contains any reference to a template param name from `names`,
/// including names nested inside generic type arguments (e.g. `R` inside `Result<Throwable, R>`).
/// Handles both `TTemplateParam` and the docblock-parser workaround where bare unqualified names
/// are emitted as `TNamedObject { fqcn: "T", type_params: [] }`.
pub(crate) fn type_refs_any_template(ty: &Type, names: &FxHashSet<Name>) -> bool {
    fn check_atomic(a: &Atomic, names: &FxHashSet<Name>) -> bool {
        match a {
            Atomic::TTemplateParam { name, .. } => names.contains(name),
            Atomic::TNamedObject { fqcn, type_params } => {
                if type_params.is_empty() && !fqcn.contains('\\') && names.contains(fqcn) {
                    return true;
                }
                type_params
                    .iter()
                    .any(|tp| tp.types.iter().any(|a| check_atomic(a, names)))
            }
            Atomic::TClassString(Some(inner)) => !inner.contains('\\') && names.contains(inner),
            _ => false,
        }
    }
    ty.types.iter().any(|a| check_atomic(a, names))
}

fn scalar_types_compatible(value_ty: &Type, prop_ty: &Type) -> bool {
    value_ty.is_subtype_structural(prop_ty)
}

pub(crate) fn property_assign_compatible(
    value_ty: &Type,
    prop_ty: &Type,
    db: &dyn crate::db::MirDatabase,
) -> bool {
    if scalar_types_compatible(value_ty, prop_ty) {
        return true;
    }
    if is_subtype(db, value_ty, prop_ty) {
        return true;
    }
    value_ty.types.iter().all(|a| match a {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TClosure { .. } | Atomic::TCallable { .. } => prop_ty.types.iter().any(|p| {
            matches!(p, Atomic::TClosure { .. } | Atomic::TCallable { .. })
                || matches!(p, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Closure")
        }),
        Atomic::TNever => true,
        Atomic::TNull => prop_ty.is_nullable(),
        _ => false,
    })
}

pub(crate) fn is_property_type_coercion(
    value_ty: &Type,
    prop_ty: &Type,
    db: &dyn crate::db::MirDatabase,
) -> bool {
    if value_ty.is_mixed() || prop_ty.is_mixed() {
        return false;
    }
    let value_core = value_ty.core_type();
    if value_core.types.is_empty() || !value_core.is_single() {
        return false;
    }
    let val_fqcn = match value_core.types.first().unwrap() {
        Atomic::TNamedObject { fqcn, type_params } if type_params.is_empty() => *fqcn,
        _ => return false,
    };
    prop_ty.types.iter().any(|p| {
        let prop_fqcn = match p {
            Atomic::TNamedObject { fqcn, type_params } if type_params.is_empty() => fqcn,
            _ => return false,
        };
        crate::db::extends_or_implements(db, prop_fqcn.as_ref(), val_fqcn.as_ref())
    })
}

#[cfg(test)]
mod range_arithmetic_tests {
    use super::*;

    fn range(min: Option<i64>, max: Option<i64>) -> Type {
        Type::single(Atomic::TIntRange { min, max })
    }

    fn lit(n: i64) -> Type {
        Type::single(Atomic::TLiteralInt(n))
    }

    #[test]
    fn add_shifts_both_bounds() {
        // int<0, 4> + 5  =>  int<5, 9>
        let r =
            infer_int_range_arithmetic(&range(Some(0), Some(4)), &lit(5), BinaryOp::Add).unwrap();
        assert_eq!(r.to_string(), "int<5, 9>");
    }

    #[test]
    fn add_keeps_unbounded_upper() {
        // int<0, max> + 5  =>  int<5, max>
        let r = infer_int_range_arithmetic(&range(Some(0), None), &lit(5), BinaryOp::Add).unwrap();
        assert_eq!(r.to_string(), "int<5, max>");
    }

    #[test]
    fn sub_lowers_min_to_negative() {
        // int<0, max> - 1  =>  int<-1, max>   (lmin - rmax, lmax - rmin)
        let r = infer_int_range_arithmetic(&range(Some(0), None), &lit(1), BinaryOp::Sub).unwrap();
        assert_eq!(r.to_string(), "int<-1, max>");
    }

    #[test]
    fn add_overflow_saturates_to_unbounded() {
        // int<i64::MAX, i64::MAX> + 1  =>  both bounds overflow to unbounded,
        // which renders as the bare `int`.
        let r = infer_int_range_arithmetic(
            &range(Some(i64::MAX), Some(i64::MAX)),
            &lit(1),
            BinaryOp::Add,
        )
        .unwrap();
        assert_eq!(r.to_string(), "int");
    }

    #[test]
    fn no_range_operand_returns_none() {
        // plain int + literal: no explicit range, so range arithmetic abstains
        assert!(
            infer_int_range_arithmetic(&Type::single(Atomic::TInt), &lit(3), BinaryOp::Add)
                .is_none()
        );
    }

    #[test]
    fn non_integer_operand_returns_none() {
        // range + string: not integer-only, abstain
        assert!(infer_int_range_arithmetic(
            &range(Some(0), None),
            &Type::single(Atomic::TString),
            BinaryOp::Add
        )
        .is_none());
    }

    #[test]
    fn mul_non_negative_ranges() {
        // non-negative × literal positive → int<0, max> (unbounded above)
        let r = infer_int_range_arithmetic(&range(Some(0), None), &lit(2), BinaryOp::Mul).unwrap();
        assert_eq!(r, range(Some(0), None));

        // bounded × bounded → bounded product
        let r = infer_int_range_arithmetic(
            &range(Some(2), Some(4)),
            &range(Some(3), Some(6)),
            BinaryOp::Mul,
        )
        .unwrap();
        assert_eq!(r, range(Some(6), Some(24)));

        // mixed-sign operand: defer to infer_arithmetic
        assert!(
            infer_int_range_arithmetic(&range(None, Some(-1)), &lit(2), BinaryOp::Mul).is_none()
        );
    }

    #[test]
    fn mod_non_negative_ranges() {
        // non-negative-int % 5 → int<0, 4>
        let r = infer_int_range_arithmetic(&range(Some(0), None), &lit(5), BinaryOp::Mod).unwrap();
        assert_eq!(r, range(Some(0), Some(4)));

        // int<0, 100> % 10 → int<0, 9>
        let r = infer_int_range_arithmetic(&range(Some(0), Some(100)), &lit(10), BinaryOp::Mod)
            .unwrap();
        assert_eq!(r, range(Some(0), Some(9)));

        // negative divisor: no range inference
        assert!(
            infer_int_range_arithmetic(&range(Some(0), None), &lit(-5), BinaryOp::Mod).is_none()
        );
    }
}
