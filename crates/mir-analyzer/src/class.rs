/// Class analyzer — validates class definitions after codebase finalization.
///
/// Checks performed (all codebase-level, no AST required):
///   - Concrete class implements all abstract parent methods
///   - Concrete class implements all interface methods
///   - Overriding method does not reduce visibility
///   - Overriding method return type is covariant with parent
///   - Overriding method does not override a final method
///   - Class does not extend a final class
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::sync::Arc;

use mir_codebase::storage::Visibility;
use mir_issues::{Issue, IssueKind, Location};

use crate::db::{class_ancestors, MirDatabase};

// ---------------------------------------------------------------------------
// ClassAnalyzer
// ---------------------------------------------------------------------------

pub struct ClassAnalyzer<'a> {
    db: &'a dyn MirDatabase,
    /// Only report issues for classes defined in these files (empty = all files).
    analyzed_files: HashSet<Arc<str>>,
    /// Source text keyed by file path, used to extract snippets for class-level issues.
    sources: HashMap<Arc<str>, &'a str>,
}

impl<'a> ClassAnalyzer<'a> {
    #[allow(dead_code)]
    pub fn new(db: &'a dyn MirDatabase) -> Self {
        Self {
            db,
            analyzed_files: HashSet::default(),
            sources: HashMap::default(),
        }
    }

    pub fn with_files(
        db: &'a dyn MirDatabase,
        files: HashSet<Arc<str>>,
        file_data: &'a [(Arc<str>, Arc<str>)],
    ) -> Self {
        let sources: HashMap<Arc<str>, &'a str> = file_data
            .iter()
            .map(|(f, s)| (f.clone(), s.as_ref()))
            .collect();
        Self {
            db,
            analyzed_files: files,
            sources,
        }
    }

    /// Ancestor chain for `fqcn` from the salsa db, or empty if the class
    /// isn't registered.
    fn ancestors(&self, fqcn: &str) -> Vec<Arc<str>> {
        class_ancestors(self.db, crate::db::Fqcn::from_str(self.db, fqcn)).0
    }

