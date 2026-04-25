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

use rayon::prelude::*;

use mir_codebase::storage::{MethodStorage, Visibility};
use mir_codebase::Codebase;
use mir_issues::{Issue, IssueKind, Location};

// ---------------------------------------------------------------------------
// ClassAnalyzer
// ---------------------------------------------------------------------------

pub struct ClassAnalyzer<'a> {
    codebase: &'a Codebase,
    /// Only report issues for classes defined in these files (empty = all files).
    analyzed_files: HashSet<Arc<str>>,
    /// Source text keyed by file path, used to extract snippets for class-level issues.
    sources: HashMap<Arc<str>, &'a str>,
}

impl<'a> ClassAnalyzer<'a> {
    pub fn new(codebase: &'a Codebase) -> Self {
        Self {
            codebase,
            analyzed_files: HashSet::new(),
            sources: HashMap::new(),
        }
    }

    pub fn with_files(
        codebase: &'a Codebase,
        files: HashSet<Arc<str>>,
        file_data: &'a [(Arc<str>, String, String)],
    ) -> Self {
        let sources: HashMap<Arc<str>, &'a str> = file_data
            .iter()
            .map(|(f, s, _)| (f.clone(), s.as_str()))
            .collect();
        Self {
            codebase,
            analyzed_files: files,
            sources,
        }
    }

