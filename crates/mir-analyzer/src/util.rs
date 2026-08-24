/// Lowercase a PHP identifier (method name, function name, class name, keyword).
///
/// PHP technically allows bytes `0x80–0xFF` in identifiers, but real-world PHP is
/// overwhelmingly ASCII. `to_ascii_lowercase` is correct for all ASCII identifiers and
/// faster than the Unicode-aware `to_lowercase`; bytes above 0x7F pass through unchanged.
///
/// **Do not use** for docblock content, string literals, or arbitrary source text —
/// those may contain non-ASCII characters that require full Unicode case folding.
#[inline]
pub(crate) fn php_ident_lowercase(s: &str) -> String {
    s.to_ascii_lowercase()
}

/// Native PHP type names that are valid to *spell* in type positions, even if
/// some of them (notably `resource`) are legacy/unsupported at runtime in some
/// declarations. These must never be treated as user-defined classes.
pub(crate) fn is_native_type_name(name: &str) -> bool {
    matches!(
        php_ident_lowercase(name).as_str(),
        "array"
            | "bool"
            | "callable"
            | "false"
            | "float"
            | "int"
            | "iterable"
            | "mixed"
            | "never"
            | "null"
            | "object"
            | "parent"
            | "resource"
            | "self"
            | "static"
            | "string"
            | "true"
            | "void"
    )
}

pub(crate) fn is_shadowable_docblock_pseudotype_alias(name: &str) -> bool {
    matches!(
        php_ident_lowercase(name).as_str(),
        "boolean" | "double" | "integer" | "number" | "numeric" | "real" | "resource"
    )
}

/// Every native PHP superglobal name (without the `$` prefix), for purity
/// checks that treat reading/writing one as touching external mutable
/// state — deliberately broader than `taint::SUPERGLOBALS` (which excludes
/// `$_SESSION`/`GLOBALS`/`argv`/`argc` since those aren't attacker-controlled
/// taint sources; purity cares about "is this external state", not "is this
/// user input", so the two lists have different membership on purpose).
pub(crate) fn is_superglobal_name(name: &str) -> bool {
    matches!(
        name,
        "GLOBALS"
            | "_SERVER"
            | "_GET"
            | "_POST"
            | "_REQUEST"
            | "_SESSION"
            | "_COOKIE"
            | "_FILES"
            | "_ENV"
    )
}

/// Real global-namespace PHP classes/interfaces commonly referenced bare in
/// **docblocks** with no explicit `use` import. Unlike a pseudo-type keyword
/// (`array`/`iterable`/…), these are actual classes — PHP's own namespace
/// resolution rules would require an import or a leading `\` for them in
/// real code, but Psalm/PHPStan resolve bare docblock references to them
/// leniently, so mir does too rather than mis-qualifying them against the
/// current namespace (the same failure mode already fixed for `iterable`'s
/// implicit `Traversable` member).
///
/// Only for docblock-type resolution — do not use this for resolving a real
/// code reference (a type hint, `instanceof`, `Foo::class`, …), where a bare
/// unqualified name genuinely does need the current namespace prepended,
/// same as real PHP.
///
/// `Countable` is deliberately excluded: unlike these others, it's common
/// enough as a user-redeclared interface name (a local `Countable` in a
/// legacy/polyfill namespace) that treating it as always-global would shadow
/// a real same-namespace declaration.
pub(crate) fn is_global_builtin_docblock_class(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "closure"
            | "traversable"
            | "iterator"
            | "iteratoraggregate"
            | "arrayaccess"
            | "generator"
            | "stringable"
            | "stdclass"
            | "throwable"
    )
}

