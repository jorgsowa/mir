//! Tests for the Phase 3 pull-based find_* queries.
//!
//! The headline contract: `find_class_like(fqcn)` and `find_function(fqn)`
//! work **without any prior `ingest_file` call** — only `set_file_text`
//! (which registers source text in salsa) plus a resolver. Pass-1
//! collection runs on demand inside `collect_file_definitions`.
//!
//! This proves the pull architecture is wired correctly end-to-end:
//!   resolver → resolve_fqcn_to_path → lookup_source_file
//!            → collect_file_definitions (lazy parse + collect)
//!            → class_in_file / function_in_file (linear scan)

use std::path::PathBuf;
use std::sync::Arc;

use mir_analyzer::db::{
    class_ancestors_by_fqcn, class_in_file, find_class_constant_in_chain,
    find_class_constant_in_class, find_class_like, find_function, find_method_in_chain,
    find_method_in_class, find_property_in_chain, find_property_in_class, function_in_file,
    ClassLike, Fqcn,
};
use mir_analyzer::{AnalysisSession, ClassResolver, PhpVersion};

struct StubResolver {
    mapping: std::collections::HashMap<String, PathBuf>,
}

impl ClassResolver for StubResolver {
    fn resolve(&self, fqcn: &str) -> Option<PathBuf> {
        self.mapping.get(fqcn).cloned()
    }
}

fn make_resolver(entries: &[(&str, &str)]) -> Arc<dyn ClassResolver> {
    let mapping = entries
        .iter()
        .map(|(k, v)| ((*k).to_string(), PathBuf::from(*v)))
        .collect();
    Arc::new(StubResolver { mapping })
}

#[test]
fn class_in_file_finds_class_after_set_file_text_only() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo {}\n"),
    );

    let db = session.snapshot_db();
    let sf = mir_analyzer::db::MirDatabase::lookup_source_file(&db, "/proj/Foo.php")
        .expect("source file must be registered after set_file_text");
    let fqcn = Fqcn::new(&db, Arc::from("App\\Foo"));
    let class = class_in_file(&db, sf, fqcn);
    assert!(
        class.is_some(),
        "class_in_file must demand collect_file_definitions and find App\\Foo"
    );
    assert_eq!(class.as_ref().unwrap().fqcn.as_ref(), "App\\Foo");
    assert_eq!(class.unwrap().short_name.as_ref(), "Foo");
}

#[test]
fn function_in_file_finds_function_after_set_file_text_only() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/helpers.php"),
        Arc::from("<?php\nnamespace App;\nfunction greet(string $n): string { return $n; }\n"),
    );

    let db = session.snapshot_db();
    let sf = mir_analyzer::db::MirDatabase::lookup_source_file(&db, "/proj/helpers.php")
        .expect("source file must be registered");
    let fqn = Fqcn::new(&db, Arc::from("App\\greet"));
    let func = function_in_file(&db, sf, fqn);
    assert!(
        func.is_some(),
        "function_in_file must demand collect_file_definitions and find App\\greet"
    );
    assert_eq!(func.unwrap().fqn.as_ref(), "App\\greet");
}

#[test]
fn find_class_like_combines_resolution_and_extraction() {
    // The headline test for the pull architecture: set up a resolver +
    // register file text, then ask find_class_like to do the whole thing
    // in one call. No `ingest_file` involved.
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo {}\n"),
    );

    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Foo"));
    let result = find_class_like(&db, fqcn);
    match result {
        Some(ClassLike::Class(c)) => {
            assert_eq!(c.fqcn.as_ref(), "App\\Foo");
        }
        other => panic!("expected Some(Class), got {other:?}"),
    }
}

#[test]
fn find_class_like_returns_interface_kind() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\HasFoo", "/proj/HasFoo.php")]));
    session.set_file_text(
        Arc::from("/proj/HasFoo.php"),
        Arc::from("<?php\nnamespace App;\ninterface HasFoo {}\n"),
    );

    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\HasFoo"));
    assert!(matches!(
        find_class_like(&db, fqcn),
        Some(ClassLike::Interface(_))
    ));
}

