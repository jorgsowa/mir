//! Pull-based find_* queries resolve after `set_file_text` alone, with Pass-1
//! collection demanded inside `collect_file_definitions`.

use std::path::PathBuf;
use std::sync::Arc;

use mir_types::Name;

use crate::db::{
    class_in_file, find_class_constant_in_chain, find_class_constant_in_class,
    find_global_constant, find_property_in_chain, find_property_in_class, function_in_file, Fqcn,
};
use crate::{AnalysisSession, ClassResolver, PhpVersion};

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
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo {}\n"),
    );

    let db = session.snapshot_db();
    let sf = crate::db::MirDatabase::lookup_source_file(&db, "/proj/Foo.php")
        .expect("source file must be registered after set_file_text");
    let fqcn = Fqcn::new(&db, Name::new("App\\Foo"));
    let class = class_in_file(&db, sf, fqcn);
    assert!(
        class.is_some(),
        "class_in_file must demand collect_file_definitions and find App\\Foo"
    );
    assert_eq!(class.as_ref().unwrap().fqcn.as_ref(), "App\\Foo");
    assert_eq!(class.as_ref().unwrap().short_name.as_ref(), "Foo");
}

#[test]
fn function_in_file_finds_function_after_set_file_text_only() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/helpers.php"),
        Arc::from("<?php\nnamespace App;\nfunction greet(string $n): string { return $n; }\n"),
    );

    let db = session.snapshot_db();
    let sf = crate::db::MirDatabase::lookup_source_file(&db, "/proj/helpers.php")
        .expect("source file must be registered");
    let fqn = Fqcn::new(&db, Name::new("App\\greet"));
    let func = function_in_file(&db, sf, fqn);
    assert!(
        func.is_some(),
        "function_in_file must demand collect_file_definitions and find App\\greet"
    );
    assert_eq!(func.as_ref().unwrap().fqn.as_ref(), "App\\greet");
}

#[test]
fn find_global_constant_finds_via_workspace_index() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.set_file_text(
        Arc::from("/proj/constants.php"),
        Arc::from("<?php\nnamespace App;\nconst ANSWER = 42;\n"),
    );

    let db = session.snapshot_db();
    let fqn = Fqcn::new(&db, Name::new("App\\ANSWER"));
    let constant = find_global_constant(&db, fqn);
    assert!(
        constant.is_some(),
        "find_global_constant must resolve through the workspace index"
    );
}

#[test]
fn find_property_in_class_finds_own_property() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo { public string $name = ''; }\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Name::new("App\\Foo"));
    assert!(find_property_in_class(&db, fqcn, "name").is_some());
}

#[test]
fn find_class_constant_in_class_finds_own_constant() {
    let mut session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(make_resolver(&[("App\\Foo", "/proj/Foo.php")]));
    session.set_file_text(
        Arc::from("/proj/Foo.php"),
        Arc::from("<?php\nnamespace App;\nclass Foo { const ANSWER = 42; }\n"),
    );
    let db = session.snapshot_db();
    let fqcn = Fqcn::new(&db, Name::new("App\\Foo"));
    assert!(find_class_constant_in_class(&db, fqcn, "ANSWER").is_some());
}

#[test]
fn find_property_in_chain_finds_inherited_property() {
    let mut session =
        AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
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
    let fqcn = Fqcn::new(&db, Name::new("App\\Child"));
    let (declared_in, _p) = find_property_in_chain(&db, fqcn, "name")
        .expect("find_property_in_chain must walk to App\\Base");
    assert_eq!(declared_in.as_ref(), "App\\Base");
}

#[test]
fn find_class_constant_in_chain_finds_inherited_constant() {
    let mut session =
        AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(make_resolver(&[
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
    let fqcn = Fqcn::new(&db, Name::new("App\\Child"));
    let (declared_in, _c) = find_class_constant_in_chain(&db, fqcn, "ANSWER")
        .expect("find_class_constant_in_chain must walk to App\\Base");
    assert_eq!(declared_in.as_ref(), "App\\Base");
}