/// A docblock-derived type may carry a bare well-known-global-builtin name
/// (`Closure`/`Traversable`/`Iterator`/`IteratorAggregate`/`ArrayAccess`/
/// `Generator`/`Stringable`/`stdClass`/`Throwable`) purely because that
/// leniency was applied once, at collection time (`collector::resolution`,
/// which has no db access and so can't know whether a same-namespace class
/// shadows the name). A native type hint has no such leniency — it always
/// resolves strictly — so when a real class exists at the namespace-qualified
/// name, the docblock's bare reference reads the same way the native hint
/// does (the local shadow), not as the actual global builtin. Comparing the
/// two without reconciling would flag a real, intentional match as
/// `MismatchingDocblockReturnType`/`ParamType`, and storing it unreconciled
/// would carry the same wrong name into every later flow-analysis/member-
/// lookup consumer of a param/return/property type (P28). Only ever narrows
/// a bare builtin name to a confirmed *existing* local class — every other
/// docblock type is returned unchanged, so this can't mask a genuine
/// mismatch.
pub(crate) fn reconcile_docblock_builtin_shadow(
    db: &dyn crate::db::MirDatabase,
    file: &str,
    ty: mir_types::Type,
) -> mir_types::Type {
    let from_docblock = ty.from_docblock;
    let possibly_undefined = ty.possibly_undefined;
    let types: Vec<mir_types::Atomic> = ty
        .types
        .into_iter()
        .map(|a| reconcile_builtin_shadow_atomic(db, file, a))
        .collect();
    let mut result = mir_types::Type::from_vec(types);
    result.from_docblock = from_docblock;
    result.possibly_undefined = possibly_undefined;
    result
}

fn reconcile_builtin_shadow_atomic(
    db: &dyn crate::db::MirDatabase,
    file: &str,
    atomic: mir_types::Atomic,
) -> mir_types::Atomic {
    use mir_types::Atomic;
    match atomic {
        Atomic::TNamedObject { fqcn, type_params } => {
            let fqcn = reconcile_builtin_shadow_name(db, file, fqcn);
            let type_params = type_params
                .iter()
                .cloned()
                .map(|tp| reconcile_docblock_builtin_shadow(db, file, tp))
                .collect();
            Atomic::TNamedObject { fqcn, type_params }
        }
        Atomic::TClassString(Some(cls)) => {
            Atomic::TClassString(Some(reconcile_builtin_shadow_name(db, file, cls)))
        }
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(reconcile_docblock_builtin_shadow(db, file, *key)),
            value: Box::new(reconcile_docblock_builtin_shadow(db, file, *value)),
        },
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(reconcile_docblock_builtin_shadow(db, file, *value)),
        },
        Atomic::TIntersection { parts } => Atomic::TIntersection {
            parts: parts
                .iter()
                .cloned()
                .map(|p| reconcile_docblock_builtin_shadow(db, file, p))
                .collect(),
        },
        other => other,
    }
}

fn reconcile_builtin_shadow_name(
    db: &dyn crate::db::MirDatabase,
    file: &str,
    fqcn: mir_types::Name,
) -> mir_types::Name {
    if !is_global_builtin_docblock_class(fqcn.as_ref()) {
        return fqcn;
    }
    let qualified = crate::db::resolve_name(db, file, fqcn.as_ref());
    if qualified != fqcn.as_ref() && crate::db::class_exists(db, &qualified) {
        mir_types::Name::from(qualified.as_str())
    } else {
        fqcn
    }
}

/// Param-list counterpart of [`reconcile_docblock_builtin_shadow`] — applied
/// to a function/method's stored `DeclaredParam` list right before it seeds
/// body-analysis flow state, so a docblock-shadowed builtin param type
/// doesn't carry its lenient bare name into the body's own flow tracking
/// (P28). Skips the allocation entirely when no param's type is
/// docblock-derived, which is the overwhelming majority case.
pub(crate) fn reconcile_declared_params_for_body(
    db: &dyn crate::db::MirDatabase,
    file: &str,
    params: std::sync::Arc<[mir_codebase::DeclaredParam]>,
) -> std::sync::Arc<[mir_codebase::DeclaredParam]> {
    let needs_reconcile = params
        .iter()
        .any(|p| p.ty.as_ref().is_some_and(|t| t.from_docblock));
    if !needs_reconcile {
        return params;
    }
    params
        .iter()
        .cloned()
        .map(|mut p| {
            p = reconcile_declared_param_docblock_shadow(db, file, p);
            if let Some(ty) = p.ty.take() {
                if ty.from_docblock {
                    p.ty = Some(std::sync::Arc::new(reconcile_docblock_builtin_shadow(
                        db,
                        file,
                        (*ty).clone(),
                    )));
                } else {
                    p.ty = Some(ty);
                }
            }
            p
        })
        .collect::<Vec<_>>()
        .into()
}

