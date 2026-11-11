//! Dead-code detector: after body analysis has recorded every reference,
//! reports private methods and properties, final classes and free functions
//! that nothing references. One pass over the reference index matches keys
//! case-insensitively where PHP names are case-insensitive.

use std::sync::Arc;

use mir_codebase::definitions::Visibility;
use mir_issues::{Issue, IssueKind, Severity};
use rustc_hash::{FxHashMap, FxHashSet};

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

/// A declaration reported as dead unless a reference to one of its keys exists.
struct Candidate {
    kind: IssueKind,
    location: Option<mir_types::Location>,
    /// Lowercased FQCN of a class-like, matched against built class names.
    class_name: Option<String>,
    /// A property that only plain `=` writes don't keep alive.
    needs_read: bool,
}

/// A reference key that keeps candidate `id` alive, optionally only when the
/// reference sits inside `within`.
struct Credit {
    id: u32,
    within: Option<Arc<mir_types::Location>>,
}

type Key = (String, Option<Arc<mir_types::Location>>);

fn anywhere(key: String) -> Key {
    (key, None)
}

#[derive(Default)]
struct Candidates {
    list: Vec<Candidate>,
    /// Normalized reference key → candidates that key keeps alive.
    by_key: FxHashMap<String, Vec<Credit>>,
}

impl Candidates {
    fn push(
        &mut self,
        kind: IssueKind,
        location: Option<mir_types::Location>,
        keys: impl IntoIterator<Item = Key>,
    ) {
        let id = self.list.len() as u32;
        self.list.push(Candidate {
            kind,
            location,
            class_name: None,
            needs_read: false,
        });
        for (key, within) in keys {
            self.by_key
                .entry(key)
                .or_default()
                .push(Credit { id, within });
        }
    }
}

/// Reference key with the case-insensitive parts (class, function and method
/// names) lowercased; property and constant names keep their case.
fn normalize_key(key: &str, out: &mut String) -> bool {
    out.clear();
    let Some((prefix, rest)) = key.split_once(':') else {
        return false;
    };
    let member_is_case_sensitive = match prefix {
        "cls" | "fn" | "meth" | "dyn" | "dyncnst" | "objvars" | "nsprefix" => false,
        "prop" | "propw" | "cnst" | "traituse" => true,
        _ => return false,
    };
    out.push_str(prefix);
    out.push(':');
    match rest.split_once("::").filter(|_| member_is_case_sensitive) {
        Some((class, member)) => {
            out.extend(class.chars().map(|c| c.to_ascii_lowercase()));
            out.push_str("::");
            out.push_str(member);
        }
        None => out.extend(rest.chars().map(|c| c.to_ascii_lowercase())),
    }
    true
}

/// A value with no object inside: writing it into a property that is never
/// read has no effect (an object could be kept alive for its destructor).
fn is_plain_data(ty: &mir_types::Type) -> bool {
    use mir_types::Atomic as A;
    !ty.types.is_empty()
        && ty.types.iter().all(|atomic| match atomic {
            A::TString
            | A::TLiteralString(_)
            | A::TNonEmptyString
            | A::TNonFalsyString
            | A::TRefinedString { .. }
            | A::TNumericString
            | A::TClassString(_)
            | A::TInterfaceString(_)
            | A::TEnumString
            | A::TTraitString
            | A::TInt
            | A::TLiteralInt(_)
            | A::TIntRange { .. }
            | A::TPositiveInt
            | A::TNegativeInt
            | A::TNonNegativeInt
            | A::TFloat
            | A::TIntegralFloat
            | A::TLiteralFloat(..)
            | A::TBool
            | A::TTrue
            | A::TFalse
            | A::TNull
            | A::TNumeric => true,
            A::TArray { key, value } | A::TNonEmptyArray { key, value } => {
                is_plain_data(key) && is_plain_data(value)
            }
            A::TList { value } | A::TNonEmptyList { value } => is_plain_data(value),
            A::TKeyedArray { properties, .. } => properties.values().all(|p| is_plain_data(&p.ty)),
            _ => false,
        })
}

type Position = (Arc<str>, u32, u16);

fn contains(span: &mir_types::Location, file: &str, line: u32, col: u16) -> bool {
    span.file.as_ref() == file
        && (line, col) >= (span.line, span.col_start)
        && (line, col) < (span.line_end, span.col_end)
}