#[test]
fn find_function_finds_via_resolver() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\greet", "/proj/helpers.php")]));
    session.set_file_text(
        Arc::from("/proj/helpers.php"),
        Arc::from("<?php\nnamespace App;\nfunction greet(): string { return 'hi'; }\n"),
    );

    let db = session.snapshot_db();
    let fqn = Fqcn::new(&db, Arc::from("App\\greet"));
    let func = find_function(&db, fqn);
    assert!(
        func.is_some(),
        "find_function must resolve and extract in one call"
    );
    assert_eq!(func.unwrap().fqn.as_ref(), "App\\greet");
}

#[test]
fn find_returns_none_when_file_not_registered() {
    // Resolver maps the FQCN to a path, but the file text was never
    // registered. The query falls through cleanly without panicking.
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));

    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Foo"));
    assert!(find_class_like(&db, fqcn).is_none());
}

/// The Phase-4 unblocker: with stub-aware resolver in place,
/// `find_class_like` can locate built-in PHP classes by FQCN without any
/// user-side setup. Pass-2 migration depends on this — Pass-2 references
/// stub classes (Exception, ArrayObject, …) constantly.
#[test]
fn find_class_like_resolves_php_builtin_via_stub_resolver() {
    use std::path::PathBuf;
    struct EmptyResolver;
    impl ClassResolver for EmptyResolver {
        fn resolve(&self, _: &str) -> Option<PathBuf> {
            None
        }
    }
    // Empty user resolver + automatic stub-aware wrap from
    // `with_class_resolver`. The stub resolver maps "ArrayObject" →
    // its bundled stub path; `ensure_stubs_loaded` registers that path
    // as a SourceFile so `find_class_like` finds it.
    let session =
        AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(Arc::new(EmptyResolver));
    session.ensure_stubs_loaded();

    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("ArrayObject"));
    let found = find_class_like(&db, fqcn);
    assert!(
        found.is_some(),
        "stub-aware resolver + SourceFile registration must let \
         find_class_like locate ArrayObject; got None"
    );
    assert!(matches!(found, Some(ClassLike::Class(_))));
}

#[test]
fn find_returns_class_from_set_file_text() {
    // With the pull-based architecture, set_file_text is enough to make a
    // class findable via workspace_symbol_index — no resolver needed.
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nclass Foo {}\n"),
    );

    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("Foo"));
    assert!(
        find_class_like(&db, fqcn).is_some(),
        "class registered via set_file_text must be findable"
    );
}

#[test]
fn find_returns_none_for_unregistered_class() {
    // A class that was never registered in any way must return None.
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("NeverRegistered"));
    assert!(find_class_like(&db, fqcn).is_none());
}

#[test]
fn find_method_in_class_finds_own_method() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo { public function bar(): void {} }\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Foo"));
    let m = find_method_in_class(&db, fqcn, "bar");
    assert!(m.is_some(), "find_method_in_class must find App\\Foo::bar");
}

#[test]
fn find_method_in_class_is_case_insensitive() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo { public function camelCase(): void {} }\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Foo"));
    assert!(find_method_in_class(&db, fqcn, "camelcase").is_some());
    assert!(find_method_in_class(&db, fqcn, "CamelCase").is_some());
    assert!(find_method_in_class(&db, fqcn, "CAMELCASE").is_some());
}

#[test]
fn find_property_in_class_finds_own_property() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo { public string $name = ''; }\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Foo"));
    assert!(find_property_in_class(&db, fqcn, "name").is_some());
}

#[test]
fn find_class_constant_in_class_finds_own_constant() {
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo { const ANSWER = 42; }\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Foo"));
    assert!(find_class_constant_in_class(&db, fqcn, "ANSWER").is_some());
}

