//! Widening of types that keep growing: loop fixed points and the global size guard.

use std::sync::Arc;

use crate::atomic::{ArrayKey, Atomic};
use crate::union::{combine_generic_arrays, Type};

/// Array nesting kept before deeper levels collapse to `array<array-key, mixed>`.
pub const MAX_ARRAY_DEPTH: usize = 8;

/// Union members before generalizing; literal strings and ints (`int-mask-of`) have own limits and don't count.
pub const MAX_UNION_MEMBERS: usize = 30;

/// Array levels [`Type::generalized`] keeps below the top one.
const LOOP_WIDENING_DEPTH: usize = 1;

impl Type {
    /// Deepest array nesting in this type: `int` is 0, `list<int>` 1, `array{0: list<int>}` 2.
    pub fn array_depth(&self) -> usize {
        self.types.iter().map(atomic_array_depth).max().unwrap_or(0)
    }

    /// Literals to base types, shapes to generic arrays, values two levels down to `mixed`; idempotent.
    pub fn generalized(&self) -> Type {
        self.generalized_to(LOOP_WIDENING_DEPTH)
    }

    fn generalized_to(&self, depth: usize) -> Type {
        let mut out = Type::empty();
        out.possibly_undefined = self.possibly_undefined;
        out.from_docblock = self.from_docblock;
        out.falsy_stripped = self.falsy_stripped;
        out.possibly_absent_offset = self.possibly_absent_offset;
        // One generic array per kind keeps the form canonical across loop passes.
        let mut folded: Vec<Atomic> = Vec::with_capacity(self.types.len());
        for atomic in &self.types {
            let mut general = generalize_atomic(atomic, depth);
            if let Some((i, combined)) = folded
                .iter()
                .enumerate()
                .find_map(|(i, f)| combine_generic_arrays(f, &general).map(|c| (i, c)))
            {
                folded.swap_remove(i);
                general = combined;
            }
            folded.push(general);
        }
        for atomic in folded {
            out.add_type(atomic);
        }
        out
    }
}

fn atomic_array_depth(atomic: &Atomic) -> usize {
    let mut deepest = 0;
    let is_array = any_array_child(atomic, |ty| {
        deepest = deepest.max(ty.array_depth());
        false
    });
    is_array.map_or(0, |_| 1 + deepest)
}

/// Whether `f` holds for a value type nested in array `atomic`; `None` for a non-array.
fn any_array_child(atomic: &Atomic, mut f: impl FnMut(&Type) -> bool) -> Option<bool> {
    match atomic {
        Atomic::TArray { value, .. }
        | Atomic::TNonEmptyArray { value, .. }
        | Atomic::TList { value }
        | Atomic::TNonEmptyList { value } => Some(f(value)),
        Atomic::TKeyedArray { properties, .. } => Some(
            properties.values().any(|p| f(&p.ty)) || properties.remainder.as_ref().is_some_and(f),
        ),
        _ => None,
    }
}

/// Whether `atomic` nests arrays more than `limit` levels deep.
pub(crate) fn exceeds_array_depth(atomic: &Atomic, limit: usize) -> bool {
    any_array_child(atomic, |ty| {
        limit == 0 || ty.types.iter().any(|a| exceeds_array_depth(a, limit - 1))
    })
    .unwrap_or(false)
}

/// `atomic` with arrays nested past [`MAX_ARRAY_DEPTH`] levels generalized away.
pub fn cap_array_depth(atomic: Atomic) -> Atomic {
    if exceeds_array_depth(&atomic, MAX_ARRAY_DEPTH) {
        truncate_depth(&atomic, MAX_ARRAY_DEPTH)
    } else {
        atomic
    }
}

fn truncate_depth(atomic: &Atomic, limit: usize) -> Atomic {
    if limit <= 1 {
        return generalize_atomic(atomic, 0);
    }
    let inner = |ty: &Type| {
        let mut out = ty.clone();
        out.types = ty
            .types
            .iter()
            .map(|a| truncate_depth(a, limit - 1))
            .collect();
        out
    };
    map_array_values(atomic, inner).unwrap_or_else(|| atomic.clone())
}

/// `atomic` with `f` applied to its nested value types, keeping its kind.
fn map_array_values(atomic: &Atomic, mut f: impl FnMut(&Type) -> Type) -> Option<Atomic> {
    Some(match atomic {
        Atomic::TArray { key, value } => Atomic::TArray {
            key: key.clone(),
            value: Box::new(f(value)),
        },
        Atomic::TNonEmptyArray { key, value } => Atomic::TNonEmptyArray {
            key: key.clone(),
            value: Box::new(f(value)),
        },
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(f(value)),
        },
        Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
            value: Box::new(f(value)),
        },
        Atomic::TKeyedArray {
            properties,
            is_open,
            is_list,
        } => Atomic::TKeyedArray {
            properties: Arc::new(properties.map_types(f)),
            is_open: *is_open,
            is_list: *is_list,
        },
        _ => return None,
    })
}

