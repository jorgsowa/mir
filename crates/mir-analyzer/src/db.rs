use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use mir_codebase::storage::{Assertion, FnParam, FunctionStorage, TemplateParam};
use mir_codebase::StubSlice;
use mir_issues::Issue;
use mir_types::Union;

// ---------------------------------------------------------------------------
// MirDatabase trait
// ---------------------------------------------------------------------------

/// Salsa database trait for mir incremental analysis.
#[salsa::db]
pub trait MirDatabase: salsa::Database {
    /// The PHP version configured for this analysis run.
    fn php_version_str(&self) -> Arc<str>;

    /// Look up the [`ClassNode`] handle registered for `fqcn`, if any.
    ///
    /// This is an untracked read — the DashMap holds Salsa input *handles*
    /// (cheap IDs), not data.  Changes to a class's *fields* (parent,
    /// interfaces, active state) are tracked through the `ClassNode` input
    /// itself, so downstream queries are still correctly invalidated.
    fn lookup_class_node(&self, fqcn: &str) -> Option<ClassNode>;

    /// Look up the [`FunctionNode`] handle registered for `fqn`, if any.
    fn lookup_function_node(&self, fqn: &str) -> Option<FunctionNode>;
}

// ---------------------------------------------------------------------------
// SourceFile input (S1)
// ---------------------------------------------------------------------------

/// Source file registered as a Salsa input.
/// Setting `text` on an existing `SourceFile` is the single write that drives
/// all downstream query invalidation.
#[salsa::input]
pub struct SourceFile {
    pub path: Arc<str>,
    pub text: Arc<str>,
}

// ---------------------------------------------------------------------------
// FileDefinitions (S1)
// ---------------------------------------------------------------------------

/// Result of the `collect_file_definitions` tracked query.
#[derive(Clone, Debug)]
pub struct FileDefinitions {
    pub slice: Arc<StubSlice>,
    pub issues: Arc<Vec<Issue>>,
}

impl PartialEq for FileDefinitions {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.slice, &other.slice) && Arc::ptr_eq(&self.issues, &other.issues)
    }
}

unsafe impl salsa::Update for FileDefinitions {
    unsafe fn maybe_update(old_pointer: *mut Self, new_value: Self) -> bool {
        unsafe { *old_pointer = new_value };
        true
    }
}

// ---------------------------------------------------------------------------
// ClassNode input (S2)
// ---------------------------------------------------------------------------

/// Salsa input representing a single class or interface in the inheritance
/// graph.  Fields are kept minimal — only what `class_ancestors` needs.
///
/// Invariant: every FQCN in the codebase that is known to the Salsa DB has
/// exactly one `ClassNode` handle, stored in `MirDb::class_nodes`.  When a
/// class is removed (file deleted or re-indexed), its node is marked
/// `active = false` rather than dropped, so dependent `class_ancestors` queries
/// can still observe the change and re-run.
#[salsa::input]
pub struct ClassNode {
    pub fqcn: Arc<str>,
    /// `false` when the class has been removed from the codebase.  Dependent
    /// queries observe this change and re-run, returning empty ancestors.
    pub active: bool,
    pub is_interface: bool,
    /// Direct parent class (classes only; `None` for interfaces).
    pub parent: Option<Arc<str>>,
    /// Directly implemented interfaces (classes only).
    pub interfaces: Arc<[Arc<str>]>,
    /// Used traits (classes only).  Traits are added to the ancestor list but
    /// their own ancestors are not recursed into, matching PHP semantics.
    pub traits: Arc<[Arc<str>]>,
    /// Directly extended interfaces (interfaces only).
    pub extends: Arc<[Arc<str>]>,
}

// ---------------------------------------------------------------------------
// FunctionNode input (S5-PR2)
// ---------------------------------------------------------------------------

/// Salsa input representing a single global function.
///
/// `inferred_return_type` is intentionally absent — it lives in
/// `FunctionStorage` until S3 promotes it to a proper tracked query.
///
/// Invariant: every FQN known to the Salsa DB has exactly one `FunctionNode`
/// handle in `MirDb::function_nodes`.  Removed functions are marked
/// `active = false` rather than dropped.
#[salsa::input]
pub struct FunctionNode {
    pub fqn: Arc<str>,
    pub active: bool,
    pub params: Arc<[FnParam]>,
    pub return_type: Option<Union>,
    pub template_params: Arc<[TemplateParam]>,
    pub assertions: Arc<[Assertion]>,
    pub throws: Arc<[Arc<str>]>,
    pub deprecated: Option<Arc<str>>,
    pub is_pure: bool,
}

