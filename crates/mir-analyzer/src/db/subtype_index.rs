//! Delta-maintained inverted inheritance index: resolved parent FQCN →
//! direct children, plus per-class declaration sites.
//!
//! The scan-based `class_subtype_files` query answered "who extends X" by
//! walking every class-like in the workspace and materializing each one's
//! ancestor chain — memoized per FQCN but invalidated wholesale whenever the
//! workspace class index changed, so under active editing every
//! goto-implementation re-paid an O(all classes) scan. This index is the
//! resolved inverse maintained incrementally: when a file's definitions are
//! (re)committed, its classes' old edges are removed and the new ones added,
//! so a query is a reverse-edge BFS in O(matching subtree) regardless of
//! edits elsewhere.
//!
//! Parent names are stored as resolved FQCNs (the collector applies
//! namespace + `use` aliases at collection time), lowercased for PHP's
//! case-insensitive class-name semantics.

use std::sync::Arc;

use rustc_hash::{FxHashMap, FxHashSet};

use mir_types::Location;

/// What kind of class-like declared the subtype (mirrors the four def
/// structs; there is no shared kind tag on them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassLikeKind {
    Class,
    Interface,
    Trait,
    Enum,
}

/// One class-like declaration as recorded in the index.
#[derive(Debug, Clone)]
pub struct SubtypeEntry {
    /// Display-form FQCN (as collected, no leading `\`).
    pub fqcn: Arc<str>,
    pub kind: ClassLikeKind,
    pub is_abstract: bool,
    /// Direct `extends` + `implements` ancestors, lowercased resolved FQCNs.
    pub supers: Box<[Arc<str>]>,
    /// Direct `use TraitName;` ancestors, lowercased resolved FQCNs. Kept
    /// separate so implementation queries can exclude trait users while
    /// visibility-scope queries include them.
    pub trait_supers: Box<[Arc<str>]>,
    /// Declaration site (line 1-based, cols 0-based code points), when known.
    pub location: Option<Location>,
}

/// A resolved subtype hit returned by queries.
#[derive(Debug, Clone)]
pub struct SubtypeSite {
    pub fqcn: Arc<str>,
    pub kind: ClassLikeKind,
    pub is_abstract: bool,
    pub file: Arc<str>,
    pub location: Option<Location>,
}

/// Normalize an FQCN for use as an edge key.
pub fn edge_key(fqcn: &str) -> Arc<str> {
    Arc::from(fqcn.trim_start_matches('\\').to_ascii_lowercase())
}

/// The unqualified (short) name of an FQCN.
pub fn short_name_of(fqcn: &str) -> &str {
    fqcn.rsplit('\\').next().unwrap_or(fqcn)
}

/// Build the index entries for one file's collected definitions. Parent and
/// interface names in the slice are already resolved FQCNs (the collector
/// applies namespace + `use` aliases), so this is pure reshaping.
pub fn entries_from_slice(slice: &mir_codebase::definitions::StubSlice) -> Vec<SubtypeEntry> {
    let mut out: Vec<SubtypeEntry> =
        Vec::with_capacity(slice.classes.len() + slice.interfaces.len() + slice.enums.len());
    for c in &slice.classes {
        out.push(SubtypeEntry {
            fqcn: c.fqcn.clone(),
            kind: ClassLikeKind::Class,
            is_abstract: c.is_abstract,
            supers: c
                .parent
                .iter()
                .chain(c.interfaces.iter())
                .map(|p| edge_key(p))
                .collect(),
            trait_supers: c.traits.iter().map(|p| edge_key(p)).collect(),
            location: c.location.clone(),
        });
    }
    for i in &slice.interfaces {
        out.push(SubtypeEntry {
            fqcn: i.fqcn.clone(),
            kind: ClassLikeKind::Interface,
            is_abstract: false,
            supers: i.extends.iter().map(|p| edge_key(p)).collect(),
            trait_supers: Box::default(),
            location: i.location.clone(),
        });
    }
    for t in &slice.traits {
        out.push(SubtypeEntry {
            fqcn: t.fqcn.clone(),
            kind: ClassLikeKind::Trait,
            is_abstract: false,
            supers: Box::default(),
            trait_supers: t.traits.iter().map(|p| edge_key(p)).collect(),
            location: t.location.clone(),
        });
    }
    for e in &slice.enums {
        out.push(SubtypeEntry {
            fqcn: e.fqcn.clone(),
            kind: ClassLikeKind::Enum,
            is_abstract: false,
            supers: e.interfaces.iter().map(|p| edge_key(p)).collect(),
            trait_supers: e.traits.iter().map(|p| edge_key(p)).collect(),
            location: e.location.clone(),
        });
    }
    out
}