/// Whether [`generalize_atomic`] would change `atomic` at the top level.
pub(crate) fn is_generalizable(atomic: &Atomic) -> bool {
    match atomic {
        Atomic::TKeyedArray { properties, .. } => !properties.is_empty(),
        Atomic::TLiteralInt(_)
        | Atomic::TIntRange { .. }
        | Atomic::TPositiveInt
        | Atomic::TNegativeInt
        | Atomic::TNonNegativeInt
        | Atomic::TLiteralString(_)
        | Atomic::TLiteralFloat(..)
        | Atomic::TTrue
        | Atomic::TFalse => true,
        _ => false,
    }
}

/// Generalizes `atomic`, keeping `depth` array levels below it before values become `mixed`.
pub(crate) fn generalize_atomic(atomic: &Atomic, depth: usize) -> Atomic {
    let value_of = |ty: &Type| -> Box<Type> {
        Box::new(if depth == 0 {
            Type::mixed()
        } else {
            ty.generalized_to(depth - 1)
        })
    };
    match atomic {
        Atomic::TLiteralInt(_)
        | Atomic::TIntRange { .. }
        | Atomic::TPositiveInt
        | Atomic::TNegativeInt
        | Atomic::TNonNegativeInt => Atomic::TInt,
        Atomic::TLiteralString(_) => Atomic::TString,
        Atomic::TLiteralFloat(..) => Atomic::TFloat,
        Atomic::TTrue | Atomic::TFalse => Atomic::TBool,
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(key.generalized_to(0)),
            value: value_of(value),
        },
        Atomic::TNonEmptyArray { key, value } => Atomic::TNonEmptyArray {
            key: Box::new(key.generalized_to(0)),
            value: value_of(value),
        },
        Atomic::TList { value } => Atomic::TList {
            value: value_of(value),
        },
        Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
            value: value_of(value),
        },
        Atomic::TKeyedArray { properties, .. } if properties.is_empty() => atomic.clone(),
        Atomic::TKeyedArray {
            properties,
            is_open,
            is_list,
        } => {
            if *is_open && properties.remainder.is_none() {
                return Atomic::TArray {
                    key: Box::new(Type::array_key()),
                    value: Box::new(Type::mixed()),
                };
            }
            let mut values = Type::empty();
            for ty in properties
                .values()
                .map(|p| &p.ty)
                .chain(properties.remainder.as_ref())
            {
                values.merge_with(ty);
            }
            let value = value_of(&values);
            let non_empty = properties.values().any(|p| !p.optional);
            if *is_list {
                return if non_empty {
                    Atomic::TNonEmptyList { value }
                } else {
                    Atomic::TList { value }
                };
            }
            let has_int = properties.keys().any(|k| matches!(k, ArrayKey::Int(_)));
            let has_string = properties.keys().any(|k| matches!(k, ArrayKey::String(_)));
            let key = Box::new(match (has_int, has_string) {
                (true, false) => Type::int(),
                (false, true) => Type::string(),
                _ => Type::array_key(),
            });
            if non_empty {
                Atomic::TNonEmptyArray { key, value }
            } else {
                Atomic::TArray { key, value }
            }
        }
        _ => atomic.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::KeyedProperty;

    fn list_of(ty: Type) -> Type {
        let mut props = crate::KeyedProperties::default();
        props.insert(
            ArrayKey::Int(0),
            KeyedProperty {
                ty,
                optional: false,
            },
        );
        Type::single(Atomic::TKeyedArray {
            properties: Arc::new(props),
            is_open: false,
            is_list: true,
        })
    }

    #[test]
    fn generalizing_a_recursive_shape_reaches_a_fixed_point() {
        let mut ty = Type::single(Atomic::TLiteralInt(0));
        ty.merge_with(&list_of(list_of(Type::single(Atomic::TLiteralInt(0)))));
        let step = |w: &Type| {
            let mut next = w.clone();
            next.merge_with(&list_of(w.clone()));
            next.generalized()
        };
        let mut widened = ty.generalized();
        for _ in 0..2 {
            widened = step(&widened);
        }
        assert_eq!(step(&widened), widened);
        assert!(widened.array_depth() <= LOOP_WIDENING_DEPTH + 1);
    }

    #[test]
    fn depth_cap_bounds_nesting() {
        let mut ty = Type::int();
        for _ in 0..20 {
            ty = list_of(ty);
        }
        let capped = cap_array_depth(ty.types[0].clone());
        assert_eq!(Type::single(capped).array_depth(), MAX_ARRAY_DEPTH);
    }

    #[test]
    fn merging_deep_shapes_caps_their_depth() {
        let mut ty = Type::int();
        for _ in 0..20 {
            let mut next = Type::empty();
            next.merge_with(&list_of(ty.clone()));
            next.merge_with(&ty);
            ty = next;
        }
        assert!(ty.array_depth() <= MAX_ARRAY_DEPTH);
    }

    #[test]
    fn wide_union_generalizes_literals() {
        let mut ty = Type::empty();
        for n in 0..100 {
            ty.merge_with(&Type::single(Atomic::TLiteralFloat(n, 0)));
        }
        assert!(ty.types.len() <= MAX_UNION_MEMBERS + 1, "{ty:?}");
        assert!(ty.types.contains(&Atomic::TFloat));
    }
}
