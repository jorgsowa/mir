use mir_types::{
    atomic::{ConditionalData, KeyedProperty},
    union::vec_to_type_params,
    Atomic, Name, Type,
};
use rustc_hash::FxHashMap;

/// Look up `alias` in `use_aliases`, falling back to a case-insensitive scan
/// if the exact-case lookup misses. PHP resolves `use` imports
/// case-insensitively; the exact-case hit above covers the common path, the
/// scan is a last resort for a differently-cased reference.
fn find_alias<'a>(alias: &str, use_aliases: &'a FxHashMap<String, String>) -> Option<&'a String> {
    use_aliases.get(alias).or_else(|| {
        use_aliases
            .iter()
            .find(|(a, _)| a.eq_ignore_ascii_case(alias))
            .map(|(_, fqcn)| fqcn)
    })
}

pub(super) fn resolve_name(
    name: &str,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> String {
    if name.starts_with('\\') {
        return name.trim_start_matches('\\').to_string();
    }
    let first_part = name.split('\\').next().unwrap_or(name);
    if let Some(resolved) = find_alias(first_part, use_aliases) {
        if name.contains('\\') {
            let rest = &name[first_part.len()..];
            return format!("{resolved}{rest}");
        }
        return resolved.clone();
    }
    if let Some(ns) = namespace {
        return format!("{ns}\\{name}");
    }
    name.to_string()
}

pub(super) fn resolve_alias_only(name: &str, use_aliases: &FxHashMap<String, String>) -> String {
    let name = name.trim_start_matches('\\');
    let first_part = name.split('\\').next().unwrap_or(name);
    if let Some(resolved) = find_alias(first_part, use_aliases) {
        if name.contains('\\') {
            let rest = &name[first_part.len()..];
            return format!("{resolved}{rest}");
        }
        return resolved.clone();
    }
    name.to_string()
}

pub(super) fn resolve_type_name(
    name: &str,
    full_qualify: bool,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Name {
    // Globally-qualified names (leading `\`) are already resolved — strip the
    // backslash and return without prepending the current namespace.
    if name.starts_with('\\') {
        return Name::from(name.trim_start_matches('\\'));
    }
    let stripped = name.trim_start_matches('\\');
    let first_part = stripped.split('\\').next().unwrap_or(stripped);
    if find_alias(first_part, use_aliases).is_some() {
        return resolve_alias_only(stripped, use_aliases).as_str().into();
    }
    if stripped.contains('\\') {
        return Name::from(stripped);
    }
    if full_qualify {
        resolve_name(stripped, namespace, use_aliases)
            .as_str()
            .into()
    } else {
        Name::from(stripped)
    }
}

pub(super) fn resolve_union_inner(
    union: Type,
    full_qualify: bool,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Type {
    let from_docblock = union.from_docblock;
    let types: Vec<Atomic> = union
        .types
        .into_iter()
        .map(|a| resolve_atomic_inner(a, full_qualify, namespace, use_aliases))
        .collect();
    let mut result = Type::from_vec(types);
    result.from_docblock = from_docblock;
    result
}

pub(super) fn resolve_atomic_inner(
    atomic: Atomic,
    full_qualify: bool,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Atomic {
    match atomic {
        Atomic::TNamedObject { fqcn, type_params } => {
            let resolved = resolve_type_name(fqcn.as_str(), full_qualify, namespace, use_aliases);
            if type_params.is_empty() {
                Atomic::TNamedObject {
                    fqcn: resolved,
                    type_params,
                }
            } else {
                let new_params: Vec<Type> = type_params
                    .iter()
                    .map(|p| resolve_union_inner(p.clone(), full_qualify, namespace, use_aliases))
                    .collect();
                Atomic::TNamedObject {
                    fqcn: resolved,
                    type_params: vec_to_type_params(new_params),
                }
            }
        }
        Atomic::TClassString(Some(cls)) => {
            let resolved = resolve_type_name(cls.as_str(), full_qualify, namespace, use_aliases);
            Atomic::TClassString(Some(resolved))
        }
        Atomic::TInterfaceString(Some(iface)) => {
            let resolved = resolve_type_name(iface.as_str(), full_qualify, namespace, use_aliases);
            Atomic::TInterfaceString(Some(resolved))
        }
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(resolve_union_inner(
                *key,
                full_qualify,
                namespace,
                use_aliases,
            )),
            value: Box::new(resolve_union_inner(
                *value,
                full_qualify,
                namespace,
                use_aliases,
            )),
        },
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(resolve_union_inner(
                *value,
                full_qualify,
                namespace,
                use_aliases,
            )),
        },
        Atomic::TNonEmptyArray { key, value } => Atomic::TNonEmptyArray {
            key: Box::new(resolve_union_inner(
                *key,
                full_qualify,
                namespace,
                use_aliases,
            )),
            value: Box::new(resolve_union_inner(
                *value,
                full_qualify,
                namespace,
                use_aliases,
            )),
        },
        Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
            value: Box::new(resolve_union_inner(
                *value,
                full_qualify,
                namespace,
                use_aliases,
            )),
        },
        Atomic::TConditional { data } => {
            let ConditionalData {
                param_name,
                subject,
                if_true,
                if_false,
            } = *data;
            Atomic::TConditional {
                data: Box::new(ConditionalData {
                    param_name,
                    subject: resolve_union_inner(subject, full_qualify, namespace, use_aliases),
                    if_true: resolve_union_inner(if_true, full_qualify, namespace, use_aliases),
                    if_false: resolve_union_inner(if_false, full_qualify, namespace, use_aliases),
                }),
            }
        }
        Atomic::TIntersection { parts } => Atomic::TIntersection {
            parts: vec_to_type_params(
                parts
                    .iter()
                    .map(|p| resolve_union_inner(p.clone(), full_qualify, namespace, use_aliases))
                    .collect(),
            ),
        },
        Atomic::TKeyedArray {
            properties,
            is_open,
            is_list,
        } => Atomic::TKeyedArray {
            properties: Box::new(
                properties
                    .into_iter()
                    .map(|(key, prop)| {
                        let resolved_ty =
                            resolve_union_inner(prop.ty, full_qualify, namespace, use_aliases);
                        (
                            key,
                            KeyedProperty {
                                ty: resolved_ty,
                                optional: prop.optional,
                            },
                        )
                    })
                    .collect(),
            ),
            is_open,
            is_list,
        },
        other => other,
    }
}

pub(super) fn fill_self_static_parent(union: Type, class_fqcn: &str) -> Type {
    let mut result = Type::empty();
    result.possibly_undefined = union.possibly_undefined;
    result.from_docblock = union.from_docblock;
    for a in union.types {
        let filled = match a {
            Atomic::TSelf { ref fqcn } if fqcn.is_empty() => Atomic::TSelf {
                fqcn: class_fqcn.into(),
            },
            Atomic::TStaticObject { ref fqcn } if fqcn.is_empty() => Atomic::TStaticObject {
                fqcn: class_fqcn.into(),
            },
            Atomic::TParent { ref fqcn } if fqcn.is_empty() => Atomic::TParent {
                fqcn: class_fqcn.into(),
            },
            other => other,
        };
        result.types.push(filled);
    }
    result
}

pub(super) fn resolve_union(
    union: Type,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Type {
    resolve_union_inner(union, true, namespace, use_aliases)
}

pub(super) fn resolve_union_doc(
    union: Type,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Type {
    resolve_union_inner(union, false, namespace, use_aliases)
}

pub(super) fn resolve_union_doc_with_aliases(
    union: Type,
    aliases: &FxHashMap<String, Type>,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Type {
    if aliases.is_empty() {
        return resolve_union_doc(union, namespace, use_aliases);
    }
    // Alias substitution first (against the still-raw, pre-FQN-resolution
    // names the alias map is keyed by), THEN FQN resolution — same ordering
    // the return-type call site already uses. `expand_aliases_only` recurses
    // into nested positions (a generic type argument, an array's key/value
    // type, …), so an alias used as `Box<IntList>` (not just a bare `IntList`)
    // now expands too; a single top-level-only check here previously missed
    // that case even though `expand_aliases_only` itself was fixed for it.
    let expanded = super::expand_aliases_only(union, aliases);
    resolve_union_doc(expanded, namespace, use_aliases)
}

pub(super) fn resolve_union_opt(
    opt: Option<Type>,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Option<Type> {
    opt.map(|u| resolve_union(u, namespace, use_aliases))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aliases(pairs: &[(&str, &str)]) -> FxHashMap<String, String> {
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn resolve_name_matches_qualified_alias_case_insensitively() {
        let use_aliases = aliases(&[("Deep", "MyApp\\Deep")]);
        let ns = Some("Client".to_string());
        assert_eq!(
            resolve_name("deep\\Service", &ns, &use_aliases),
            "MyApp\\Deep\\Service",
            "a differently-cased qualified reference must still resolve via the import"
        );
    }

    #[test]
    fn resolve_name_matches_unqualified_alias_case_insensitively() {
        let use_aliases = aliases(&[("Service", "MyApp\\Deep\\Service")]);
        let ns = Some("Client".to_string());
        assert_eq!(
            resolve_name("service", &ns, &use_aliases),
            "MyApp\\Deep\\Service"
        );
    }

    #[test]
    fn resolve_type_name_matches_qualified_alias_case_insensitively() {
        let use_aliases = aliases(&[("Deep", "MyApp\\Deep")]);
        let ns = Some("Client".to_string());
        assert_eq!(
            resolve_type_name("deep\\Service", true, &ns, &use_aliases).as_str(),
            "MyApp\\Deep\\Service"
        );
    }
}
