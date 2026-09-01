use super::*;

impl<'a> ClassAnalyzer<'a> {
    pub(super) fn check_circular_class_inheritance(&self, issues: &mut Vec<Issue>) {
        let mut globally_done: HashSet<String> = HashSet::default();

        let mut class_keys: Vec<mir_types::Name> = crate::db::workspace_classes(self.db)
            .iter()
            .filter(|fqcn| {
                let here = crate::db::Fqcn::from_str(self.db, fqcn.as_str());
                crate::db::find_class_like(self.db, here)
                    .map(|c| c.is_class())
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        class_keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        for start_fqcn in &class_keys {
            if globally_done.contains(start_fqcn.as_str()) {
                continue;
            }

            // Walk the parent chain, tracking order for cycle reporting.
            let mut chain: Vec<Arc<str>> = Vec::new();
            let mut chain_set: HashSet<String> = HashSet::default();
            let mut current: Arc<str> = Arc::from(start_fqcn.as_str());

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

    pub(super) fn check_circular_interface_inheritance(&self, issues: &mut Vec<Issue>) {
        let mut globally_done: HashSet<String> = HashSet::default();

        let mut iface_keys: Vec<mir_types::Name> = crate::db::workspace_classes(self.db)
            .iter()
            .filter(|fqcn| {
                let here = crate::db::Fqcn::from_str(self.db, fqcn.as_str());
                crate::db::find_class_like(self.db, here)
                    .map(|c| c.is_interface())
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        iface_keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        for start_fqcn in &iface_keys {
            if globally_done.contains(start_fqcn.as_str()) {
                continue;
            }
            let mut in_stack: Vec<Arc<str>> = Vec::new();
            let mut stack_set: HashSet<String> = HashSet::default();
            self.dfs_interface_cycle(
                Arc::from(start_fqcn.as_str()),
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

    // -----------------------------------------------------------------------
    // Check: circular trait composition (trait A { use B; } trait B { use A; })
    // -----------------------------------------------------------------------

    pub(super) fn check_circular_trait_composition(&self, issues: &mut Vec<Issue>) {
        let mut globally_done: HashSet<String> = HashSet::default();

        let mut trait_keys: Vec<mir_types::Name> = crate::db::workspace_classes(self.db)
            .iter()
            .filter(|fqcn| {
                let here = crate::db::Fqcn::from_str(self.db, fqcn.as_str());
                crate::db::find_class_like(self.db, here)
                    .map(|c| c.is_trait())
                    .unwrap_or(false)
            })
            .copied()
            .collect();
        trait_keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        for start_fqcn in &trait_keys {
            if globally_done.contains(start_fqcn.as_str()) {
                continue;
            }
            let mut in_stack: Vec<Arc<str>> = Vec::new();
            let mut stack_set: HashSet<String> = HashSet::default();
            self.dfs_trait_cycle(
                Arc::from(start_fqcn.as_str()),
                &mut in_stack,
                &mut stack_set,
                &mut globally_done,
                issues,
            );
        }
    }

    fn dfs_trait_cycle(
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
                .filter(|n| self.class_in_analyzed_files(n))
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
                let short_name = offender.rsplit('\\').next().unwrap_or(offender.as_ref());
                let mut issue = Issue::new(
                    IssueKind::InvalidTraitUse {
                        trait_name: short_name.to_string(),
                        reason: format!("{short_name} has a circular trait composition chain"),
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
        let used_traits: Vec<Arc<str>> = crate::db::find_class_like(self.db, here)
            .map(|c| c.class_traits().to_vec())
            .unwrap_or_default();

        for used in used_traits {
            self.dfs_trait_cycle(used, in_stack, stack_set, globally_done, issues);
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

    pub(super) fn check_missing_constructor(
        &self,
        fqcn: &Arc<str>,
        location: Option<&Location>,
        issues: &mut Vec<Issue>,
    ) {
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        let ancestors = crate::db::class_ancestors_by_fqcn(self.db, here);
        // Index (in `ancestors`, self-first) of the class whose constructor
        // actually runs for `new fqcn()`. That constructor can only reach
        // properties declared at or above its own class in the chain — it
        // can't initialize a property a more-derived subclass added later,
        // even though PHP still resolves it as the effective constructor
        // (e.g. a shared base class with a `setUp()`-style subclass convention).
        // A mixin-provided constructor isn't positioned in `ancestors` at
        // all; treat that as covering everything, matching prior behavior.
        let ctor_idx = crate::db::find_method_in_chain(self.db, here, "__construct")
            .map(|(owner, _)| ancestors.iter().position(|a| *a == owner).unwrap_or(0));
        let has_uninitialized = ancestors.iter().enumerate().any(|(idx, ancestor)| {
            if ctor_idx.is_some_and(|ci| ci <= idx) {
                return false;
            }
            let anc_here = crate::db::Fqcn::from_str(self.db, ancestor.as_ref());
            if let Some(class) = crate::db::find_class_like(self.db, anc_here) {
                if let Some(props) = class.own_properties() {
                    return props.values().any(|p| {
                         // A hooked property is virtual when default-less (its
                         // value is computed on access), so it never forces a
                         // constructor.
                         !p.has_hook
                              && p.has_native_type
                               && p.default.is_none()
                               && p.ty.as_deref().is_some_and(|ty| !ty.is_nullable())
                    });
                }
            }
            false
        });
        if !has_uninitialized {
            return;
        }
        let loc = issue_location(
            location,
            location.and_then(|l| self.sources.get(&l.file).copied()),
        );
        let mut issue = Issue::new(
            IssueKind::MissingConstructor {
                class: fqcn.to_string(),
            },
            loc,
        );
        if let Some(snippet) = extract_snippet(location, &self.sources) {
            issue = issue.with_snippet(snippet);
        }
        issues.push(issue);
    }
}
