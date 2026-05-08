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
        for fqcn in self.db.active_class_node_fqcns() {
            let Some(class_node) = self.db.lookup_class_node(fqcn.as_ref()) else {
                continue;
            };
            if class_node.is_interface(self.db)
                || class_node.is_trait(self.db)
                || class_node.is_enum(self.db)
            {
                continue;
            }
            let fqcn_str = fqcn.as_ref();

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
                    let (file, line) = location_from_storage(&method.location(self.db));
                    issues.push(Issue::new(
                        IssueKind::UnusedMethod {
                            class: fqcn_str.to_string(),
                            method: name.to_string(),
                        },
                        Location {
                            file,
                            line,
                            line_end: line,
                            col_start: 0,
                            col_end: 0,
                        },
                    ));
                }
            }

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
                    let (file, line) = location_from_storage(&prop.location(self.db));
                    issues.push(Issue::new(
                        IssueKind::UnusedProperty {
                            class: fqcn_str.to_string(),
                            property: name.to_string(),
                        },
                        Location {
                            file,
                            line,
                            line_end: line,
                            col_start: 0,
                            col_end: 0,
                        },
                    ));
                }
            }
        }

        // --- Non-referenced free functions ---
        let stub_vfs = StubVfs::new();
        for fqn in self.db.active_function_node_fqns() {
            let Some(node) = self.db.lookup_function_node(fqn.as_ref()) else {
                continue;
            };
            if !node.active(self.db) {
                continue;
            }
            let location = node.location(self.db);
            // Skip PHP built-in and extension functions loaded from stubs —
            // they are not user-defined dead code.
            if let Some(loc) = &location {
                if stub_vfs.is_stub_file(loc.file.as_ref()) {
                    continue;
                }
            }
            if !self.db.has_reference(fqn.as_ref()) {
                let (file, line) = location_from_storage(&location);
                issues.push(Issue::new(
                    IssueKind::UnusedFunction {
                        name: node.short_name(self.db).to_string(),
                    },
                    Location {
                        file,
                        line,
                        line_end: line,
                        col_start: 0,
                        col_end: 0,
                    },
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

fn location_from_storage(
    loc: &Option<mir_codebase::storage::Location>,
) -> (std::sync::Arc<str>, u32) {
    match loc {
        Some(l) => (l.file.clone(), 1), // byte offset → line mapping not available here
        None => (std::sync::Arc::from("<unknown>"), 1),
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
        let salsa = salsa.lock();
        let issues = DeadCodeAnalyzer::new(&salsa.0).analyze();
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
