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

use crate::db::MirDatabase;

/// Per-file issues (`None` for a reused file) and issues tied to no single file.
pub type IssuesByFile = (Vec<(Arc<str>, Option<Vec<Issue>>)>, Vec<Issue>);

enum AnalyzedDef {
    Class(Arc<str>, crate::db::ClassLike),
    Interface(Arc<str>, Arc<mir_codebase::definitions::InterfaceDef>),
    Enum(Arc<str>, Arc<mir_codebase::definitions::EnumDef>),
    Trait(Arc<mir_codebase::definitions::TraitDef>),
}

impl AnalyzedDef {
    fn file(&self) -> Option<&Arc<str>> {
        let location = match self {
            AnalyzedDef::Class(_, class) => class.location(),
            AnalyzedDef::Interface(_, iface) => iface.location.as_ref(),
            AnalyzedDef::Enum(_, def) => def.location.as_ref(),
            AnalyzedDef::Trait(def) => def.location.as_ref(),
        };
        location.map(|l| &l.file)
    }
}

// ---------------------------------------------------------------------------
// ClassAnalyzer
// ---------------------------------------------------------------------------

pub struct ClassAnalyzer<'a> {
    db: &'a dyn MirDatabase,
    /// Only report issues for classes defined in these files (empty = all files).
    analyzed_files: Arc<HashSet<Arc<str>>>,
    /// Source text keyed by file path, used to extract snippets for class-level issues.
    sources: Arc<HashMap<Arc<str>, &'a str>>,
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
            analyzed_files: Arc::new(files),
            sources: Arc::new(sources),
        }
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

    /// Class-level checks of one plain class.
    fn class_issues(&self, fqcn: &Arc<str>, class: &crate::db::ClassLike) -> Vec<Issue> {
        let mut issues = Vec::new();
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
            // The body pass reports extending an enum or trait as a compile error;
            // the member checks below would only add cascades.
            if parent_pulled
                .as_ref()
                .is_some_and(|p| p.is_enum() || p.is_trait())
            {
                return issues;
            }
            if parent_pulled.is_some() {
                if parent_is_final {
                    self.push_located_issue(
                        &mut issues,
                        IssueKind::InvalidExtendClass {
                            parent: parent_fqcn.to_string(),
                            child: fqcn.to_string(),
                            reason: None,
                        },
                        location.as_ref(),
                    );
                }
                // A `readonly class` (PHP 8.2 whole-class modifier) must
                // extend another `readonly` class, and vice versa — both
                // directions are a hard PHP fatal.
                let parent_readonly = parent_pulled
                    .as_ref()
                    .map(|c| c.is_readonly())
                    .unwrap_or(false);
                if class.is_readonly() != parent_readonly {
                    self.push_located_issue(
                        &mut issues,
                        IssueKind::ReadonlyClassExtendsMismatch {
                            parent: parent_fqcn.to_string(),
                            child: fqcn.to_string(),
                            parent_readonly,
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
        for iface_fqcn in class.interfaces() {
            let iface_here = crate::db::Fqcn::from_str(self.db, iface_fqcn.as_ref());
            if let Some(iface) = crate::db::find_class_like(self.db, iface_here) {
                if let Some((used, canonical_str)) =
                    crate::fqcn_case_mismatch(iface_fqcn.as_ref(), iface.fqcn().as_ref())
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
        // ---- 1c. Sealed supertypes and interface `@phpstan-require-extends` ---
        let direct_supertypes = parent_fqcn.iter().chain(class.interfaces().iter());
        for sup in direct_supertypes {
            let sealed = match crate::db::find_class_like(
                self.db,
                crate::db::Fqcn::from_str(self.db, sup.as_ref()),
            ) {
                Some(crate::db::ClassLike::Class(c)) => c.sealed.clone(),
                Some(crate::db::ClassLike::Interface(i)) => i.sealed.clone(),
                _ => continue,
            };
            if !sealed.is_empty() && !sealed.iter().any(|c| c.eq_ignore_ascii_case(fqcn)) {
                let allowed: Vec<&str> = sealed.iter().map(|c| c.as_ref()).collect();
                self.push_located_issue(
                    &mut issues,
                    IssueKind::InvalidExtendClass {
                        parent: sup.to_string(),
                        child: fqcn.to_string(),
                        reason: Some(format!(
                            "Class {fqcn} cannot extend sealed {sup}, which allows only {}",
                            allowed.join("|")
                        )),
                    },
                    location.as_ref(),
                );
            }
        }
        let ancestors =
            crate::db::class_ancestors_by_fqcn(self.db, crate::db::Fqcn::from_str(self.db, fqcn));
        for ancestor in ancestors.iter() {
            let Some(crate::db::ClassLike::Interface(iface)) = crate::db::find_class_like(
                self.db,
                crate::db::Fqcn::from_str(self.db, ancestor.as_ref()),
            ) else {
                continue;
            };
            for required in iface.require_extends.iter() {
                if !ancestors.iter().any(|a| a.eq_ignore_ascii_case(required)) {
                    self.push_located_issue(
                        &mut issues,
                        IssueKind::InvalidExtendClass {
                            parent: required.to_string(),
                            child: fqcn.to_string(),
                            reason: Some(format!(
                                "Class {fqcn} implements {ancestor}, which requires extending {required}"
                            )),
                        },
                        location.as_ref(),
                    );
                }
            }
        }

        for trait_fqcn in class.class_traits() {
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

        // The checks below look the class up by name; a same-named enum,
        // interface or trait (a polyfill behind a version check) would answer.
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        if !crate::db::find_class_like(self.db, here).is_some_and(|c| c.is_class()) {
            return issues;
        }
        // A compile error in the body pass; the enum contracts' members would cascade.
        if crate::db::extends_or_implements(self.db, fqcn.as_ref(), "UnitEnum") {
            return issues;
        }

        self.check_constant_overrides(fqcn, class.own_constants(), &mut issues);
        if let crate::db::ClassLike::Class(def) = &class {
            self.check_ambiguous_constants(
                fqcn,
                "Class",
                &def.own_constants,
                def.parent.as_ref(),
                &def.interfaces,
                location.as_ref(),
                &mut issues,
            );
            self.check_trait_method_collisions(def, &mut issues);
            // A readonly class makes every property readonly, its traits' too.
            if !def.is_readonly {
                self.check_trait_member_compatibility(
                    fqcn,
                    Some(&def.own_properties),
                    def.parent.as_deref(),
                    &def.own_constants,
                    &def.traits,
                    &def.trait_use_locations,
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
            return issues;
        }

        // A parent that brings `Traversable` is internal (it iterates) or reported itself.
        let parent_traversable = parent_fqcn
            .as_ref()
            .is_some_and(|p| crate::db::extends_or_implements(self.db, p.as_ref(), "Traversable"));
        if !parent_traversable
            && crate::db::extends_or_implements(self.db, fqcn.as_ref(), "Traversable")
            && !crate::db::extends_or_implements(self.db, fqcn.as_ref(), "Iterator")
            && !crate::db::extends_or_implements(self.db, fqcn.as_ref(), "IteratorAggregate")
        {
            self.push_located_issue(
                &mut issues,
                IssueKind::CompileError {
                    message: format!(
                        "Class {fqcn} must implement interface Traversable as part of either Iterator or IteratorAggregate"
                    ),
                    code: "traversable-without-iterator".to_string(),
                },
                location.as_ref(),
            );
        }

        // ---- 2. Abstract parent methods must be implemented ----------------
        self.check_abstract_methods_implemented(fqcn, location.as_ref(), &mut issues);
        self.check_abstract_property_hooks(fqcn, false, location.as_ref(), &mut issues);

        // ---- 3. Interface methods must be implemented ----------------------
        self.check_interface_methods_implemented(fqcn, location.as_ref(), &mut issues);

        // ---- 4. Method override compatibility ------------------------------
        self.check_overrides(fqcn, location.as_ref(), &mut issues);

        // ---- 5. Magic method casing ----------------------------------------
        self.check_magic_method_casing(fqcn, &mut issues);

        // ---- 6. Missing constructor ----------------------------------------
        self.check_missing_constructor(fqcn, location.as_ref(), &mut issues);
        issues
    }

    /// Run all class-level checks and return every discovered issue.
    pub fn analyze_all(&self) -> Vec<Issue> {
        let mut issues: Vec<Issue> = self
            .analyzed_defs()
            .iter()
            .flat_map(|def| self.def_issues(def))
            .collect();
        issues.extend(self.cycle_issues());
        issues
    }

    /// Class-like definitions of the analyzed files in check order: classes,
    /// interfaces, enums, then traits, each sorted by FQCN.
    fn analyzed_defs(&self) -> Vec<AnalyzedDef> {
        // Decomposing per file via `collect_file_definitions` means vendor / stub
        // classes are never materialized (they aren't in `analyzed_files`), which
        // is the dominant cost on cold start over a large `vendor/`.
        let files = &self.analyzed_files;
        let mut defs: Vec<AnalyzedDef> = crate::db::analyzed_class_defs(self.db, files)
            .into_iter()
            .map(|(fqcn, class)| AnalyzedDef::Class(fqcn, class))
            .collect();
        defs.extend(
            crate::db::analyzed_interface_defs(self.db, files)
                .into_iter()
                .map(|(fqcn, iface)| AnalyzedDef::Interface(fqcn, iface)),
        );
        defs.extend(
            crate::db::analyzed_enum_defs(self.db, files)
                .into_iter()
                .map(|(fqcn, def)| AnalyzedDef::Enum(fqcn, def)),
        );
        defs.extend(
            crate::db::analyzed_trait_defs(self.db, files)
                .into_iter()
                .map(|(_, def)| AnalyzedDef::Trait(def)),
        );
        defs
    }

    fn def_issues(&self, def: &AnalyzedDef) -> Vec<Issue> {
        match def {
            AnalyzedDef::Class(fqcn, class) => self.class_issues(fqcn, class),
            AnalyzedDef::Interface(fqcn, iface) => self.interface_issues(fqcn, iface),
            AnalyzedDef::Enum(fqcn, def) => self.enum_issues(fqcn, def),
            AnalyzedDef::Trait(def) => self.trait_issues(def),
        }
    }

    /// The checks of [`Self::analyze_all`] spread over the rayon pool, each
    /// worker on its own clone of `db`. Per-definition issues come back grouped
    /// by declaring file, in the order the files first appear; files in `reuse`
    /// are not checked and come back as `None`. The second element holds the
    /// cross-file cycle issues and those of definitions without a location.
    pub fn analyze_by_file_par<D>(&self, db: D, reuse: &HashSet<Arc<str>>) -> IssuesByFile
    where
        D: MirDatabase + Clone + Send,
    {
        use rayon::prelude::*;
        let defs = self.analyzed_defs();
        let (analyzed_files, sources) = (&self.analyzed_files, &self.sources);
        let per_def: Vec<Option<Vec<Issue>>> = defs
            .par_iter()
            .map_with(db, |db, def| {
                if def.file().is_some_and(|f| reuse.contains(f)) {
                    return None;
                }
                let analyzer = ClassAnalyzer {
                    db: &*db as &dyn MirDatabase,
                    analyzed_files: Arc::clone(analyzed_files),
                    sources: Arc::clone(sources),
                };
                Some(analyzer.def_issues(def))
            })
            .collect();

        let mut by_file: Vec<(Arc<str>, Option<Vec<Issue>>)> = Vec::new();
        let mut slot_of: HashMap<Arc<str>, usize> = HashMap::default();
        let mut unlocated = Vec::new();
        for (def, issues) in defs.iter().zip(per_def) {
            let Some(file) = def.file() else {
                unlocated.extend(issues.unwrap_or_default());
                continue;
            };
            let slot = *slot_of.entry(file.clone()).or_insert_with(|| {
                let reused = reuse.contains(file);
                by_file.push((file.clone(), (!reused).then(Vec::new)));
                by_file.len() - 1
            });
            if let (Some(acc), Some(issues)) = (by_file[slot].1.as_mut(), issues) {
                acc.extend(issues);
            }
        }
        unlocated.extend(self.cycle_issues());
        (by_file, unlocated)
    }

    /// Interface-level `#[Override]`, constant and extends checks.
    fn interface_issues(
        &self,
        iface_fqcn: &Arc<str>,
        iface: &Arc<mir_codebase::definitions::InterfaceDef>,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();
        {
            self.check_overrides(iface_fqcn, None, &mut issues);
            self.check_constant_overrides(iface_fqcn, &iface.own_constants, &mut issues);
            let location = iface.location.clone();
            self.check_ambiguous_constants(
                iface_fqcn,
                "Interface",
                &iface.own_constants,
                None,
                &iface.extends,
                location.as_ref(),
                &mut issues,
            );
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
            // Same generic-bound check as class `@implements`/`@extends` type
            // args above (1c) — an interface's own `@template-extends
            // Base<Unrelated>` type args were parsed and used for binding
            // substitution elsewhere but never bound-checked here.
            for (parent_iface_fqcn, args) in iface.extends_type_args.iter() {
                if !args.is_empty() {
                    self.check_generic_type_args(
                        parent_iface_fqcn.as_ref(),
                        args,
                        location.as_ref(),
                        &mut issues,
                    );
                }
            }
            self.check_magic_method_casing(iface_fqcn, &mut issues);
        }
        issues
    }

    /// Enum-level implements casing, magic method and contract checks.
    fn enum_issues(
        &self,
        enum_fqcn: &Arc<str>,
        enum_def: &Arc<mir_codebase::definitions::EnumDef>,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();
        {
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
            self.check_magic_method_casing(enum_fqcn, &mut issues);
            self.check_enum_magic_methods(enum_def, &mut issues);
            self.check_enum_builtin_redeclarations(enum_def, &mut issues);
            self.check_abstract_methods_implemented(enum_fqcn, location.as_ref(), &mut issues);
            self.check_abstract_property_hooks(enum_fqcn, true, location.as_ref(), &mut issues);
            self.check_trait_member_compatibility(
                enum_fqcn,
                None,
                None,
                &enum_def.own_constants,
                &enum_def.traits,
                &enum_def.trait_use_locations,
                location.as_ref(),
                &mut issues,
            );
            self.check_constant_overrides(enum_fqcn, &enum_def.own_constants, &mut issues);
            self.check_ambiguous_constants(
                enum_fqcn,
                "Enum",
                &enum_def.own_constants,
                None,
                &enum_def.interfaces,
                location.as_ref(),
                &mut issues,
            );

            // Same generic-bound check as class `@implements`/`@extends` type
            // args above (1c) — an enum's own `@implements Interface<Arg>`
            // type args were parsed and used for binding substitution
            // elsewhere but never bound-checked here, unlike the identical
            // class case.
            for (iface, args) in enum_def.implements_type_args.iter() {
                self.check_generic_type_args(iface.as_ref(), args, location.as_ref(), &mut issues);
            }

            // ---- Enum interface methods must be implemented --------------------
            self.check_enum_interface_methods_implemented(
                enum_fqcn,
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
            self.check_overrides(enum_fqcn, location.as_ref(), &mut issues);
        }
        issues
    }

    /// `DeprecatedTrait`: the trait uses a deprecated trait.
    fn trait_issues(&self, trait_def: &mir_codebase::definitions::TraitDef) -> Vec<Issue> {
        let mut issues = Vec::new();
        {
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
        issues
    }

    /// Circular inheritance and trait composition, which span files.
    fn cycle_issues(&self) -> Vec<Issue> {
        let mut issues = Vec::new();
        self.check_circular_class_inheritance(&mut issues);
        self.check_circular_interface_inheritance(&mut issues);
        self.check_circular_trait_composition(&mut issues);

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
