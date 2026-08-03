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
    allow_builtin_shortcut: bool,
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
    // Docblock-only leniency: a bare `Closure`/`Traversable`/`Generator`/…
    // without a `use` import is assumed to mean the global builtin, since
    // docblocks are commonly written without imports. A *native* type hint
    // (or any other real code reference — `extends`, `instanceof`, …) has no
    // such ambiguity: PHP's own name-resolution rules always qualify a bare
    // name against the current namespace regardless of whether it happens to
    // collide with a builtin name, so `allow_builtin_shortcut` must be false
    // for those callers or a same-namespace class named e.g. `Generator`
    // resolves to the wrong (builtin) FQCN.
    if allow_builtin_shortcut && crate::util::is_global_builtin_docblock_class(stripped) {
        return Name::from(stripped);
    }
    if !full_qualify {
        return Name::from(stripped);
    }
    // A qualified name already prefixed with the current namespace has
    // already been resolved once — e.g. spliced in from a pre-resolved
    // `@psalm-type`/`@phpstan-type` alias value (`build_type_aliases` resolves
    // the alias body eagerly so a cross-file `@psalm-import-type` carries its
    // *defining* file's namespace, not the importing file's), which
    // `resolve_union_doc_with_aliases` then re-runs FQN resolution over
    // wholesale after substitution. Re-prepending here would double the
    // namespace (`App\User` -> `App\App\User`). A qualified name genuinely
    // written by the user that happens to start with the current namespace's
    // own name is the rare case sacrificed to this guard.
    if let Some(ns) = namespace {
        if let Some(rest) = stripped.strip_prefix(ns.as_str()) {
            if rest.starts_with('\\') {
                return Name::from(stripped);
            }
        }
    }
    resolve_name(stripped, namespace, use_aliases)
        .as_str()
        .into()
}