pub(crate) fn reconcile_declared_param_docblock_shadow(
    db: &dyn crate::db::MirDatabase,
    default_file: &str,
    mut param: mir_codebase::DeclaredParam,
) -> mir_codebase::DeclaredParam {
    let Some(raw) = param.doc_type_raw.as_deref() else {
        return param;
    };
    let Some(ty) = param.ty.as_deref() else {
        return param;
    };
    if !ty.from_docblock {
        return param;
    }
    let raw = raw.trim();
    if !is_shadowable_docblock_pseudotype_alias(raw) {
        return param;
    }
    let file = param.doc_type_file.as_deref().unwrap_or(default_file);
    let fqcn = crate::db::resolve_name(db, file, raw);
    if !crate::db::class_exists(db, &fqcn) {
        return param;
    }
    let mut resolved = mir_types::Type::single(mir_types::Atomic::TNamedObject {
        fqcn: mir_types::Name::from(fqcn.as_str()),
        type_params: mir_types::union::empty_type_params(),
    });
    resolved.from_docblock = ty.from_docblock;
    resolved.possibly_undefined = ty.possibly_undefined;
    resolved.falsy_stripped = ty.falsy_stripped;
    resolved.possibly_absent_offset = ty.possibly_absent_offset;
    param.ty = Some(std::sync::Arc::new(resolved));
    param
}

/// Property counterpart of [`reconcile_docblock_builtin_shadow`] (P28's
/// class-member sibling). Unlike a param/return type, a stored `@var`
/// property type has no `db`-resolvable file context readily at hand where
/// it's read back (a property can be read from any file, not just its
/// declaring one) — but `PropertyDef` already stores the property's native
/// type hint *separately* (`native_ty`), and that native hint always
/// resolves strictly (never leniently), so it's already the confirmed-
/// correct FQCN whenever a same-namespace class shadows a builtin. Reconcile
/// against it directly: no db query needed at all. Only ever narrows a bare
/// builtin name in `ty` to `native_ty`'s FQCN for the same bare name —
/// every other case (no native hint, non-builtin docblock name, disagreeing
/// class names) is left exactly as collected.
pub(crate) fn reconcile_property_ty_against_native(
    ty: mir_types::Type,
    native_ty: Option<&mir_types::Type>,
) -> mir_types::Type {
    let Some(native_ty) = native_ty else {
        return ty;
    };
    let from_docblock = ty.from_docblock;
    let possibly_undefined = ty.possibly_undefined;
    let types: Vec<mir_types::Atomic> = ty
        .types
        .into_iter()
        .map(|a| reconcile_prop_shadow_atomic(a, native_ty))
        .collect();
    let mut result = mir_types::Type::from_vec(types);
    result.from_docblock = from_docblock;
    result.possibly_undefined = possibly_undefined;
    result
}

fn reconcile_prop_shadow_atomic(
    atomic: mir_types::Atomic,
    native_ty: &mir_types::Type,
) -> mir_types::Atomic {
    use mir_types::Atomic;
    match atomic {
        Atomic::TNamedObject { fqcn, type_params } => {
            let fqcn = reconcile_prop_shadow_name(fqcn, native_ty);
            let type_params = type_params
                .iter()
                .cloned()
                .map(|tp| reconcile_property_ty_against_native(tp, Some(native_ty)))
                .collect();
            Atomic::TNamedObject { fqcn, type_params }
        }
        Atomic::TClassString(Some(cls)) => {
            Atomic::TClassString(Some(reconcile_prop_shadow_name(cls, native_ty)))
        }
        other => other,
    }
}

/// Whether `native_ty` names, at any atom, the same class `bare_name`
/// leniently resolved to as a global builtin — i.e. a same-namespace class
/// literally called `Generator`/`Closure`/etc., confirmed by the native
/// hint's own strict resolution.
fn reconcile_prop_shadow_name(
    fqcn: mir_types::Name,
    native_ty: &mir_types::Type,
) -> mir_types::Name {
    if !is_global_builtin_docblock_class(fqcn.as_ref()) {
        return fqcn;
    }
    let bare_lower = fqcn.as_ref().to_ascii_lowercase();
    let shadow = native_ty.types.iter().find_map(|atom| {
        let candidate = match atom {
            mir_types::Atomic::TNamedObject { fqcn: n, .. } => Some(n),
            mir_types::Atomic::TClassString(Some(n)) => Some(n),
            _ => None,
        }?;
        let candidate_bare = candidate.as_ref().rsplit('\\').next().unwrap_or("");
        (candidate.as_ref() != fqcn.as_ref() && candidate_bare.eq_ignore_ascii_case(&bare_lower))
            .then_some(*candidate)
    });
    shadow.unwrap_or(fqcn)
}