    /// Run all class-level checks and return every discovered issue.
    pub fn analyze_all(&self) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Only plain classes defined in the analyzed file set, already sorted by
        // FQCN. Decomposing per file via `collect_file_definitions` means vendor
        // / stub classes are never materialized (they aren't in `analyzed_files`),
        // which is the dominant cost on cold start over a large `vendor/`.
        for (fqcn, class) in crate::db::analyzed_class_defs(self.db, &self.analyzed_files) {
            let fqcn = &fqcn;
            let location: Option<Location> = class.location().cloned();
            let parent_fqcn: Option<Arc<str>> = class.parent().cloned();
            let is_abstract = class.is_abstract();

            // ---- 1. Final-class extension check / deprecated parent check ------
            if let Some(parent_fqcn) = parent_fqcn.as_ref() {
                let parent_here = crate::db::Fqcn::from_str(self.db, parent_fqcn.as_ref());
                let parent_pulled = crate::db::find_class_like(self.db, parent_here);
                let parent_is_final = parent_pulled
                    .as_ref()
                    .map(|c| c.is_final())
                    .unwrap_or(false);
                let parent_deprecated: Option<Arc<str>> =
                    parent_pulled.as_ref().and_then(|c| c.deprecated().cloned());
                if let Some(canonical) = parent_pulled.as_ref() {
                    let used_short = parent_fqcn
                        .rsplit('\\')
                        .next()
                        .unwrap_or(parent_fqcn.as_ref());
                    let canonical_short = canonical
                        .fqcn()
                        .rsplit('\\')
                        .next()
                        .unwrap_or(canonical.fqcn().as_ref());
                    if used_short != canonical_short
                        && used_short.eq_ignore_ascii_case(canonical_short)
                    {
                        let loc = issue_location(
                            location.as_ref(),
                            location
                                .as_ref()
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::WrongCaseClass {
                                used: used_short.to_string(),
                                canonical: canonical_short.to_string(),
                            },
                            loc,
                        );
                        if let Some(snippet) = extract_snippet(location.as_ref(), &self.sources) {
                            issue = issue.with_snippet(snippet);
                        }
                        issues.push(issue);
                    }
                }
                if parent_pulled.is_some() {
                    if parent_is_final {
                        let loc = issue_location(
                            location.as_ref(),
                            location
                                .as_ref()
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::InvalidExtendClass {
                                parent: parent_fqcn.to_string(),
                                child: fqcn.to_string(),
                            },
                            loc,
                        );
                        if let Some(snippet) = extract_snippet(location.as_ref(), &self.sources) {
                            issue = issue.with_snippet(snippet);
                        }
                        issues.push(issue);
                    }
                    if let Some(msg) = parent_deprecated {
                        let loc = issue_location(
                            location.as_ref(),
                            location
                                .as_ref()
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::DeprecatedClass {
                                name: parent_fqcn.to_string(),
                                message: Some(msg).filter(|m| !m.is_empty()),
                            },
                            loc,
                        );
                        if let Some(snippet) = extract_snippet(location.as_ref(), &self.sources) {
                            issue = issue.with_snippet(snippet);
                        }
                        issues.push(issue);
                    }
                }
            }

            // ---- 1b. Deprecated interface / trait checks -----------------------
            {
                let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                if let Some(cls) = crate::db::find_class_like(self.db, here) {
                    for iface_fqcn in cls.interfaces() {
                        let iface_here = crate::db::Fqcn::from_str(self.db, iface_fqcn.as_ref());
                        if let Some(iface) = crate::db::find_class_like(self.db, iface_here) {
                            let used_short = iface_fqcn
                                .rsplit('\\')
                                .next()
                                .unwrap_or(iface_fqcn.as_ref());
                            let canonical_short = iface
                                .fqcn()
                                .rsplit('\\')
                                .next()
                                .unwrap_or(iface.fqcn().as_ref());
                            if used_short != canonical_short
                                && used_short.eq_ignore_ascii_case(canonical_short)
                            {
                                let loc = issue_location(
                                    location.as_ref(),
                                    location
                                        .as_ref()
                                        .and_then(|l| self.sources.get(&l.file).copied()),
                                );
                                let mut issue = Issue::new(
                                    IssueKind::WrongCaseClass {
                                        used: used_short.to_string(),
                                        canonical: canonical_short.to_string(),
                                    },
                                    loc,
                                );
                                if let Some(snippet) =
                                    extract_snippet(location.as_ref(), &self.sources)
                                {
                                    issue = issue.with_snippet(snippet);
                                }
                                issues.push(issue);
                            }
                            if let Some(msg) = iface.deprecated() {
                                let loc = issue_location(
                                    location.as_ref(),
                                    location
                                        .as_ref()
                                        .and_then(|l| self.sources.get(&l.file).copied()),
                                );
                                let mut issue = Issue::new(
                                    IssueKind::DeprecatedInterface {
                                        name: iface_fqcn.to_string(),
                                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                    },
                                    loc,
                                );
                                if let Some(snippet) =
                                    extract_snippet(location.as_ref(), &self.sources)
                                {
                                    issue = issue.with_snippet(snippet);
                                }
                                issues.push(issue);
                            }
                        }
                    }
                    for trait_fqcn in cls.class_traits() {
                        let trait_here = crate::db::Fqcn::from_str(self.db, trait_fqcn.as_ref());
                        if let Some(t) = crate::db::find_class_like(self.db, trait_here) {
                            let used_short = trait_fqcn
                                .rsplit('\\')
                                .next()
                                .unwrap_or(trait_fqcn.as_ref());
                            let canonical_short =
                                t.fqcn().rsplit('\\').next().unwrap_or(t.fqcn().as_ref());
                            if used_short != canonical_short
                                && used_short.eq_ignore_ascii_case(canonical_short)
                            {
                                let loc = issue_location(
                                    location.as_ref(),
                                    location
                                        .as_ref()
                                        .and_then(|l| self.sources.get(&l.file).copied()),
                                );
                                let mut issue = Issue::new(
                                    IssueKind::WrongCaseClass {
                                        used: used_short.to_string(),
                                        canonical: canonical_short.to_string(),
                                    },
                                    loc,
                                );
                                if let Some(snippet) =
                                    extract_snippet(location.as_ref(), &self.sources)
                                {
                                    issue = issue.with_snippet(snippet);
                                }
                                issues.push(issue);
                            }
                            if let Some(msg) = t.deprecated() {
                                let loc = issue_location(
                                    location.as_ref(),
                                    location
                                        .as_ref()
                                        .and_then(|l| self.sources.get(&l.file).copied()),
                                );
                                let mut issue = Issue::new(
                                    IssueKind::DeprecatedTrait {
                                        name: trait_fqcn.to_string(),
                                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                    },
                                    loc,
                                );
                                if let Some(snippet) =
                                    extract_snippet(location.as_ref(), &self.sources)
                                {
                                    issue = issue.with_snippet(snippet);
                                }
                                issues.push(issue);
                            }
                        }
                    }
                }
            }

            // Skip abstract classes for "must implement" checks
            if is_abstract {
                // Still check override compatibility for abstract classes
                self.check_overrides(fqcn, location.as_ref(), &mut issues);
                self.check_magic_method_casing(fqcn, &mut issues);
                continue;
            }

            // ---- 2. Abstract parent methods must be implemented ----------------
            self.check_abstract_methods_implemented(fqcn, location.as_ref(), &mut issues);

            // ---- 3. Interface methods must be implemented ----------------------
            self.check_interface_methods_implemented(fqcn, location.as_ref(), &mut issues);

            // ---- 4. Method override compatibility ------------------------------
            self.check_overrides(fqcn, location.as_ref(), &mut issues);

            // ---- 5. Magic method casing ----------------------------------------
            self.check_magic_method_casing(fqcn, &mut issues);
        }

