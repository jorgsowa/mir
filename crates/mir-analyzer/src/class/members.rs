use super::*;

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