/// Declaring entries for one FQCN: `(file, entry)` pairs. More than one file
/// can declare the same FQCN (duplicated symbol); keep all, keyed by file.
type DeclSites = Vec<(Arc<str>, Arc<SubtypeEntry>)>;

#[derive(Default, Debug)]
pub struct SubtypeIndex {
    /// parent (lowercased FQCN) → children (lowercased FQCN). Edge origin
    /// (extends/implements vs trait use) lives on the child's entry.
    children: FxHashMap<Arc<str>, FxHashSet<Arc<str>>>,
    /// file → class-likes it declared at last commit.
    by_file: FxHashMap<Arc<str>, Vec<Arc<SubtypeEntry>>>,
    /// class (lowercased FQCN) → declaring entries.
    decls: FxHashMap<Arc<str>, DeclSites>,
}

impl SubtypeIndex {
    /// Replace `file`'s class-like declarations wholesale.
    pub fn set_file_classes(&mut self, file: &Arc<str>, entries: Vec<SubtypeEntry>) {
        self.clear_file(file.as_ref());
        if entries.is_empty() {
            return;
        }
        let mut stored: Vec<Arc<SubtypeEntry>> = Vec::with_capacity(entries.len());
        for entry in entries {
            let entry = Arc::new(entry);
            let child_key = edge_key(&entry.fqcn);
            for parent in entry.supers.iter().chain(entry.trait_supers.iter()) {
                self.children
                    .entry(parent.clone())
                    .or_default()
                    .insert(child_key.clone());
            }
            self.decls
                .entry(child_key)
                .or_default()
                .push((file.clone(), entry.clone()));
            stored.push(entry);
        }
        self.by_file.insert(file.clone(), stored);
    }

    /// Remove every declaration recorded for `file`.
    pub fn clear_file(&mut self, file: &str) {
        let Some(old) = self.by_file.remove(file) else {
            return;
        };
        for entry in old {
            let child_key = edge_key(&entry.fqcn);
            // Drop the child edge only when no other file still declares a
            // class-like with the same FQCN and the same parent.
            if let Some(sites) = self.decls.get_mut(&child_key) {
                sites.retain(|(f, _)| f.as_ref() != file);
                if sites.is_empty() {
                    self.decls.remove(&child_key);
                }
            }
            let still_declared: Vec<&Arc<SubtypeEntry>> = self
                .decls
                .get(&child_key)
                .map(|sites| sites.iter().map(|(_, e)| e).collect())
                .unwrap_or_default();
            for parent in entry.supers.iter().chain(entry.trait_supers.iter()) {
                let survives = still_declared
                    .iter()
                    .any(|e| e.supers.contains(parent) || e.trait_supers.contains(parent));
                if !survives {
                    if let Some(set) = self.children.get_mut(parent) {
                        set.remove(&child_key);
                        if set.is_empty() {
                            self.children.remove(parent);
                        }
                    }
                }
            }
        }
    }

