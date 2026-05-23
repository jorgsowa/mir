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

        // Walk only plain classes (not interfaces/traits/enums); private
        // members on the other kinds aren't subject to dead-code reporting.
        let class_fqcns: Vec<Arc<str>> = crate::db::workspace_classes(self.db)
            .iter()
            .cloned()
            .collect();

        for fqcn in &class_fqcns {
            let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
            let pulled = crate::db::find_class_like(self.db, here);
            let is_class = pulled.as_ref().map(|c| c.is_class()).unwrap_or(false);
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
            }
        }

        // --- Non-referenced free functions ---
        let stub_vfs = StubVfs::new();
        let fqns: Vec<Arc<str>> = crate::db::workspace_functions(self.db)
            .iter()
            .cloned()
            .collect();
        for fqn in fqns {
            let here = crate::db::Fqcn::from_str(self.db, fqn.as_ref());
            let pulled = crate::db::find_function(self.db, here);
            let Some(f) = pulled.as_ref() else {
                continue;
            };
            let (location, short_name) = (f.location.clone(), f.short_name.to_string());
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
    use crate::session::AnalysisSession;
    use crate::PhpVersion;

    #[test]
    fn builtin_functions_not_flagged_as_unused() {
        // The dead-code pass must not produce UnusedFunction for PHP built-ins
        // (strlen, array_map, etc.) even when they are never called in user code.
        // This test bypasses the fixture runner's file-path filter to verify the
        // fix directly on the DeadCodeAnalyzer output.
        let session = AnalysisSession::new(PhpVersion::LATEST);
        session.ensure_all_stubs_loaded();
        let db = session.snapshot_db();
        let issues = DeadCodeAnalyzer::new(&db).analyze();
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