fn has_private_member(class: &crate::db::ClassLike) -> bool {
    class
        .own_methods()
        .values()
        .any(|m| m.visibility == Visibility::Private)
        || class
            .own_constants()
            .values()
            .any(|c| c.visibility == Some(Visibility::Private))
        || class
            .own_properties()
            .is_some_and(|props| props.values().any(|p| p.visibility == Visibility::Private))
}

fn credit_references(
    candidates: &Candidates,
    credits: &[Credit],
    positions: &[Position],
    used: &mut [bool],
) {
    for credit in credits {
        let own_span = candidates.list[credit.id as usize].location.as_ref();
        // A reference from inside the declaration itself (recursion,
        // `new self`) doesn't keep it alive.
        if positions.iter().any(|(file, line, col)| {
            !own_span.is_some_and(|s| contains(s, file, *line, *col))
                && credit
                    .within
                    .as_ref()
                    .is_none_or(|w| contains(w, file, *line, *col))
        }) {
            used[credit.id as usize] = true;
        }
    }
}

pub struct DeadCodeAnalyzer<'a> {
    db: &'a dyn MirDatabase,
    /// Only report dead code defined in these files (empty = all files).
    /// Threading the project file set keeps the pass from materializing and
    /// flagging private members of lazily-loaded vendor classes (whose bodies
    /// are never analyzed, so every private member looks unreferenced).
    analyzed_files: FxHashSet<Arc<str>>,
}

impl<'a> DeadCodeAnalyzer<'a> {
    pub fn with_files(db: &'a dyn MirDatabase, analyzed_files: FxHashSet<Arc<str>>) -> Self {
        Self { db, analyzed_files }
    }

    pub fn analyze(&self) -> Vec<Issue> {
        let mut candidates = Candidates::default();
        let class_likes = crate::db::analyzed_class_likes(self.db, &self.analyzed_files);
        self.collect_private_members(&class_likes, &mut candidates);
        self.collect_classes(&class_likes, &mut candidates);
        self.collect_functions(&mut candidates);

        let mut used = vec![false; candidates.list.len()];
        let mut normalized = String::new();
        let mut namespace_prefixes: Vec<String> = Vec::new();
        // Property references are settled once every pure write is known.
        let mut property_refs: Vec<(String, Vec<Position>)> = Vec::new();
        let mut pure_writes: FxHashMap<String, FxHashSet<Position>> = FxHashMap::default();
        self.db.for_each_reference(&mut |symbol, positions| {
            if !normalize_key(symbol, &mut normalized) {
                return;
            }
            if let Some(prefix) = normalized.strip_prefix("nsprefix:") {
                namespace_prefixes.push(prefix.to_string());
                return;
            }
            if let Some(member) = normalized.strip_prefix("propw:") {
                let key = format!("prop:{member}");
                if candidates.by_key.contains_key(&key) {
                    pure_writes.entry(key).or_default().extend(positions);
                }
                return;
            }
            let Some(credits) = candidates.by_key.get(normalized.as_str()) else {
                return;
            };
            if credits.iter().all(|c| used[c.id as usize]) {
                return;
            }
            if normalized.starts_with("prop:")
                && credits
                    .iter()
                    .any(|c| candidates.list[c.id as usize].needs_read)
            {
                property_refs.push((normalized.clone(), positions.collect()));
                return;
            }
            for position in positions {
                credit_references(
                    &candidates,
                    credits,
                    std::slice::from_ref(&position),
                    &mut used,
                );
                if credits.iter().all(|c| used[c.id as usize]) {
                    break;
                }
            }
        });
        let no_writes = FxHashSet::default();
        for (key, positions) in property_refs {
            let writes = pure_writes.get(&key).unwrap_or(&no_writes);
            let reads: Vec<Position> = positions
                .iter()
                .filter(|p| !writes.contains(*p))
                .cloned()
                .collect();
            for credit in &candidates.by_key[&key] {
                let considered = if candidates.list[credit.id as usize].needs_read {
                    &reads
                } else {
                    &positions
                };
                credit_references(
                    &candidates,
                    std::slice::from_ref(credit),
                    considered,
                    &mut used,
                );
            }
        }

        for (candidate, used) in candidates.list.iter().zip(used.iter_mut()) {
            let Some(class) = &candidate.class_name else {
                continue;
            };
            *used = *used
                || namespace_prefixes
                    .iter()
                    .any(|p| class.starts_with(p.as_str()))
                || matches!(&candidate.kind, IssueKind::UnusedClass { class } if self.is_test_case(class));
        }

        candidates
            .list
            .into_iter()
            .zip(used)
            .filter(|(_, used)| !used)
            .map(|(candidate, _)| {
                let mut issue = Issue::new(
                    candidate.kind,
                    crate::diagnostics::storage_loc_to_location(candidate.location.as_ref()),
                );
                issue.severity = Severity::Info;
                issue
            })
            .collect()
    }