    /// Transitive subtypes of `fqcn` (excluding `fqcn` itself), BFS over the
    /// reverse edges. `include_trait_users` controls whether an edge that
    /// exists only via `use Trait;` counts as a subtype relation.
    pub fn subtypes_of(&self, fqcn: &str, include_trait_users: bool) -> Vec<SubtypeSite> {
        let root = edge_key(fqcn);
        let mut visited: FxHashSet<Arc<str>> = FxHashSet::default();
        visited.insert(root.clone());
        let mut queue: Vec<Arc<str>> = vec![root];
        let mut out: Vec<SubtypeSite> = Vec::new();
        while let Some(parent) = queue.pop() {
            let Some(children) = self.children.get(&parent) else {
                continue;
            };
            for child in children {
                if visited.contains(child) {
                    continue;
                }
                let Some(sites) = self.decls.get(child) else {
                    // Edge to a class we have no declaration for (declared in
                    // an uncommitted file) — nothing to report or recurse into.
                    continue;
                };
                // The edge qualifies when any declaring site reaches `parent`
                // through extends/implements — or trait use when requested.
                // A child reached only through a non-qualifying edge is not a
                // subtype and must not be descended into from here; it stays
                // unvisited so a qualifying edge elsewhere can still reach it.
                let mut qualifies = false;
                for (file, entry) in sites {
                    let via_super = entry.supers.contains(&parent);
                    let via_trait = include_trait_users && entry.trait_supers.contains(&parent);
                    if !via_super && !via_trait {
                        continue;
                    }
                    qualifies = true;
                    out.push(SubtypeSite {
                        fqcn: entry.fqcn.clone(),
                        kind: entry.kind,
                        is_abstract: entry.is_abstract,
                        file: file.clone(),
                        location: entry.location.clone(),
                    });
                }
                if qualifies {
                    visited.insert(child.clone());
                    queue.push(child.clone());
                }
            }
        }
        out.sort_by(|a, b| {
            a.file.cmp(&b.file).then_with(|| {
                let la = a.location.as_ref().map(|l| l.line).unwrap_or(0);
                let lb = b.location.as_ref().map(|l| l.line).unwrap_or(0);
                la.cmp(&lb)
            })
        });
        out.dedup_by(|a, b| a.fqcn == b.fqcn && a.file == b.file);
        out
    }

    /// Like [`Self::subtypes_of`], but when the exact FQCN yields nothing,
    /// retry from every parent key sharing the target's short name. This is
    /// the written-form leniency the old name-matching implementation had:
    /// `interface Runner {}` in the global namespace is matched by
    /// `implements Runner` inside `namespace App`, and `use App\Animal` on
    /// the cursor side still matches a bare `extends Animal` declared in the
    /// global namespace.
    pub fn subtypes_of_lenient(&self, fqcn: &str, include_trait_users: bool) -> Vec<SubtypeSite> {
        let exact = self.subtypes_of(fqcn, include_trait_users);
        if !exact.is_empty() {
            return exact;
        }
        let root = edge_key(fqcn);
        let short = short_name_of(&root).to_string();
        let mut out: Vec<SubtypeSite> = Vec::new();
        let mut alt_roots: Vec<Arc<str>> = self
            .children
            .keys()
            .filter(|k| k.as_ref() != root.as_ref() && short_name_of(k) == short)
            .cloned()
            .collect();
        alt_roots.sort();
        for key in alt_roots {
            out.extend(self.subtypes_of(&key, include_trait_users));
        }
        out.sort_by(|a, b| a.file.cmp(&b.file).then(a.fqcn.cmp(&b.fqcn)));
        out.dedup_by(|a, b| a.fqcn == b.fqcn && a.file == b.file);
        out
    }