#[test]
fn class_ancestors_walks_parent_chain() {
    let session = AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
        ("App\\Base", "/proj/Base.php"),
        ("App\\Mid", "/proj/Mid.php"),
        ("App\\Leaf", "/proj/Leaf.php"),
    ]));
    session.set_file_text(
        Arc::from("/proj/Base.php"),
        Arc::from("<?php\nnamespace App;\nclass Base {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Mid.php"),
        Arc::from("<?php\nnamespace App;\nclass Mid extends Base {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Leaf.php"),
        Arc::from("<?php\nnamespace App;\nclass Leaf extends Mid {}\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Leaf"));
    let ancestors = class_ancestors_by_fqcn(&db, fqcn);
    let names: Vec<&str> = ancestors.iter().map(|s| s.as_ref()).collect();
    assert_eq!(names, vec!["App\\Leaf", "App\\Mid", "App\\Base"]);
}

#[test]
fn find_method_in_chain_finds_inherited_method() {
    let session = AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
        ("App\\Base", "/proj/Base.php"),
        ("App\\Child", "/proj/Child.php"),
    ]));
    session.set_file_text(
        Arc::from("/proj/Base.php"),
        Arc::from("<?php\nnamespace App;\nclass Base { public function inherited(): void {} }\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Child.php"),
        Arc::from("<?php\nnamespace App;\nclass Child extends Base {}\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Child"));
    let (declared_in, _m) = find_method_in_chain(&db, fqcn, "inherited")
        .expect("find_method_in_chain must walk to App\\Base");
    assert_eq!(declared_in.as_ref(), "App\\Base");
}

#[test]
fn find_property_in_chain_finds_inherited_property() {
    let session = AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
        ("App\\Base", "/proj/Base.php"),
        ("App\\Child", "/proj/Child.php"),
    ]));
    session.set_file_text(
        Arc::from("/proj/Base.php"),
        Arc::from("<?php\nnamespace App;\nclass Base { public string $name = ''; }\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Child.php"),
        Arc::from("<?php\nnamespace App;\nclass Child extends Base {}\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Child"));
    let (declared_in, _p) = find_property_in_chain(&db, fqcn, "name")
        .expect("find_property_in_chain must walk to App\\Base");
    assert_eq!(declared_in.as_ref(), "App\\Base");
}

#[test]
fn find_class_constant_in_chain_finds_inherited_constant() {
    let session = AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
        ("App\\Base", "/proj/Base.php"),
        ("App\\Child", "/proj/Child.php"),
    ]));
    session.set_file_text(
        Arc::from("/proj/Base.php"),
        Arc::from("<?php\nnamespace App;\nclass Base { const ANSWER = 42; }\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Child.php"),
        Arc::from("<?php\nnamespace App;\nclass Child extends Base {}\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\Child"));
    let (declared_in, _c) = find_class_constant_in_chain(&db, fqcn, "ANSWER")
        .expect("find_class_constant_in_chain must walk to App\\Base");
    assert_eq!(declared_in.as_ref(), "App\\Base");
}

#[test]
fn ancestor_walk_handles_cycles() {
    // Intentional cycle in @extends-style docblock or genuinely circular
    // PHP code: A extends B, B extends A. We should not loop forever; the
    // walk should terminate at the first duplicate.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
        ("App\\A", "/proj/A.php"),
        ("App\\B", "/proj/B.php"),
    ]));
    session.set_file_text(
        Arc::from("/proj/A.php"),
        Arc::from("<?php\nnamespace App;\nclass A extends B {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/B.php"),
        Arc::from("<?php\nnamespace App;\nclass B extends A {}\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Arc::from("App\\A"));
    let ancestors = class_ancestors_by_fqcn(&db, fqcn);
    assert_eq!(ancestors.len(), 2, "cycle must terminate after 2 entries");
    let names: Vec<&str> = ancestors.iter().map(|s| s.as_ref()).collect();
    assert!(names.contains(&"App\\A"));
    assert!(names.contains(&"App\\B"));
}
