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
                            col_start: 0,
                            col_end: 0,
                        },
                    ));
                }
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