    /// Private members of classes and enums; interface members are public and
    /// a trait's private members belong to whichever class composes it.
    fn collect_private_members(
        &self,
        class_likes: &[(Arc<str>, crate::db::ClassLike)],
        candidates: &mut Candidates,
    ) {
        for (fqcn, class) in class_likes {
            if !matches!(
                class,
                crate::db::ClassLike::Class(_) | crate::db::ClassLike::Enum(_)
            ) || !has_private_member(class)
            {
                continue;
            }
            let fqcn_str = fqcn.as_ref();
            let fqcn_lower = fqcn_str.to_ascii_lowercase();

            // A `$obj->$name`/`$obj->$name()`/`Class::$$name` access anywhere
            // may reach any private member, so it keeps all of them alive.
            let dyn_key = format!("dyn:{fqcn_lower}");

            // A trait body calling `$this->m()` reaches the composing class's
            // own `m`: recorded as `traituse:` when the trait lacks `m`, or
            // under the trait's own `m` it overrides. Walk the transitive
            // ancestors so nested composition (`use Outer` where `Outer` uses
            // `Inner`) is covered.
            let here = crate::db::Fqcn::from_str(self.db, fqcn_str);
            let used_traits: Vec<(String, Option<Arc<mir_types::Location>>)> =
                crate::db::class_ancestors_by_fqcn(self.db, here)
                    .iter()
                    .filter(|a| a.as_ref() != fqcn_str)
                    .filter_map(|a| {
                        let ancestor_here = crate::db::Fqcn::from_str(self.db, a.as_ref());
                        match crate::db::find_class_like(self.db, ancestor_here) {
                            Some(crate::db::ClassLike::Trait(t)) => {
                                Some((a.to_ascii_lowercase(), t.location.clone().map(Arc::new)))
                            }
                            _ => None,
                        }
                    })
                    .collect();

            // `@dataProvider name` / `#[DataProvider('name')]` methods are invoked by
            // PHPUnit via reflection, never through an ordinary call site.
            let data_provider_names: FxHashSet<String> = class
                .own_methods()
                .iter()
                .flat_map(|(_, m)| m.data_provider_targets.iter())
                .map(|s| s.to_ascii_lowercase())
                .collect();

            for (name, method) in class.own_methods().iter() {
                if method.visibility != Visibility::Private {
                    continue;
                }
                let name_lower = name.to_ascii_lowercase();
                if MAGIC_METHODS.contains(&name_lower.as_str())
                    || data_provider_names.contains(&name_lower)
                {
                    continue;
                }
                let keys = [
                    anywhere(format!("meth:{fqcn_lower}::{name_lower}")),
                    anywhere(dyn_key.clone()),
                ]
                .into_iter()
                .chain(used_traits.iter().flat_map(|(t, span)| {
                    [
                        anywhere(format!("traituse:{t}::{name_lower}")),
                        (format!("meth:{t}::{name_lower}"), span.clone()),
                    ]
                }));
                candidates.push(
                    IssueKind::UnusedMethod {
                        class: fqcn_str.to_string(),
                        method: method.name.to_string(),
                    },
                    method.location.clone(),
                    keys,
                );
            }

            let class_span = class.location().cloned().map(Arc::new);
            for (name, constant) in class.own_constants().iter() {
                if constant.visibility != Some(Visibility::Private) {
                    continue;
                }
                let keys = [
                    anywhere(format!("cnst:{fqcn_lower}::{name}")),
                    anywhere(format!("dyncnst:{fqcn_lower}")),
                    (format!("cnst:self::{name}"), class_span.clone()),
                ]
                .into_iter()
                .chain(
                    used_traits
                        .iter()
                        .map(|(t, span)| (format!("cnst:{t}::{name}"), span.clone())),
                );
                candidates.push(
                    IssueKind::UnusedConstant {
                        class: fqcn_str.to_string(),
                        constant: name.to_string(),
                    },
                    constant.location.clone(),
                    keys,
                );
            }

            let Some(props) = class.own_properties() else {
                continue;
            };
            // Without `__serialize`, `serialize()` reads every property of a
            // class built to be unserialized.
            let declares = |m: &str| class.own_methods().contains_key(m);
            let natively_serialized = !declares("__serialize")
                && ["__sleep", "__wakeup", "__unserialize"]
                    .into_iter()
                    .any(declares);
            for (name, prop) in props.iter() {
                if prop.visibility != Visibility::Private {
                    continue;
                }
                let keys = [
                    anywhere(format!("prop:{fqcn_lower}::{name}")),
                    anywhere(dyn_key.clone()),
                    anywhere(format!("objvars:{fqcn_lower}")),
                ]
                .into_iter()
                .chain(used_traits.iter().flat_map(|(t, span)| {
                    [
                        anywhere(format!("traituse:{t}::{name}")),
                        (format!("prop:{t}::{name}"), span.clone()),
                    ]
                }));
                candidates.push(
                    IssueKind::UnusedProperty {
                        class: fqcn_str.to_string(),
                        property: name.to_string(),
                    },
                    prop.location.clone(),
                    keys,
                );
                if let Some(last) = candidates.list.last_mut() {
                    last.needs_read =
                        !natively_serialized && prop.ty.as_deref().is_some_and(is_plain_data);
                }
            }
        }
    }