    /// Run all class-level checks and return every discovered issue.
    pub fn analyze_all(&self) -> Vec<Issue> {
        let class_keys: Vec<Arc<str>> = self
            .codebase
            .classes
            .iter()
            .map(|e| e.key().clone())
            .collect();

        // Per-class checks are independent — run them in parallel.
        let mut issues: Vec<Issue> = class_keys
            .par_iter()
            .flat_map(|fqcn| {
                let mut class_issues = Vec::new();

                let cls = match self.codebase.classes.get(fqcn.as_ref()) {
                    Some(c) => c,
                    None => return class_issues,
                };

                // Skip classes from vendor / stub files — only check user-analyzed files
                if !self.analyzed_files.is_empty() {
                    let in_analyzed = cls
                        .location
                        .as_ref()
                        .map(|loc| self.analyzed_files.contains(&loc.file))
                        .unwrap_or(false);
                    if !in_analyzed {
                        return class_issues;
                    }
                }

                // ---- 1. Final-class extension check / deprecated parent check ------
                if let Some(parent_fqcn) = &cls.parent {
                    if let Some(parent) = self.codebase.classes.get(parent_fqcn.as_ref()) {
                        if parent.is_final {
                            let loc = issue_location(
                                cls.location.as_ref(),
                                fqcn,
                                cls.location
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
                            if let Some(snippet) =
                                extract_snippet(cls.location.as_ref(), &self.sources)
                            {
                                issue = issue.with_snippet(snippet);
                            }
                            class_issues.push(issue);
                        }
                        if let Some(msg) = parent.deprecated.clone() {
                            let loc = issue_location(
                                cls.location.as_ref(),
                                fqcn,
                                cls.location
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
                            if let Some(snippet) =
                                extract_snippet(cls.location.as_ref(), &self.sources)
                            {
                                issue = issue.with_snippet(snippet);
                            }
                            class_issues.push(issue);
                        }
                    }
                }

                // Skip abstract classes for "must implement" checks
                if cls.is_abstract {
                    // Still check override compatibility for abstract classes
                    self.check_overrides(&cls, &mut class_issues);
                    return class_issues;
                }

                // ---- 2. Abstract parent methods must be implemented ----------------
                self.check_abstract_methods_implemented(&cls, &mut class_issues);

                // ---- 3. Interface methods must be implemented ----------------------
                self.check_interface_methods_implemented(&cls, &mut class_issues);

                // ---- 4. Method override compatibility ------------------------------
                self.check_overrides(&cls, &mut class_issues);

                class_issues
            })
            .collect();

        // ---- 5. Circular inheritance detection (must remain serial — uses shared memoization) ---
        self.check_circular_class_inheritance(&mut issues);
        self.check_circular_interface_inheritance(&mut issues);

        issues
    }

    // -----------------------------------------------------------------------
    // Check: all abstract methods from ancestor chain are implemented
    // -----------------------------------------------------------------------

    fn check_abstract_methods_implemented(
        &self,
        cls: &mir_codebase::storage::ClassStorage,
        issues: &mut Vec<Issue>,
    ) {
        let fqcn = &cls.fqcn;

        // Walk every ancestor class and collect abstract methods
        for ancestor_fqcn in &cls.all_parents {
            // Collect abstract method names first, then drop the DashMap guard before
            // calling get_method (which re-enters the same DashMap).
            let abstract_methods: Vec<Arc<str>> = {
                let Some(ancestor) = self.codebase.classes.get(ancestor_fqcn.as_ref()) else {
                    continue;
                };
                ancestor
                    .own_methods
                    .iter()
                    .filter(|(_, m)| m.is_abstract)
                    .map(|(_, m)| m.name.clone())
                    .collect()
            };

            for method_name in abstract_methods {
                // Check if the concrete class (or any closer ancestor) provides it
                if self
                    .codebase
                    .get_method(fqcn.as_ref(), method_name.as_ref())
                    .map(|m| !m.is_abstract)
                    .unwrap_or(false)
                {
                    continue; // implemented
                }

                let loc = issue_location(
                    cls.location.as_ref(),
                    fqcn,
                    cls.location
                        .as_ref()
                        .and_then(|l| self.sources.get(&l.file).copied()),
                );
                let mut issue = Issue::new(
                    IssueKind::UnimplementedAbstractMethod {
                        class: fqcn.to_string(),
                        method: method_name.to_string(),
                    },
                    loc,
                );
                if let Some(snippet) = extract_snippet(cls.location.as_ref(), &self.sources) {
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
        cls: &mir_codebase::storage::ClassStorage,
        issues: &mut Vec<Issue>,
    ) {
        let fqcn = &cls.fqcn;

        // Collect all interfaces (direct + from ancestors)
        let all_ifaces: Vec<Arc<str>> = cls
            .all_parents
            .iter()
            .filter(|p| self.codebase.interfaces.contains_key(p.as_ref()))
            .cloned()
            .collect();

        for iface_fqcn in &all_ifaces {
            // Collect method names first, then drop the interface guard before calling
            // get_method (which re-enters self.codebase.interfaces when walking ancestors).
            let method_names: Vec<Arc<str>> =
                match self.codebase.interfaces.get(iface_fqcn.as_ref()) {
                    Some(iface) => iface.own_methods.values().map(|m| m.name.clone()).collect(),
                    None => continue,
                };

            for method_name in method_names {
                // PHP method names are case-insensitive; normalize before lookup so that
                // a hand-written stub key like "jsonSerialize" matches the collector's
                // lowercased key "jsonserialize" stored in own_methods.
                let method_name_lower = method_name.to_lowercase();
                // Check if the class provides a concrete implementation
                let implemented = self
                    .codebase
                    .get_method(fqcn.as_ref(), &method_name_lower)
                    .map(|m| !m.is_abstract)
                    .unwrap_or(false);

                if !implemented {
                    let loc = issue_location(
                        cls.location.as_ref(),
                        fqcn,
                        cls.location
                            .as_ref()
                            .and_then(|l| self.sources.get(&l.file).copied()),
                    );
                    let mut issue = Issue::new(
                        IssueKind::UnimplementedInterfaceMethod {
                            class: fqcn.to_string(),
                            interface: iface_fqcn.to_string(),
                            method: method_name.to_string(),
                        },
                        loc,
                    );
                    if let Some(snippet) = extract_snippet(cls.location.as_ref(), &self.sources) {
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

    fn check_overrides(&self, cls: &mir_codebase::storage::ClassStorage, issues: &mut Vec<Issue>) {
        let fqcn = &cls.fqcn;

        for (method_name, own_method) in &cls.own_methods {
            // PHP does not enforce constructor signature compatibility
            if method_name.as_ref() == "__construct" {
                continue;
            }

            // Find parent definition (if any) — search ancestor chain
            let parent_method = self.find_parent_method(cls, method_name.as_ref());

            let parent = match parent_method {
                Some(m) => m,
                None => continue, // not an override
            };

            let loc = issue_location(
                own_method.location.as_ref(),
                fqcn,
                own_method
                    .location
                    .as_ref()
                    .and_then(|l| self.sources.get(&l.file).copied()),
            );

            // ---- a. Cannot override a final method -------------------------
            if parent.is_final {
                let mut issue = Issue::new(
                    IssueKind::FinalMethodOverridden {
                        class: fqcn.to_string(),
                        method: method_name.to_string(),
                        parent: parent.fqcn.to_string(),
                    },
                    loc.clone(),
                );
                if let Some(snippet) = extract_snippet(own_method.location.as_ref(), &self.sources)
                {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }

            // ---- b. Visibility must not be reduced -------------------------
            if visibility_reduced(own_method.visibility, parent.visibility) {
                let mut issue = Issue::new(
                    IssueKind::OverriddenMethodAccess {
                        class: fqcn.to_string(),
                        method: method_name.to_string(),
                    },
                    loc.clone(),
                );
                if let Some(snippet) = extract_snippet(own_method.location.as_ref(), &self.sources)
                {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }

            // ---- c. Return type must be covariant --------------------------
            // Only check when both sides have an explicit return type.
            // Skip when:
            //   - Parent type is from a docblock (PHP doesn't enforce docblock override compat)
            //   - Either type contains a named object (needs codebase for inheritance check)
            //   - Either type contains TSelf/TStaticObject (always compatible with self)
            if let (Some(child_ret), Some(parent_ret)) =
                (&own_method.return_type, &parent.return_type)
            {
                let parent_from_docblock = parent_ret.from_docblock;
                let involves_named_objects = self.type_has_named_objects(child_ret)
                    || self.type_has_named_objects(parent_ret);
                let involves_self_static = self.type_has_self_or_static(child_ret)
                    || self.type_has_self_or_static(parent_ret);

                if !parent_from_docblock
                    && !involves_named_objects
                    && !involves_self_static
                    && !child_ret.is_subtype_of_simple(parent_ret)
                    && !parent_ret.is_mixed()
                    && !child_ret.is_mixed()
                    && !self.return_type_has_template(parent_ret)
                {
                    issues.push(
                        Issue::new(
                            IssueKind::MethodSignatureMismatch {
                                class: fqcn.to_string(),
                                method: method_name.to_string(),
                                detail: format!(
                                    "return type '{}' is not a subtype of parent '{}'",
                                    child_ret, parent_ret
                                ),
                            },
                            loc.clone(),
                        )
                        .with_snippet(method_name.to_string()),
                    );
                }
            }

            // ---- d. Required param count must not increase -----------------
            let parent_required = parent
                .params
                .iter()
                .filter(|p| !p.is_optional && !p.is_variadic)
                .count();
            let child_required = own_method
                .params
                .iter()
                .filter(|p| !p.is_optional && !p.is_variadic)
                .count();

            if child_required > parent_required {
                issues.push(
                    Issue::new(
                        IssueKind::MethodSignatureMismatch {
                            class: fqcn.to_string(),
                            method: method_name.to_string(),
                            detail: format!(
                                "overriding method requires {} argument(s) but parent requires {}",
                                child_required, parent_required
                            ),
                        },
                        loc.clone(),
                    )
                    .with_snippet(method_name.to_string()),
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
            let shared_len = parent.params.len().min(own_method.params.len());
            for i in 0..shared_len {
                let parent_param = &parent.params[i];
                let child_param = &own_method.params[i];

                let (parent_ty, child_ty) = match (&parent_param.ty, &child_param.ty) {
                    (Some(p), Some(c)) => (p, c),
                    _ => continue,
                };

                if parent_ty.is_mixed()
                    || child_ty.is_mixed()
                    || self.type_has_named_objects(parent_ty)
                    || self.type_has_named_objects(child_ty)
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
                                method: method_name.to_string(),
                                detail: format!(
                                    "parameter ${} type '{}' is narrower than parent type '{}'",
                                    child_param.name, child_ty, parent_ty
                                ),
                            },
                            loc.clone(),
                        )
                        .with_snippet(method_name.to_string()),
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
            Atomic::TClassString(Some(inner)) => !self.codebase.type_exists(inner.as_ref()),
            Atomic::TNamedObject { fqcn, type_params } => {
                // Bare name with no namespace separator is likely a template param
                (!fqcn.contains('\\') && !self.codebase.type_exists(fqcn.as_ref()))
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
    fn type_has_named_objects(&self, ty: &mir_types::Union) -> bool {
        use mir_types::Atomic;
        ty.types.iter().any(|a| match a {
            Atomic::TNamedObject { .. } => true,
            Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
                self.type_has_named_objects(key) || self.type_has_named_objects(value)
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                self.type_has_named_objects(value)
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

    /// Find a method with the given name in the closest ancestor (not the class itself).
    fn find_parent_method(
        &self,
        cls: &mir_codebase::storage::ClassStorage,
        method_name: &str,
    ) -> Option<Arc<MethodStorage>> {
        // Walk all_parents in order (closest ancestor first)
        for ancestor_fqcn in &cls.all_parents {
            if let Some(ancestor_cls) = self.codebase.classes.get(ancestor_fqcn.as_ref()) {
                if let Some(m) = ancestor_cls.own_methods.get(method_name) {
                    return Some(Arc::clone(m));
                }
            } else if let Some(iface) = self.codebase.interfaces.get(ancestor_fqcn.as_ref()) {
                if let Some(m) = iface.own_methods.get(method_name) {
                    return Some(Arc::clone(m));
                }
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
            .codebase
            .classes
            .iter()
            .map(|e| e.key().clone())
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
                        let cls = self.codebase.classes.get(offender.as_ref());
                        let loc = issue_location(
                            cls.as_ref().and_then(|c| c.location.as_ref()),
                            offender,
                            cls.as_ref()
                                .and_then(|c| c.location.as_ref())
                                .and_then(|l| self.sources.get(&l.file).copied()),
                        );
                        let mut issue = Issue::new(
                            IssueKind::CircularInheritance {
                                class: offender.to_string(),
                            },
                            loc,
                        );
                        if let Some(snippet) = extract_snippet(
                            cls.as_ref().and_then(|c| c.location.as_ref()),
                            &self.sources,
                        ) {
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
                    .codebase
                    .classes
                    .get(current.as_ref())
                    .and_then(|c| c.parent.clone());

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
            .codebase
            .interfaces
            .iter()
            .map(|e| e.key().clone())
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
                let iface = self.codebase.interfaces.get(offender.as_ref());
                let loc = issue_location(
                    iface.as_ref().and_then(|i| i.location.as_ref()),
                    offender,
                    iface
                        .as_ref()
                        .and_then(|i| i.location.as_ref())
                        .and_then(|l| self.sources.get(&l.file).copied()),
                );
                let mut issue = Issue::new(
                    IssueKind::CircularInheritance {
                        class: offender.to_string(),
                    },
                    loc,
                );
                if let Some(snippet) = extract_snippet(
                    iface.as_ref().and_then(|i| i.location.as_ref()),
                    &self.sources,
                ) {
                    issue = issue.with_snippet(snippet);
                }
                issues.push(issue);
            }
            return;
        }

        stack_set.insert(fqcn.to_string());
        in_stack.push(fqcn.clone());

        let extends = self
            .codebase
            .interfaces
            .get(fqcn.as_ref())
            .map(|i| i.extends.clone())
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
        self.codebase
            .classes
            .get(fqcn.as_ref())
            .map(|c| {
                c.location
                    .as_ref()
                    .map(|loc| self.analyzed_files.contains(&loc.file))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    fn iface_in_analyzed_files(&self, fqcn: &Arc<str>) -> bool {
        if self.analyzed_files.is_empty() {
            return true;
        }
        self.codebase
            .interfaces
            .get(fqcn.as_ref())
            .map(|i| {
                i.location
                    .as_ref()
                    .map(|loc| self.analyzed_files.contains(&loc.file))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
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

/// Build an issue location from the stored codebase Location (which carries line/col as
/// Unicode char-count columns). Falls back to a dummy location using the FQCN as the file
/// path when no Location is stored.
fn issue_location(
    storage_loc: Option<&mir_codebase::storage::Location>,
    fqcn: &Arc<str>,
    source: Option<&str>,
) -> Location {
    match storage_loc {
        Some(loc) => {
            // Calculate col_end from the end byte offset if source is available.
            let col_end = if let Some(src) = source {
                if loc.end > loc.start {
                    let end_offset = (loc.end as usize).min(src.len());
                    // Find the line start containing the end offset.
                    let line_start = src[..end_offset].rfind('\n').map(|p| p + 1).unwrap_or(0);
                    // Count Unicode chars from line start to end offset.
                    let col_end = src[line_start..end_offset].chars().count() as u16;

                    // Count Unicode chars from line start to start offset.
                    let col_start_offset = (loc.start as usize).min(src.len());
                    let col_start_line = src[..col_start_offset]
                        .rfind('\n')
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    let col_start = src[col_start_line..col_start_offset].chars().count() as u16;

                    col_end.max(col_start + 1)
                } else {
                    // Single-char span: end = start + 1.
                    let col_start_offset = (loc.start as usize).min(src.len());
                    let col_start_line = src[..col_start_offset]
                        .rfind('\n')
                        .map(|p| p + 1)
                        .unwrap_or(0);
                    src[col_start_line..col_start_offset].chars().count() as u16 + 1
                }
            } else {
                loc.col + 1
            };

            // col_start: use loc.col (already a char-count) or recompute from source.
            let col_start = if let Some(src) = source {
                let col_start_offset = (loc.start as usize).min(src.len());
                let col_start_line = src[..col_start_offset]
                    .rfind('\n')
                    .map(|p| p + 1)
                    .unwrap_or(0);
                src[col_start_line..col_start_offset].chars().count() as u16
            } else {
                loc.col
            };

            Location {
                file: loc.file.clone(),
                line: loc.line,
                col_start,
                col_end,
            }
        }
        None => Location {
            file: fqcn.clone(),
            line: 1,
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
    let start = loc.start as usize;
    let end = loc.end as usize;
    if start >= src.len() {
        return None;
    }
    let end = end.min(src.len());
    let span_text = &src[start..end];
    // Take only the first line to keep the snippet concise.
    let first_line = span_text.lines().next().unwrap_or(span_text);
    Some(first_line.trim().to_string())
}
