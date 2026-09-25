//! Class, function, and method lookups resolve after `set_file_text` alone —
//! no `ingest_file` — through a resolver or the workspace index.

use std::path::PathBuf;
use std::sync::Arc;

use mir_analyzer::db::ClassLike;
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

fn find_class_like(session: &AnalysisSession, fqcn: &str) -> Option<ClassLike> {
    session
        .snapshot()
        .find_class_like(fqcn)
        .expect("not cancelled")
}

fn declaring_class_of_method(
    session: &AnalysisSession,
    class: &str,
    method: &str,
) -> Option<String> {
    session
        .snapshot()
        .find_method_in_chain(class, method)
        .expect("not cancelled")
        .map(|(declared_in, _)| declared_in.to_string())
}

#[test]
fn find_class_like_combines_resolution_and_extraction() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo {}\n"),
    );

    match find_class_like(&session, "App\\Foo") {
        Some(ClassLike::Class(c)) => assert_eq!(c.fqcn.as_ref(), "App\\Foo"),
        other => panic!("expected Some(Class), got {other:?}"),
    }
}

#[test]
fn find_class_like_returns_interface_kind() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\HasFoo", "/proj/HasFoo.php")]));
    session.set_file_text(
        Arc::from("/proj/HasFoo.php"),
        Arc::from("<?php\nnamespace App;\ninterface HasFoo {}\n"),
    );

    assert!(matches!(
        find_class_like(&session, "App\\HasFoo"),
        Some(ClassLike::Interface(_))
    ));
}

#[test]
fn find_function_finds_via_resolver() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\greet", "/proj/helpers.php")]));
    session.set_file_text(
        Arc::from("/proj/helpers.php"),
        Arc::from("<?php\nnamespace App;\nfunction greet(): string { return 'hi'; }\n"),
    );

    let func = session
        .snapshot()
        .find_function("App\\greet")
        .expect("not cancelled")
        .expect("find_function must resolve and extract in one call");
    assert_eq!(func.fqn.as_ref(), "App\\greet");
}

#[test]
fn find_returns_none_when_file_not_registered() {
    // Resolver maps the FQCN to a path, but the file text was never
    // registered. The query falls through cleanly without panicking.
    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));

    assert!(find_class_like(&session, "App\\Foo").is_none());
}

/// Built-in PHP classes resolve by FQCN through the stub-aware resolver
/// wrap, with no user-side setup.
#[test]
fn find_class_like_resolves_php_builtin_via_stub_resolver() {
    struct EmptyResolver;
    impl ClassResolver for EmptyResolver {
        fn resolve(&self, _: &str) -> Option<PathBuf> {
            None
        }
    }
    let mut session =
        AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(Arc::new(EmptyResolver));
    session.ensure_all_stubs();

    assert!(
        matches!(
            find_class_like(&session, "ArrayObject"),
            Some(ClassLike::Class(_))
        ),
        "stub-aware resolver must locate ArrayObject"
    );
}

#[test]
fn find_returns_class_from_set_file_text() {
    // set_file_text alone makes a class findable via the workspace index; no
    // resolver needed.
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nclass Foo {}\n"),
    );

    assert!(
        find_class_like(&session, "Foo").is_some(),
        "class registered via set_file_text must be findable"
    );
    assert!(session.contains_class("Foo"));
}

#[test]
fn find_returns_none_for_unregistered_class() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    assert!(find_class_like(&session, "NeverRegistered").is_none());
    assert!(!session.contains_class("NeverRegistered"));
}

#[test]
fn find_method_finds_own_method() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo { public function bar(): void {} }\n"),
    );

    assert_eq!(
        declaring_class_of_method(&session, "App\\Foo", "bar").as_deref(),
        Some("App\\Foo")
    );
}

#[test]
fn method_lookup_is_case_insensitive() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo { public function camelCase(): void {} }\n"),
    );

    for name in ["camelcase", "CamelCase", "CAMELCASE"] {
        assert!(session.contains_method("App\\Foo", name), "{name}");
        assert!(
            declaring_class_of_method(&session, "App\\Foo", name).is_some(),
            "{name}"
        );
    }
}

#[test]
fn find_method_in_chain_finds_inherited_method() {
    let mut session =
        AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
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

    assert_eq!(
        declaring_class_of_method(&session, "App\\Child", "inherited").as_deref(),
        Some("App\\Base")
    );
}

#[test]
fn ancestors_walk_parent_chain() {
    let mut session =
        AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
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

    let ancestors = session.ancestors_of("App\\Leaf");
    let names: Vec<&str> = ancestors.iter().map(|s| s.as_ref()).collect();
    assert_eq!(names, vec!["App\\Mid", "App\\Base"]);
}

#[test]
fn ancestor_walk_handles_cycles() {
    // A extends B, B extends A: the walk must terminate at the first duplicate.
    let mut session =
        AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
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

    let ancestors = session.ancestors_of("App\\A");
    let names: Vec<&str> = ancestors.iter().map(|s| s.as_ref()).collect();
    assert_eq!(names, vec!["App\\B"], "cycle must terminate");
}

#[test]
fn subtype_files_resolves_plain_fqn_and_aliased_extends() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/Base.php"),
        Arc::from("<?php\nnamespace App;\nclass Base { protected function boot() {} }\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Child.php"),
        Arc::from("<?php\nnamespace App;\nclass Child extends Base {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Grand.php"),
        Arc::from("<?php\nnamespace App;\nclass Grand extends \\App\\Base {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Aliased.php"),
        Arc::from("<?php\nnamespace App\\Sub;\nuse App\\Base as TheBase;\nclass Aliased extends TheBase {}\n"),
    );
    session.set_file_text(
        Arc::from("/proj/Stranger.php"),
        Arc::from("<?php\nnamespace App;\nclass Stranger {}\n"),
    );

    let mut files = session.subtype_files("App\\Base");
    files.sort();
    assert_eq!(
        files,
        vec![
            Arc::<str>::from("/proj/Aliased.php"),
            Arc::<str>::from("/proj/Child.php"),
            Arc::<str>::from("/proj/Grand.php"),
        ],
        "subtype_files must find subclasses regardless of plain/FQN/aliased extends, \
         and exclude the base's own file and unrelated classes"
    );
}
