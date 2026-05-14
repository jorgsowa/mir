use std::sync::Arc;

use mir_codebase::storage::{
    ConstantStorage, MethodStorage, PropertyStorage, TemplateParam, Visibility,
};

// Import everything from parent module (mod.rs re-exports)
use super::*;

// Tests

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::*;
    use salsa::Setter as _;

    fn upsert_class(
        db: &mut MirDb,
        fqcn: &str,
        parent: Option<Arc<str>>,
        extends: Arc<[Arc<str>]>,
        is_interface: bool,
    ) -> ClassNode {
        db.upsert_class_node(ClassNodeFields {
            is_interface,
            parent,
            extends,
            ..ClassNodeFields::for_class(Arc::from(fqcn))
        })
    }

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
    fn analyze_file_accumulates_parse_errors() {
        let db = MirDb::default();
        // Unterminated string literal — guaranteed parser diagnostic.
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/parse_err.php"),
            Arc::from("<?php $x = \"unterminated"),
        );
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        analyze_file(&db, file, input);
        let issues: Vec<&IssueAccumulator> = analyze_file::accumulated(&db, file, input);
        assert!(
            !issues.is_empty(),
            "expected parse error to surface as accumulated IssueAccumulator"
        );
        assert!(matches!(
            issues[0].0.kind,
            mir_issues::IssueKind::ParseError { .. }
        ));
    }

    #[test]
    fn analyze_file_clean_input_accumulates_nothing() {
        let db = MirDb::default();
        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/clean.php"),
            Arc::from("<?php class Foo {}"),
        );
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        analyze_file(&db, file, input);
        let issues: Vec<&IssueAccumulator> = analyze_file::accumulated(&db, file, input);
        let refs: Vec<&RefLocAccumulator> = analyze_file::accumulated(&db, file, input);
        assert!(issues.is_empty());
        assert!(refs.is_empty());
    }

    #[test]
    fn analyze_file_calls_pass2_for_undefined_class() {
        let mut db = MirDb::default();
        // Load stubs so we have a baseline codebase
        for slice in crate::stubs::builtin_stub_slices_for_version(crate::PhpVersion::LATEST) {
            db.ingest_stub_slice(&slice);
        }

        let file = SourceFile::new(
            &db,
            Arc::from("/tmp/test_pass2.php"),
            Arc::from("<?php function foo() { new UndefinedClass(); }"),
        );
        let input = AnalyzeFileInput::new(&db, Arc::from("8.2"));
        analyze_file(&db, file, input);
        let issues: Vec<&IssueAccumulator> = analyze_file::accumulated(&db, file, input);

        assert!(
            !issues.is_empty(),
            "Pass2Driver should emit UndefinedClass issue"
        );
        assert!(issues
            .iter()
            .any(|acc| matches!(acc.0.kind, mir_issues::IssueKind::UndefinedClass { .. })));
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
        let node = upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        let ancestors = class_ancestors(&db, node);
        assert!(ancestors.0.is_empty(), "root class has no ancestors");
    }

    #[test]
    fn class_ancestors_single_parent() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        let child = upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        let ancestors = class_ancestors(&db, child);
        assert_eq!(ancestors.0.len(), 1);
        assert_eq!(ancestors.0[0].as_ref(), "Base");
    }

    #[test]
    fn class_ancestors_transitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "GrandParent", None, Arc::from([]), false);
        upsert_class(
            &mut db,
            "Parent",
            Some(Arc::from("GrandParent")),
            Arc::from([]),
            false,
        );
        let child = upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Parent")),
            Arc::from([]),
            false,
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
        let node_a = upsert_class(&mut db, "A", Some(Arc::from("A")), Arc::from([]), false);
        let ancestors = class_ancestors(&db, node_a);
        // Cycle recovery: empty list (A's ancestors exclude itself).
        assert!(ancestors.0.is_empty(), "cycle must yield empty ancestors");
    }

    #[test]
    fn class_ancestors_inactive_node_returns_empty() {
        let mut db = MirDb::default();
        let node = upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        db.deactivate_class_node("Foo");
        let ancestors = class_ancestors(&db, node);
        assert!(ancestors.0.is_empty(), "inactive node must yield empty");
    }

    #[test]
    fn class_ancestors_recomputes_on_parent_change() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        let child = upsert_class(&mut db, "Child", None, Arc::from([]), false);

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
        upsert_class(&mut db, "Countable", None, Arc::from([]), true);
        let child_iface = upsert_class(
            &mut db,
            "Collection",
            None,
            Arc::from([Arc::from("Countable")]),
            true,
        );
        let ancestors = class_ancestors(&db, child_iface);
        assert_eq!(ancestors.0.len(), 1);
        assert_eq!(ancestors.0[0].as_ref(), "Countable");
    }

    #[test]
    fn type_exists_via_db_tracks_active_state() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        assert!(type_exists_via_db(&db, "Foo"));
        assert!(!type_exists_via_db(&db, "Bar"));
        db.deactivate_class_node("Foo");
        assert!(!type_exists_via_db(&db, "Foo"));
    }

    #[test]
    fn clone_preserves_class_node_lookups() {
        // PR10a: each parallel batch worker gets its own MirDb clone.
        // Verify the clone observes the same registered nodes.
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        let cloned = db.clone();
        assert!(
            type_exists_via_db(&cloned, "Foo"),
            "clone must observe nodes registered before clone()"
        );
        assert!(
            !type_exists_via_db(&cloned, "Bar"),
            "clone must not observe nodes that were never registered"
        );
        // Clones must also resolve ancestors through the same shared storage.
        let foo_node = cloned.lookup_class_node("Foo").expect("registered");
        let ancestors = class_ancestors(&cloned, foo_node);
        assert!(ancestors.0.is_empty(), "Foo has no ancestors");
    }

    // -----------------------------------------------------------------
    // Helpers for method-related fixtures
    // -----------------------------------------------------------------

    fn upsert_class_with_traits(
        db: &mut MirDb,
        fqcn: &str,
        parent: Option<Arc<str>>,
        traits: &[&str],
        is_interface: bool,
        is_trait: bool,
    ) -> ClassNode {
        db.upsert_class_node(ClassNodeFields {
            is_interface,
            is_trait,
            parent,
            traits: Arc::from(
                traits
                    .iter()
                    .map(|t| Arc::<str>::from(*t))
                    .collect::<Vec<_>>(),
            ),
            ..ClassNodeFields::for_class(Arc::from(fqcn))
        })
    }

    fn upsert_method(db: &mut MirDb, fqcn: &str, name: &str, is_abstract: bool) -> MethodNode {
        let storage = MethodStorage {
            name: Arc::from(name),
            fqcn: Arc::from(fqcn),
            params: Arc::from([].as_slice()),
            return_type: None,
            inferred_return_type: None,
            visibility: Visibility::Public,
            is_static: false,
            is_abstract,
            is_final: false,
            is_constructor: name == "__construct",
            template_params: vec![],
            assertions: vec![],
            throws: vec![],
            deprecated: None,
            is_internal: false,
            is_pure: false,
            location: None,
            docstring: None,
        };
        db.upsert_method_node(&storage)
    }

    fn upsert_enum(db: &mut MirDb, fqcn: &str, interfaces: &[&str], is_backed: bool) -> ClassNode {
        db.upsert_class_node(ClassNodeFields {
            interfaces: Arc::from(
                interfaces
                    .iter()
                    .map(|i| Arc::<str>::from(*i))
                    .collect::<Vec<_>>(),
            ),
            is_backed_enum: is_backed,
            ..ClassNodeFields::for_enum(Arc::from(fqcn))
        })
    }

    // -----------------------------------------------------------------
    // method_exists_via_db
    // -----------------------------------------------------------------

    #[test]
    fn method_exists_via_db_finds_own_method() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "bar", false);
        assert!(method_exists_via_db(&db, "Foo", "bar"));
        assert!(!method_exists_via_db(&db, "Foo", "missing"));
    }

    #[test]
    fn method_exists_via_db_walks_parent() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_method(&mut db, "Base", "inherited", false);
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        assert!(method_exists_via_db(&db, "Child", "inherited"));
    }

    #[test]
    fn method_exists_via_db_walks_traits_transitively() {
        let mut db = MirDb::default();
        upsert_class_with_traits(&mut db, "InnerTrait", None, &[], false, true);
        upsert_method(&mut db, "InnerTrait", "deep_trait_method", false);
        upsert_class_with_traits(&mut db, "OuterTrait", None, &["InnerTrait"], false, true);
        upsert_class_with_traits(&mut db, "Foo", None, &["OuterTrait"], false, false);
        assert!(method_exists_via_db(&db, "Foo", "deep_trait_method"));
    }

    #[test]
    fn method_exists_via_db_is_case_insensitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "doStuff", false);
        // Stored with original case; lookup must lowercase internally.
        assert!(method_exists_via_db(&db, "Foo", "DoStuff"));
        assert!(method_exists_via_db(&db, "Foo", "DOSTUFF"));
    }

    #[test]
    fn method_exists_via_db_unknown_class_returns_false() {
        let db = MirDb::default();
        assert!(!method_exists_via_db(&db, "Nope", "anything"));
    }

    #[test]
    fn method_exists_via_db_inactive_class_returns_false() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "bar", false);
        db.deactivate_class_node("Foo");
        assert!(!method_exists_via_db(&db, "Foo", "bar"));
    }

    #[test]
    fn method_exists_via_db_finds_abstract_methods() {
        // Existence-only: abstracts count.  This is the difference vs.
        // method_is_concretely_implemented.
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "abstr", true);
        assert!(method_exists_via_db(&db, "Foo", "abstr"));
    }

    // -----------------------------------------------------------------
    // method_is_concretely_implemented
    // -----------------------------------------------------------------

    #[test]
    fn method_is_concretely_implemented_skips_abstract() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "abstr", true);
        assert!(!method_is_concretely_implemented(&db, "Foo", "abstr"));
    }

    #[test]
    fn method_is_concretely_implemented_finds_concrete_in_trait() {
        let mut db = MirDb::default();
        upsert_class_with_traits(&mut db, "MyTrait", None, &[], false, true);
        upsert_method(&mut db, "MyTrait", "provided", false);
        upsert_class_with_traits(&mut db, "Foo", None, &["MyTrait"], false, false);
        assert!(method_is_concretely_implemented(&db, "Foo", "provided"));
    }

    #[test]
    fn method_is_concretely_implemented_skips_interface_definitions() {
        // Interfaces don't supply implementations, regardless of how
        // their methods are stored.
        let mut db = MirDb::default();
        upsert_class(&mut db, "I", None, Arc::from([]), true);
        upsert_method(&mut db, "I", "m", false);
        upsert_class(&mut db, "C", None, Arc::from([Arc::from("I")]), false);
        // C "implements" I but has no own implementation.
        assert!(!method_is_concretely_implemented(&db, "C", "m"));
    }

    // -----------------------------------------------------------------
    // extends_or_implements_via_db
    // -----------------------------------------------------------------

    #[test]
    fn extends_or_implements_via_db_self_match() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        assert!(extends_or_implements_via_db(&db, "Foo", "Foo"));
    }

    #[test]
    fn extends_or_implements_via_db_transitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Animal", None, Arc::from([]), false);
        upsert_class(
            &mut db,
            "Mammal",
            Some(Arc::from("Animal")),
            Arc::from([]),
            false,
        );
        upsert_class(
            &mut db,
            "Dog",
            Some(Arc::from("Mammal")),
            Arc::from([]),
            false,
        );
        assert!(extends_or_implements_via_db(&db, "Dog", "Animal"));
        assert!(extends_or_implements_via_db(&db, "Dog", "Mammal"));
        assert!(!extends_or_implements_via_db(&db, "Animal", "Dog"));
    }

    #[test]
    fn extends_or_implements_via_db_unknown_returns_false() {
        let db = MirDb::default();
        assert!(!extends_or_implements_via_db(&db, "Nope", "Foo"));
    }

    #[test]
    fn extends_or_implements_via_db_unit_enum_implicit() {
        let mut db = MirDb::default();
        upsert_enum(&mut db, "Status", &[], false);
        assert!(extends_or_implements_via_db(&db, "Status", "UnitEnum"));
        assert!(extends_or_implements_via_db(&db, "Status", "\\UnitEnum"));
        // Pure enum is NOT a BackedEnum.
        assert!(!extends_or_implements_via_db(&db, "Status", "BackedEnum"));
    }

    #[test]
    fn extends_or_implements_via_db_backed_enum_implicit() {
        let mut db = MirDb::default();
        upsert_enum(&mut db, "Status", &[], true);
        assert!(extends_or_implements_via_db(&db, "Status", "UnitEnum"));
        assert!(extends_or_implements_via_db(&db, "Status", "BackedEnum"));
        assert!(extends_or_implements_via_db(&db, "Status", "\\BackedEnum"));
    }

    #[test]
    fn extends_or_implements_via_db_enum_declared_interface() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Stringable", None, Arc::from([]), true);
        upsert_enum(&mut db, "Status", &["Stringable"], false);
        assert!(extends_or_implements_via_db(&db, "Status", "Stringable"));
    }

    // -----------------------------------------------------------------
    // has_unknown_ancestor_via_db
    // -----------------------------------------------------------------

    #[test]
    fn has_unknown_ancestor_via_db_clean_chain_returns_false() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        assert!(!has_unknown_ancestor_via_db(&db, "Child"));
    }

    #[test]
    fn has_unknown_ancestor_via_db_missing_parent_returns_true() {
        let mut db = MirDb::default();
        // Child claims to extend Missing, but Missing isn't registered.
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Missing")),
            Arc::from([]),
            false,
        );
        assert!(has_unknown_ancestor_via_db(&db, "Child"));
    }

    #[test]
    fn class_template_params_via_db_returns_registered_params() {
        use mir_types::Variance;
        let mut db = MirDb::default();
        let tp = TemplateParam {
            name: Arc::from("T"),
            bound: None,
            defining_entity: Arc::from("Box"),
            variance: Variance::Invariant,
        };
        db.upsert_class_node(ClassNodeFields {
            template_params: Arc::from([tp.clone()]),
            ..ClassNodeFields::for_class(Arc::from("Box"))
        });
        let got = class_template_params_via_db(&db, "Box").expect("registered");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name.as_ref(), "T");

        assert!(class_template_params_via_db(&db, "Missing").is_none());
        db.deactivate_class_node("Box");
        assert!(class_template_params_via_db(&db, "Box").is_none());
    }

    // -----------------------------------------------------------------
    // lookup_method_in_chain
    // -----------------------------------------------------------------

    fn upsert_class_with_mixins(
        db: &mut MirDb,
        fqcn: &str,
        parent: Option<Arc<str>>,
        mixins: &[&str],
    ) -> ClassNode {
        db.upsert_class_node(ClassNodeFields {
            parent,
            mixins: Arc::from(
                mixins
                    .iter()
                    .map(|m| Arc::<str>::from(*m))
                    .collect::<Vec<_>>(),
            ),
            ..ClassNodeFields::for_class(Arc::from(fqcn))
        })
    }

    #[test]
    fn lookup_method_in_chain_finds_own_then_ancestor() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_method(&mut db, "Base", "shared", false);
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        upsert_method(&mut db, "Child", "shared", false);
        // Own wins over ancestor.
        let found = lookup_method_in_chain(&db, "Child", "shared").expect("own");
        assert_eq!(found.fqcn(&db).as_ref(), "Child");
        // Inherited-only resolves to ancestor.
        upsert_method(&mut db, "Base", "only_in_base", false);
        let found = lookup_method_in_chain(&db, "Child", "only_in_base").expect("ancestor");
        assert_eq!(found.fqcn(&db).as_ref(), "Base");
    }

    #[test]
    fn lookup_method_in_chain_walks_trait_of_traits() {
        let mut db = MirDb::default();
        upsert_class_with_traits(&mut db, "InnerTrait", None, &[], false, true);
        upsert_method(&mut db, "InnerTrait", "deep", false);
        upsert_class_with_traits(&mut db, "OuterTrait", None, &["InnerTrait"], false, true);
        upsert_class_with_traits(&mut db, "Foo", None, &["OuterTrait"], false, false);
        let found = lookup_method_in_chain(&db, "Foo", "deep").expect("transitive trait");
        assert_eq!(found.fqcn(&db).as_ref(), "InnerTrait");
    }

    #[test]
    fn lookup_method_in_chain_walks_mixins() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "MixinTarget", None, Arc::from([]), false);
        upsert_method(&mut db, "MixinTarget", "magic", false);
        upsert_class_with_mixins(&mut db, "Host", None, &["MixinTarget"]);
        let found = lookup_method_in_chain(&db, "Host", "magic").expect("via @mixin");
        assert_eq!(found.fqcn(&db).as_ref(), "MixinTarget");
    }

    #[test]
    fn lookup_method_in_chain_mixin_cycle_does_not_hang() {
        let mut db = MirDb::default();
        // A → B → A (mutual @mixin); neither defines the method.
        upsert_class_with_mixins(&mut db, "A", None, &["B"]);
        upsert_class_with_mixins(&mut db, "B", None, &["A"]);
        assert!(lookup_method_in_chain(&db, "A", "missing").is_none());
    }

    #[test]
    fn lookup_method_in_chain_is_case_insensitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_method(&mut db, "Foo", "doStuff", false);
        assert!(lookup_method_in_chain(&db, "Foo", "DOSTUFF").is_some());
        assert!(lookup_method_in_chain(&db, "Foo", "dostuff").is_some());
    }

    #[test]
    fn lookup_method_in_chain_unknown_returns_none() {
        let db = MirDb::default();
        assert!(lookup_method_in_chain(&db, "Nope", "anything").is_none());
    }

    // -----------------------------------------------------------------
    // lookup_property_in_chain
    // -----------------------------------------------------------------

    fn upsert_property(db: &mut MirDb, fqcn: &str, name: &str, is_readonly: bool) -> PropertyNode {
        let storage = PropertyStorage {
            name: Arc::from(name),
            ty: None,
            inferred_ty: None,
            visibility: Visibility::Public,
            is_static: false,
            is_readonly,
            default: None,
            location: None,
        };
        let owner = Arc::<str>::from(fqcn);
        db.upsert_property_node(&owner, &storage);
        db.lookup_property_node(fqcn, name).expect("registered")
    }

    #[test]
    fn lookup_property_in_chain_own_then_ancestor() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_property(&mut db, "Base", "x", false);
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        // Inherited resolves to Base.
        let found = lookup_property_in_chain(&db, "Child", "x").expect("ancestor");
        assert_eq!(found.fqcn(&db).as_ref(), "Base");
        // Own override wins.
        upsert_property(&mut db, "Child", "x", true);
        let found = lookup_property_in_chain(&db, "Child", "x").expect("own");
        assert_eq!(found.fqcn(&db).as_ref(), "Child");
        assert!(found.is_readonly(&db));
    }

    #[test]
    fn lookup_property_in_chain_walks_mixins() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "MixinTarget", None, Arc::from([]), false);
        upsert_property(&mut db, "MixinTarget", "exposed", false);
        upsert_class_with_mixins(&mut db, "Host", None, &["MixinTarget"]);
        let found = lookup_property_in_chain(&db, "Host", "exposed").expect("via @mixin");
        assert_eq!(found.fqcn(&db).as_ref(), "MixinTarget");
    }

    #[test]
    fn lookup_property_in_chain_mixin_cycle_does_not_hang() {
        let mut db = MirDb::default();
        upsert_class_with_mixins(&mut db, "A", None, &["B"]);
        upsert_class_with_mixins(&mut db, "B", None, &["A"]);
        assert!(lookup_property_in_chain(&db, "A", "missing").is_none());
    }

    #[test]
    fn lookup_property_in_chain_is_case_sensitive() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_property(&mut db, "Foo", "myProp", false);
        assert!(lookup_property_in_chain(&db, "Foo", "myProp").is_some());
        // Property names are case-sensitive in PHP.
        assert!(lookup_property_in_chain(&db, "Foo", "MyProp").is_none());
    }

    #[test]
    fn lookup_property_in_chain_inactive_returns_none() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_property(&mut db, "Foo", "x", false);
        db.deactivate_class_node("Foo");
        assert!(lookup_property_in_chain(&db, "Foo", "x").is_none());
    }

    // -----------------------------------------------------------------
    // class_constant_exists_in_chain
    // -----------------------------------------------------------------

    fn upsert_constant(db: &mut MirDb, fqcn: &str, name: &str) {
        let storage = ConstantStorage {
            name: Arc::from(name),
            ty: mir_types::Union::mixed(),
            visibility: None,
            is_final: false,
            location: None,
        };
        let owner = Arc::<str>::from(fqcn);
        db.upsert_class_constant_node(&owner, &storage);
    }

    #[test]
    fn class_constant_exists_in_chain_finds_own() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_constant(&mut db, "Foo", "MAX");
        assert!(class_constant_exists_in_chain(&db, "Foo", "MAX"));
        assert!(!class_constant_exists_in_chain(&db, "Foo", "MIN"));
    }

    #[test]
    fn class_constant_exists_in_chain_walks_parent() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Base", None, Arc::from([]), false);
        upsert_constant(&mut db, "Base", "VERSION");
        upsert_class(
            &mut db,
            "Child",
            Some(Arc::from("Base")),
            Arc::from([]),
            false,
        );
        assert!(class_constant_exists_in_chain(&db, "Child", "VERSION"));
    }

    #[test]
    fn class_constant_exists_in_chain_walks_interface() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "I", None, Arc::from([]), true);
        upsert_constant(&mut db, "I", "TYPE");
        // A class that implements I — interfaces go in the `interfaces`
        // slot, not the `extends` slot which is interface-only.
        db.upsert_class_node(ClassNodeFields {
            interfaces: Arc::from([Arc::from("I")]),
            ..ClassNodeFields::for_class(Arc::from("Impl"))
        });
        assert!(class_constant_exists_in_chain(&db, "Impl", "TYPE"));
    }

    #[test]
    fn class_constant_exists_in_chain_walks_direct_trait() {
        let mut db = MirDb::default();
        upsert_class_with_traits(&mut db, "T", None, &[], false, true);
        upsert_constant(&mut db, "T", "FROM_TRAIT");
        upsert_class_with_traits(&mut db, "Foo", None, &["T"], false, false);
        assert!(class_constant_exists_in_chain(&db, "Foo", "FROM_TRAIT"));
    }

    #[test]
    fn class_constant_exists_in_chain_unknown_class_returns_false() {
        let db = MirDb::default();
        assert!(!class_constant_exists_in_chain(&db, "Nope", "ANY"));
    }

    #[test]
    fn class_constant_exists_in_chain_inactive_returns_false() {
        let mut db = MirDb::default();
        upsert_class(&mut db, "Foo", None, Arc::from([]), false);
        upsert_constant(&mut db, "Foo", "X");
        db.deactivate_class_node("Foo");
        db.deactivate_class_constants("Foo");
        assert!(!class_constant_exists_in_chain(&db, "Foo", "X"));
    }

    /// Validates the S3-deadlock premise.  After `for_each_with` returns,
    /// all worker clones must drop so that a subsequent setter on the
    /// canonical db (strong-count==1) does not block on
    /// `Storage::cancel_others`.  Wrapped in a join-with-timeout so a
    /// regression hangs for at most 30s instead of forever.
    #[test]
    fn parallel_reads_then_serial_write_does_not_deadlock() {
        use rayon::prelude::*;
        use std::sync::mpsc;
        use std::time::Duration;

        let (tx, rx) = mpsc::channel::<()>();
        std::thread::spawn(move || {
            let mut db = MirDb::default();
            let storage = mir_codebase::storage::FunctionStorage {
                fqn: Arc::from("foo"),
                short_name: Arc::from("foo"),
                params: Arc::from([].as_slice()),
                return_type: None,
                inferred_return_type: None,
                template_params: vec![],
                assertions: vec![],
                throws: vec![],
                deprecated: None,
                is_pure: false,
                location: None,
                docstring: None,
            };
            let node = db.upsert_function_node(&storage);

            // Parallel sweep with cloned dbs; each worker reads via &dyn MirDatabase.
            let db_for_sweep = db.clone();
            (0..256u32)
                .into_par_iter()
                .for_each_with(db_for_sweep, |db, _| {
                    let _ = node.return_type(&*db as &dyn MirDatabase);
                });

            // Sweep is done — clones owned by `for_each_with` are dropped.
            // If any worker-thread retains thread-local Salsa state pointing
            // at a clone, this setter will hang in `Storage::cancel_others`.
            node.set_return_type(&mut db)
                .to(Some(Arc::new(Union::mixed())));
            assert_eq!(node.return_type(&db), Some(Arc::new(Union::mixed())));
            tx.send(()).unwrap();
        });

        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(()) => {}
            Err(_) => {
                panic!("S3 deadlock repro: setter after for_each_with did not return within 30s")
            }
        }
    }

    /// Pins the actual root cause of the original S3 deadlock: a sibling
    /// `MirDb` clone (e.g. the `class_db` used by `ClassAnalyzer` in
    /// `project.rs`) being alive when a setter runs blocks
    /// `Storage::cancel_others` indefinitely.  Dropping the sibling before
    /// the setter unblocks it.
    ///
    /// This is the regression guard for `commit_inferred_return_types`: if
    /// a future refactor hoists a clone past the commit point, this test
    /// fails (either the "while sibling alive, setter is blocked" half
    /// or the "after drop, setter completes" half).
    #[test]
    fn sibling_clone_blocks_setter_until_dropped() {
        use std::sync::mpsc;
        use std::time::Duration;

        let mut db = MirDb::default();
        let storage = mir_codebase::storage::FunctionStorage {
            fqn: Arc::from("foo"),
            short_name: Arc::from("foo"),
            params: Arc::from([].as_slice()),
            return_type: None,
            inferred_return_type: None,
            template_params: vec![],
            assertions: vec![],
            throws: vec![],
            deprecated: None,
            is_pure: false,
            location: None,
            docstring: None,
        };
        let node = db.upsert_function_node(&storage);

        let sibling = db.clone();

        // Move the writer into a worker thread so we can probe its progress
        // without blocking the test.  Channel signals when the setter returns.
        let (tx, rx) = mpsc::channel::<()>();
        let writer = std::thread::spawn(move || {
            node.set_return_type(&mut db)
                .to(Some(Arc::new(Union::mixed())));
            tx.send(()).unwrap();
        });

        // While the sibling clone is alive the setter must NOT make progress —
        // strong-count > 1 forces `cancel_others` to wait.
        match rx.recv_timeout(Duration::from_millis(500)) {
            Err(mpsc::RecvTimeoutError::Timeout) => { /* expected */ }
            Ok(()) => panic!(
                "setter completed while sibling clone was alive — strong-count==1 \
                 invariant of `cancel_others` is broken; commit_inferred_return_types \
                 cannot rely on tight-scoping clones"
            ),
            Err(e) => panic!("unexpected channel error: {e:?}"),
        }

        // Drop the sibling.  Strong-count drops to 1 and the setter unblocks.
        drop(sibling);

        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(()) => {}
            Err(_) => panic!("setter did not complete within 5s after sibling clone dropped"),
        }
        writer.join().expect("writer thread panicked");
    }
}
