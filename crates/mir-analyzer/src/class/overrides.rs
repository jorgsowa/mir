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
        let own_methods: Vec<(Arc<str>, Arc<mir_codebase::definitions::MethodDef>)> = class
            .own_methods()
            .iter()
            .map(|(k, m)| (k.clone(), m.clone()))
            .collect();

        // Members composed via `use Trait;` behave like "own" methods for override
        // purposes: this class inherits the trait's *implementation*, so a conflict
        // against the real parent/interfaces must be checked even when the class
        // body never redeclares the method — previously only literally-declared
        // `own_methods()` were checked, making trait-composed overrides invisible to
        // every check below (final/static/visibility/return/param).
        //
        // `class_ancestors_by_fqcn` DFS-visits the FULL transitive trait subtree
        // before ever reaching the real parent/interfaces (traits are always
        // ordered first in `ancestor_fqcns`), so the contiguous run of Trait-kind
        // entries immediately after `here` is exactly this class's own composed
        // traits — a parent class's own trait-contributed methods are correctly
        // left alone here (they're checked by that parent's own pass instead).
        let mut seen_method_keys: HashSet<Arc<str>> =
            own_methods.iter().map(|(k, _)| k.clone()).collect();
        let mut trait_composed_methods: Vec<(Arc<str>, Arc<mir_codebase::definitions::MethodDef>)> =
            Vec::new();
        for anc in crate::db::class_ancestors_by_fqcn(self.db, here)
            .iter()
            .skip(1)
        {
            let anc_here = crate::db::Fqcn::from_str(self.db, anc.as_ref());
            let Some(crate::db::ClassLike::Trait(t)) =
                crate::db::find_class_like(self.db, anc_here)
            else {
                break;
            };
            for key in t.own_methods.iter().map(|(k, _)| k.clone()) {
                if !seen_method_keys.insert(key.clone()) {
                    continue;
                }
                // Resolve through the precedence-aware walker (not a plain lookup
                // on this one trait) so `insteadof`/`as` conflicts between two
                // composed traits pick the actual winning copy.
                if let Some((owner, m)) =
                    crate::db::find_method_respecting_precedence(self.db, here, key.as_ref())
                {
                    if crate::db::class_kind(self.db, owner.as_ref()).is_some_and(|k| k.is_trait) {
                        // `self`/`static` inside a trait method are bound to the
                        // trait's own FQCN in its declaration, but PHP resolves both
                        // to the *composing* class at the use site — rebind before
                        // treating this signature as the composing class's "own",
                        // otherwise e.g. a `static` return type is compared as
                        // `static(Trait)` against the parent's `static(Parent)` and
                        // spuriously fails covariance.
                        let rebound = Self::rebind_self_static_in_method(&m, fqcn);
                        trait_composed_methods.push((key, rebound));
                    }
                }
            }
        }
        let own_methods = own_methods
            .into_iter()
            .chain(trait_composed_methods)
            .collect::<Vec<_>>();

        // What this class's own `@extends`/`@implements` chain binds an ancestor's
        // template params to (e.g. `@extends Box<int>` -> T => int). Lets an ancestor
        // method's still-templated param/return type be checked against a concrete
        // type instead of being skipped outright just because it mentions a template.
        let inherited_bindings =
            crate::db::inherited_template_bindings(self.db, fqcn.as_ref(), &HashMap::default());

        // `insteadof` exclusions declared by ANY class in the ancestor chain
        // (e.g. `use T1, T2 { T2::f insteadof T1; }`) — collected once so the
        // per-method ancestor walk below can skip a trait's LOSING copy of a
        // method instead of treating it as a real "parent" to check against.
        let mut excluded_trait_methods: HashSet<(Arc<str>, Arc<str>)> = HashSet::default();
        for anc in crate::db::class_ancestors_by_fqcn(self.db, here).iter() {
            let anc_fqcn = crate::db::Fqcn::from_str(self.db, anc.as_ref());
            if let Some(crate::db::ClassLike::Class(cls)) =
                crate::db::find_class_like(self.db, anc_fqcn)
            {
                for (method_lower, losers) in cls.trait_insteadof.iter() {
                    for loser in losers {
                        excluded_trait_methods.insert((loser.clone(), method_lower.clone()));
                    }
                }
            }
        }

        for (_, own) in own_methods {
            let method_name: Arc<str> = own.name.clone();

            // PHP does not enforce constructor signature compatibility
            if method_name.as_ref() == "__construct" {
                continue;
            }

            // Find parent definition (if any) — search ancestor chain
            let method_name_lower: Arc<str> = if method_name.bytes().any(|b| b.is_ascii_uppercase())
            {
                Arc::from(crate::util::php_ident_lowercase(&method_name).as_str())
            } else {
                method_name.clone()
            };
            // Collect ALL ancestors (skipping self) that define this method.
            // The first one is the "primary parent" for structural checks (final,
            // visibility, static, abstract). All are checked for signature
            // compatibility (return type, param types) so that conflicts across
            // multiple interfaces are caught.
            let all_parent_methods: Vec<(Arc<str>, Arc<mir_codebase::definitions::MethodDef>)> =
                crate::db::class_ancestors_by_fqcn(self.db, here)
                    .iter()
                    .skip(1)
                    .filter_map(|anc| {
                        // A trait excluded via `insteadof` for this method
                        // never contributes its own copy — it lost precedence
                        // to another trait's version.
                        if excluded_trait_methods
                            .contains(&(anc.clone(), method_name_lower.clone()))
                        {
                            return None;
                        }
                        let here2 = crate::db::Fqcn::from_str(self.db, anc.as_ref());
                        if let Some(m) = crate::db::find_method_in_class(
                            self.db,
                            here2,
                            method_name_lower.as_ref(),
                        ) {
                            return Some((anc.clone(), m));
                        }
                        // Trait method alias (`use T { orig as alias; }`): the
                        // alias name is invisible to a plain own-methods
                        // lookup on the using class, so resolve it explicitly
                        // — mirrors the alias handling
                        // `find_method_respecting_precedence` already does
                        // for normal calls, which this ancestor walk
                        // otherwise never consults.
                        let crate::db::ClassLike::Class(cls) =
                            crate::db::find_class_like(self.db, here2)?
                        else {
                            return None;
                        };
                        let (opt_trait_fqcn, orig_method, vis_override, alias_cased) =
                            cls.trait_aliases.get(method_name_lower.as_ref())?;
                        let search_traits: Vec<Arc<str>> = match opt_trait_fqcn {
                            Some(t) => vec![t.clone()],
                            None => cls.traits.clone(),
                        };
                        for trait_fqcn in &search_traits {
                            let trait_here =
                                crate::db::Fqcn::from_str(self.db, trait_fqcn.as_ref());
                            if let Some(m) = crate::db::find_method_in_class(
                                self.db,
                                trait_here,
                                orig_method.as_ref(),
                            ) {
                                let mut m_clone = (*m).clone();
                                m_clone.name = alias_cased.clone();
                                if let Some(vis) = vis_override {
                                    m_clone.visibility = *vis;
                                }
                                return Some((anc.clone(), Arc::new(m_clone)));
                            }
                        }
                        None
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
            // abstract in an abstract class is always legal. A trait's abstract
            // method is exempt too — confirmed live: unlike a class body directly
            // re-declaring a parent method abstract (always a fatal, regardless
            // of signature), a trait's abstract requirement against an inherited
            // concrete method is only ever rejected for a genuine signature
            // mismatch, which the return-type/param checks below already catch.
            //
            // These structural checks (a0/a/b/c) scan ALL ancestors, not just
            // the first, for the same reason the return-type/param loops below
            // do: traits are always ordered before the real parent class, so a
            // trait's compatible copy of a method must not shadow a genuine
            // conflict against the parent (or an interface) further down the
            // chain.
            let is_body_declared = class.own_methods().contains_key(method_name_lower.as_ref());
            if own.is_abstract && is_body_declared {
                if let Some((parent_fqcn, _)) = all_parent_methods.iter().find(|(pf, p)| {
                    !p.is_abstract
                        && !crate::db::class_kind(self.db, pf.as_ref())
                            .is_some_and(|k| k.is_interface)
                }) {
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
            }

            // ---- a. Cannot override a final method -------------------------
            if let Some((final_parent_fqcn, _)) =
                all_parent_methods.iter().find(|(_, p)| p.is_final)
            {
                let mut issue = Issue::new(
                    IssueKind::FinalMethodOverridden {
                        class: fqcn.to_string(),
                        method: method_name_lower.to_string(),
                        parent: final_parent_fqcn.to_string(),
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
            if let Some((static_parent_fqcn, static_parent)) = all_parent_methods
                .iter()
                .find(|(_, p)| p.is_static != own.is_static)
            {
                let detail = if static_parent.is_static {
                    format!(
                        "cannot override static method {}::{}() with a non-static method",
                        static_parent_fqcn, method_name_lower
                    )
                } else {
                    format!(
                        "cannot override non-static method {}::{}() with a static method",
                        static_parent_fqcn, method_name_lower
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
            if all_parent_methods
                .iter()
                .any(|(_, p)| visibility_reduced(own.visibility, p.visibility))
            {
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
            if let Some(child_ret_raw) = own_return_type.as_ref() {
                // A child override can itself just repeat the ancestor's bare template
                // (e.g. copying `@return T` instead of the interface's own bound type) —
                // substitute the same inherited bindings on both sides so the comparison
                // isn't a false mismatch between "T" and what T concretely resolves to.
                let child_ret = if self.return_type_has_template(child_ret_raw) {
                    child_ret_raw.substitute_templates(&inherited_bindings)
                } else {
                    child_ret_raw.clone()
                };
                let child_ret = &child_ret;
                let child_file = own_location.as_ref().map(|l| l.file.as_ref()).unwrap_or("");
                for (idx, (p_fqcn, p)) in all_parent_methods.iter().enumerate() {
                    let Some(parent_ret_raw) = p.return_type.as_deref() else {
                        continue;
                    };
                    // Substitute this class's own inherited bindings (e.g. `@extends
                    // Box<int>` -> T => int) before deciding whether the ancestor's
                    // return type is still an unresolved template — a docblock-only
                    // return type is normally an intentional, unenforced refinement,
                    // but a generic contract this class itself concretely bound is not.
                    let had_template = self.return_type_has_template(parent_ret_raw);
                    let parent_ret = if had_template {
                        parent_ret_raw.substitute_templates(&inherited_bindings)
                    } else {
                        parent_ret_raw.clone()
                    };
                    let parent_ret = &parent_ret;
                    if (parent_ret_raw.from_docblock && !had_template)
                        || parent_ret.is_mixed()
                        || child_ret.is_mixed()
                        || self.return_type_has_template(parent_ret)
                        || self.return_type_has_template(child_ret)
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
                        // unions as before. An array-of-named-object return
                        // (`array<int, T>`) falls through named_object_return_compatible's
                        // structural catch-all (no class-hierarchy awareness for arrays),
                        // so also try the codebase-aware array check the ordinary
                        // return-statement checker already uses for exactly this shape.
                        crate::stmt::named_object_return_compatible(
                            child_ret, parent_ret, self.db, child_file,
                        ) || crate::stmt::return_arrays_compatible(
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

            // ---- d/d2/d3/e. Param-side checks against EVERY ancestor --------
            // Like section c (return type), a param-side LSP violation must be
            // caught against ANY ancestor that defines this method, not just
            // the "primary" one — otherwise a class implementing two
            // interfaces with conflicting param contracts is only checked
            // against whichever interface happens to be listed first.
            let own_params = own.params.clone();
            // Two ancestors (e.g. a trait and an interface implemented by the
            // same class, both declaring that trait's method) commonly share
            // byte-identical signatures — dedup by the STRUCTURAL shape of
            // the violation (excluding which ancestor triggered it) so that
            // case still reports once, using the first ancestor's wording
            // (matching pre-existing fixtures), while genuinely differing
            // per-ancestor contracts (the diamond this loop exists to catch)
            // still each get their own diagnostic.
            let mut seen_count_violation: HashSet<(usize, usize)> = HashSet::default();
            let mut seen_fewer_params: HashSet<(usize, usize)> = HashSet::default();
            let mut seen_byref_violation: HashSet<(usize, bool)> = HashSet::default();
            let mut seen_narrowing: HashSet<(usize, String, String)> = HashSet::default();
            for (anc_fqcn, anc_parent) in all_parent_methods.iter() {
                let parent_params = anc_parent.params.clone();

                // ---- d. Required param count must not increase -------------
                let parent_required = parent_params
                    .iter()
                    .filter(|p| !p.is_optional && !p.is_variadic)
                    .count();
                let child_required = own_params
                    .iter()
                    .filter(|p| !p.is_optional && !p.is_variadic)
                    .count();

                if child_required > parent_required
                    && seen_count_violation.insert((child_required, parent_required))
                {
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

                // ---- d2. Child must not declare fewer parameters than parent -
                // A child accepting fewer positional params cannot handle every
                // call the parent could (an LSP violation PHP rejects). A
                // trailing variadic absorbs the extras, so it is exempt.
                // Constructors are exempt from signature compatibility in PHP,
                // and private parent methods are not real overrides.
                if method_name_lower.as_ref() != "__construct"
                    && anc_parent.visibility != Visibility::Private
                    && own_params.len() < parent_params.len()
                    && !own_params.iter().any(|p| p.is_variadic)
                    && seen_fewer_params.insert((own_params.len(), parent_params.len()))
                {
                    issues.push(
                        Issue::new(
                            IssueKind::MethodSignatureMismatch {
                                class: fqcn.to_string(),
                                method: method_name_lower.to_string(),
                                detail: format!(
                                    "method has fewer parameters ({}) than parent {}::{}() ({})",
                                    own_params.len(),
                                    anc_fqcn,
                                    method_name_lower,
                                    parent_params.len()
                                ),
                            },
                            loc.clone(),
                        )
                        .with_snippet(method_name_lower.to_string()),
                    );
                }

                // ---- d3. by-reference-ness of shared params must match ------
                // A parameter that is by-value in the parent but by-reference
                // in the child (or vice versa) changes the calling contract —
                // PHP rejects the override. Constructors are exempt.
                if method_name_lower.as_ref() != "__construct" {
                    let shared = parent_params.len().min(own_params.len());
                    if let Some(i) =
                        (0..shared).find(|&i| parent_params[i].is_byref != own_params[i].is_byref)
                    {
                        if seen_byref_violation.insert((i, parent_params[i].is_byref)) {
                            issues.push(
                                Issue::new(
                                    IssueKind::MethodSignatureMismatch {
                                        class: fqcn.to_string(),
                                        method: method_name_lower.to_string(),
                                        detail: format!(
                                            "parameter ${} must {}be passed by reference to match parent {}::{}()",
                                            own_params[i].name.as_ref().trim_start_matches('$'),
                                            if parent_params[i].is_byref { "" } else { "not " },
                                            anc_fqcn,
                                            method_name_lower
                                        ),
                                    },
                                    loc.clone(),
                                )
                                .with_snippet(method_name_lower.to_string()),
                            );
                        }
                    }
                }

                // ---- e. Param types must not be narrowed (contravariance) ---
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

                    let (parent_ty_raw, child_ty_raw) = match (&parent_param.ty, &child_param.ty) {
                        (Some(p), Some(c)) => (p, c),
                        _ => continue,
                    };

                    // As with return types: substitute this class's own inherited bindings
                    // before giving up on a still-templated param type, so a concretely
                    // bound generic contract (`@extends Box<int>`) is still checked.
                    let parent_had_template = self.return_type_has_template(parent_ty_raw);
                    let parent_ty = parent_ty_raw.substitute_templates(&inherited_bindings);
                    let child_ty = child_ty_raw.substitute_templates(&inherited_bindings);
                    let parent_ty = &parent_ty;
                    let child_ty = &child_ty;

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
                        // refinement, not an LSP violation — so ordinarily only compare
                        // native hints. But when the parent's own type is a generic
                        // contract (`@param T`), this class's `@extends`/`@implements`
                        // fixed T to a concrete type, not something the child author
                        // discretionarily narrowed — so the child's concrete type (however
                        // it's declared) must still honor it, mirroring the return-type
                        // covariance carve-out above.
                        let checkable = parent_had_template
                            || (!parent_ty.from_docblock && !child_ty.from_docblock);
                        if checkable
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
                        if seen_narrowing.insert((i, child_ty.to_string(), parent_ty.to_string())) {
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
                        }
                        break; // one issue per (method, ancestor) is enough
                    }
                }
            }
        }

        // ---- Property visibility must not be reduced -------------------------
        // Same trait-composition gap as methods above: a property declared only
        // in a `use`d trait is otherwise invisible to every check in this loop.
        let mut own_properties: Vec<(Arc<str>, mir_codebase::definitions::PropertyDef)> = class
            .own_properties()
            .map(|props| props.iter().map(|(k, p)| (k.clone(), p.clone())).collect())
            .unwrap_or_default();
        let mut seen_prop_keys: HashSet<Arc<str>> =
            own_properties.iter().map(|(k, _)| k.clone()).collect();
        for anc in crate::db::class_ancestors_by_fqcn(self.db, here)
            .iter()
            .skip(1)
        {
            let anc_here = crate::db::Fqcn::from_str(self.db, anc.as_ref());
            let Some(crate::db::ClassLike::Trait(t)) =
                crate::db::find_class_like(self.db, anc_here)
            else {
                break;
            };
            for (key, prop) in t.own_properties.iter() {
                if seen_prop_keys.insert(key.clone()) {
                    own_properties.push((key.clone(), prop.clone()));
                }
            }
        }
        for (_, own_prop) in own_properties {
            let prop_name = own_prop.name.clone();
            // Look up the same property name in ancestors, skipping self (first
            // entry) AND this class's own composed-trait prefix — a trait that
            // contributes `own_prop` itself is not a "parent" to compare against,
            // it's where `own_prop` came from (mirrors the method loop's parent
            // walk, which naturally skips past a self-match by looking for a
            // *violating* ancestor rather than just the first same-named one).
            let parent_prop = crate::db::class_ancestors_by_fqcn(self.db, here)
                .iter()
                .skip(1)
                .skip_while(|anc| {
                    let anc_here = crate::db::Fqcn::from_str(self.db, anc.as_ref());
                    matches!(
                        crate::db::find_class_like(self.db, anc_here),
                        Some(crate::db::ClassLike::Trait(_))
                    )
                })
                .find_map(|anc| {
                    let anc_here = crate::db::Fqcn::from_str(self.db, anc.as_ref());
                    crate::db::find_property_in_class(self.db, anc_here, prop_name.as_ref())
                        .map(|p| (anc.clone(), p))
                });
            let Some((parent_fqcn, parent_prop)) = parent_prop else {
                continue;
            };
            // Only enforce visibility rules against real PHP properties.
            // Magic @property/@property-read/@property-write entries (from_docblock=true)
            // are not runtime properties and establish no visibility contract.
            if !parent_prop.from_docblock
                && visibility_reduced(own_prop.visibility, parent_prop.visibility)
            {
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
            // PHP fatal-errors when a redeclared property flips native `readonly`-ness in
            // either direction. Only real PHP properties carry this contract — `@readonly`
            // is advisory and not runtime-enforced, so skip docblock-only entries.
            if !own_prop.from_docblock
                && !parent_prop.from_docblock
                && own_prop.has_native_readonly != parent_prop.has_native_readonly
            {
                let loc = issue_location(
                    own_prop.location.as_ref(),
                    own_prop
                        .location
                        .as_ref()
                        .and_then(|l| self.sources.get(&l.file).copied()),
                );
                let mut issue = Issue::new(
                    IssueKind::ReadonlyPropertyRedeclarationMismatch {
                        parent_class: parent_fqcn.to_string(),
                        class: fqcn.to_string(),
                        property: prop_name.to_string(),
                        parent_readonly: parent_prop.has_native_readonly,
                    },
                    loc,
                );
                if let Some(snippet) = extract_snippet(own_prop.location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }
            // PHP fatal-errors when a redeclared property flips `static`-ness in either
            // direction ("Cannot redeclare static X::$y as non static Y::$y" and vice
            // versa). Only real PHP properties carry this contract — `@property` docblock
            // entries are virtual (no runtime static/instance distinction), so skip
            // docblock-only entries just like the readonly check above.
            if !own_prop.from_docblock
                && !parent_prop.from_docblock
                && own_prop.is_static != parent_prop.is_static
            {
                let loc = issue_location(
                    own_prop.location.as_ref(),
                    own_prop
                        .location
                        .as_ref()
                        .and_then(|l| self.sources.get(&l.file).copied()),
                );
                let mut issue = Issue::new(
                    IssueKind::StaticPropertyRedeclarationMismatch {
                        parent_class: parent_fqcn.to_string(),
                        class: fqcn.to_string(),
                        property: prop_name.to_string(),
                        parent_static: parent_prop.is_static,
                    },
                    loc,
                );
                if let Some(snippet) = extract_snippet(own_prop.location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }
            // PHP requires redeclared typed properties to keep the same type (invariant).
            // Only flag when both sides carry a native type hint — docblock-only types are
            // not enforced by the runtime. Compare `native_ty`, not `ty`: `ty` folds in any
            // `@var` docblock refinement, which PHP's redeclaration rule never checks.
            if own_prop.has_native_type && parent_prop.has_native_type {
                if let (Some(own_t), Some(parent_t)) = (
                    own_prop.native_ty.as_deref(),
                    parent_prop.native_ty.as_deref(),
                ) {
                    let same_type = own_t.is_subtype_structural(parent_t)
                        && parent_t.is_subtype_structural(own_t);
                    if !same_type {
                        let loc = issue_location(
                            own_prop.location.as_ref(),
                            own_prop
                                .location
                                .as_ref()
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::PropertyTypeRedeclarationMismatch {
                                class: fqcn.to_string(),
                                property: prop_name.to_string(),
                                expected: format!("{}", parent_t),
                                actual: format!("{}", own_t),
                            },
                            loc,
                        );
                        if let Some(snippet) =
                            extract_snippet(own_prop.location.as_ref(), &self.sources)
                        {
                            issue = issue.with_snippet(snippet);
                        }
                        issues.push(issue);
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Rebinds any `self`/`static` atomic in `ty` from wherever it was declared
    /// to `to_fqcn` — PHP resolves both to the class actually using a trait
    /// method, not the trait's own declaring FQCN.
    fn rebind_self_static(ty: &mir_types::Type, to_fqcn: &Arc<str>) -> mir_types::Type {
        use mir_types::Atomic;
        if !ty
            .types
            .iter()
            .any(|a| matches!(a, Atomic::TSelf { .. } | Atomic::TStaticObject { .. }))
        {
            return ty.clone();
        }
        let mut result = mir_types::Type::empty();
        result.possibly_undefined = ty.possibly_undefined;
        result.from_docblock = ty.from_docblock;
        for atomic in &ty.types {
            let rebound = match atomic {
                Atomic::TSelf { .. } => Atomic::TSelf {
                    fqcn: mir_types::Name::new(to_fqcn.as_ref()),
                },
                Atomic::TStaticObject { .. } => Atomic::TStaticObject {
                    fqcn: mir_types::Name::new(to_fqcn.as_ref()),
                },
                other => other.clone(),
            };
            result.add_type(rebound);
        }
        result
    }

    /// Applies [`rebind_self_static`] to a trait method's params/return type
    /// before it's checked as if it were the composing class's own method.
    fn rebind_self_static_in_method(
        m: &Arc<mir_codebase::definitions::MethodDef>,
        to_fqcn: &Arc<str>,
    ) -> Arc<mir_codebase::definitions::MethodDef> {
        let needs_rebind = m
            .return_type
            .as_deref()
            .is_some_and(Self::type_has_self_or_static_atomic)
            || m.params.iter().any(|p| {
                p.ty.as_deref()
                    .is_some_and(Self::type_has_self_or_static_atomic)
            });
        if !needs_rebind {
            return m.clone();
        }
        let mut m_clone = (**m).clone();
        m_clone.return_type = m_clone
            .return_type
            .as_deref()
            .map(|t| Arc::new(Self::rebind_self_static(t, to_fqcn)));
        m_clone.params = m_clone
            .params
            .iter()
            .map(|p| {
                let mut p = p.clone();
                p.ty =
                    p.ty.as_deref()
                        .map(|t| Arc::new(Self::rebind_self_static(t, to_fqcn)));
                p
            })
            .collect();
        Arc::new(m_clone)
    }

    fn type_has_self_or_static_atomic(ty: &mir_types::Type) -> bool {
        use mir_types::Atomic;
        ty.types
            .iter()
            .any(|a| matches!(a, Atomic::TSelf { .. } | Atomic::TStaticObject { .. }))
    }

    /// Returns true if the type contains template params or class-strings with unknown types.
    /// Used to suppress MethodSignatureMismatch on generic parent return types.
    /// Checks recursively into array key/value types.
    fn return_type_has_template(&self, ty: &mir_types::Type) -> bool {
        use mir_types::Atomic;
        ty.types.iter().any(|atomic| match atomic {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TClassString(Some(inner)) | Atomic::TInterfaceString(Some(inner)) => {
                !crate::db::class_exists(self.db, inner.as_ref())
            }
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
            Atomic::TIntersection { parts } => {
                parts.iter().any(|p| self.return_type_has_template(p))
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
            Atomic::TIntersection { parts } => parts.iter().any(Self::type_has_named_objects),
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
