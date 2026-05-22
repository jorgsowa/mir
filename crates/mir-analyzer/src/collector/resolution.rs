use mir_types::{Atomic, Union};
use rustc_hash::FxHashMap;
use std::sync::Arc;

pub(super) fn resolve_name(
    name: &str,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> String {
    if name.starts_with('\\') {
        return name.trim_start_matches('\\').to_string();
    }
    let first_part = name.split('\\').next().unwrap_or(name);
    if let Some(resolved) = use_aliases.get(first_part) {
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
    if let Some(resolved) = use_aliases.get(first_part) {
        if name.contains('\\') {
            let rest = &name[first_part.len()..];
            return format!("{resolved}{rest}");
        }
        return resolved.clone();
    }
    name.to_string()
}

pub(super) fn resolve_type_name(
    name: &Arc<str>,
    full_qualify: bool,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Arc<str> {
    let stripped = name.trim_start_matches('\\');
    let first_part = stripped.split('\\').next().unwrap_or(stripped);
    if use_aliases.contains_key(first_part) {
        return resolve_alias_only(stripped, use_aliases).into();
    }
    if stripped.contains('\\') {
        return Arc::from(stripped);
    }
    if full_qualify {
        resolve_name(stripped, namespace, use_aliases).into()
    } else {
        Arc::from(stripped)
    }
}

pub(super) fn resolve_union_inner(
    union: Union,
    full_qualify: bool,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Union {
    let from_docblock = union.from_docblock;
    let types: Vec<Atomic> = union
        .types
        .into_iter()
        .map(|a| resolve_atomic_inner(a, full_qualify, namespace, use_aliases))
        .collect();
    let mut result = Union::from_vec(types);
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
            let resolved = resolve_type_name(&fqcn, full_qualify, namespace, use_aliases);
            Atomic::TNamedObject {
                fqcn: resolved,
                type_params,
            }
        }
        Atomic::TClassString(Some(cls)) => {
            let resolved = resolve_type_name(&cls, full_qualify, namespace, use_aliases);
            Atomic::TClassString(Some(resolved))
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
        other => other,
    }
}

pub(super) fn fill_self_static_parent(union: Union, class_fqcn: &str) -> Union {
    let mut result = Union::empty();
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
    union: Union,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Union {
    resolve_union_inner(union, true, namespace, use_aliases)
}

pub(super) fn resolve_union_doc(
    union: Union,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Union {
    resolve_union_inner(union, false, namespace, use_aliases)
}

pub(super) fn resolve_union_doc_with_aliases(
    union: Union,
    aliases: &FxHashMap<String, Union>,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Union {
    if aliases.is_empty() {
        return resolve_union_doc(union, namespace, use_aliases);
    }

    let from_docblock = union.from_docblock;
    let mut result = Union::empty();
    result.possibly_undefined = union.possibly_undefined;
    result.from_docblock = from_docblock;

    for atomic in union.types {
        match atomic {
            Atomic::TNamedObject { fqcn, type_params } if type_params.is_empty() => {
                if let Some(alias_ty) = aliases.get(fqcn.as_ref()) {
                    result.merge_with(alias_ty);
                } else {
                    result.add_type(resolve_atomic_inner(
                        Atomic::TNamedObject { fqcn, type_params },
                        false,
                        namespace,
                        use_aliases,
                    ));
                }
            }
            other => result.add_type(resolve_atomic_inner(other, false, namespace, use_aliases)),
        }
    }

    result
}

pub(super) fn resolve_union_opt(
    opt: Option<Union>,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Option<Union> {
    opt.map(|u| resolve_union(u, namespace, use_aliases))
}