        // ---- 5. Interface-level #[Override] check + extends casing --------
        // Interfaces are not included in the class loop above, so scan them
        // separately for #[Override] on methods that have no parent interface method.
        for (iface_fqcn, iface) in crate::db::analyzed_interface_defs(self.db, &self.analyzed_files)
        {
            self.check_overrides(&iface_fqcn, None, &mut issues);
            let location = iface.location.clone();
            for parent_iface_fqcn in iface.extends.iter() {
                let here = crate::db::Fqcn::from_str(self.db, parent_iface_fqcn.as_ref());
                if let Some(canonical) = crate::db::find_class_like(self.db, here) {
                    let used_short = parent_iface_fqcn
                        .rsplit('\\')
                        .next()
                        .unwrap_or(parent_iface_fqcn.as_ref());
                    let canonical_short = canonical
                        .fqcn()
                        .rsplit('\\')
                        .next()
                        .unwrap_or(canonical.fqcn().as_ref());
                    if used_short != canonical_short
                        && used_short.eq_ignore_ascii_case(canonical_short)
                    {
                        let loc = issue_location(
                            location.as_ref(),
                            location
                                .as_ref()
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::WrongCaseClass {
                                used: used_short.to_string(),
                                canonical: canonical_short.to_string(),
                            },
                            loc,
                        );
                        if let Some(snippet) = extract_snippet(location.as_ref(), &self.sources) {
                            issue = issue.with_snippet(snippet);
                        }
                        issues.push(issue);
                    }
                }
            }
            self.check_magic_method_casing(&iface_fqcn, &mut issues);
        }

        // ---- 5b. Enum-level implements casing + magic methods ---------------
        for (enum_fqcn, enum_def) in crate::db::analyzed_enum_defs(self.db, &self.analyzed_files) {
            let location = enum_def.location.clone();
            for iface_fqcn in enum_def.interfaces.iter() {
                let here = crate::db::Fqcn::from_str(self.db, iface_fqcn.as_ref());
                if let Some(canonical) = crate::db::find_class_like(self.db, here) {
                    let used_short = iface_fqcn
                        .rsplit('\\')
                        .next()
                        .unwrap_or(iface_fqcn.as_ref());
                    let canonical_short = canonical
                        .fqcn()
                        .rsplit('\\')
                        .next()
                        .unwrap_or(canonical.fqcn().as_ref());
                    if used_short != canonical_short
                        && used_short.eq_ignore_ascii_case(canonical_short)
                    {
                        let loc = issue_location(
                            location.as_ref(),
                            location
                                .as_ref()
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::WrongCaseClass {
                                used: used_short.to_string(),
                                canonical: canonical_short.to_string(),
                            },
                            loc,
                        );
                        if let Some(snippet) = extract_snippet(location.as_ref(), &self.sources) {
                            issue = issue.with_snippet(snippet);
                        }
                        issues.push(issue);
                    }
                }
            }
            self.check_magic_method_casing(&enum_fqcn, &mut issues);
        }