pub(super) fn resolve_union_inner(
    union: Type,
    full_qualify: bool,
    allow_builtin_shortcut: bool,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Type {
    let from_docblock = union.from_docblock;
    let types: Vec<Atomic> = union
        .types
        .into_iter()
        .map(|a| {
            resolve_atomic_inner(
                a,
                full_qualify,
                allow_builtin_shortcut,
                namespace,
                use_aliases,
            )
        })
        .collect();
    let mut result = Type::from_vec(types);
    result.from_docblock = from_docblock;
    result
}

pub(super) fn resolve_atomic_inner(
    atomic: Atomic,
    full_qualify: bool,
    allow_builtin_shortcut: bool,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Atomic {
    macro_rules! ru {
        ($t:expr) => {
            resolve_union_inner(
                $t,
                full_qualify,
                allow_builtin_shortcut,
                namespace,
                use_aliases,
            )
        };
    }
    match atomic {
        Atomic::TNamedObject { fqcn, type_params } => {
            let resolved = resolve_type_name(
                fqcn.as_str(),
                full_qualify,
                allow_builtin_shortcut,
                namespace,
                use_aliases,
            );
            if type_params.is_empty() {
                Atomic::TNamedObject {
                    fqcn: resolved,
                    type_params,
                }
            } else {
                let new_params: Vec<Type> = type_params.iter().map(|p| ru!(p.clone())).collect();
                Atomic::TNamedObject {
                    fqcn: resolved,
                    type_params: vec_to_type_params(new_params),
                }
            }
        }
        Atomic::TClassString(Some(cls)) => {
            let resolved = resolve_type_name(
                cls.as_str(),
                full_qualify,
                allow_builtin_shortcut,
                namespace,
                use_aliases,
            );
            Atomic::TClassString(Some(resolved))
        }
        Atomic::TInterfaceString(Some(iface)) => {
            let resolved = resolve_type_name(
                iface.as_str(),
                full_qualify,
                allow_builtin_shortcut,
                namespace,
                use_aliases,
            );
            Atomic::TInterfaceString(Some(resolved))
        }
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(ru!(*key)),
            value: Box::new(ru!(*value)),
        },
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(ru!(*value)),
        },
        Atomic::TNonEmptyArray { key, value } => Atomic::TNonEmptyArray {
            key: Box::new(ru!(*key)),
            value: Box::new(ru!(*value)),
        },
        Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
            value: Box::new(ru!(*value)),
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
                    subject: ru!(subject),
                    if_true: ru!(if_true),
                    if_false: ru!(if_false),
                }),
            }
        }
        Atomic::TIntersection { parts } => Atomic::TIntersection {
            parts: vec_to_type_params(parts.iter().map(|p| ru!(p.clone())).collect()),
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
                        let resolved_ty = ru!(prop.ty);
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
        // `callable(T): R` / `Closure(T): R` — a class name embedded in one
        // of these signatures (including inside a `@psalm-type` alias body)
        // previously never went through `use`-import/namespace resolution at
        // all, since no arm here recursed into either variant. Mirrors the
        // identical fix already applied to the sibling `expand_aliases_in_atomic`.
        Atomic::TCallable {
            params,
            return_type,
        } => {
            Atomic::TCallable {
                params: params.map(|ps| {
                    ps.iter()
                        .map(|p| mir_types::atomic::FnParam {
                            ty: p.ty.as_ref().map(|t| {
                                mir_types::compact::SimpleType::from_union(ru!(t.to_union()))
                            }),
                            out_ty: p.out_ty.as_ref().map(|t| {
                                mir_types::compact::SimpleType::from_union(ru!(t.to_union()))
                            }),
                            ..p.clone()
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                }),
                return_type: return_type.map(|rt| Box::new(ru!(*rt))),
            }
        }
        Atomic::TClosure { data } => {
            Atomic::TClosure {
                data: Box::new(mir_types::atomic::ClosureData {
                    params: data
                        .params
                        .iter()
                        .map(|p| mir_types::atomic::FnParam {
                            ty: p.ty.as_ref().map(|t| {
                                mir_types::compact::SimpleType::from_union(ru!(t.to_union()))
                            }),
                            out_ty: p.out_ty.as_ref().map(|t| {
                                mir_types::compact::SimpleType::from_union(ru!(t.to_union()))
                            }),
                            ..p.clone()
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                    return_type: ru!(data.return_type),
                    this_type: data.this_type.map(|t| ru!(t)),
                }),
            }
        }
        other => other,
    }
}

fn is_self_static_parent_keyword(name: &Name) -> bool {
    matches!(
        crate::util::php_ident_lowercase(name.as_ref()).as_str(),
        "self" | "static" | "parent"
    )
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
            // `class-string<self>`/`class-string<static>`/`class-string<parent>` parse
            // with the keyword stored literally as the inner name (there's no sentinel
            // atom to fill unlike bare TSelf/TStaticObject/TParent above) — substitute
            // it here the same way, or it's never resolved and silently misses real bugs
            // through a `Foo::method()::other()` chain.
            Atomic::TClassString(Some(ref name)) if is_self_static_parent_keyword(name) => {
                Atomic::TClassString(Some(class_fqcn.into()))
            }
            Atomic::TInterfaceString(Some(ref name)) if is_self_static_parent_keyword(name) => {
                Atomic::TInterfaceString(Some(class_fqcn.into()))
            }
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
    // Native type hints resolve exactly like any other real code reference
    // (`extends`, `instanceof`, …): no builtin-name leniency — a same-namespace
    // class named e.g. `Generator` must win over the global builtin.
    resolve_union_inner(union, true, false, namespace, use_aliases)
}

pub(super) fn resolve_union_doc(
    union: Type,
    namespace: &Option<String>,
    use_aliases: &FxHashMap<String, String>,
) -> Type {
    // A bare same-namespace class name in a docblock (`@param Foo $x` inside
    // `namespace App;`, referring to `App\Foo`) must resolve exactly like a
    // native type hint does — `full_qualify=false` used to leave it bare
    // specifically to avoid mis-qualifying real global classes like `Closure`
    // against the current namespace, but that also silently left every
    // genuine sibling-class reference unqualified. `resolve_type_name` exempts
    // real global builtins on its own (`allow_builtin_shortcut=true` here) for
    // docblocks specifically, since they're commonly written without imports.
    resolve_union_inner(union, true, true, namespace, use_aliases)
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
            resolve_type_name("deep\\Service", true, false, &ns, &use_aliases).as_str(),
            "MyApp\\Deep\\Service"
        );
    }
}
