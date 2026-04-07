/// Class analyzer — validates class definitions after codebase finalization.
///
/// Checks performed (all codebase-level, no AST required):
///   - Concrete class implements all abstract parent methods
///   - Concrete class implements all interface methods
///   - Overriding method does not reduce visibility
///   - Overriding method return type is covariant with parent
///   - Overriding method does not override a final method
///   - Class does not extend a final class
use std::collections::HashSet;
use std::sync::Arc;

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
}

impl<'a> ClassAnalyzer<'a> {
    pub fn new(codebase: &'a Codebase) -> Self {
        Self {
            codebase,
            analyzed_files: HashSet::new(),
        }
    }

    pub fn with_files(codebase: &'a Codebase, files: HashSet<Arc<str>>) -> Self {
        Self {
            codebase,
            analyzed_files: files,
        }
    }

    /// Run all class-level checks and return every discovered issue.
    pub fn analyze_all(&self) -> Vec<Issue> {
        let mut issues = Vec::new();

        let class_keys: Vec<Arc<str>> = self
            .codebase
            .classes
            .iter()
            .map(|e| e.key().clone())
            .collect();

        for fqcn in &class_keys {
            let cls = match self.codebase.classes.get(fqcn.as_ref()) {
                Some(c) => c,
                None => continue,
            };

            // Skip classes from vendor / stub files — only check user-analyzed files
            if !self.analyzed_files.is_empty() {
                let in_analyzed = cls
                    .location
                    .as_ref()
                    .map(|loc| self.analyzed_files.contains(&loc.file))
                    .unwrap_or(false);
                if !in_analyzed {
                    continue;
                }
            }

            let loc = dummy_location(fqcn);

            // ---- 1. Final-class extension check --------------------------------
            if let Some(parent_fqcn) = &cls.parent {
                if let Some(parent) = self.codebase.classes.get(parent_fqcn.as_ref()) {
                    if parent.is_final {
                        issues.push(Issue::new(
                            IssueKind::FinalClassExtended {
                                parent: parent_fqcn.to_string(),
                                child: fqcn.to_string(),
                            },
                            loc.clone(),
                        ));
                    }
                }
            }

            // Skip abstract classes for "must implement" checks
            if cls.is_abstract {
                // Still check override compatibility for abstract classes
                self.check_overrides(&cls, &mut issues);
                continue;
            }

            // ---- 2. Abstract parent methods must be implemented ----------------
            self.check_abstract_methods_implemented(&cls, &mut issues);

            // ---- 3. Interface methods must be implemented ----------------------
            self.check_interface_methods_implemented(&cls, &mut issues);

            // ---- 4. Method override compatibility ------------------------------
            self.check_overrides(&cls, &mut issues);
        }

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
            let ancestor = match self.codebase.classes.get(ancestor_fqcn.as_ref()) {
                Some(a) => a,
                None => continue,
            };

            for (method_name, method) in &ancestor.own_methods {
                if !method.is_abstract {
                    continue;
                }

                // Check if the concrete class (or any closer ancestor) provides it
                if cls
                    .get_method(method_name.as_ref())
                    .map(|m| !m.is_abstract)
                    .unwrap_or(false)
                {
                    continue; // implemented
                }

                issues.push(Issue::new(
                    IssueKind::UnimplementedAbstractMethod {
                        class: fqcn.to_string(),
                        method: method_name.to_string(),
                    },
                    dummy_location(fqcn),
                ));
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
            let iface = match self.codebase.interfaces.get(iface_fqcn.as_ref()) {
                Some(i) => i,
                None => continue,
            };

            for (method_name, _method) in &iface.own_methods {
                // Check if the class provides a concrete implementation
                let implemented = cls
                    .get_method(method_name.as_ref())
                    .map(|m| !m.is_abstract)
                    .unwrap_or(false);

                if !implemented {
                    issues.push(Issue::new(
                        IssueKind::UnimplementedInterfaceMethod {
                            class: fqcn.to_string(),
                            interface: iface_fqcn.to_string(),
                            method: method_name.to_string(),
                        },
                        dummy_location(fqcn),
                    ));
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Check: override compatibility
    // -----------------------------------------------------------------------

    fn check_overrides(&self, cls: &mir_codebase::storage::ClassStorage, issues: &mut Vec<Issue>) {
        let fqcn = &cls.fqcn;
        // Use the actual source file if available, otherwise fall back to fqcn.
        let class_file: Arc<str> = cls
            .location
            .as_ref()
            .map(|l| l.file.clone())
            .unwrap_or_else(|| fqcn.clone());

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

            let loc = Location {
                file: class_file.clone(),
                line: 1,
                col_start: 0,
                col_end: 0,
            };

            // ---- a. Cannot override a final method -------------------------
            if parent.is_final {
                issues.push(Issue::new(
                    IssueKind::FinalMethodOverridden {
                        class: fqcn.to_string(),
                        method: method_name.to_string(),
                        parent: parent.fqcn.to_string(),
                    },
                    loc.clone(),
                ));
            }

            // ---- b. Visibility must not be reduced -------------------------
            if visibility_reduced(own_method.visibility, parent.visibility) {
                issues.push(Issue::new(
                    IssueKind::OverriddenMethodAccess {
                        class: fqcn.to_string(),
                        method: method_name.to_string(),
                    },
                    loc.clone(),
                ));
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
    ) -> Option<MethodStorage> {
        // Walk all_parents in order (closest ancestor first)
        for ancestor_fqcn in &cls.all_parents {
            if let Some(ancestor_cls) = self.codebase.classes.get(ancestor_fqcn.as_ref()) {
                if let Some(m) = ancestor_cls.own_methods.get(method_name) {
                    return Some(m.clone());
                }
            } else if let Some(iface) = self.codebase.interfaces.get(ancestor_fqcn.as_ref()) {
                if let Some(m) = iface.own_methods.get(method_name) {
                    return Some(m.clone());
                }
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

/// Create a placeholder location (class-level issues don't have a precise span yet).
fn dummy_location(fqcn: &Arc<str>) -> Location {
    Location {
        file: fqcn.clone(),
        line: 1,
        col_start: 0,
        col_end: 0,
    }
}
