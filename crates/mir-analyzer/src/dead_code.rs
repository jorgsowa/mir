/// Dead-code detector (M18).
///
/// After Pass 2 has recorded all method/property/function references into the
/// codebase, this analyzer walks every class and reports:
///
/// - `UnusedMethod`   — private method that is never called
/// - `UnusedProperty` — private property that is never read
/// - `UnusedFunction` — non-public free function that is never called
///
/// Magic methods (`__construct`, `__destruct`, `__toString`, etc.) and
/// constructors are excluded because they are called implicitly.
use std::sync::Arc;

use mir_codebase::storage::Visibility;
use mir_issues::{Issue, IssueKind, Location, Severity};

use crate::db::MirDatabase;
use crate::stubs::StubVfs;

// Magic PHP methods that are invoked implicitly — never flag these as unused.
const MAGIC_METHODS: &[&str] = &[
    "__construct",
    "__destruct",
    "__call",
    "__callstatic",
    "__get",
    "__set",
    "__isset",
    "__unset",
    "__sleep",
    "__wakeup",
    "__serialize",
    "__unserialize",
    "__tostring",
    "__invoke",
    "__set_state",
    "__clone",
    "__debuginfo",
];

pub struct DeadCodeAnalyzer<'a> {
    db: &'a dyn MirDatabase,
}

impl<'a> DeadCodeAnalyzer<'a> {
    pub fn new(db: &'a dyn MirDatabase) -> Self {
        Self { db }
    }

    pub fn analyze(&self) -> Vec<Issue> {
        let mut issues = Vec::new();

        // --- Private methods / properties on classes ---
        // Walk only class-kind nodes (not interfaces/traits/enums); private
        // members on the other kinds aren't subject to dead-code reporting.
        //
        // Phase 4: enumerate via workspace_classes + push-path
        // active_class_node_fqcns for completeness; dedupe by FQCN.
        // Phase 5 drops the push-path leg.
        let pull_classes: Vec<Arc<str>> = crate::db::workspace_classes(self.db)
            .iter()
            .cloned()
            .collect();
        let push_classes: Vec<Arc<str>> = self.db.active_class_node_fqcns();
        let mut seen_classes: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();
        let class_fqcns: Vec<Arc<str>> = pull_classes
            .into_iter()
            .chain(push_classes)
            .filter(|f| seen_classes.insert(f.clone()))
            .collect();

        for fqcn in &class_fqcns {
            // Prefer the pull-path snapshot; if missing, fall back to the
            // push-path node. Both yield the same is-class predicate and
            // own_methods / own_properties iteration shape.
            let here = crate::db::Fqcn::new(self.db, fqcn.clone());
            let pulled = crate::db::find_class_like(self.db, here);
            let is_class = match pulled.as_ref() {
                Some(c) => c.is_class(),
                None => self
                    .db
                    .lookup_class_node(fqcn.as_ref())
                    .map(|n| {
                        !n.is_interface(self.db) && !n.is_trait(self.db) && !n.is_enum(self.db)
                    })
                    .unwrap_or(false),
            };
            if !is_class {
                continue;
            }
            let fqcn_str = fqcn.as_ref();

            // Methods.
            if let Some(class) = pulled.as_ref() {
                for (name, method) in class.own_methods().iter() {
                    if method.visibility != Visibility::Private {
                        continue;
                    }
                    let name_lower = name.to_ascii_lowercase();
                    if MAGIC_METHODS.contains(&name_lower.as_str()) {
                        continue;
                    }
                    if !self.db.has_reference(&format!(
                        "{}::{}",
                        fqcn_str,
                        name.to_ascii_lowercase()
                    )) {
                        let location = location_from_storage(&method.location);
                        issues.push(Issue::new(
                            IssueKind::UnusedMethod {
                                class: fqcn_str.to_string(),
                                method: name.to_string(),
                            },
                            location,
                        ));
                    }
                }
            } else {
                for method in self.db.class_own_methods(fqcn_str) {
                    if !method.active(self.db) {
                        continue;
                    }
                    if method.visibility(self.db) != Visibility::Private {
                        continue;
                    }
                    let name = method.name(self.db);
                    let name_lower = name.to_lowercase();
                    if MAGIC_METHODS.contains(&name_lower.as_str()) {
                        continue;
                    }
                    if !self
                        .db
                        .has_reference(&format!("{}::{}", fqcn_str, name.to_lowercase()))
                    {
                        let location = location_from_storage(&method.location(self.db));
                        issues.push(Issue::new(
                            IssueKind::UnusedMethod {
                                class: fqcn_str.to_string(),
                                method: name.to_string(),
                            },
                            location,
                        ));
                    }
                }
            }

            // Properties.
            if let Some(class) = pulled.as_ref() {
                if let Some(props) = class.own_properties() {
                    for (name, prop) in props.iter() {
                        if prop.visibility != Visibility::Private {
                            continue;
                        }
                        if !self
                            .db
                            .has_reference(&format!("{}::{}", fqcn_str, name.as_ref()))
                        {
                            let location = location_from_storage(&prop.location);
                            issues.push(Issue::new(
                                IssueKind::UnusedProperty {
                                    class: fqcn_str.to_string(),
                                    property: name.to_string(),
                                },
                                location,
                            ));
                        }
                    }
                }
            } else {
                for prop in self.db.class_own_properties(fqcn_str) {
                    if !prop.active(self.db) {
                        continue;
                    }
                    if prop.visibility(self.db) != Visibility::Private {
                        continue;
                    }
                    let name = prop.name(self.db);
                    if !self
                        .db
                        .has_reference(&format!("{}::{}", fqcn_str, name.as_ref()))
                    {
                        let location = location_from_storage(&prop.location(self.db));
                        issues.push(Issue::new(
                            IssueKind::UnusedProperty {
                                class: fqcn_str.to_string(),
                                property: name.to_string(),
                            },
                            location,
                        ));
                    }
                }
            }
        }

        // --- Non-referenced free functions ---
        // Phase 4: enumerate via workspace_functions (pull path) and
        // fall back to active_function_node_fqns for fixtures not
        // registered as SourceFiles. Phase 5 removes the fallback.
        let stub_vfs = StubVfs::new();
        let pull_fns: Vec<Arc<str>> = crate::db::workspace_functions(self.db)
            .iter()
            .cloned()
            .collect();
        let push_fns: Vec<Arc<str>> = self.db.active_function_node_fqns();
        let mut seen: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();
        let fqns: Vec<Arc<str>> = pull_fns
            .into_iter()
            .chain(push_fns)
            .filter(|f| seen.insert(f.clone()))
            .collect();
        for fqn in fqns {
            // Prefer the pull-path snapshot for the function's location +
            // short name; fall back to the push-path node.
            let here = crate::db::Fqcn::new(self.db, fqn.clone());
            let pulled = crate::db::find_function(self.db, here);
            let (location, short_name) = if let Some(f) = pulled.as_ref() {
                (f.location.clone(), f.short_name.to_string())
            } else {
                let Some(node) = self.db.lookup_function_node(fqn.as_ref()) else {
                    continue;
                };
                if !node.active(self.db) {
                    continue;
                }
                (node.location(self.db), node.short_name(self.db).to_string())
            };
            // Skip PHP built-in and extension functions loaded from stubs —
            // they are not user-defined dead code.
            if let Some(loc) = &location {
                if stub_vfs.is_stub_file(loc.file.as_ref()) {
                    continue;
                }
            }
            if !self.db.has_reference(fqn.as_ref()) {
                let location = location_from_storage(&location);
                issues.push(Issue::new(
                    IssueKind::UnusedFunction { name: short_name },
                    location,
                ));
            }
        }

        // Downgrade all dead-code issues to Info
        for issue in &mut issues {
            issue.severity = Severity::Info;
        }

        issues
    }
}

