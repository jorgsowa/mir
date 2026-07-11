/// Class analyzer — validates class definitions after codebase finalization.
///
/// Checks performed (all codebase-level, no AST required):
///   - Concrete class implements all abstract parent methods
///   - Concrete class implements all interface methods
///   - Overriding method does not reduce visibility
///   - Overriding method return type is covariant with parent
///   - Overriding method does not override a final method
///   - Class does not extend a final class
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::sync::Arc;

use mir_codebase::definitions::Visibility;
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
        class_ancestors(self.db, crate::db::Fqcn::from_str(self.db, fqcn)).0
    }

    /// Build an `Issue` of `kind` at `location` (with a source snippet when
    /// available) and push it. Centralizes the location/snippet boilerplate
    /// repeated by every check in `analyze_all`.
    fn push_located_issue(
        &self,
        issues: &mut Vec<Issue>,
        kind: IssueKind,
        location: Option<&mir_types::Location>,
    ) {
        let source = location.and_then(|l| self.sources.get(&l.file).copied());
        let loc = issue_location(location, source);
        let mut issue = Issue::new(kind, loc);
        if let Some(snippet) = extract_snippet(location, &self.sources) {
            issue = issue.with_snippet(snippet);
        }
        issues.push(issue);
    }

    /// Run all class-level checks and return every discovered issue.
    pub fn analyze_all(&self) -> Vec<Issue> {
        let mut issues = Vec::new();

        // Only plain classes defined in the analyzed file set, already sorted by
        // FQCN. Decomposing per file via `collect_file_definitions` means vendor
        // / stub classes are never materialized (they aren't in `analyzed_files`),
        // which is the dominant cost on cold start over a large `vendor/`.
        for (fqcn, class) in crate::db::analyzed_class_defs(self.db, &self.analyzed_files) {
            let fqcn = &fqcn;
            let location: Option<Location> = class.location().cloned();
            let parent_fqcn: Option<Arc<str>> = class.parent().cloned();
            let is_abstract = class.is_abstract();

            // ---- 1. Final-class extension check / deprecated parent check ------
            if let Some(parent_fqcn) = parent_fqcn.as_ref() {
                let parent_here = crate::db::Fqcn::from_str(self.db, parent_fqcn.as_ref());
                let parent_pulled = crate::db::find_class_like(self.db, parent_here);
                let parent_is_final = parent_pulled
                    .as_ref()
                    .map(|c| c.is_final())
                    .unwrap_or(false);
                let parent_deprecated: Option<Arc<str>> =
                    parent_pulled.as_ref().and_then(|c| c.deprecated().cloned());
                if let Some(canonical) = parent_pulled.as_ref() {
                    if let Some((used, canonical_str)) =
                        crate::fqcn_case_mismatch(parent_fqcn.as_ref(), canonical.fqcn().as_ref())
                    {
                        self.push_located_issue(
                            &mut issues,
                            IssueKind::WrongCaseClass {
                                used,
                                canonical: canonical_str,
                            },
                            location.as_ref(),
                        );
                    }
                }
                if parent_pulled.is_some() {
                    if parent_is_final {
                        self.push_located_issue(
                            &mut issues,
                            IssueKind::InvalidExtendClass {
                                parent: parent_fqcn.to_string(),
                                child: fqcn.to_string(),
                            },
                            location.as_ref(),
                        );
                    }
                    if let Some(msg) = parent_deprecated {
                        self.push_located_issue(
                            &mut issues,
                            IssueKind::DeprecatedClass {
                                name: parent_fqcn.to_string(),
                                message: Some(msg).filter(|m| !m.is_empty()),
                            },
                            location.as_ref(),
                        );
                    }
                }
            }

            // ---- 1b. Deprecated interface / trait checks -----------------------
            {
                let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                if let Some(cls) = crate::db::find_class_like(self.db, here) {
                    for iface_fqcn in cls.interfaces() {
                        let iface_here = crate::db::Fqcn::from_str(self.db, iface_fqcn.as_ref());
                        if let Some(iface) = crate::db::find_class_like(self.db, iface_here) {
                            if let Some((used, canonical_str)) = crate::fqcn_case_mismatch(
                                iface_fqcn.as_ref(),
                                iface.fqcn().as_ref(),
                            ) {
                                self.push_located_issue(
                                    &mut issues,
                                    IssueKind::WrongCaseClass {
                                        used,
                                        canonical: canonical_str,
                                    },
                                    location.as_ref(),
                                );
                            }
                            if let Some(msg) = iface.deprecated() {
                                self.push_located_issue(
                                    &mut issues,
                                    IssueKind::DeprecatedInterface {
                                        name: iface_fqcn.to_string(),
                                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                    },
                                    location.as_ref(),
                                );
                            }
                        }
                    }
                    for trait_fqcn in cls.class_traits() {
                        let trait_here = crate::db::Fqcn::from_str(self.db, trait_fqcn.as_ref());
                        if let Some(t) = crate::db::find_class_like(self.db, trait_here) {
                            if let Some((used, canonical_str)) =
                                crate::fqcn_case_mismatch(trait_fqcn.as_ref(), t.fqcn().as_ref())
                            {
                                self.push_located_issue(
                                    &mut issues,
                                    IssueKind::WrongCaseClass {
                                        used,
                                        canonical: canonical_str,
                                    },
                                    location.as_ref(),
                                );
                            }
                            if let Some(msg) = t.deprecated() {
                                self.push_located_issue(
                                    &mut issues,
                                    IssueKind::DeprecatedTrait {
                                        name: trait_fqcn.to_string(),
                                        message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                    },
                                    location.as_ref(),
                                );
                            }
                        }
                    }
                }
            }

            // ---- 1c. Generic type-arg bound checks -----------------------------
            // `@implements`/`@extends` type args were never checked against the
            // target's own `@template T of Bound` — only method/function calls and
            // (as of the `new`-bound fix) constructor-argument inference were.
            // A concrete violation here (e.g. `@implements Container<Unrelated>`
            // where `Container`'s `T` is bounded to `Base`) was silently accepted.
            for (iface, args) in class.implements_type_args() {
                self.check_generic_type_args(iface.as_ref(), args, location.as_ref(), &mut issues);
            }
            if let Some(parent_fqcn) = parent_fqcn.as_ref() {
                let extends_args = class.extends_type_args();
                if !extends_args.is_empty() {
                    self.check_generic_type_args(
                        parent_fqcn.as_ref(),
                        extends_args,
                        location.as_ref(),
                        &mut issues,
                    );
                }
            }

            // Skip abstract classes for "must implement" checks
            if is_abstract {
                // Still check override compatibility for abstract classes
                self.check_overrides(fqcn, location.as_ref(), &mut issues);
                self.check_magic_method_casing(fqcn, &mut issues);
                continue;
            }

            // ---- 2. Abstract parent methods must be implemented ----------------
            self.check_abstract_methods_implemented(fqcn, location.as_ref(), &mut issues);

            // ---- 3. Interface methods must be implemented ----------------------
            self.check_interface_methods_implemented(fqcn, location.as_ref(), &mut issues);

            // ---- 4. Method override compatibility ------------------------------
            self.check_overrides(fqcn, location.as_ref(), &mut issues);

            // ---- 5. Magic method casing ----------------------------------------
            self.check_magic_method_casing(fqcn, &mut issues);

            // ---- 6. Missing constructor ----------------------------------------
            self.check_missing_constructor(fqcn, location.as_ref(), &mut issues);
        }

        // ---- 5. Interface-level #[Override] check + extends casing --------
        // Interfaces are not included in the class loop above, so scan them
        // separately for #[Override] on methods that have no parent interface method.
        for (iface_fqcn, iface) in crate::db::analyzed_interface_defs(self.db, &self.analyzed_files)
        {
            self.check_overrides(&iface_fqcn, None, &mut issues);
            let location = iface.location.clone();
            for parent_iface_fqcn in iface.extends.iter() {
                let here = crate::db::Fqcn::from_str(self.db, parent_iface_fqcn.as_ref());
                if let Some(canonical) = crate::db::find_class_like(self.db, here) {
                    if let Some((used, canonical_str)) = crate::fqcn_case_mismatch(
                        parent_iface_fqcn.as_ref(),
                        canonical.fqcn().as_ref(),
                    ) {
                        self.push_located_issue(
                            &mut issues,
                            IssueKind::WrongCaseClass {
                                used,
                                canonical: canonical_str,
                            },
                            location.as_ref(),
                        );
                    }
                    if let Some(msg) = canonical.deprecated() {
                        self.push_located_issue(
                            &mut issues,
                            IssueKind::DeprecatedInterface {
                                name: parent_iface_fqcn.to_string(),
                                message: Some(msg.clone()).filter(|m| !m.is_empty()),
                            },
                            location.as_ref(),
                        );
                    }
                }
            }
            self.check_magic_method_casing(&iface_fqcn, &mut issues);
        }

        // ---- 5b. Enum-level implements casing + magic methods ---------------
        for (enum_fqcn, enum_def) in crate::db::analyzed_enum_defs(self.db, &self.analyzed_files) {
            let location = enum_def.location.clone();
            for iface_fqcn in enum_def.interfaces.iter() {
                let here = crate::db::Fqcn::from_str(self.db, iface_fqcn.as_ref());
                if let Some(canonical) = crate::db::find_class_like(self.db, here) {
                    if let Some((used, canonical_str)) =
                        crate::fqcn_case_mismatch(iface_fqcn.as_ref(), canonical.fqcn().as_ref())
                    {
                        self.push_located_issue(
                            &mut issues,
                            IssueKind::WrongCaseClass {
                                used,
                                canonical: canonical_str,
                            },
                            location.as_ref(),
                        );
                    }
                    if let Some(msg) = canonical.deprecated() {
                        self.push_located_issue(
                            &mut issues,
                            IssueKind::DeprecatedInterface {
                                name: iface_fqcn.to_string(),
                                message: Some(msg.clone()).filter(|m| !m.is_empty()),
                            },
                            location.as_ref(),
                        );
                    }
                }
            }
            self.check_magic_method_casing(&enum_fqcn, &mut issues);

            // ---- Enum interface methods must be implemented --------------------
            self.check_enum_interface_methods_implemented(
                &enum_fqcn,
                location.as_ref(),
                &mut issues,
            );

            // ---- Enum method override/signature compatibility ------------------
            // check_enum_interface_methods_implemented only verifies a method
            // with the right NAME exists — it never compares signatures against
            // the interface method it's satisfying, so a covariance violation,
            // wrong param count, or static/instance mismatch on an enum method
            // went completely undetected. `ClassLike::Enum::ancestor_fqcns`
            // already walks the enum's interface chain, so check_overrides
            // (built for classes/interfaces) works unchanged here too.
            self.check_overrides(&enum_fqcn, location.as_ref(), &mut issues);
        }

        // ---- 5c. DeprecatedTrait: trait uses a deprecated trait ---------------
        for (trait_fqcn, trait_def) in crate::db::analyzed_trait_defs(self.db, &self.analyzed_files)
        {
            let _ = &trait_fqcn;
            let location = trait_def.location.clone();
            for used_trait_fqcn in trait_def.traits.iter() {
                let fqcn_key = crate::db::Fqcn::from_str(self.db, used_trait_fqcn.as_ref());
                let Some(canonical) = crate::db::find_class_like(self.db, fqcn_key) else {
                    continue;
                };
                if let Some(msg) = canonical.deprecated() {
                    self.push_located_issue(
                        &mut issues,
                        IssueKind::DeprecatedTrait {
                            name: used_trait_fqcn.to_string(),
                            message: Some(msg.clone()).filter(|m| !m.is_empty()),
                        },
                        location.as_ref(),
                    );
                }
            }
        }

        // ---- 6. Circular inheritance detection --------------------------------
        self.check_circular_class_inheritance(&mut issues);
        self.check_circular_interface_inheritance(&mut issues);

        issues
    }

    /// Check `@implements Target<args>` / `@extends Target<args>` type args
    /// against `Target`'s own declared `@template ... of Bound` constraints —
    /// the same bound-violation check applied to call-site/constructor-site
    /// template bindings, but for the type args a class declares when
    /// implementing/extending a generic interface/class.
    fn check_generic_type_args(
        &self,
        target_fqcn: &str,
        args: &[mir_types::Type],
        location: Option<&mir_types::Location>,
        issues: &mut Vec<Issue>,
    ) {
        let Some(target_tps) = crate::db::class_template_params(self.db, target_fqcn) else {
            return;
        };
        let bindings: HashMap<mir_types::Name, mir_types::Type> = target_tps
            .iter()
            .zip(args.iter())
            .map(|(tp, ty)| (tp.name, ty.clone()))
            .collect();
        if bindings.is_empty() {
            return;
        }
        for (name, inferred, bound) in crate::generic::check_template_bounds_with_inheritance(
            self.db,
            &bindings,
            &target_tps,
            &HashSet::default(),
            None,
        ) {
            self.push_located_issue(
                issues,
                IssueKind::InvalidTemplateParam {
                    name: name.to_string(),
                    expected_bound: format!("{bound}"),
                    actual: format!("{inferred}"),
                },
                location,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Check: all abstract methods from ancestor chain are implemented
    // -----------------------------------------------------------------------
}

mod cycles;
mod members;
mod overrides;

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
/// Clamps `line_end`/`col_end` to the declaration line so SARIF/WASM consumers
/// see a tight range instead of the entire class body.
/// Falls back to `storage_loc_to_location(None)` when no Location is stored.
fn issue_location(storage_loc: Option<&mir_types::Location>, source: Option<&str>) -> Location {
    let Some(loc) = storage_loc else {
        return crate::diagnostics::storage_loc_to_location(None);
    };
    let (line_end, col_end) = source
        .and_then(|src| src.lines().nth(loc.line.saturating_sub(1) as usize))
        .map(|decl_line| {
            let char_count = decl_line.chars().count() as u16;
            (loc.line, char_count.max(loc.col_start + 1))
        })
        .unwrap_or((loc.line, loc.col_end));
    Location {
        file: loc.file.clone(),
        line: loc.line,
        line_end,
        col_start: loc.col_start,
        col_end,
    }
}

fn canonical_magic_name(lower: &str) -> Option<&'static str> {
    match lower {
        "__construct" => Some("__construct"),
        "__destruct" => Some("__destruct"),
        "__call" => Some("__call"),
        "__callstatic" => Some("__callStatic"),
        "__get" => Some("__get"),
        "__set" => Some("__set"),
        "__isset" => Some("__isset"),
        "__unset" => Some("__unset"),
        "__sleep" => Some("__sleep"),
        "__wakeup" => Some("__wakeup"),
        "__serialize" => Some("__serialize"),
        "__unserialize" => Some("__unserialize"),
        "__tostring" => Some("__toString"),
        "__invoke" => Some("__invoke"),
        "__set_state" => Some("__set_state"),
        "__clone" => Some("__clone"),
        "__debuginfo" => Some("__debugInfo"),
        _ => None,
    }
}

/// Extract the first line of source text covered by `storage_loc` as a snippet.
fn extract_snippet(
    storage_loc: Option<&mir_types::Location>,
    sources: &HashMap<Arc<str>, &str>,
) -> Option<String> {
    let loc = storage_loc?;
    let src = *sources.get(&loc.file)?;
    // Walk to the 1-based start line (loc.line is already 1-based).
    let line_idx = loc.line.saturating_sub(1) as usize;
    let line_text = src.lines().nth(line_idx)?;
    Some(line_text.trim().to_string())
}