        // ---- 6. Circular inheritance detection --------------------------------
        self.check_circular_class_inheritance(&mut issues);
        self.check_circular_interface_inheritance(&mut issues);

        issues
    }

    // -----------------------------------------------------------------------
    // Check: all abstract methods from ancestor chain are implemented
    // -----------------------------------------------------------------------

    fn check_abstract_methods_implemented(
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

    fn check_interface_methods_implemented(
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
    fn scalar_return_types_compatible(
        child_ret: &mir_types::Type,
        parent_ret: &mir_types::Type,
    ) -> bool {
        child_ret.is_subtype_structural(parent_ret)
    }

    /// Returns true when a child's scalar param type has been illegally narrowed
    /// relative to the parent (contravariance violation).
    /// Only called after confirming neither side contains named objects, self/static,
    /// templates, or mixed — those cases are skipped by the caller.
    fn scalar_param_type_narrowed(parent_ty: &mir_types::Type, child_ty: &mir_types::Type) -> bool {
        !parent_ty.is_subtype_structural(child_ty)
    }

    fn check_magic_method_casing(&self, fqcn: &Arc<str>, issues: &mut Vec<Issue>) {
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

    fn check_overrides(
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
            if own.is_abstract && !parent.is_abstract {
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
                    let involves_named_objects = Self::type_has_named_objects(child_ret)
                        || Self::type_has_named_objects(parent_ret);
                    let involves_self_static = self.type_has_self_or_static(child_ret)
                        || self.type_has_self_or_static(parent_ret);
                    let compatible = if (involves_named_objects || involves_self_static)
                        && self.type_has_only_object_atoms(child_ret)
                        && self.type_has_only_object_atoms(parent_ret)
                    {
                        crate::stmt::named_object_return_compatible(
                            child_ret, parent_ret, self.db, child_file,
                        )
                    } else if involves_named_objects || involves_self_static {
                        true // mixed scalar+object union — skip (G5 gap)
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
            //   - Either type contains a named object (needs codebase for inheritance check)
            //   - Either type contains TSelf/TStaticObject
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
                    || Self::type_has_named_objects(parent_ty)
                    || Self::type_has_named_objects(child_ty)
                    || self.type_has_self_or_static(parent_ty)
                    || self.type_has_self_or_static(child_ty)
                    || self.return_type_has_template(parent_ty)
                    || self.return_type_has_template(child_ty)
                {
                    continue;
                }

                if Self::scalar_param_type_narrowed(parent_ty, child_ty) {
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

    /// Returns true if the type contains TSelf or TStaticObject (late-static types).
    /// These are always considered compatible with their bound class type.
    fn type_has_self_or_static(&self, ty: &mir_types::Type) -> bool {
        use mir_types::Atomic;
        ty.types
            .iter()
            .any(|a| matches!(a, Atomic::TSelf { .. } | Atomic::TStaticObject { .. }))
    }

    /// Returns true if every atom in the union is handled by `named_object_return_compatible`:
    /// object types (named/self/static/parent), null, void, never, and class-string variants.
    /// Unions that also contain scalar atoms (int, string, …) are not fully handled there
    /// and must fall back to the skip path (G5 gap).
    fn type_has_only_object_atoms(&self, ty: &mir_types::Type) -> bool {
        use mir_types::Atomic;
        ty.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TNamedObject { .. }
                    | Atomic::TSelf { .. }
                    | Atomic::TStaticObject { .. }
                    | Atomic::TParent { .. }
                    | Atomic::TNull
                    | Atomic::TVoid
                    | Atomic::TNever
                    | Atomic::TClassString(_)
            )
        })
    }

    // -----------------------------------------------------------------------
    // Check: circular class inheritance (class A extends B extends A)
    // -----------------------------------------------------------------------

    fn check_circular_class_inheritance(&self, issues: &mut Vec<Issue>) {
        let mut globally_done: HashSet<String> = HashSet::default();

        let mut class_keys: Vec<Arc<str>> = crate::db::workspace_classes(self.db)
            .iter()
            .filter(|fqcn| {
                let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                crate::db::find_class_like(self.db, here)
                    .map(|c| c.is_class())
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        class_keys.sort();

        for start_fqcn in &class_keys {
            if globally_done.contains(start_fqcn.as_ref()) {
                continue;
            }

            // Walk the parent chain, tracking order for cycle reporting.
            let mut chain: Vec<Arc<str>> = Vec::new();
            let mut chain_set: HashSet<String> = HashSet::default();
            let mut current: Arc<str> = start_fqcn.clone();

            loop {
                if globally_done.contains(current.as_ref()) {
                    // Known safe — stop here.
                    for node in &chain {
                        globally_done.insert(node.to_string());
                    }
                    break;
                }
                if !chain_set.insert(current.to_string()) {
                    // current is already in chain → cycle detected.
                    let cycle_start = chain
                        .iter()
                        .position(|p| p.as_ref() == current.as_ref())
                        .unwrap_or(0);
                    let cycle_nodes = &chain[cycle_start..];

                    // Report on the lexicographically last class in the cycle
                    // that belongs to an analyzed file (or any if filter is empty).
                    let offender = cycle_nodes
                        .iter()
                        .filter(|n| self.class_in_analyzed_files(n))
                        .max_by(|a, b| a.as_ref().cmp(b.as_ref()));

                    if let Some(offender) = offender {
                        let here = crate::db::Fqcn::from_str(self.db, offender.as_ref());
                        let location: Option<Location> = crate::db::find_class_like(self.db, here)
                            .and_then(|c| c.location().cloned());
                        let loc = issue_location(
                            location.as_ref(),
                            location
                                .as_ref()
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::CircularInheritance {
                                class: offender.to_string(),
                            },
                            loc,
                        );
                        if let Some(snippet) = extract_snippet(location.as_ref(), &self.sources) {
                            issue = issue.with_snippet(snippet);
                        }
                        issues.push(issue);
                    }

                    for node in &chain {
                        globally_done.insert(node.to_string());
                    }
                    break;
                }

                chain.push(current.clone());

                let here = crate::db::Fqcn::from_str(self.db, current.as_ref());
                let parent: Option<Arc<str>> =
                    crate::db::find_class_like(self.db, here).and_then(|c| c.parent().cloned());

                match parent {
                    Some(p) => current = p,
                    None => {
                        for node in &chain {
                            globally_done.insert(node.to_string());
                        }
                        break;
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Check: circular interface inheritance (interface I1 extends I2 extends I1)
    // -----------------------------------------------------------------------

    fn check_circular_interface_inheritance(&self, issues: &mut Vec<Issue>) {
        let mut globally_done: HashSet<String> = HashSet::default();

        let mut iface_keys: Vec<Arc<str>> = crate::db::workspace_classes(self.db)
            .iter()
            .filter(|fqcn| {
                let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                crate::db::find_class_like(self.db, here)
                    .map(|c| c.is_interface())
                    .unwrap_or(false)
            })
            .cloned()
            .collect();
        iface_keys.sort();

        for start_fqcn in &iface_keys {
            if globally_done.contains(start_fqcn.as_ref()) {
                continue;
            }
            let mut in_stack: Vec<Arc<str>> = Vec::new();
            let mut stack_set: HashSet<String> = HashSet::default();
            self.dfs_interface_cycle(
                start_fqcn.clone(),
                &mut in_stack,
                &mut stack_set,
                &mut globally_done,
                issues,
            );
        }
    }

    fn dfs_interface_cycle(
        &self,
        fqcn: Arc<str>,
        in_stack: &mut Vec<Arc<str>>,
        stack_set: &mut HashSet<String>,
        globally_done: &mut HashSet<String>,
        issues: &mut Vec<Issue>,
    ) {
        if globally_done.contains(fqcn.as_ref()) {
            return;
        }
        if stack_set.contains(fqcn.as_ref()) {
            // Cycle: find cycle nodes from in_stack.
            let cycle_start = in_stack
                .iter()
                .position(|p| p.as_ref() == fqcn.as_ref())
                .unwrap_or(0);
            let cycle_nodes = &in_stack[cycle_start..];

            let offender = cycle_nodes
                .iter()
                .filter(|n| self.iface_in_analyzed_files(n))
                .max_by(|a, b| a.as_ref().cmp(b.as_ref()));

            if let Some(offender) = offender {
                let here = crate::db::Fqcn::from_str(self.db, offender.as_ref());
                let location =
                    crate::db::find_class_like(self.db, here).and_then(|c| c.location().cloned());
                let loc = issue_location(
                    location.as_ref(),
                    location
                        .as_ref()
                        .and_then(|l| self.sources.get(&l.file).copied()),
                );
                let mut issue = Issue::new(
                    IssueKind::CircularInheritance {
                        class: offender.to_string(),
                    },
                    loc,
                );
                if let Some(snippet) = extract_snippet(location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }
            return;
        }

        stack_set.insert(fqcn.to_string());
        in_stack.push(fqcn.clone());

        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        let extends: Vec<Arc<str>> = crate::db::find_class_like(self.db, here)
            .map(|c| c.extends().to_vec())
            .unwrap_or_default();

        for parent in extends {
            self.dfs_interface_cycle(parent, in_stack, stack_set, globally_done, issues);
        }

        in_stack.pop();
        stack_set.remove(fqcn.as_ref());
        globally_done.insert(fqcn.to_string());
    }

    fn class_in_analyzed_files(&self, fqcn: &Arc<str>) -> bool {
        if self.analyzed_files.is_empty() {
            return true;
        }
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        crate::db::find_class_like(self.db, here)
            .and_then(|c| c.location().cloned())
            .map(|loc| self.analyzed_files.contains(&loc.file))
            .unwrap_or(false)
    }

    fn iface_in_analyzed_files(&self, fqcn: &Arc<str>) -> bool {
        // Same lookup path as `class_in_analyzed_files`.
        self.class_in_analyzed_files(fqcn)
    }
}

/// Returns true if `child_vis` is strictly less visible than `parent_vis`.
fn visibility_reduced(child_vis: Visibility, parent_vis: Visibility) -> bool {
    // Public > Protected > Private (in terms of access)
    // Reducing means going from more visible to less visible.
    matches!(
        (parent_vis, child_vis),
        (Visibility::Public, Visibility::Protected)
            | (Visibility::Public, Visibility::Private)
            | (Visibility::Protected, Visibility::Private)
    )
}

/// Build an issue location from the stored codebase Location.
/// Clamps `line_end`/`col_end` to the declaration line so SARIF/WASM consumers
/// see a tight range instead of the entire class body.
/// Falls back to `storage_loc_to_location(None)` when no Location is stored.
fn issue_location(storage_loc: Option<&mir_types::Location>, source: Option<&str>) -> Location {
    let Some(loc) = storage_loc else {
        return crate::diagnostics::storage_loc_to_location(None);
    };
    let (line_end, col_end) = source
        .and_then(|src| src.lines().nth(loc.line.saturating_sub(1) as usize))
        .map(|decl_line| {
            let char_count = decl_line.chars().count() as u16;
            (loc.line, char_count.max(loc.col_start + 1))
        })
        .unwrap_or((loc.line, loc.col_end));
    Location {
        file: loc.file.clone(),
        line: loc.line,
        line_end,
        col_start: loc.col_start,
        col_end,
    }
}

fn canonical_magic_name(lower: &str) -> Option<&'static str> {
    match lower {
        "__construct" => Some("__construct"),
        "__destruct" => Some("__destruct"),
        "__call" => Some("__call"),
        "__callstatic" => Some("__callStatic"),
        "__get" => Some("__get"),
        "__set" => Some("__set"),
        "__isset" => Some("__isset"),
        "__unset" => Some("__unset"),
        "__sleep" => Some("__sleep"),
        "__wakeup" => Some("__wakeup"),
        "__serialize" => Some("__serialize"),
        "__unserialize" => Some("__unserialize"),
        "__tostring" => Some("__toString"),
        "__invoke" => Some("__invoke"),
        "__set_state" => Some("__set_state"),
        "__clone" => Some("__clone"),
        "__debuginfo" => Some("__debugInfo"),
        _ => None,
    }
}

/// Extract the first line of source text covered by `storage_loc` as a snippet.
fn extract_snippet(
    storage_loc: Option<&mir_types::Location>,
    sources: &HashMap<Arc<str>, &str>,
) -> Option<String> {
    let loc = storage_loc?;
    let src = *sources.get(&loc.file)?;
    // Walk to the 1-based start line (loc.line is already 1-based).
    let line_idx = loc.line.saturating_sub(1) as usize;
    let line_text = src.lines().nth(line_idx)?;
    Some(line_text.trim().to_string())
}
