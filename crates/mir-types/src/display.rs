use std::fmt;

use crate::atomic::Atomic;
use crate::union::Type;

/// Write `items` separated by `sep` straight into the formatter, avoiding the
/// intermediate `Vec<String>` + `join` allocations.
fn write_joined<T: fmt::Display>(
    f: &mut fmt::Formatter<'_>,
    items: impl IntoIterator<Item = T>,
    sep: &str,
) -> fmt::Result {
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            f.write_str(sep)?;
        }
        write!(f, "{item}")?;
    }
    Ok(())
}

/// True when `t` is precisely `mixed` and nothing else — deliberately
/// stricter than [`Type::is_mixed`], which also treats an unconstrained
/// template parameter as mixed. A template placeholder is a meaningful part
/// of a generic signature (e.g. `array<TKey, TValue>`) and must still be
/// printed, whereas a literal, unconstrained `mixed` is a default that adds
/// no information and can be collapsed away.
fn is_exactly_mixed(t: &Type) -> bool {
    matches!(t.types.as_slice(), [Atomic::TMixed])
}

/// Write a comma-separated callable/closure parameter type list, printing
/// `mixed` for untyped params.
fn write_param_types(f: &mut fmt::Formatter<'_>, params: &[crate::atomic::FnParam]) -> fmt::Result {
    for (i, p) in params.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
        }
        match &p.ty {
            Some(ty) => write!(f, "{ty}")?,
            None => f.write_str("mixed")?,
        }
    }
    Ok(())
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.types.is_empty() {
            return write!(f, "never");
        }
        write_joined(f, self.types.iter(), "|")
    }
}