    /// FQCNs (display form) of every class-like currently missing a
    /// declaration entry but referenced as a parent from `fqcn`'s subtree.
    /// Used by completeness passes to decide which frontier names still need
    /// their declaring files committed.
    pub fn has_decl(&self, fqcn: &str) -> bool {
        self.decls.contains_key(&edge_key(fqcn))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(fqcn: &str, supers: &[&str], traits: &[&str]) -> SubtypeEntry {
        SubtypeEntry {
            fqcn: Arc::from(fqcn),
            kind: ClassLikeKind::Class,
            is_abstract: false,
            supers: supers.iter().map(|s| edge_key(s)).collect(),
            trait_supers: traits.iter().map(|s| edge_key(s)).collect(),
            location: None,
        }
    }

    #[test]
    fn direct_and_transitive_subtypes() {
        let mut idx = SubtypeIndex::default();
        let a: Arc<str> = Arc::from("a.php");
        let b: Arc<str> = Arc::from("b.php");
        idx.set_file_classes(&a, vec![entry("App\\Child", &["App\\Base"], &[])]);
        idx.set_file_classes(&b, vec![entry("App\\Grand", &["App\\Child"], &[])]);
        let subs = idx.subtypes_of("App\\Base", false);
        assert_eq!(subs.len(), 2);
        assert!(subs.iter().any(|s| s.fqcn.as_ref() == "App\\Child"));
        assert!(subs.iter().any(|s| s.fqcn.as_ref() == "App\\Grand"));
    }

    #[test]
    fn recommit_replaces_edges() {
        let mut idx = SubtypeIndex::default();
        let a: Arc<str> = Arc::from("a.php");
        idx.set_file_classes(&a, vec![entry("App\\Child", &["App\\Base"], &[])]);
        assert_eq!(idx.subtypes_of("App\\Base", false).len(), 1);
        idx.set_file_classes(&a, vec![entry("App\\Child", &["App\\Other"], &[])]);
        assert!(idx.subtypes_of("App\\Base", false).is_empty());
        assert_eq!(idx.subtypes_of("App\\Other", false).len(), 1);
    }

    #[test]
    fn trait_users_only_when_requested() {
        let mut idx = SubtypeIndex::default();
        let a: Arc<str> = Arc::from("a.php");
        idx.set_file_classes(&a, vec![entry("App\\User", &[], &["App\\Helper"])]);
        assert!(idx.subtypes_of("App\\Helper", false).is_empty());
        assert_eq!(idx.subtypes_of("App\\Helper", true).len(), 1);
    }

    #[test]
    fn clear_file_keeps_other_files_edges() {
        let mut idx = SubtypeIndex::default();
        let a: Arc<str> = Arc::from("a.php");
        let b: Arc<str> = Arc::from("b.php");
        idx.set_file_classes(&a, vec![entry("App\\Child", &["App\\Base"], &[])]);
        idx.set_file_classes(&b, vec![entry("App\\Child", &["App\\Base"], &[])]);
        idx.clear_file("a.php");
        assert_eq!(idx.subtypes_of("App\\Base", false).len(), 1);
        idx.clear_file("b.php");
        assert!(idx.subtypes_of("App\\Base", false).is_empty());
    }

    #[test]
    fn case_insensitive_parent_match() {
        let mut idx = SubtypeIndex::default();
        let a: Arc<str> = Arc::from("a.php");
        idx.set_file_classes(&a, vec![entry("App\\Child", &["App\\BASE"], &[])]);
        assert_eq!(idx.subtypes_of("app\\base", false).len(), 1);
        assert_eq!(idx.subtypes_of("\\App\\Base", false).len(), 1);
    }

    #[test]
    fn diamond_hierarchy_visits_once() {
        let mut idx = SubtypeIndex::default();
        let a: Arc<str> = Arc::from("a.php");
        idx.set_file_classes(
            &a,
            vec![
                entry("I\\Left", &["I\\Top"], &[]),
                entry("I\\Right", &["I\\Top"], &[]),
                entry("I\\Bottom", &["I\\Left", "I\\Right"], &[]),
            ],
        );
        let subs = idx.subtypes_of("I\\Top", false);
        assert_eq!(subs.len(), 3);
    }
}
