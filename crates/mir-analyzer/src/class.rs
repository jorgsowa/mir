/// Class analyzer — validates class definitions after codebase finalization.
///
/// Checks performed (all codebase-level, no AST required):
///   - Concrete class implements all abstract parent methods
///   - Concrete class implements all interface methods
///   - Overriding method does not reduce visibility
///   - Overriding method return type is covariant with parent
///   - Overriding method does not override a final method
///   - Class does not extend a final class
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use mir_codebase::storage::{Location as StorageLocation, Visibility};
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
            analyzed_files: HashSet::new(),
            sources: HashMap::new(),
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
        // Phase 4 H1: keyed by Fqcn now.
        class_ancestors(
            self.db,
            crate::db::Fqcn::new(self.db, Arc::<str>::from(fqcn)),
        )
        .0
    }

    /// Run all class-level checks and return every discovered issue.
    pub fn analyze_all(&self) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Phase 4: enumerate via workspace_classes (pull) merged with
        // active_class_node_fqcns (push fallback). Filter to plain
        // classes only.
        let pull_classes: Vec<Arc<str>> = crate::db::workspace_classes(self.db)
            .iter()
            .cloned()
            .collect();
        let push_classes: Vec<Arc<str>> = self.db.active_class_node_fqcns();
        let mut seen: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();
        let mut class_keys: Vec<Arc<str>> = pull_classes
            .into_iter()
            .chain(push_classes)
            .filter(|f| seen.insert(f.clone()))
            .filter(|fqcn| {
                let here = crate::db::Fqcn::new(self.db, fqcn.clone());
                if let Some(c) = crate::db::find_class_like(self.db, here) {
                    return c.is_class();
                }
                self.db
                    .lookup_class_node(fqcn.as_ref())
                    .map(|n| {
                        !n.is_interface(self.db) && !n.is_trait(self.db) && !n.is_enum(self.db)
                    })
                    .unwrap_or(false)
            })
            .collect();
        // Sort for deterministic issue order across runs.
        class_keys.sort();

        for fqcn in &class_keys {
            // Pull-first + push-fallback class data.
            let here = crate::db::Fqcn::new(self.db, fqcn.clone());
            let pulled = crate::db::find_class_like(self.db, here);
            let push_node = self
                .db
                .lookup_class_node(fqcn.as_ref())
                .filter(|n| n.active(self.db));
            let location: Option<StorageLocation> = pulled
                .as_ref()
                .and_then(|c| c.location().cloned())
                .or_else(|| push_node.and_then(|n| n.location(self.db)));
            let parent_fqcn: Option<Arc<str>> = pulled
                .as_ref()
                .and_then(|c| c.parent().cloned())
                .or_else(|| push_node.and_then(|n| n.parent(self.db)));
            let is_abstract = pulled
                .as_ref()
                .map(|c| c.is_abstract())
                .or_else(|| push_node.map(|n| n.is_abstract(self.db)))
                .unwrap_or(false);
            if pulled.is_none() && push_node.is_none() {
                continue;
            }

            // Skip classes from vendor / stub files — only check user-analyzed files
            if !self.analyzed_files.is_empty() {
                let in_analyzed = location
                    .as_ref()
                    .map(|loc| self.analyzed_files.contains(&loc.file))
                    .unwrap_or(false);
                if !in_analyzed {
                    continue;
                }
            }

            // ---- 1. Final-class extension check / deprecated parent check ------
            if let Some(parent_fqcn) = parent_fqcn.as_ref() {
                let parent_here = crate::db::Fqcn::new(self.db, parent_fqcn.clone());
                let parent_pulled = crate::db::find_class_like(self.db, parent_here);
                let parent_push = self
                    .db
                    .lookup_class_node(parent_fqcn.as_ref())
                    .filter(|n| n.active(self.db));
                let parent_is_final = parent_pulled
                    .as_ref()
                    .map(|c| c.is_final())
                    .or_else(|| parent_push.map(|n| n.is_final(self.db)))
                    .unwrap_or(false);
                let parent_deprecated: Option<Arc<str>> = parent_pulled
                    .as_ref()
                    .and_then(|c| c.deprecated().cloned())
                    .or_else(|| parent_push.and_then(|n| n.deprecated(self.db)));
                if parent_pulled.is_some() || parent_push.is_some() {
                    if parent_is_final {
                        let loc = issue_location(
                            location.as_ref(),
                            fqcn,
                            location
                                .as_ref()
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::FinalClassExtended {
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
                            fqcn,
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

            // Skip abstract classes for "must implement" checks
            if is_abstract {
                // Still check override compatibility for abstract classes
                self.check_overrides(fqcn, location.as_ref(), &mut issues);
                continue;
            }

            // ---- 2. Abstract parent methods must be implemented ----------------
            self.check_abstract_methods_implemented(fqcn, location.as_ref(), &mut issues);

            // ---- 3. Interface methods must be implemented ----------------------
            self.check_interface_methods_implemented(fqcn, location.as_ref(), &mut issues);

            // ---- 4. Method override compatibility ------------------------------
            self.check_overrides(fqcn, location.as_ref(), &mut issues);
        }

        // ---- 5. Circular inheritance detection --------------------------------
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
        cls_location: Option<&StorageLocation>,
        issues: &mut Vec<Issue>,
    ) {
        // Walk every ancestor class and collect abstract methods
        let ancestors = self.ancestors(fqcn);
        for ancestor_fqcn in &ancestors {
            // Read abstract method names from the salsa db.  PR52 wired
            // pruning into `ingest_codebase`, so `method_nodes` no longer
            // accumulates stale stub entries when a user file shadows a
            // bundled-stub class with a different method set.
            let abstract_methods: Vec<Arc<str>> = self
                .db
                .class_own_methods(ancestor_fqcn.as_ref())
                .into_iter()
                .filter(|m| m.active(self.db) && m.is_abstract(self.db))
                .map(|m| m.name(self.db))
                .collect();

            for method_name in abstract_methods {
                // Check if the concrete class (or any closer ancestor) provides it
                if crate::db::method_is_concretely_implemented(
                    self.db,
                    fqcn.as_ref(),
                    method_name.as_ref(),
                ) {
                    continue; // implemented
                }

                let loc = issue_location(
                    cls_location,
                    fqcn,
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
        cls_location: Option<&StorageLocation>,
        issues: &mut Vec<Issue>,
    ) {
        // Collect all interfaces (direct + from ancestors)
        let all_ifaces: Vec<Arc<str>> = self
            .ancestors(fqcn)
            .into_iter()
            .filter(|p| {
                crate::db::class_kind_via_db(self.db, p.as_ref()).is_some_and(|k| k.is_interface)
            })
            .collect();

        for iface_fqcn in &all_ifaces {
            // Read method names from the salsa db.  PR52 wired pruning into
            // `ingest_codebase`, so `method_nodes` no longer accumulates stale
            // stub entries when a user file shadows a bundled-stub interface.
            let method_nodes = self.db.class_own_methods(iface_fqcn.as_ref());
            if method_nodes.is_empty() {
                // Skip interfaces with no registered methods (unregistered or
                // empty marker interfaces).
                continue;
            }
            let method_names: Vec<Arc<str>> = method_nodes
                .into_iter()
                .filter(|m| m.active(self.db) && !m.is_virtual(self.db))
                .map(|m| m.name(self.db))
                .collect();

            for method_name in method_names {
                // PHP method names are case-insensitive; normalize before lookup so that
                // a hand-written stub key like "jsonSerialize" matches the collector's
                // lowercased key "jsonserialize" stored in own_methods.
                let method_name_lower = method_name.to_lowercase();
                // Check if the class provides a concrete implementation
                let implemented = crate::db::method_is_concretely_implemented(
                    self.db,
                    fqcn.as_ref(),
                    &method_name_lower,
                );

                if !implemented {
                    let loc = issue_location(
                        cls_location,
                        fqcn,
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

    fn check_overrides(
        &self,
        fqcn: &Arc<str>,
        _cls_location: Option<&StorageLocation>,
        issues: &mut Vec<Issue>,
    ) {
        let own_methods = self.db.class_own_methods(fqcn.as_ref());
        for own in own_methods {
            if !own.active(self.db) {
                continue;
            }
            let method_name: Arc<str> = own.name(self.db);

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
            let parent_method = self.find_parent_method(fqcn, method_name_lower.as_ref());

            let parent = match parent_method {
                Some(m) => m,
                None => continue, // not an override
            };

            let own_location = own.location(self.db);
            let loc = issue_location(
                own_location.as_ref(),
                fqcn,
                own_location
                    .as_ref()
                    .and_then(|l| self.sources.get(&l.file).copied()),
            );

            // ---- a. Cannot override a final method -------------------------
            if parent.is_final(self.db) {
                let mut issue = Issue::new(
                    IssueKind::FinalMethodOverridden {
                        class: fqcn.to_string(),
                        method: method_name_lower.to_string(),
                        parent: parent.fqcn(self.db).to_string(),
                    },
                    loc.clone(),
                );
                if let Some(snippet) = extract_snippet(own_location.as_ref(), &self.sources) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }

            // ---- b. Visibility must not be reduced -------------------------
            if visibility_reduced(own.visibility(self.db), parent.visibility(self.db)) {
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

            // ---- c. Return type must be covariant --------------------------
            // Only check when both sides have an explicit return type.
            // Skip when:
            //   - Parent type is from a docblock (PHP doesn't enforce docblock override compat)
            //   - Either type is mixed
            //   - Parent type contains a template param
            let parent_return_type = parent.return_type(self.db);
            let own_return_type = own.return_type(self.db);
            if let (Some(child_ret), Some(parent_ret)) =
                (own_return_type.as_ref(), parent_return_type.as_ref())
            {
                let parent_from_docblock = parent_ret.from_docblock;
                let involves_named_objects = Self::type_has_named_objects(child_ret)
                    || Self::type_has_named_objects(parent_ret);
                let involves_self_static = self.type_has_self_or_static(child_ret)
                    || self.type_has_self_or_static(parent_ret);

                if !parent_from_docblock
                    && !parent_ret.is_mixed()
                    && !child_ret.is_mixed()
                    && !self.return_type_has_template(parent_ret)
                {
                    let child_file = own_location.as_ref().map(|l| l.file.as_ref()).unwrap_or("");

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
                        child_ret.is_subtype_of_simple(parent_ret)
                    };

                    if !compatible {
                        issues.push(
                            Issue::new(
                                IssueKind::MethodSignatureMismatch {
                                    class: fqcn.to_string(),
                                    method: method_name_lower.to_string(),
                                    detail: format!(
                                        "return type '{child_ret}' is not a subtype of parent '{parent_ret}'"
                                    ),
                                },
                                loc.clone(),
                            )
                            .with_snippet(method_name_lower.to_string()),
                        );
                    }
                }
            }

            // ---- d. Required param count must not increase -----------------
            let parent_params = parent.params(self.db);
            let own_params = own.params(self.db);
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

                // Contravariance: parent_ty must be subtype of child_ty.
                // If not, child has narrowed the param type.
                if !parent_ty.is_subtype_of_simple(child_ty) {
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
    fn return_type_has_template(&self, ty: &mir_types::Union) -> bool {
        use mir_types::Atomic;
        ty.types.iter().any(|atomic| match atomic {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TClassString(Some(inner)) => {
                !crate::db::type_exists_via_db(self.db, inner.as_ref())
            }
            Atomic::TNamedObject { fqcn, type_params } => {
                // Bare name with no namespace separator is likely a template param
                (!fqcn.contains('\\') && !crate::db::type_exists_via_db(self.db, fqcn.as_ref()))
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
    fn type_has_named_objects(ty: &mir_types::Union) -> bool {
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
    fn type_has_self_or_static(&self, ty: &mir_types::Union) -> bool {
        use mir_types::Atomic;
        ty.types
            .iter()
            .any(|a| matches!(a, Atomic::TSelf { .. } | Atomic::TStaticObject { .. }))
    }

    /// Returns true if every atom in the union is handled by `named_object_return_compatible`:
    /// object types (named/self/static/parent), null, void, never, and class-string variants.
    /// Unions that also contain scalar atoms (int, string, …) are not fully handled there
    /// and must fall back to the skip path (G5 gap).
    fn type_has_only_object_atoms(&self, ty: &mir_types::Union) -> bool {
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

    /// Find a method with the given name in the closest ancestor (not the class itself).
    /// Returns the parent's `MethodNode` (db-tracked).
    fn find_parent_method(
        &self,
        fqcn: &Arc<str>,
        method_name_lower: &str,
    ) -> Option<crate::db::MethodNode> {
        let ancestors = self.ancestors(fqcn);
        for ancestor_fqcn in &ancestors {
            if let Some(node) = self
                .db
                .lookup_method_node(ancestor_fqcn.as_ref(), method_name_lower)
                .filter(|n| n.active(self.db))
            {
                return Some(node);
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Check: circular class inheritance (class A extends B extends A)
    // -----------------------------------------------------------------------

    fn check_circular_class_inheritance(&self, issues: &mut Vec<Issue>) {
        let mut globally_done: HashSet<String> = HashSet::new();

        let mut class_keys: Vec<Arc<str>> = self
            .db
            .active_class_node_fqcns()
            .into_iter()
            .filter(|fqcn| {
                self.db
                    .lookup_class_node(fqcn.as_ref())
                    .map(|n| {
                        !n.is_interface(self.db) && !n.is_trait(self.db) && !n.is_enum(self.db)
                    })
                    .unwrap_or(false)
            })
            .collect();
        class_keys.sort();

        for start_fqcn in &class_keys {
            if globally_done.contains(start_fqcn.as_ref()) {
                continue;
            }

            // Walk the parent chain, tracking order for cycle reporting.
            let mut chain: Vec<Arc<str>> = Vec::new();
            let mut chain_set: HashSet<String> = HashSet::new();
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
                        let location = self
                            .db
                            .lookup_class_node(offender.as_ref())
                            .filter(|n| n.active(self.db))
                            .and_then(|n| n.location(self.db));
                        let loc = issue_location(
                            location.as_ref(),
                            offender,
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

                let parent = self
                    .db
                    .lookup_class_node(current.as_ref())
                    .filter(|n| n.active(self.db))
                    .and_then(|n| n.parent(self.db));

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
        let mut globally_done: HashSet<String> = HashSet::new();

        let mut iface_keys: Vec<Arc<str>> = self
            .db
            .active_class_node_fqcns()
            .into_iter()
            .filter(|fqcn| {
                self.db
                    .lookup_class_node(fqcn.as_ref())
                    .map(|n| n.is_interface(self.db))
                    .unwrap_or(false)
            })
            .collect();
        iface_keys.sort();

        for start_fqcn in &iface_keys {
            if globally_done.contains(start_fqcn.as_ref()) {
                continue;
            }
            let mut in_stack: Vec<Arc<str>> = Vec::new();
            let mut stack_set: HashSet<String> = HashSet::new();
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
                let location = self
                    .db
                    .lookup_class_node(offender.as_ref())
                    .filter(|n| n.active(self.db))
                    .and_then(|n| n.location(self.db));
                let loc = issue_location(
                    location.as_ref(),
                    offender,
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

        let extends: Vec<Arc<str>> = self
            .db
            .lookup_class_node(fqcn.as_ref())
            .filter(|n| n.active(self.db))
            .map(|n| n.extends(self.db).to_vec())
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
        self.db
            .lookup_class_node(fqcn.as_ref())
            .filter(|n| n.active(self.db))
            .and_then(|n| n.location(self.db))
            .map(|loc| self.analyzed_files.contains(&loc.file))
            .unwrap_or(false)
    }

    fn iface_in_analyzed_files(&self, fqcn: &Arc<str>) -> bool {
        // Same lookup path as `class_in_analyzed_files` — interface and class
        // nodes share `ClassNode` storage, distinguished by `is_interface`.
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
/// Falls back to a dummy location using the FQCN as the file path when no
/// Location is stored.
fn issue_location(
    storage_loc: Option<&mir_codebase::storage::Location>,
    fqcn: &Arc<str>,
    _source: Option<&str>,
) -> Location {
    match storage_loc {
        Some(loc) => Location {
            file: loc.file.clone(),
            line: loc.line,
            line_end: loc.line_end,
            col_start: loc.col_start,
            col_end: loc.col_end,
        },
        None => Location {
            file: fqcn.clone(),
            line: 1,
            line_end: 1,
            col_start: 0,
            col_end: 0,
        },
    }
}

/// Extract the first line of source text covered by `storage_loc` as a snippet.
fn extract_snippet(
    storage_loc: Option<&mir_codebase::storage::Location>,
    sources: &HashMap<Arc<str>, &str>,
) -> Option<String> {
    let loc = storage_loc?;
    let src = *sources.get(&loc.file)?;
    // Walk to the 1-based start line (loc.line is already 1-based).
    let line_idx = loc.line.saturating_sub(1) as usize;
    let line_text = src.lines().nth(line_idx)?;
    Some(line_text.trim().to_string())
}