impl fmt::Display for Atomic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atomic::TString => write!(f, "string"),
            Atomic::TLiteralString(s) => write!(f, "\"{s}\""),
            Atomic::TCallableString => write!(f, "callable-string"),
            Atomic::TClassString(None) => write!(f, "class-string"),
            Atomic::TClassString(Some(cls)) => write!(f, "class-string<{cls}>"),
            Atomic::TNonEmptyString => write!(f, "non-empty-string"),
            Atomic::TNumericString => write!(f, "numeric-string"),

            Atomic::TInt => write!(f, "int"),
            Atomic::TLiteralInt(n) => write!(f, "{n}"),
            Atomic::TIntRange { min, max } => match (min, max) {
                (None, None) => write!(f, "int"),
                (lo, hi) => {
                    let lo = lo.map_or_else(|| "min".to_string(), |n| n.to_string());
                    let hi = hi.map_or_else(|| "max".to_string(), |n| n.to_string());
                    write!(f, "int<{lo}, {hi}>")
                }
            },
            Atomic::TPositiveInt => write!(f, "positive-int"),
            Atomic::TNegativeInt => write!(f, "negative-int"),
            Atomic::TNonNegativeInt => write!(f, "non-negative-int"),

            Atomic::TFloat | Atomic::TIntegralFloat => write!(f, "float"),
            Atomic::TLiteralFloat(high, low) => {
                let bits = ((*high as u64) << 32) | (*low as u32 as u64);
                let value = f64::from_bits(bits);
                write!(f, "{value}")
            }

            Atomic::TBool => write!(f, "bool"),
            Atomic::TTrue => write!(f, "true"),
            Atomic::TFalse => write!(f, "false"),

            Atomic::TNull => write!(f, "null"),
            Atomic::TVoid => write!(f, "void"),
            Atomic::TNever => write!(f, "never"),
            Atomic::TMixed => write!(f, "mixed"),
            Atomic::TScalar => write!(f, "scalar"),
            Atomic::TNumeric => write!(f, "numeric"),

            Atomic::TObject => write!(f, "object"),
            Atomic::TNamedObject { fqcn, type_params } => {
                // `Traversable<mixed, mixed>`/`Generator<mixed, mixed, mixed, mixed>`
                // carry no more information than the bare class name — every
                // param resolving to the unconstrained default is exactly the
                // case an omitted type-param list would represent.
                if type_params.is_empty() || type_params.iter().all(is_exactly_mixed) {
                    write!(f, "{fqcn}")
                } else {
                    write!(f, "{fqcn}<")?;
                    write_joined(f, type_params.iter(), ", ")?;
                    f.write_str(">")
                }
            }
            Atomic::TStaticObject { fqcn } => write!(f, "static({fqcn})"),
            Atomic::TSelf { fqcn } => write!(f, "self({fqcn})"),
            Atomic::TParent { fqcn } => write!(f, "parent({fqcn})"),

            Atomic::TCallable {
                params: None,
                return_type: None,
            } => write!(f, "callable"),
            Atomic::TCallable {
                params: Some(params),
                return_type,
            } => {
                f.write_str("callable(")?;
                write_param_types(f, params)?;
                match return_type {
                    Some(r) => write!(f, "): {r}"),
                    None => f.write_str("): mixed"),
                }
            }
            Atomic::TCallable {
                params: None,
                return_type: Some(ret),
            } => {
                write!(f, "callable(): {ret}")
            }
            Atomic::TClosure { data } => {
                f.write_str("Closure(")?;
                write_param_types(f, &data.params)?;
                write!(f, "): {}", data.return_type)
            }

            Atomic::TArray { key, value } => {
                // `array<mixed, mixed>` and `array<array-key, mixed>` are both
                // just `array` — `array-key` (int|string) is already the
                // maximal legal key domain, so it's as much a "default" key
                // as `mixed` is.
                if is_exactly_mixed(value) && (is_exactly_mixed(key) || key.is_array_key()) {
                    write!(f, "array")
                } else {
                    write!(f, "array<{key}, {value}>")
                }
            }
            Atomic::TList { value } => {
                if is_exactly_mixed(value) {
                    write!(f, "list")
                } else {
                    write!(f, "list<{value}>")
                }
            }
            Atomic::TNonEmptyArray { key, value } => {
                if is_exactly_mixed(value) && (is_exactly_mixed(key) || key.is_array_key()) {
                    write!(f, "non-empty-array")
                } else {
                    write!(f, "non-empty-array<{key}, {value}>")
                }
            }
            Atomic::TNonEmptyList { value } => {
                if is_exactly_mixed(value) {
                    write!(f, "non-empty-list")
                } else {
                    write!(f, "non-empty-list<{value}>")
                }
            }
            Atomic::TKeyedArray { properties, .. } => {
                f.write_str("array{")?;
                for (i, (k, v)) in properties.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    match k {
                        crate::atomic::ArrayKey::String(s) => write!(f, "'{s}'")?,
                        crate::atomic::ArrayKey::Int(n) => write!(f, "{n}")?,
                    }
                    if v.optional {
                        f.write_str("?")?;
                    }
                    write!(f, ": {}", v.ty)?;
                }
                f.write_str("}")
            }

            Atomic::TTemplateParam { name, .. } => write!(f, "{name}"),
            Atomic::TConditional { data } => {
                let (subject, if_true, if_false) = (&data.subject, &data.if_true, &data.if_false);
                match &data.param_name {
                    Some(name) => write!(f, "(${name} is {subject} ? {if_true} : {if_false})"),
                    None => write!(f, "({subject} is ? {if_true} : {if_false})"),
                }
            }

            Atomic::TInterfaceString(None) => write!(f, "interface-string"),
            Atomic::TInterfaceString(Some(iface)) => write!(f, "interface-string<{iface}>"),
            Atomic::TEnumString => write!(f, "enum-string"),
            Atomic::TTraitString => write!(f, "trait-string"),
            Atomic::TLiteralEnumCase {
                enum_fqcn,
                case_name,
            } => {
                write!(f, "{enum_fqcn}::{case_name}")
            }

            Atomic::TIntersection { parts } => {
                let mut iter = parts.iter();
                if let Some(first) = iter.next() {
                    write!(f, "{first}")?;
                    for part in iter {
                        write!(f, "&{part}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int_range_unbounded_displays_as_int() {
        assert_eq!(
            format!(
                "{}",
                Atomic::TIntRange {
                    min: None,
                    max: None
                }
            ),
            "int"
        );
    }

    #[test]
    fn int_range_bounded_min_displays_range() {
        assert_eq!(
            format!(
                "{}",
                Atomic::TIntRange {
                    min: Some(0),
                    max: None
                }
            ),
            "int<0, max>"
        );
    }

    #[test]
    fn int_range_bounded_max_displays_range() {
        assert_eq!(
            format!(
                "{}",
                Atomic::TIntRange {
                    min: None,
                    max: Some(100)
                }
            ),
            "int<min, 100>"
        );
    }

    #[test]
    fn int_range_fully_bounded_displays_range() {
        assert_eq!(
            format!(
                "{}",
                Atomic::TIntRange {
                    min: Some(1),
                    max: Some(10)
                }
            ),
            "int<1, 10>"
        );
    }

    #[test]
    fn unbounded_int_range_in_union_displays_as_int() {
        let mut u = Type::empty();
        u.add_type(Atomic::TIntRange {
            min: None,
            max: None,
        });
        u.add_type(Atomic::TFalse);
        assert_eq!(format!("{u}"), "int|false");
    }

    #[test]
    fn array_of_mixed_mixed_collapses_to_array() {
        let atomic = Atomic::TArray {
            key: Box::new(Type::mixed()),
            value: Box::new(Type::mixed()),
        };
        assert_eq!(format!("{atomic}"), "array");
    }

    #[test]
    fn array_of_array_key_mixed_collapses_to_array() {
        let atomic = Atomic::TArray {
            key: Box::new(Type::array_key()),
            value: Box::new(Type::mixed()),
        };
        assert_eq!(format!("{atomic}"), "array");
    }

    #[test]
    fn array_of_int_mixed_does_not_collapse() {
        let atomic = Atomic::TArray {
            key: Box::new(Type::int()),
            value: Box::new(Type::mixed()),
        };
        assert_eq!(format!("{atomic}"), "array<int, mixed>");
    }

    #[test]
    fn array_of_mixed_string_does_not_collapse() {
        let atomic = Atomic::TArray {
            key: Box::new(Type::mixed()),
            value: Box::new(Type::string()),
        };
        assert_eq!(format!("{atomic}"), "array<mixed, string>");
    }

    #[test]
    fn non_empty_array_of_mixed_mixed_collapses() {
        let atomic = Atomic::TNonEmptyArray {
            key: Box::new(Type::mixed()),
            value: Box::new(Type::mixed()),
        };
        assert_eq!(format!("{atomic}"), "non-empty-array");
    }

    #[test]
    fn list_of_mixed_collapses_to_list() {
        let atomic = Atomic::TList {
            value: Box::new(Type::mixed()),
        };
        assert_eq!(format!("{atomic}"), "list");
    }

    #[test]
    fn list_of_string_does_not_collapse() {
        let atomic = Atomic::TList {
            value: Box::new(Type::string()),
        };
        assert_eq!(format!("{atomic}"), "list<string>");
    }

    #[test]
    fn non_empty_list_of_mixed_collapses() {
        let atomic = Atomic::TNonEmptyList {
            value: Box::new(Type::mixed()),
        };
        assert_eq!(format!("{atomic}"), "non-empty-list");
    }

    #[test]
    fn named_object_all_mixed_params_collapses_to_bare_name() {
        let atomic = Atomic::TNamedObject {
            fqcn: "Traversable".into(),
            type_params: crate::union::vec_to_type_params(vec![Type::mixed(), Type::mixed()]),
        };
        assert_eq!(format!("{atomic}"), "Traversable");
    }

    #[test]
    fn named_object_with_one_concrete_param_does_not_collapse() {
        let atomic = Atomic::TNamedObject {
            fqcn: "Traversable".into(),
            type_params: crate::union::vec_to_type_params(vec![Type::int(), Type::mixed()]),
        };
        assert_eq!(format!("{atomic}"), "Traversable<int, mixed>");
    }

    #[test]
    fn named_object_with_mixed_bounded_template_param_does_not_collapse() {
        // An unresolved `T` template parameter (even one bounded by `mixed`)
        // is a meaningful part of a generic signature and must never be
        // confused with a literal, information-free `mixed` default.
        let template_param = Atomic::TTemplateParam {
            name: "T".into(),
            as_type: Box::new(Type::mixed()),
            defining_entity: "MyClass".into(),
        };
        let atomic = Atomic::TNamedObject {
            fqcn: "MyClass".into(),
            type_params: crate::union::vec_to_type_params(vec![Type::single(template_param)]),
        };
        assert_eq!(format!("{atomic}"), "MyClass<T>");
    }
}