    /// Every class-like declared in the analyzed files: inheritance, type
    /// hints and docblocks all record a class reference.
    fn collect_classes(
        &self,
        class_likes: &[(Arc<str>, crate::db::ClassLike)],
        candidates: &mut Candidates,
    ) {
        let stub_vfs = StubVfs::new();
        for (fqcn, class) in class_likes {
            let location = class.location();
            if class.is_api()
                || location.is_some_and(|loc| stub_vfs.is_stub_file(loc.file.as_ref()))
            {
                continue;
            }
            let class_name = fqcn.to_ascii_lowercase();
            candidates.push(
                IssueKind::UnusedClass {
                    class: fqcn.to_string(),
                },
                location.cloned(),
                [anywhere(format!("cls:{class_name}"))],
            );
            if let Some(last) = candidates.list.last_mut() {
                last.class_name = Some(class_name);
            }
        }
    }

    /// Test runners (PHPUnit `TestCase`, framework `*TestCase` bases) load
    /// test classes themselves.
    fn is_test_case(&self, fqcn: &str) -> bool {
        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        crate::db::class_ancestors_by_fqcn(self.db, here)
            .iter()
            .skip(1)
            .any(|a| {
                let short = a.rsplit('\\').next().unwrap_or(a);
                short.to_ascii_lowercase().ends_with("testcase")
            })
    }

    fn collect_functions(&self, candidates: &mut Candidates) {
        let stub_vfs = StubVfs::new();
        for (fqn, f) in crate::db::analyzed_function_defs(self.db, &self.analyzed_files) {
            if f.location
                .as_ref()
                .is_some_and(|loc| stub_vfs.is_stub_file(loc.file.as_ref()))
            {
                continue;
            }
            candidates.push(
                IssueKind::UnusedFunction {
                    name: f.short_name.to_string(),
                },
                f.location.clone(),
                [anywhere(format!("fn:{}", fqn.to_ascii_lowercase()))],
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_utils::fresh_session;
    use crate::PhpVersion;

    #[test]
    fn builtin_functions_not_flagged_as_unused() {
        // The dead-code pass must not produce UnusedFunction for PHP built-ins
        // (strlen, array_map, etc.) even when they are never called in user code.
        // This test bypasses the fixture runner's file-path filter to verify the
        // fix directly on the DeadCodeAnalyzer output.
        let mut session = fresh_session(PhpVersion::LATEST);
        session.ensure_all_stubs();
        let db = session.snapshot_db();
        let issues = DeadCodeAnalyzer::with_files(&db, Default::default()).analyze();
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