// ---------------------------------------------------------------------------
// Ancestors return type (S2)
// ---------------------------------------------------------------------------

/// The computed ancestor list for a class or interface.
///
/// Uses content equality so Salsa's cycle-convergence check can detect
/// fixpoints correctly (two empty lists from different iterations are equal).
#[derive(Clone, Debug, Default)]
pub struct Ancestors(pub Vec<Arc<str>>);

impl PartialEq for Ancestors {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(&other.0)
                .all(|(a, b)| a.as_ref() == b.as_ref())
    }
}

unsafe impl salsa::Update for Ancestors {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

// ---------------------------------------------------------------------------
// class_ancestors tracked query (S2)
// ---------------------------------------------------------------------------

fn ancestors_initial(_db: &dyn MirDatabase, _id: salsa::Id, _node: ClassNode) -> Ancestors {
    Ancestors(vec![])
}

fn ancestors_cycle(
    _db: &dyn MirDatabase,
    _cycle: &salsa::Cycle,
    _last: &Ancestors,
    _value: Ancestors,
    _node: ClassNode,
) -> Ancestors {
    // PHP class cycles are a compile-time error.  Break immediately with an
    // empty list so the fixpoint converges on the first iteration.
    Ancestors(vec![])
}

/// Salsa tracked query: compute the transitive ancestor list for a class or
/// interface.
///
/// Ancestors are accumulated in the same order as `Codebase::ensure_finalized`:
/// parent → parent's ancestors → implemented interfaces + their ancestors →
/// used traits (class); or: extended interfaces + their ancestors (interface).
///
/// Cycle recovery returns an empty list on the first iteration, which is
/// correct because PHP forbids circular inheritance.
#[salsa::tracked(cycle_fn = ancestors_cycle, cycle_initial = ancestors_initial)]
pub fn class_ancestors(db: &dyn MirDatabase, node: ClassNode) -> Ancestors {
    if !node.active(db) {
        return Ancestors(vec![]);
    }

    let mut all: Vec<Arc<str>> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    let add = |fqcn: &Arc<str>, all: &mut Vec<Arc<str>>, seen: &mut HashSet<String>| {
        if seen.insert(fqcn.to_string()) {
            all.push(fqcn.clone());
        }
    };

    if node.is_interface(db) {
        for e in node.extends(db).iter() {
            add(e, &mut all, &mut seen);
            if let Some(parent_node) = db.lookup_class_node(e) {
                for a in class_ancestors(db, parent_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
    } else {
        if let Some(ref p) = node.parent(db) {
            add(p, &mut all, &mut seen);
            if let Some(parent_node) = db.lookup_class_node(p) {
                for a in class_ancestors(db, parent_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
        for iface in node.interfaces(db).iter() {
            add(iface, &mut all, &mut seen);
            if let Some(iface_node) = db.lookup_class_node(iface) {
                for a in class_ancestors(db, iface_node).0 {
                    add(&a, &mut all, &mut seen);
                }
            }
        }
        for t in node.traits(db).iter() {
            add(t, &mut all, &mut seen);
        }
    }

    Ancestors(all)
}

// ---------------------------------------------------------------------------
// collect_file_definitions tracked query (S1)
// ---------------------------------------------------------------------------

#[salsa::tracked]
pub fn collect_file_definitions(db: &dyn MirDatabase, file: SourceFile) -> FileDefinitions {
    let path = file.path(db);
    let text = file.text(db);

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, &text);

    let mut all_issues: Vec<Issue> = parsed
        .errors
        .iter()
        .map(|err| {
            Issue::new(
                mir_issues::IssueKind::ParseError {
                    message: err.to_string(),
                },
                mir_issues::Location {
                    file: path.clone(),
                    line: 1,
                    line_end: 1,
                    col_start: 0,
                    col_end: 0,
                },
            )
        })
        .collect();

    let collector =
        crate::collector::DefinitionCollector::new_for_slice(path, &text, &parsed.source_map);
    let (slice, collector_issues) = collector.collect_slice(&parsed.program);
    all_issues.extend(collector_issues);

    FileDefinitions {
        slice: Arc::new(slice),
        issues: Arc::new(all_issues),
    }
}

// ---------------------------------------------------------------------------
// MirDb concrete database
// ---------------------------------------------------------------------------

/// Concrete in-process Salsa database.
#[salsa::db]
#[derive(Default)]
pub struct MirDb {
    storage: salsa::Storage<Self>,
    /// FQCN → ClassNode handle registry (not tracked by Salsa; see
    /// `lookup_class_node` for the rationale).
    class_nodes: HashMap<Arc<str>, ClassNode>,
    /// FQN → FunctionNode handle registry.
    function_nodes: HashMap<Arc<str>, FunctionNode>,
}

#[salsa::db]
impl salsa::Database for MirDb {}

#[salsa::db]
impl MirDatabase for MirDb {
    fn php_version_str(&self) -> Arc<str> {
        Arc::from("8.2")
    }

    fn lookup_class_node(&self, fqcn: &str) -> Option<ClassNode> {
        self.class_nodes.get(fqcn).copied()
    }

    fn lookup_function_node(&self, fqn: &str) -> Option<FunctionNode> {
        self.function_nodes.get(fqn).copied()
    }
}

impl MirDb {
    /// Create or update the `ClassNode` for `fqcn`.
    ///
    /// If a handle already exists, its fields are updated in-place so Salsa
    /// can track the change.  A new handle is created only on first registration.
    pub fn upsert_class_node(
        &mut self,
        fqcn: Arc<str>,
        is_interface: bool,
        parent: Option<Arc<str>>,
        interfaces: Arc<[Arc<str>]>,
        traits: Arc<[Arc<str>]>,
        extends: Arc<[Arc<str>]>,
    ) -> ClassNode {
        use salsa::Setter as _;
        if let Some(&node) = self.class_nodes.get(&fqcn) {
            node.set_active(self).to(true);
            node.set_is_interface(self).to(is_interface);
            node.set_parent(self).to(parent);
            node.set_interfaces(self).to(interfaces);
            node.set_traits(self).to(traits);
            node.set_extends(self).to(extends);
            node
        } else {
            let node = ClassNode::new(
                self,
                fqcn.clone(),
                true,
                is_interface,
                parent,
                interfaces,
                traits,
                extends,
            );
            self.class_nodes.insert(fqcn, node);
            node
        }
    }

    /// Mark the `ClassNode` for `fqcn` as inactive.
    ///
    /// Dependent `class_ancestors` queries will observe the change and re-run,
    /// returning an empty list.
    pub fn deactivate_class_node(&mut self, fqcn: &str) {
        use salsa::Setter as _;
        if let Some(&node) = self.class_nodes.get(fqcn) {
            node.set_active(self).to(false);
        }
    }

    /// Create or update the `FunctionNode` for the given `FunctionStorage`.
    pub fn upsert_function_node(&mut self, storage: &FunctionStorage) -> FunctionNode {
        use salsa::Setter as _;
        let fqn = &storage.fqn;
        if let Some(&node) = self.function_nodes.get(fqn.as_ref()) {
            node.set_active(self).to(true);
            node.set_params(self)
                .to(Arc::from(storage.params.as_slice()));
            node.set_return_type(self).to(storage.return_type.clone());
            node.set_template_params(self)
                .to(Arc::from(storage.template_params.as_slice()));
            node.set_assertions(self)
                .to(Arc::from(storage.assertions.as_slice()));
            node.set_throws(self)
                .to(Arc::from(storage.throws.as_slice()));
            node.set_deprecated(self).to(storage.deprecated.clone());
            node.set_is_pure(self).to(storage.is_pure);
            node
        } else {
            let node = FunctionNode::new(
                self,
                fqn.clone(),
                true,
                Arc::from(storage.params.as_slice()),
                storage.return_type.clone(),
                Arc::from(storage.template_params.as_slice()),
                Arc::from(storage.assertions.as_slice()),
                Arc::from(storage.throws.as_slice()),
                storage.deprecated.clone(),
                storage.is_pure,
            );
            self.function_nodes.insert(fqn.clone(), node);
            node
        }
    }

    /// Mark the `FunctionNode` for `fqn` as inactive.
    pub fn deactivate_function_node(&mut self, fqn: &str) {
        use salsa::Setter as _;
        if let Some(&node) = self.function_nodes.get(fqn) {
            node.set_active(self).to(false);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use salsa::Setter as _;

    #[test]
    fn mirdb_constructs() {
        let _db = MirDb::default();
    }

    #[test]
    fn source_file_input_roundtrip() {
        let db = MirDb::default();
        let file = SourceFile::new(&db, Arc::from("/tmp/test.php"), Arc::from("<?php echo 1;"));
        assert_eq!(file.path(&db).as_ref(), "/tmp/test.php");
        assert_eq!(file.text(&db).as_ref(), "<?php echo 1;");
    }

    #[test]
    fn collect_file_definitions_basic() {
        let db = MirDb::default();
        let src = Arc::from("<?php class Foo {}");
        let file = SourceFile::new(&db, Arc::from("/tmp/foo.php"), src);
        let defs = collect_file_definitions(&db, file);
        assert!(defs.issues.is_empty());
        assert_eq!(defs.slice.classes.len(), 1);
        assert_eq!(defs.slice.classes[0].fqcn.as_ref(), "Foo");
    }

    #[test]
    fn collect_file_definitions_memoized() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/memo.php"),
            Arc::from("<?php class Bar {}"),
        );

        let defs1 = collect_file_definitions(&db, file);
        let defs2 = collect_file_definitions(&db, file);
        assert!(
            Arc::ptr_eq(&defs1.slice, &defs2.slice),
            "unchanged file must return the memoized result"
        );
    }

    #[test]
    fn collect_file_definitions_recomputes_on_change() {
        let mut db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/memo2.php"),
            Arc::from("<?php class Foo {}"),
        );

        let defs1 = collect_file_definitions(&db, file);
        file.set_text(&mut db)
            .to(Arc::from("<?php class Foo {} class Bar {}"));
        let defs2 = collect_file_definitions(&db, file);

        assert!(
            !Arc::ptr_eq(&defs1.slice, &defs2.slice),
            "changed file must produce a new result"
        );
        assert_eq!(defs2.slice.classes.len(), 2);
    }

    #[test]
    fn class_ancestors_empty_for_root_class() {
        let mut db = MirDb::default();
        let node = db.upsert_class_node(
            Arc::from("Foo"),
            false,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        let ancestors = class_ancestors(&db, node);
        assert!(ancestors.0.is_empty(), "root class has no ancestors");
    }

    #[test]
    fn class_ancestors_single_parent() {
        let mut db = MirDb::default();
        db.upsert_class_node(
            Arc::from("Base"),
            false,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        let child = db.upsert_class_node(
            Arc::from("Child"),
            false,
            Some(Arc::from("Base")),
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        let ancestors = class_ancestors(&db, child);
        assert_eq!(ancestors.0.len(), 1);
        assert_eq!(ancestors.0[0].as_ref(), "Base");
    }

    #[test]
    fn class_ancestors_transitive() {
        let mut db = MirDb::default();
        db.upsert_class_node(
            Arc::from("GrandParent"),
            false,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        db.upsert_class_node(
            Arc::from("Parent"),
            false,
            Some(Arc::from("GrandParent")),
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        let child = db.upsert_class_node(
            Arc::from("Child"),
            false,
            Some(Arc::from("Parent")),
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        let ancestors = class_ancestors(&db, child);
        assert_eq!(ancestors.0.len(), 2);
        assert_eq!(ancestors.0[0].as_ref(), "Parent");
        assert_eq!(ancestors.0[1].as_ref(), "GrandParent");
    }

    #[test]
    fn class_ancestors_cycle_returns_empty() {
        let mut db = MirDb::default();
        // A extends A — not valid PHP, but we must not panic.
        let node_a = db.upsert_class_node(
            Arc::from("A"),
            false,
            Some(Arc::from("A")),
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        let ancestors = class_ancestors(&db, node_a);
        // Cycle recovery: empty list (A's ancestors exclude itself).
        assert!(ancestors.0.is_empty(), "cycle must yield empty ancestors");
    }

    #[test]
    fn class_ancestors_inactive_node_returns_empty() {
        let mut db = MirDb::default();
        let node = db.upsert_class_node(
            Arc::from("Foo"),
            false,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        db.deactivate_class_node("Foo");
        let ancestors = class_ancestors(&db, node);
        assert!(ancestors.0.is_empty(), "inactive node must yield empty");
    }

    #[test]
    fn class_ancestors_recomputes_on_parent_change() {
        let mut db = MirDb::default();
        db.upsert_class_node(
            Arc::from("Base"),
            false,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        let child = db.upsert_class_node(
            Arc::from("Child"),
            false,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );

        let before = class_ancestors(&db, child);
        assert!(before.0.is_empty());

        // Add Base as parent of Child.
        child.set_parent(&mut db).to(Some(Arc::from("Base")));

        let after = class_ancestors(&db, child);
        assert_eq!(after.0.len(), 1);
        assert_eq!(after.0[0].as_ref(), "Base");
    }

    #[test]
    fn interface_ancestors_via_extends() {
        let mut db = MirDb::default();
        db.upsert_class_node(
            Arc::from("Countable"),
            true,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([]),
        );
        let child_iface = db.upsert_class_node(
            Arc::from("Collection"),
            true,
            None,
            Arc::from([]),
            Arc::from([]),
            Arc::from([Arc::from("Countable")]),
        );
        let ancestors = class_ancestors(&db, child_iface);
        assert_eq!(ancestors.0.len(), 1);
        assert_eq!(ancestors.0[0].as_ref(), "Countable");
    }
}
