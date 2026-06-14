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
                let method_name_lower = method_name.to_lowercase();
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
                } else {
                    // When the implementation is provided by a trait (not the class's own method
                    // or a parent class method), check signature compatibility against the
                    // interface. The regular check_overrides only covers own methods.
                    let class_fqcn_key = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                    if let Some((impl_class, impl_method)) =
                        crate::db::find_method_respecting_precedence(
                            self.db,
                            class_fqcn_key,
                            &method_name_lower,
                        )
                    {
                        if crate::db::class_kind(self.db, impl_class.as_ref())
                            .is_some_and(|k| k.is_trait)
                        {
                            if let Some(iface_method) =
                                crate::db::find_method_in_class(self.db, here, &method_name_lower)
                            {
                                let impl_params = &impl_method.params;
                                let iface_params = &iface_method.params;
                                let impl_required = impl_params
                                    .iter()
                                    .filter(|p| !p.is_optional && !p.is_variadic)
                                    .count();
                                let iface_required = iface_params
                                    .iter()
                                    .filter(|p| !p.is_optional && !p.is_variadic)
                                    .count();
                                let has_variadic = impl_params.iter().any(|p| p.is_variadic);

                                let detail = if !has_variadic
                                    && impl_params.len() < iface_params.len()
                                {
                                    Some(format!(
                                        "method has fewer parameters ({}) than interface {}::{}() ({})",
                                        impl_params.len(),
                                        iface_fqcn,
                                        method_name_lower,
                                        iface_params.len()
                                    ))
                                } else if impl_required > iface_required {
                                    Some(format!(
                                        "overriding method requires {impl_required} argument(s) but interface requires {iface_required}"
                                    ))
                                } else {
                                    None
                                };

                                if let Some(detail) = detail {
                                    let loc = issue_location(
                                        cls_location,
                                        cls_location
                                            .and_then(|l| self.sources.get(&l.file).copied()),
                                    );
                                    let mut issue = Issue::new(
                                        IssueKind::MethodSignatureMismatch {
                                            class: fqcn.to_string(),
                                            method: method_name_lower.to_string(),
                                            detail,
                                        },
                                        loc,
                                    );
                                    if let Some(snippet) =
                                        extract_snippet(cls_location, &self.sources)
                                    {
                                        issue = issue.with_snippet(snippet);
                                    }
                                    issues.push(issue);
                                }
                            }
                        }
                    }
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
        let own_methods: Vec<Arc<mir_codebase::storage::MethodDef>> =
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
