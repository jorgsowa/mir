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
use mir_codebase::Codebase;
use mir_issues::{Issue, IssueKind, Location, Severity};

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
    codebase: &'a Codebase,
}

impl<'a> DeadCodeAnalyzer<'a> {
    pub fn new(codebase: &'a Codebase) -> Self {
        Self { codebase }
    }

    pub fn analyze(&self) -> Vec<Issue> {
        let mut issues = Vec::new();

        // --- Private methods / properties on classes ---
        for entry in self.codebase.classes.iter() {
            let cls = entry.value();
            let fqcn = cls.fqcn.as_ref();

            for (method_name, method) in &cls.own_methods {
                if method.visibility != Visibility::Private {
                    continue;
                }
                let name = method_name.as_ref();
                if MAGIC_METHODS.contains(&name) {
                    continue;
                }
                if !self.codebase.is_method_referenced(fqcn, name) {
                    let (file, line) = location_from_storage(&method.location);
                    issues.push(Issue::new(
                        IssueKind::UnusedMethod {
                            class: fqcn.to_string(),
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

            for (prop_name, prop) in &cls.own_properties {
                if prop.visibility != Visibility::Private {
                    continue;
                }
                let name = prop_name.as_ref();
                if !self.codebase.is_property_referenced(fqcn, name) {
                    let (file, line) = location_from_storage(&prop.location);
                    issues.push(Issue::new(
                        IssueKind::UnusedProperty {
                            class: fqcn.to_string(),
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
        for entry in self.codebase.functions.iter() {
            let func = entry.value();
            let fqn = func.fqn.as_ref();
            // Skip PHP built-in and extension functions loaded from stubs —
            // they are not user-defined dead code.
            if let Some(loc) = &func.location {
                if stub_vfs.is_stub_file(loc.file.as_ref()) {
                    continue;
                }
            }
            if !self.codebase.is_function_referenced(fqn) {
                let (file, line) = location_from_storage(&func.location);
                issues.push(Issue::new(
                    IssueKind::UnusedFunction {
                        name: func.short_name.to_string(),
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
        let issues = DeadCodeAnalyzer::new(analyzer.codebase()).analyze();
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