fn location_from_storage(loc: &Option<mir_codebase::storage::Location>) -> Location {
    match loc {
        Some(l) => Location {
            file: l.file.clone(),
            line: l.line,
            line_end: l.line_end,
            col_start: l.col_start,
            col_end: l.col_end,
        },
        None => Location {
            file: std::sync::Arc::from("<unknown>"),
            line: 1,
            line_end: 1,
            col_start: 0,
            col_end: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::ProjectAnalyzer;

    #[test]
    fn builtin_functions_not_flagged_as_unused() {
        // The dead-code pass must not produce UnusedFunction for PHP built-ins
        // (strlen, array_map, etc.) even when they are never called in user code.
        // This test bypasses the fixture runner's file-path filter to verify the
        // fix directly on the DeadCodeAnalyzer output.
        let analyzer = ProjectAnalyzer::new();
        analyzer.load_stubs();
        let salsa = analyzer.salsa_db_for_test();
        let issues = DeadCodeAnalyzer::new(&*salsa).analyze();
        let builtin_false_positives: Vec<_> = issues
            .iter()
            .filter(|i| {
                matches!(&i.kind, IssueKind::UnusedFunction { name } if
                    matches!(name.as_str(), "strlen" | "array_map" | "json_encode" | "preg_match")
                )
            })
            .collect();
        assert!(
            builtin_false_positives.is_empty(),
            "Expected no UnusedFunction for PHP builtins, got: {:?}",
            builtin_false_positives
                .iter()
                .map(|i| i.kind.message())
                .collect::<Vec<_>>()
        );
    }
}
