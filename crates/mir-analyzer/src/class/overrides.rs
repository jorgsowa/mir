use super::*;

impl<'a> ClassAnalyzer<'a> {
    pub(super) fn check_overrides(
        &self,
        fqcn: &Arc<str>,
        _cls_location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        let Some(class) = crate::db::find_class_like(self.db, here) else {
            return;
        };
        let own_methods: Vec<(Arc<str>, Arc<mir_codebase::storage::MethodDef>)> = class
            .own_methods()
            .iter()
            .map(|(k, m)| (k.clone(), m.clone()))
            .collect();
        for (_, own) in own_methods {
            let method_name: Arc<str> = own.name.clone();

            // PHP does not enforce constructor signature compatibility
            if method_name.as_ref() == "__construct" {
                continue;
            }

            // Find parent definition (if any) — search ancestor chain
            let method_name_lower: Arc<str> = if method_name.chars().all(|c| !c.is_uppercase()) {
                method_name.clone()
            } else {
                Arc::from(method_name.to_lowercase().as_str())
            };
            // Collect ALL ancestors (skipping self) that define this method.
            // The first one is the "primary parent" for structural checks (final,
            // visibility, static, abstract). All are checked for signature
            // compatibility (return type, param types) so that conflicts across
            // multiple interfaces are caught.
            let all_parent_methods: Vec<(Arc<str>, Arc<mir_codebase::storage::MethodDef>)> =
                crate::db::class_ancestors_by_fqcn(self.db, here)
                    .iter()
                    .skip(1)
                    .filter_map(|anc| {
                        let here2 = crate::db::Fqcn::from_str(self.db, anc.as_ref());
                        crate::db::find_method_in_class(self.db, here2, method_name_lower.as_ref())
                            .map(|m| (anc.clone(), m))
                    })
                    .collect();
            let parent_method = all_parent_methods.first().cloned();

            let own_location = own.location.clone();
            let loc = issue_location(
                own_location.as_ref(),
                own_location
                    .as_ref()
                    .and_then(|l| self.sources.get(&l.file).copied()),
            );

            let (parent_fqcn, parent) = match parent_method {
                Some(m) => m,
                None => {
                    // #[Override] declared but no parent method exists.
                    if own.is_override {
                        let mut issue = Issue::new(
                            IssueKind::InvalidOverride {
                                class: fqcn.to_string(),
                                method: method_name_lower.to_string(),
                                detail: "no parent method exists to override".to_string(),
                            },
                            loc,
                        );
                        if let Some(snippet) = extract_snippet(own_location.as_ref(), &self.sources)
                        {
                            issue = issue.with_snippet(snippet);
                        }
                        issues.push(issue);
                    }
                    continue;
                }
            };

            // #[Override] with a private parent method — private methods are
            // not visible to subclasses and cannot be overridden.
            if own.is_override && parent.visibility == Visibility::Private {
                let mut issue = Issue::new(
                    IssueKind::InvalidOverride {
                        class: fqcn.to_string(),
                        method: method_name_lower.to_string(),
                        detail: format!(
                            "parent method {}::{}() is private",
                            parent_fqcn, method_name_lower
                        ),
                    },
                    loc.clone(),
                );
                if let Some(snippet) = extract_snippet(own_location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }

            // ---- a0. Cannot re-declare a concrete method as abstract --------
            // PHP rejects making a concrete parent method abstract in a subclass.
            // Interface methods are implicitly abstract, so re-declaring them
            // abstract in an abstract class is always legal.
            let parent_is_interface = crate::db::class_kind(self.db, parent_fqcn.as_ref())
                .is_some_and(|k| k.is_interface);
            if own.is_abstract && !parent.is_abstract && !parent_is_interface {
                issues.push(
                    Issue::new(
                        IssueKind::MethodSignatureMismatch {
                            class: fqcn.to_string(),
                            method: method_name_lower.to_string(),
                            detail: format!(
                                "cannot make non-abstract method {}::{}() abstract",
                                parent_fqcn, method_name_lower
                            ),
                        },
                        loc.clone(),
                    )
                    .with_snippet(method_name_lower.to_string()),
                );
            }

            // ---- a. Cannot override a final method -------------------------
            if parent.is_final {
                let mut issue = Issue::new(
                    IssueKind::FinalMethodOverridden {
                        class: fqcn.to_string(),
                        method: method_name_lower.to_string(),
                        parent: parent_fqcn.to_string(),
                    },
                    loc.clone(),
                );
                if let Some(snippet) = extract_snippet(own_location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }

            // ---- b. Static/non-static mismatch --------------------------------
            // A non-static child method cannot override a static parent method
            // and vice versa — PHP treats these as different methods in practice
            // but the static contract is part of the signature.
            if parent.is_static != own.is_static {
                let detail = if parent.is_static {
                    format!(
                        "cannot override static method {}::{}() with a non-static method",
                        parent_fqcn, method_name_lower
                    )
                } else {
                    format!(
                        "cannot override non-static method {}::{}() with a static method",
                        parent_fqcn, method_name_lower
                    )
                };
                let mut issue = Issue::new(
                    IssueKind::MethodSignatureMismatch {
                        class: fqcn.to_string(),
                        method: method_name_lower.to_string(),
                        detail,
                    },
                    loc.clone(),
                );
                if let Some(snippet) = extract_snippet(own_location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }

            // ---- c. Visibility must not be reduced -------------------------
            if visibility_reduced(own.visibility, parent.visibility) {
                let mut issue = Issue::new(
                    IssueKind::OverriddenMethodAccess {
                        class: fqcn.to_string(),
                        method: method_name_lower.to_string(),
                    },
                    loc.clone(),
                );
                if let Some(snippet) = extract_snippet(own_location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }

            // ---- c. Return type must be covariant (check ALL ancestors) -----
            // Check every ancestor that defines this method: a class implementing
            // two interfaces with conflicting return types must be flagged even if
            // it satisfies the first interface's contract.
            let own_return_type = own.return_type.as_deref().cloned();
            if let Some(child_ret) = own_return_type.as_ref() {
                let child_file = own_location.as_ref().map(|l| l.file.as_ref()).unwrap_or("");
                for (idx, (p_fqcn, p)) in all_parent_methods.iter().enumerate() {
                    let Some(parent_ret) = p.return_type.as_deref() else {
                        continue;
                    };
                    if parent_ret.from_docblock
                        || parent_ret.is_mixed()
                        || child_ret.is_mixed()
                        || self.return_type_has_template(parent_ret)
                    {
                        continue;
                    }
                    let child_has_object = Self::type_has_named_objects(child_ret)
                        || self.type_has_self_or_static(child_ret);
                    let parent_has_object = Self::type_has_named_objects(parent_ret)
                        || self.type_has_self_or_static(parent_ret);
                    let compatible = if child_has_object && parent_has_object {
                        // Both sides involve objects: named_object_return_compatible now
                        // splits mixed object+scalar unions per atom (G5), so it covers
                        // `string|Cat` vs `string|Animal` directly — not just purely-object
                        // unions as before.
                        crate::stmt::named_object_return_compatible(
                            child_ret, parent_ret, self.db, child_file,
                        )
                    } else if child_has_object || parent_has_object {
                        // Object vs. disjoint scalar (e.g. stdClass vs int): handled by the
                        // ImplementedReturnTypeMismatch check, so skip here to avoid a
                        // duplicate diagnostic.
                        true
                    } else {
                        Self::scalar_return_types_compatible(child_ret, parent_ret)
                    };
                    if !compatible {
                        // Primary parent uses the original message format for
                        // backwards-compatibility with existing fixtures. Additional
                        // ancestors include the declaring class to clarify which
                        // contract is violated.
                        let detail = if idx == 0 {
                            format!(
                                "return type '{child_ret}' is not a subtype of parent '{parent_ret}'"
                            )
                        } else {
                            format!(
                                "return type '{child_ret}' is not a subtype of {p_fqcn}::{}() '{parent_ret}'",
                                method_name_lower
                            )
                        };
                        issues.push(
                            Issue::new(
                                IssueKind::MethodSignatureMismatch {
                                    class: fqcn.to_string(),
                                    method: method_name_lower.to_string(),
                                    detail,
                                },
                                loc.clone(),
                            )
                            .with_snippet(method_name_lower.to_string()),
                        );
                    }
                }
            }

            // ---- d. Required param count must not increase -----------------
            let parent_params = parent.params.clone();
            let own_params = own.params.clone();
            let parent_required = parent_params
                .iter()
                .filter(|p| !p.is_optional && !p.is_variadic)
                .count();
            let child_required = own_params
                .iter()
                .filter(|p| !p.is_optional && !p.is_variadic)
                .count();

            if child_required > parent_required {
                issues.push(
                    Issue::new(
                        IssueKind::MethodSignatureMismatch {
                            class: fqcn.to_string(),
                            method: method_name_lower.to_string(),
                            detail: format!(
                                "overriding method requires {child_required} argument(s) but parent requires {parent_required}"
                            ),
                        },
                        loc.clone(),
                    )
                    .with_snippet(method_name_lower.to_string()),
                );
            }

            // ---- d2. Child must not declare fewer parameters than parent -----
            // A child accepting fewer positional params cannot handle every call
            // the parent could (an LSP violation PHP rejects). A trailing
            // variadic absorbs the extras, so it is exempt. Constructors are
            // exempt from signature compatibility in PHP, and private parent
            // methods are not real overrides.
            if method_name_lower.as_ref() != "__construct"
                && parent.visibility != Visibility::Private
                && own_params.len() < parent_params.len()
                && !own_params.iter().any(|p| p.is_variadic)
            {
                issues.push(
                    Issue::new(
                        IssueKind::MethodSignatureMismatch {
                            class: fqcn.to_string(),
                            method: method_name_lower.to_string(),
                            detail: format!(
                                "method has fewer parameters ({}) than parent {}::{}() ({})",
                                own_params.len(),
                                parent_fqcn,
                                method_name_lower,
                                parent_params.len()
                            ),
                        },
                        loc.clone(),
                    )
                    .with_snippet(method_name_lower.to_string()),
                );
            }

            // ---- d3. by-reference-ness of shared params must match -----------
            // A parameter that is by-value in the parent but by-reference in the
            // child (or vice versa) changes the calling contract — PHP rejects
            // the override. Constructors are exempt.
            if method_name_lower.as_ref() != "__construct" {
                let shared = parent_params.len().min(own_params.len());
                if let Some(i) =
                    (0..shared).find(|&i| parent_params[i].is_byref != own_params[i].is_byref)
                {
                    issues.push(
                        Issue::new(
                            IssueKind::MethodSignatureMismatch {
                                class: fqcn.to_string(),
                                method: method_name_lower.to_string(),
                                detail: format!(
                                    "parameter ${} must {}be passed by reference to match parent {}::{}()",
                                    own_params[i].name.as_ref().trim_start_matches('$'),
                                    if parent_params[i].is_byref { "" } else { "not " },
                                    parent_fqcn,
                                    method_name_lower
                                ),
                            },
                            loc.clone(),
                        )
                        .with_snippet(method_name_lower.to_string()),
                    );
                }
            }

            // ---- e. Param types must not be narrowed (contravariance) --------
            // For each positional param present in both parent and child:
            //   parent_param_type must be a subtype of child_param_type.
            //   (Child may widen; it must not narrow.)
            // Skip when:
            //   - Either side has no type hint
            //   - Either type is mixed
            //   - Either type contains TSelf/TStaticObject (late-static semantics)
            //   - Either type contains a template param
            let shared_len = parent_params.len().min(own_params.len());
            for i in 0..shared_len {
                let parent_param = &parent_params[i];
                let child_param = &own_params[i];

                let (parent_ty, child_ty) = match (&parent_param.ty, &child_param.ty) {
                    (Some(p), Some(c)) => (p, c),
                    _ => continue,
                };

                if parent_ty.is_mixed()
                    || child_ty.is_mixed()
                    || self.type_has_self_or_static(parent_ty)
                    || self.type_has_self_or_static(child_ty)
                    || self.return_type_has_template(parent_ty)
                    || self.return_type_has_template(child_ty)
                {
                    continue;
                }

                // Object (or mixed object+scalar) params resolve narrowing through
                // the codebase inheritance graph: a contravariance violation is when
                // the parent type is NOT a subtype of the child type (the child
                // accepts strictly fewer values than the parent contract). We only
                // decide this when every named class involved is known to the
                // codebase — an unknown class would make `is_subtype` falsely report
                // narrowing. Pure-scalar params keep the structural check. (G4)
                let involves_objects = Self::type_has_named_objects(parent_ty)
                    || Self::type_has_named_objects(child_ty);
                let narrowed = if involves_objects {
                    // Parameter contravariance is a native-signature concept: PHP only
                    // enforces it on declared type hints. A docblock `@param` that
                    // narrows to subclasses (native hint unchanged) is an intentional
                    // refinement, not an LSP violation — so only compare native hints.
                    if !parent_ty.from_docblock
                        && !child_ty.from_docblock
                        && self.all_object_classes_known(parent_ty)
                        && self.all_object_classes_known(child_ty)
                    {
                        !crate::subtype::is_subtype(self.db, parent_ty, child_ty)
                    } else {
                        false
                    }
                } else {
                    Self::scalar_param_type_narrowed(parent_ty, child_ty)
                };

                if narrowed {
                    issues.push(
                        Issue::new(
                            IssueKind::MethodSignatureMismatch {
                                class: fqcn.to_string(),
                                method: method_name_lower.to_string(),
                                detail: format!(
                                    "parameter ${} type '{}' is narrower than parent type '{}'",
                                    child_param.name, child_ty, parent_ty
                                ),
                            },
                            loc.clone(),
                        )
                        .with_snippet(method_name_lower.to_string()),
                    );
                    break; // one issue per method is enough
                }
            }
        }

        // ---- Property visibility must not be reduced -------------------------
        let own_properties: Vec<(Arc<str>, mir_codebase::storage::PropertyDef)> = class
            .own_properties()
            .map(|props| props.iter().map(|(k, p)| (k.clone(), p.clone())).collect())
            .unwrap_or_default();
        for (_, own_prop) in own_properties {
            let prop_name = own_prop.name.clone();
            // Look up the same property name in ancestors (skip self = first entry)
            let parent_prop = crate::db::class_ancestors_by_fqcn(self.db, here)
                .iter()
                .skip(1)
                .find_map(|anc| {
                    let anc_here = crate::db::Fqcn::from_str(self.db, anc.as_ref());
                    crate::db::find_property_in_class(self.db, anc_here, prop_name.as_ref())
                        .map(|p| (anc.clone(), p))
                });
            let Some((_parent_fqcn, parent_prop)) = parent_prop else {
                continue;
            };
            if visibility_reduced(own_prop.visibility, parent_prop.visibility) {
                let loc = issue_location(
                    own_prop.location.as_ref(),
                    own_prop
                        .location
                        .as_ref()
                        .and_then(|l| self.sources.get(&l.file).copied()),
                );
                let mut issue = Issue::new(
                    IssueKind::OverriddenPropertyAccess {
                        class: fqcn.to_string(),
                        property: prop_name.to_string(),
                    },
                    loc,
                );
                if let Some(snippet) = extract_snippet(own_prop.location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Returns true if the type contains template params or class-strings with unknown types.
    /// Used to suppress MethodSignatureMismatch on generic parent return types.
    /// Checks recursively into array key/value types.
    fn return_type_has_template(&self, ty: &mir_types::Type) -> bool {
        use mir_types::Atomic;
        ty.types.iter().any(|atomic| match atomic {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TClassString(Some(inner)) => !crate::db::class_exists(self.db, inner.as_ref()),
            Atomic::TNamedObject { fqcn, type_params } => {
                // Bare name with no namespace separator is likely a template param
                (!fqcn.contains('\\') && !crate::db::class_exists(self.db, fqcn.as_ref()))
                    // Also check if any type params are templates
                    || type_params.iter().any(|tp| self.return_type_has_template(tp))
            }
            Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
                self.return_type_has_template(key) || self.return_type_has_template(value)
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                self.return_type_has_template(value)
            }
            _ => false,
        })
    }

    /// Returns true if the type contains any named-object atomics (TNamedObject)
    /// at any level (including inside array key/value types).
    /// Named-object subtyping requires codebase inheritance lookup, so we skip
    /// the simple structural check for these.
    fn type_has_named_objects(ty: &mir_types::Type) -> bool {
        use mir_types::Atomic;
        ty.types.iter().any(|a| match a {
            Atomic::TNamedObject { .. } => true,
            Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
                Self::type_has_named_objects(key) || Self::type_has_named_objects(value)
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                Self::type_has_named_objects(value)
            }
            _ => false,
        })
    }

    /// Returns true if every named-object class referenced anywhere in `ty`
    /// (including inside generic type arguments and array key/value types) is known
    /// to the codebase. Used to gate object-aware param contravariance (G4): when an
    /// involved class is unknown, `is_subtype` cannot resolve its hierarchy and would
    /// falsely report narrowing, so the check is skipped instead.
    fn all_object_classes_known(&self, ty: &mir_types::Type) -> bool {
        use mir_types::Atomic;
        ty.types.iter().all(|a| match a {
            Atomic::TNamedObject { fqcn, type_params } => {
                crate::db::class_exists(self.db, fqcn.as_ref())
                    && type_params.iter().all(|p| self.all_object_classes_known(p))
            }
            Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
                self.all_object_classes_known(key) && self.all_object_classes_known(value)
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                self.all_object_classes_known(value)
            }
            _ => true,
        })
    }

    /// Returns true if the type contains TSelf or TStaticObject (late-static types).
    /// These are always considered compatible with their bound class type.
    fn type_has_self_or_static(&self, ty: &mir_types::Type) -> bool {
        use mir_types::Atomic;
        ty.types
            .iter()
            .any(|a| matches!(a, Atomic::TSelf { .. } | Atomic::TStaticObject { .. }))
    }

    // -----------------------------------------------------------------------
    // Check: circular class inheritance (class A extends B extends A)
}
