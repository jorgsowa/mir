use super::*;

impl<'a> ClassAnalyzer<'a> {
    pub(super) fn check_abstract_methods_implemented(
        &self,
        fqcn: &Arc<str>,
        cls_location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        // Walk every ancestor class and collect abstract methods
        let ancestors = self.ancestors(fqcn);
        for ancestor_fqcn in &ancestors {
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
        let all_ifaces: Vec<Arc<str>> = self
            .ancestors(fqcn)
            .into_iter()
            .filter(|p| crate::db::class_kind(self.db, p.as_ref()).is_some_and(|k| k.is_interface))
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
                let implemented =
                    crate::db::is_method_concretely_implemented(self.db, enum_fqcn.as_ref(), &lower);

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
        !parent_ty.is_subtype_structural(child_ty)
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
