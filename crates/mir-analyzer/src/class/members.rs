use super::*;
use mir_codebase::definitions::{ConstantDef, PropertyDef};

impl<'a> ClassAnalyzer<'a> {
    pub(super) fn check_abstract_methods_implemented(
        &self,
        fqcn: &Arc<str>,
        cls_location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        // Walk every ancestor and collect abstract methods. Uses the
        // fully-recursive `class_ancestors_by_fqcn` (skipping index 0, `fqcn`
        // itself) so an abstract method declared only in a trait-of-a-trait is
        // still considered here.
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        let ancestors = crate::db::class_ancestors_by_fqcn(self.db, here);
        for ancestor_fqcn in ancestors.iter().skip(1) {
            let here = crate::db::Fqcn::from_str(self.db, ancestor_fqcn.as_ref());
            let abstract_methods: Vec<Arc<str>> = crate::db::find_class_like(self.db, here)
                .map(|c| {
                    c.own_methods()
                        .iter()
                        .filter(|(_, m)| m.is_abstract)
                        .map(|(_, m)| m.name.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            for method_name in abstract_methods {
                // Check if the concrete class (or any closer ancestor) provides it
                if crate::db::is_method_concretely_implemented(
                    self.db,
                    fqcn.as_ref(),
                    method_name.as_ref(),
                ) {
                    continue; // implemented
                }

                let loc = issue_location(
                    cls_location,
                    cls_location.and_then(|l| self.sources.get(&l.file).copied()),
                );
                let mut issue = Issue::new(
                    IssueKind::UnimplementedAbstractMethod {
                        class: fqcn.to_string(),
                        method: method_name.to_string(),
                    },
                    loc,
                );
                if let Some(snippet) = extract_snippet(cls_location, &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Check: all interface methods are implemented
    // -----------------------------------------------------------------------

    pub(super) fn check_interface_methods_implemented(
        &self,
        fqcn: &Arc<str>,
        cls_location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        // Collect all interfaces (direct + from ancestors)
        let all_ifaces: Vec<Arc<str>> =
            crate::db::class_ancestors_by_fqcn(self.db, crate::db::Fqcn::from_str(self.db, fqcn))
                .iter()
                .skip(1)
                .filter(|p| {
                    crate::db::class_kind(self.db, p.as_ref()).is_some_and(|k| k.is_interface)
                })
                .cloned()
                .collect();

        for iface_fqcn in &all_ifaces {
            let here = crate::db::Fqcn::from_str(self.db, iface_fqcn.as_ref());
            let method_names: Vec<Arc<str>> = match crate::db::find_class_like(self.db, here) {
                Some(c) => c
                    .own_methods()
                    .iter()
                    .filter(|(_, m)| !m.is_virtual)
                    .map(|(_, m)| m.name.clone())
                    .collect(),
                None => continue,
            };
            if method_names.is_empty() {
                continue;
            }

            for method_name in method_names {
                // PHP method names are case-insensitive; normalize before lookup so that
                // a hand-written stub key like "jsonSerialize" matches the collector's
                // lowercased key "jsonserialize" stored in own_methods.
                let method_name_lower = crate::util::php_ident_lowercase(&method_name);
                // Check if the class provides a concrete implementation
                let implemented = crate::db::is_method_concretely_implemented(
                    self.db,
                    fqcn.as_ref(),
                    &method_name_lower,
                );

                if !implemented {
                    let loc = issue_location(
                        cls_location,
                        cls_location.and_then(|l| self.sources.get(&l.file).copied()),
                    );
                    let mut issue = Issue::new(
                        IssueKind::UnimplementedInterfaceMethod {
                            class: fqcn.to_string(),
                            interface: iface_fqcn.to_string(),
                            method: method_name.to_string(),
                        },
                        loc,
                    );
                    if let Some(snippet) = extract_snippet(cls_location, &self.sources) {
                        issue = issue.with_snippet(snippet);
                    }
                    issues.push(issue);
                }
                // When the implementation is provided by a trait, signature
                // compatibility against the interface is checked by
                // `check_overrides` itself, which treats trait-composed methods
                // as this class's "own" for override purposes.
            }
        }
    }

    // -----------------------------------------------------------------------
    // Check: enum interface methods implemented
    // -----------------------------------------------------------------------

    /// Built-in PHP enum interfaces fulfilled implicitly by the runtime.
    /// Their methods (cases, from, tryFrom) are synthesized and must not be
    /// required from user-defined enum own_methods.
    fn is_builtin_enum_interface(fqcn: &str) -> bool {
        let bare = fqcn.to_lowercase();
        let bare = bare.trim_start_matches('\\');
        matches!(
            bare,
            "unitenum" | "backedenum" | "intbackedenum" | "stringbackedenum"
        )
    }

    /// Enums cannot extend parent classes, but can satisfy an interface via
    /// their own methods or a `use`d trait's methods. Walk all transitively
    /// required interfaces — skipping built-in PHP enum contracts —
    /// and emit `UnimplementedInterfaceMethod` for each missing method.
    pub(super) fn check_enum_interface_methods_implemented(
        &self,
        enum_fqcn: &Arc<str>,
        cls_location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        let here = crate::db::Fqcn::from_str(self.db, enum_fqcn.as_ref());

        // `class_ancestors_by_fqcn` starts from the enum itself and DFS-walks
        // its interface chain (via `ancestor_fqcns` which returns `e.interfaces`
        // for enums). Filter to user-defined interfaces only.
        let iface_fqcns: Vec<Arc<str>> = crate::db::class_ancestors_by_fqcn(self.db, here)
            .iter()
            .filter(|f| {
                !Self::is_builtin_enum_interface(f.as_ref())
                    && crate::db::class_kind(self.db, f.as_ref()).is_some_and(|k| k.is_interface)
            })
            .cloned()
            .collect();

        if iface_fqcns.is_empty() {
            return;
        }

        for iface_fqcn in &iface_fqcns {
            let iface_here = crate::db::Fqcn::from_str(self.db, iface_fqcn.as_ref());
            let method_names: Vec<Arc<str>> = match crate::db::find_class_like(self.db, iface_here)
            {
                Some(c) => c
                    .own_methods()
                    .iter()
                    .filter(|(_, m)| !m.is_virtual)
                    .map(|(_, m)| m.name.clone())
                    .collect(),
                None => continue,
            };

            for method_name in method_names {
                let lower = crate::util::php_ident_lowercase(&method_name);
                // `class_ancestors_by_fqcn` already walks an enum's `use`d traits
                // (via `ancestor_fqcns`'s `ClassLike::Enum` arm), so the same
                // ancestor-walking check used for classes correctly credits a
                // trait-provided method as satisfying the interface.
                let implemented = crate::db::is_method_concretely_implemented(
                    self.db,
                    enum_fqcn.as_ref(),
                    &lower,
                );

                if !implemented {
                    let loc = issue_location(
                        cls_location,
                        cls_location.and_then(|l| self.sources.get(&l.file).copied()),
                    );
                    let mut issue = Issue::new(
                        IssueKind::UnimplementedInterfaceMethod {
                            class: enum_fqcn.to_string(),
                            interface: iface_fqcn.to_string(),
                            method: method_name.to_string(),
                        },
                        loc,
                    );
                    if let Some(snippet) = extract_snippet(cls_location, &self.sources) {
                        issue = issue.with_snippet(snippet);
                    }
                    issues.push(issue);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Check: override compatibility
    // -----------------------------------------------------------------------

    /// Returns true if both scalar return types are compatible (covariant).
    /// Only called when neither side contains named objects or self/static —
    /// those cases are handled by named_object_return_compatible.
    pub(super) fn scalar_return_types_compatible(
        child_ret: &mir_types::Type,
        parent_ret: &mir_types::Type,
    ) -> bool {
        child_ret.is_subtype_structural(parent_ret)
    }

    /// Returns true when a child's scalar param type has been illegally narrowed
    /// relative to the parent (contravariance violation).
    /// Only called after confirming neither side contains named objects, self/static,
    /// templates, or mixed — those cases are skipped by the caller.
    pub(super) fn scalar_param_type_narrowed(
        parent_ty: &mir_types::Type,
        child_ty: &mir_types::Type,
    ) -> bool {
        // `A&B` accepts everything its part `A` does, so a child widened to `A` is valid
        let intersection_part_fits = matches!(
            parent_ty.types.as_slice(),
            [mir_types::Atomic::TIntersection { parts }]
                if parts.iter().any(|part| part.is_subtype_structural(child_ty))
        );
        !(intersection_part_fits || parent_ty.is_subtype_structural(child_ty))
    }

    pub(super) fn check_magic_method_casing(&self, fqcn: &Arc<str>, issues: &mut Vec<Issue>) {
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        let Some(class) = crate::db::find_class_like(self.db, here) else {
            return;
        };
        let own_methods: Vec<Arc<mir_codebase::definitions::MethodDef>> =
            class.own_methods().iter().map(|(_, m)| m.clone()).collect();
        for own in own_methods {
            let method_name = own.name.as_ref();
            let lower = method_name.to_ascii_lowercase();
            let Some(canonical) = canonical_magic_name(&lower) else {
                continue;
            };
            if method_name == canonical {
                continue;
            }
            let own_location = own.location.clone();
            let loc = issue_location(
                own_location.as_ref(),
                own_location
                    .as_ref()
                    .and_then(|l| self.sources.get(&l.file).copied()),
            );
            let mut issue = Issue::new(
                IssueKind::WrongCaseMethod {
                    class: fqcn.to_string(),
                    used: method_name.to_string(),
                    canonical: canonical.to_string(),
                },
                loc,
            );
            if let Some(snippet) = extract_snippet(own_location.as_ref(), &self.sources) {
                issue = issue.with_snippet(snippet);
            }
            issues.push(issue);
        }
    }
}

impl ClassAnalyzer<'_> {
    /// PHP rejects these magic methods on an enum, including ones a used trait brings in.
    pub(super) fn check_enum_magic_methods(
        &self,
        enum_def: &mir_codebase::definitions::EnumDef,
        issues: &mut Vec<Issue>,
    ) {
        let allows_debug_info =
            crate::db::db_php_version(self.db) >= crate::php_version::PhpVersion::new(8, 6);
        let is_forbidden = |name: &str| match canonical_magic_name(&name.to_ascii_lowercase()) {
            None | Some("__call" | "__callStatic" | "__invoke") => None,
            Some("__debugInfo") if allows_debug_info => None,
            Some(canonical) => Some(canonical),
        };
        let mut report = |canonical: &str, location: Option<&Location>| {
            self.push_located_issue(
                issues,
                IssueKind::CompileError {
                    message: format!(
                        "Enum {} cannot include magic method {canonical}",
                        enum_def.fqcn
                    ),
                    code: "enum-magic-method".to_string(),
                },
                location,
            );
        };
        // The enum's own definition, not a lookup by name: a polyfill may declare
        // a class of the same name behind a version check.
        for (_, method) in enum_def.own_methods.iter() {
            if let Some(canonical) = is_forbidden(&method.name) {
                report(canonical, method.location.as_ref());
            }
        }
        let mut seen = HashSet::default();
        for used in enum_def.traits.iter() {
            let here = crate::db::Fqcn::from_str(self.db, used.as_ref());
            for trait_fqcn in crate::db::class_ancestors_by_fqcn(self.db, here).iter() {
                if !seen.insert(trait_fqcn.clone()) {
                    continue;
                }
                let trait_here = crate::db::Fqcn::from_str(self.db, trait_fqcn.as_ref());
                let Some(trait_like) =
                    crate::db::find_class_like(self.db, trait_here).filter(|t| t.is_trait())
                else {
                    continue;
                };
                for (_, method) in trait_like.own_methods().iter() {
                    if let Some(canonical) = is_forbidden(&method.name) {
                        report(canonical, enum_def.location.as_ref());
                    }
                }
            }
        }
    }
}

/// The method name PHP declares on every enum (`cases`) or backed enum (`from`, `tryFrom`).
pub(super) fn enum_builtin_method(
    enum_def: &mir_codebase::definitions::EnumDef,
    method_name: &str,
) -> Option<&'static str> {
    match method_name.to_ascii_lowercase().as_str() {
        "cases" => Some("cases"),
        "from" if enum_def.scalar_type.is_some() => Some("from"),
        "tryfrom" if enum_def.scalar_type.is_some() => Some("tryfrom"),
        _ => None,
    }
}

impl ClassAnalyzer<'_> {
    /// PHP rejects an enum method named like one it declares itself; a trait's copy is shadowed.
    pub(super) fn check_enum_builtin_redeclarations(
        &self,
        enum_def: &mir_codebase::definitions::EnumDef,
        issues: &mut Vec<Issue>,
    ) {
        for (_, method) in enum_def.own_methods.iter() {
            if let Some(builtin) = enum_builtin_method(enum_def, &method.name) {
                self.push_located_issue(
                    issues,
                    IssueKind::CompileError {
                        message: format!("Cannot redeclare {}::{builtin}()", enum_def.fqcn),
                        code: "duplicate-method".to_string(),
                    },
                    method.location.as_ref(),
                );
            }
        }
    }
}

impl ClassAnalyzer<'_> {
    /// PHP rejects a class constant that overrides a `final` one or narrows its visibility.
    pub(super) fn check_constant_overrides(
        &self,
        fqcn: &Arc<str>,
        own_constants: &mir_codebase::definitions::MemberMap<
            mir_codebase::definitions::ConstantDef,
        >,
        issues: &mut Vec<Issue>,
    ) {
        use mir_codebase::definitions::Visibility;
        if own_constants.is_empty() {
            return;
        }
        let rank = |v: Visibility| match v {
            Visibility::Public => 0,
            Visibility::Protected => 1,
            Visibility::Private => 2,
        };
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        let ancestors = crate::db::class_ancestors_by_fqcn(self.db, here);
        for (name, own) in own_constants.iter() {
            for ancestor_fqcn in ancestors.iter().skip(1) {
                let ancestor_here = crate::db::Fqcn::from_str(self.db, ancestor_fqcn.as_ref());
                let Some(ancestor) =
                    crate::db::find_class_like(self.db, ancestor_here).filter(|c| !c.is_trait())
                else {
                    continue;
                };
                let Some(inherited) = ancestor.own_constants().get(name) else {
                    continue;
                };
                let inherited_vis = inherited.visibility.unwrap_or(Visibility::Public);
                let own_vis = own.visibility.unwrap_or(Visibility::Public);
                let ancestor_name = ancestor.fqcn();
                let error = if inherited_vis == Visibility::Private {
                    None
                } else if inherited.is_final {
                    Some((
                        format!(
                            "{fqcn}::{name} cannot override final constant {ancestor_name}::{name}"
                        ),
                        "final-constant-override",
                    ))
                } else if rank(own_vis) > rank(inherited_vis) {
                    let kind = if ancestor.is_interface() {
                        "interface"
                    } else {
                        "class"
                    };
                    let (required, weaker) = match inherited_vis {
                        Visibility::Protected => ("protected", " or weaker"),
                        _ => ("public", ""),
                    };
                    Some((
                        format!(
                            "Access level to {fqcn}::{name} must be {required} (as in {kind} {ancestor_name}){weaker}"
                        ),
                        "constant-access-level",
                    ))
                } else {
                    None
                };
                if let Some((message, code)) = error {
                    self.push_located_issue(
                        issues,
                        IssueKind::CompileError {
                            message,
                            code: code.to_string(),
                        },
                        own.location.as_ref(),
                    );
                }
                break;
            }
        }
    }
}

impl ClassAnalyzer<'_> {
    /// PHP rejects two used traits that bring different bodies for one method the
    /// class neither declares nor resolves with `insteadof`.
    pub(super) fn check_trait_method_collisions(
        &self,
        class: &mir_codebase::definitions::ClassDef,
        issues: &mut Vec<Issue>,
    ) {
        if class.traits.len() < 2 {
            return;
        }
        // method key → (origin trait, method) per direct trait; `None` when the
        // trait's own nested uses make the origin ambiguous.
        type Provided =
            HashMap<Arc<str>, Option<(Arc<str>, Arc<mir_codebase::definitions::MethodDef>)>>;
        let provided_by = |direct: &Arc<str>| -> Provided {
            let mut provided = Provided::default();
            let mut seen = HashSet::default();
            let mut stack = vec![(direct.clone(), true)];
            while let Some((trait_fqcn, is_direct)) = stack.pop() {
                if !seen.insert(trait_fqcn.to_ascii_lowercase()) {
                    continue;
                }
                let here = crate::db::Fqcn::from_str(self.db, trait_fqcn.as_ref());
                let Some(crate::db::ClassLike::Trait(t)) =
                    crate::db::find_class_like(self.db, here)
                else {
                    continue;
                };
                for (key, method) in t.own_methods.iter() {
                    if method.is_abstract {
                        continue;
                    }
                    let origin = Some((t.fqcn.clone(), method.clone()));
                    match provided.get(key) {
                        None => {
                            provided.insert(key.clone(), origin);
                        }
                        // The direct trait's own method wins over nested ones.
                        Some(_) if is_direct => {
                            provided.insert(key.clone(), origin);
                        }
                        Some(Some((other, _))) if !other.eq_ignore_ascii_case(&t.fqcn) => {
                            provided.insert(key.clone(), None);
                        }
                        Some(_) => {}
                    }
                }
                stack.extend(t.traits.iter().map(|n| (n.clone(), false)));
            }
            provided
        };
        let mut applied = Provided::default();
        let mut reported = HashSet::default();
        for direct in &class.traits {
            let mut provided: Vec<_> = provided_by(direct).into_iter().collect();
            provided.sort_by(|a, b| a.0.cmp(&b.0));
            for (key, origin) in provided {
                if class.own_methods.contains_key(&key)
                    || class.trait_insteadof.get(&key).is_some_and(|excluded| {
                        excluded.iter().any(|e| e.eq_ignore_ascii_case(direct))
                    })
                {
                    continue;
                }
                let Some(earlier) = applied.get(&key) else {
                    applied.insert(key, origin);
                    continue;
                };
                let (Some((first_trait, first)), Some((trait_fqcn, method))) = (earlier, &origin)
                else {
                    continue;
                };
                if first_trait.eq_ignore_ascii_case(trait_fqcn) || !reported.insert(key.clone()) {
                    continue;
                }
                let location = class
                    .trait_use_locations
                    .iter()
                    .find(|(used, _)| used.eq_ignore_ascii_case(direct))
                    .map(|(_, loc)| loc)
                    .or(class.location.as_ref());
                self.push_located_issue(
                    issues,
                    IssueKind::CompileError {
                        message: format!(
                            "Trait method {trait_fqcn}::{} has not been applied as {}::{}, because of collision with {first_trait}::{}",
                            method.name, class.fqcn, method.name, first.name
                        ),
                        code: "trait-method-collision".to_string(),
                    },
                    location,
                );
            }
        }
    }
}

impl ClassAnalyzer<'_> {
    /// PHP rejects a used trait's property or constant that the class, or an earlier
    /// trait, declares differently.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_trait_member_compatibility(
        &self,
        fqcn: &Arc<str>,
        own_properties: Option<&mir_codebase::definitions::MemberMap<PropertyDef>>,
        own_constants: &mir_codebase::definitions::MemberMap<ConstantDef>,
        traits: &[Arc<str>],
        trait_use_locations: &[(Arc<str>, Location)],
        location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        let mut properties: HashMap<Arc<str>, (Arc<str>, PropertyDef)> = HashMap::default();
        for (name, prop) in own_properties.into_iter().flatten() {
            if !prop.from_docblock {
                properties.insert(name.clone(), (fqcn.clone(), prop.clone()));
            }
        }
        let mut constants: HashMap<Arc<str>, (Arc<str>, ConstantDef)> = own_constants
            .iter()
            .map(|(name, c)| (name.clone(), (fqcn.clone(), c.clone())))
            .collect();
        for used in traits {
            let here = crate::db::Fqcn::from_str(self.db, used.as_ref());
            let Some(crate::db::ClassLike::Trait(t)) = crate::db::find_class_like(self.db, here)
            else {
                continue;
            };
            let location = trait_use_locations
                .iter()
                .find(|(u, _)| u.eq_ignore_ascii_case(used))
                .map(|(_, loc)| loc)
                .or(location);
            let mut report = |kind: &str, member: &str, first: &str| {
                self.push_located_issue(
                    issues,
                    IssueKind::CompileError {
                        message: format!(
                            "{first} and {} define the same {kind} ({member}) in the composition of {fqcn}. However, the definition differs and is considered incompatible. Class was composed",
                            t.fqcn
                        ),
                        code: format!("trait-{kind}-conflict"),
                    },
                    location,
                );
            };
            for (name, prop) in t.own_properties.iter() {
                if prop.from_docblock {
                    continue;
                }
                match properties.get(name) {
                    Some((first, earlier)) => {
                        if !trait_properties_compatible(earlier, prop) {
                            report("property", &format!("${name}"), first);
                        }
                    }
                    None => {
                        properties.insert(name.clone(), (t.fqcn.clone(), prop.clone()));
                    }
                }
            }
            for (name, constant) in t.own_constants.iter() {
                match constants.get(name) {
                    Some((first, earlier)) => {
                        if !trait_constants_compatible(earlier, constant) {
                            report("constant", name, first);
                        }
                    }
                    None => {
                        constants.insert(name.clone(), (t.fqcn.clone(), constant.clone()));
                    }
                }
            }
        }
    }
}

/// A value known exactly enough to compare with `===`; `None` when it is not.
fn comparable_value(ty: &mir_types::Type) -> Option<&mir_types::Atomic> {
    use mir_types::Atomic;
    match ty.types.as_slice() {
        [a @ (Atomic::TLiteralInt(_)
        | Atomic::TLiteralString(_)
        | Atomic::TLiteralFloat(..)
        | Atomic::TTrue
        | Atomic::TFalse
        | Atomic::TNull)] => Some(a),
        _ => None,
    }
}

/// Property defaults are typed by kind only (`int`, not `1`).
fn scalar_kind(ty: &mir_types::Type) -> Option<u8> {
    use mir_types::Atomic;
    match ty.types.as_slice() {
        [Atomic::TInt | Atomic::TLiteralInt(_)] => Some(0),
        [Atomic::TString | Atomic::TLiteralString(_)] => Some(1),
        [Atomic::TFloat | Atomic::TLiteralFloat(..)] => Some(2),
        [Atomic::TBool | Atomic::TTrue | Atomic::TFalse] => Some(3),
        [Atomic::TNull] => Some(4),
        _ => None,
    }
}

fn values_differ(a: &mir_types::Type, b: &mir_types::Type) -> bool {
    if let (Some(x), Some(y)) = (comparable_value(a), comparable_value(b)) {
        return x != y;
    }
    matches!((scalar_kind(a), scalar_kind(b)), (Some(x), Some(y)) if x != y)
}

fn trait_properties_compatible(a: &PropertyDef, b: &PropertyDef) -> bool {
    if a.visibility != b.visibility
        || a.is_static != b.is_static
        || a.has_native_readonly != b.has_native_readonly
    {
        return false;
    }
    match (a.native_ty.as_deref(), b.native_ty.as_deref()) {
        (None, None) => {}
        (Some(x), Some(y)) => {
            let unresolved = |t: &mir_types::Type| {
                t.types.iter().any(|at| {
                    matches!(
                        at,
                        mir_types::Atomic::TSelf { .. }
                            | mir_types::Atomic::TStaticObject { .. }
                            | mir_types::Atomic::TParent { .. }
                    )
                })
            };
            if !unresolved(x)
                && !unresolved(y)
                && !(x.is_subtype_structural(y) && y.is_subtype_structural(x))
            {
                return false;
            }
        }
        _ => return false,
    }
    // An untyped property without an initializer is `null`; a typed one is uninitialized.
    let null = mir_types::Type::null();
    let default = |p: &PropertyDef| match p.default.as_deref() {
        Some(d) => Some(d.clone()),
        None if p.native_ty.is_none() => Some(null.clone()),
        None => None,
    };
    match (default(a), default(b)) {
        // An initializer typed `mixed` is unknown, and a promoted property's is its parameter's.
        (Some(x), _) | (_, Some(x)) if x.is_mixed() => true,
        (Some(x), Some(y)) => !values_differ(&x, &y),
        (None, None) => true,
        _ => false,
    }
}

fn trait_constants_compatible(a: &ConstantDef, b: &ConstantDef) -> bool {
    let visibility = |c: &ConstantDef| c.visibility.unwrap_or(Visibility::Public);
    let value = |c: &ConstantDef| c.literal_value.clone().unwrap_or_else(|| c.ty.clone());
    visibility(a) == visibility(b)
        && a.is_final == b.is_final
        && !values_differ(&value(a), &value(b))
}

impl ClassAnalyzer<'_> {
    /// The non-private constants `source` makes visible, each with the class-like declaring it.
    fn visible_constants(&self, source: &str) -> Vec<(Arc<str>, Arc<str>)> {
        let here = crate::db::Fqcn::from_str(self.db, source);
        let ancestors = crate::db::class_ancestors_by_fqcn(self.db, here);
        let mut seen = HashSet::default();
        let mut visible = Vec::new();
        for (idx, anc) in ancestors.iter().enumerate() {
            let anc_here = crate::db::Fqcn::from_str(self.db, anc.as_ref());
            let Some(class_like) = crate::db::find_class_like(self.db, anc_here) else {
                continue;
            };
            // A trait's constants belong to the class using it, which precedes it here.
            let declaring = if class_like.is_trait() {
                ancestors[..idx]
                    .iter()
                    .rev()
                    .find(|a| {
                        crate::db::class_kind(self.db, a.as_ref()).is_some_and(|k| !k.is_trait)
                    })
                    .cloned()
                    .unwrap_or_else(|| class_like.fqcn().clone())
            } else {
                class_like.fqcn().clone()
            };
            for (name, constant) in class_like.own_constants().iter() {
                if constant.visibility == Some(Visibility::Private) || !seen.insert(name.clone()) {
                    continue;
                }
                visible.push((name.clone(), declaring.clone()));
            }
        }
        visible
    }

    /// PHP rejects a constant inherited from two class-likes that declare it separately
    /// (before 8.1 also an own constant overriding an interface's).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_ambiguous_constants(
        &self,
        fqcn: &Arc<str>,
        kind: &str,
        own_constants: &mir_codebase::definitions::MemberMap<ConstantDef>,
        parent: Option<&Arc<str>>,
        interfaces: &[Arc<str>],
        location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        let version = crate::db::db_php_version(self.db);
        let before_81 = version < crate::php_version::PhpVersion::new(8, 1);
        let kind = if version < crate::php_version::PhpVersion::new(8, 2) {
            "Class"
        } else {
            kind
        };
        let mut inherited: HashMap<Arc<str>, Arc<str>> = HashMap::default();
        let mut reported = HashSet::default();
        let sources = parent
            .into_iter()
            .map(|p| (p, false))
            .chain(interfaces.iter().map(|i| (i, true)));
        for (source, is_interface) in sources {
            for (name, declaring) in self.visible_constants(source) {
                let message = if own_constants.contains_key(&name) {
                    if !(before_81 && is_interface) {
                        continue;
                    }
                    format!(
                        "Cannot inherit previously-inherited or override constant {name} from interface {declaring}"
                    )
                } else {
                    match inherited.get(&name) {
                        None => {
                            inherited.insert(name, declaring);
                            continue;
                        }
                        Some(first) if first.eq_ignore_ascii_case(&declaring) => continue,
                        Some(_) if before_81 => format!(
                            "Cannot inherit previously-inherited or override constant {name} from interface {declaring}"
                        ),
                        Some(first) => format!(
                            "{kind} {fqcn} inherits both {first}::{name} and {declaring}::{name}, which is ambiguous"
                        ),
                    }
                };
                if !reported.insert(name.clone()) {
                    continue;
                }
                self.push_located_issue(
                    issues,
                    IssueKind::CompileError {
                        message,
                        code: "ambiguous-constant".to_string(),
                    },
                    location,
                );
            }
        }
    }
}

impl ClassAnalyzer<'_> {
    /// PHP requires a concrete class or enum to implement every abstract property hook
    /// it inherits, as it does abstract methods.
    pub(super) fn check_abstract_property_hooks(
        &self,
        fqcn: &Arc<str>,
        is_enum: bool,
        location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        let ancestors = crate::db::class_ancestors_by_fqcn(self.db, here);
        let class_likes: Vec<_> = ancestors
            .iter()
            .filter_map(|a| {
                crate::db::find_class_like(self.db, crate::db::Fqcn::from_str(self.db, a.as_ref()))
            })
            .collect();
        let mut missing = Vec::new();
        let mut seen = HashSet::default();
        for declaring in class_likes.iter().skip(1) {
            let Some(props) = declaring.own_properties() else {
                continue;
            };
            for (name, prop) in props.iter() {
                if prop.abstract_hooks == 0 || prop.from_docblock || !seen.insert(name.clone()) {
                    continue;
                }
                let implementation = class_likes
                    .iter()
                    .filter_map(|c| c.own_properties()?.get(name))
                    .find(|p| !p.from_docblock);
                let provided = match implementation {
                    // A virtual property has only the hooks it declares; a backed one both.
                    Some(p) if p.abstract_hooks == 0 && p.virtual_hooks != 0 => p.virtual_hooks,
                    Some(p) if p.abstract_hooks == 0 => 3,
                    _ => 0,
                };
                let required = class_likes
                    .iter()
                    .filter_map(|c| c.own_properties()?.get(name))
                    .fold(0, |acc, p| acc | p.abstract_hooks);
                for (bit, hook) in [(1, "get"), (2, "set")] {
                    if required & bit != 0 && provided & bit == 0 {
                        let owner = class_likes
                            .iter()
                            .find(|c| {
                                c.own_properties()
                                    .and_then(|ps| ps.get(name))
                                    .is_some_and(|p| p.abstract_hooks & bit != 0)
                            })
                            .map(|c| c.fqcn().clone())
                            .unwrap_or_else(|| declaring.fqcn().clone());
                        missing.push(format!("{owner}::${name}::{hook}"));
                    }
                }
            }
        }
        if missing.is_empty() {
            return;
        }
        let count = missing.len();
        let plural = if count == 1 { "" } else { "s" };
        let list = missing.join(", ");
        // PHP 8.4's texts: a plural "remaining methods", and "private" for enums.
        let php84 = crate::db::db_php_version(self.db) < crate::php_version::PhpVersion::new(8, 5);
        let message = if is_enum {
            let private = if php84 { "private " } else { "" };
            format!("Enum {fqcn} must implement {count} abstract {private}method{plural} ({list})")
        } else {
            let remaining = if php84 { "s" } else { plural };
            format!(
                "Class {fqcn} contains {count} abstract method{plural} and must therefore be declared abstract or implement the remaining method{remaining} ({list})"
            )
        };
        self.push_located_issue(
            issues,
            IssueKind::CompileError {
                message,
                code: "unimplemented-property-hook".to_string(),
            },
            location,
        );
    }
}
