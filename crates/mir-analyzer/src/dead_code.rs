/// Dead-code detector (M18).
///
/// After body analysis has recorded all method/property/function references into the
/// codebase, this analyzer walks every class and reports:
///
/// - `UnusedMethod`   — private method that is never called
/// - `UnusedProperty` — private property that is never read
/// - `UnusedFunction` — non-public free function that is never called
///
/// Magic methods (`__construct`, `__destruct`, `__toString`, etc.) and
/// constructors are excluded because they are called implicitly.
use std::sync::Arc;

use mir_codebase::definitions::Visibility;
use mir_issues::{Issue, IssueKind, Severity};

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
    /// Only report dead code defined in these files (empty = all files).
    /// Threading the project file set keeps the pass from materializing and
    /// flagging private members of lazily-loaded vendor classes (whose bodies
    /// are never analyzed, so every private member looks unreferenced).
    analyzed_files: rustc_hash::FxHashSet<Arc<str>>,
}

impl<'a> DeadCodeAnalyzer<'a> {
    #[allow(dead_code)]
    pub fn new(db: &'a dyn MirDatabase) -> Self {
        Self {
            db,
            analyzed_files: rustc_hash::FxHashSet::default(),
        }
    }

    pub fn with_files(
        db: &'a dyn MirDatabase,
        analyzed_files: rustc_hash::FxHashSet<Arc<str>>,
    ) -> Self {
        Self { db, analyzed_files }
    }

    pub fn analyze(&self) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Walk only plain classes (not interfaces/traits/enums); private
        // members on the other kinds aren't subject to dead-code reporting.
        // `analyzed_class_defs` already restricts to the project file set and
        // returns plain classes only, so vendor classes are never enumerated.
        for (fqcn, class) in crate::db::analyzed_class_defs(self.db, &self.analyzed_files) {
            let fqcn_str = fqcn.as_ref();

            // Methods.
            for (name, method) in class.own_methods().iter() {
                if method.visibility != Visibility::Private {
                    continue;
                }
                let name_lower = name.to_ascii_lowercase();
                if MAGIC_METHODS.contains(&name_lower.as_str()) {
                    continue;
                }
                if !self
                    .db
                    .has_reference(&format!("meth:{}::{}", fqcn_str, name_lower))
                {
                    let location =
                        crate::diagnostics::storage_loc_to_location(method.location.as_ref());
                    issues.push(Issue::new(
                        IssueKind::UnusedMethod {
                            class: fqcn_str.to_string(),
                            method: name.to_string(),
                        },
                        location,
                    ));
                }
            }

            // Properties.
            if let Some(props) = class.own_properties() {
                for (name, prop) in props.iter() {
                    if prop.visibility != Visibility::Private {
                        continue;
                    }
                    if !self
                        .db
                        .has_reference(&format!("prop:{}::{}", fqcn_str, name.as_ref()))
                    {
                        let location =
                            crate::diagnostics::storage_loc_to_location(prop.location.as_ref());
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

        // --- Non-referenced classes ---
        let stub_vfs = StubVfs::new();
        for (fqcn, class) in crate::db::analyzed_class_defs(self.db, &self.analyzed_files) {
            let fqcn_str = fqcn.as_ref();

            // Skip abstract and non-final classes — they may be used via inheritance or type hints
            // in ways the reference tracker doesn't capture. Only flag final classes, which are
            // true leaf types that can only be used by being directly instantiated/referenced.
            if class.is_abstract() || !class.is_final() {
                continue;
            }

            // Skip vendor/stub files.
            let location = match &class {
                crate::db::ClassLike::Class(c) => c.location.clone(),
                _ => continue,
            };
            if let Some(loc) = &location {
                if stub_vfs.is_stub_file(loc.file.as_ref()) {
                    continue;
                }
                if !self.analyzed_files.is_empty()
                    && !self.analyzed_files.contains(loc.file.as_ref())
                {
                    continue;
                }
            }

            if !self.db.has_reference(&format!("cls:{fqcn_str}")) {
                let loc = crate::diagnostics::storage_loc_to_location(location.as_ref());
                issues.push(Issue::new(
                    IssueKind::UnusedClass {
                        class: fqcn_str.to_string(),
                    },
                    loc,
                ));
            }
        }

        // --- Non-referenced free functions ---
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
                // Restrict to the project file set (when scoped), mirroring the
                // class-member pass: a function from a lazily-loaded vendor file
                // is not user dead code.
                if !self.analyzed_files.is_empty()
                    && !self.analyzed_files.contains(loc.file.as_ref())
                {
                    continue;
                }
            }
            if !self.db.has_reference(&format!("fn:{}", fqn.as_ref())) {
                let location = crate::diagnostics::storage_loc_to_location(location.as_ref());
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
        session.ensure_all_stubs();
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
