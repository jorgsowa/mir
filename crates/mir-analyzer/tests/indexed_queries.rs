//! Inverted-index query contracts: `indexed_references_to`,
//! `indexed_subtype_classes`, `indexed_method_implementations`.
//!
//! Files land through `set_file_text` only (the LSP bulk-population path) —
//! the queries' own completeness passes must analyze/commit them on demand,
//! with no `ingest_file` help.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, Name, PhpVersion};

fn session_with(files: &[(&str, &str)]) -> AnalysisSession {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    for (path, text) in files {
        session.set_file_text(Arc::from(*path), Arc::from(*text));
    }
    session
}

fn paths(files: &[(&str, &str)]) -> Vec<Arc<str>> {
    files.iter().map(|(p, _)| Arc::from(*p)).collect()
}

#[test]
fn method_references_across_uncommitted_files() {
    let files = [
        (
            "svc.php",
            "<?php\nnamespace App;\nclass Service { public function process(): void {} }\n",
        ),
        (
            "user.php",
            "<?php\nnamespace App;\nclass User { public function go(Service $s): void { $s->process(); } }\n",
        ),
        (
            "other.php",
            "<?php\nnamespace Other;\nclass Free { public function process(): void {} public function go(): void { $this->process(); } }\n",
        ),
    ];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(
            &Name::method("App\\Service", "process"),
            &paths(&files),
            false,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "only App\\Service::process call sites: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "user.php");

    // The unrelated Other\Free::process must have its own, separate refs.
    let other = session
        .indexed_references_to(
            &Name::method("Other\\Free", "process"),
            &paths(&files),
            false,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(other.len(), 1);
    assert_eq!(other[0].0.as_ref(), "other.php");
}

#[test]
fn include_declaration_appends_name_span() {
    let files = [
        (
            "decl.php",
            "<?php\nclass Widget { public function render(): void {} }\n",
        ),
        (
            "use.php",
            "<?php\nfunction f(Widget $w): void { $w->render(); }\n",
        ),
    ];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(
            &Name::method("Widget", "render"),
            &paths(&files),
            true,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(refs.len(), 2, "call site + declaration: {refs:?}");
    let decl = refs
        .iter()
        .find(|(f, _)| f.as_ref() == "decl.php")
        .expect("declaration entry present");
    // Line 2 (1-based), name token `render` at char col 31.
    assert_eq!(decl.1.start.line, 2);
    assert_eq!(decl.1.start.column, 31);
    assert_eq!(decl.1.end.column, 31 + "render".len() as u32);
}

#[test]
fn constructor_references_at_new_sites() {
    let files = [
        (
            "order.php",
            "<?php\nnamespace Shop;\nclass Order { public function __construct(public int $id) {} }\n",
        ),
        (
            "checkout.php",
            "<?php\nnamespace Shop;\nclass Checkout { public function run(): Order { return new Order(1); } }\n",
        ),
    ];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(
            &Name::method("Shop\\Order", "__construct"),
            &paths(&files),
            false,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(refs.len(), 1, "the new Order(1) site: {refs:?}");
    assert_eq!(refs[0].0.as_ref(), "checkout.php");
}

#[test]
fn class_function_property_constant_references() {
    let files = [
        (
            "defs.php",
            "<?php\nnamespace App;\nclass Cfg { public const MODE = 'x'; public string $name = ''; }\nfunction helper(): void {}\nconst LIMIT = 10;\n",
        ),
        (
            "use.php",
            "<?php\nnamespace App;\nfunction consume(Cfg $c): string {\n    helper();\n    $m = Cfg::MODE;\n    $l = LIMIT;\n    return $c->name;\n}\n",
        ),
    ];
    let session = session_with(&files);
    let all = paths(&files);

    let cls = session
        .indexed_references_to(&Name::Class(Arc::from("App\\Cfg")), &all, false, &|| false)
        .expect("not cancelled");
    assert!(
        cls.iter().any(|(f, _)| f.as_ref() == "use.php"),
        "type hint + static access record cls refs: {cls:?}"
    );

    let f = session
        .indexed_references_to(
            &Name::Function(Arc::from("App\\helper")),
            &all,
            false,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(f.len(), 1, "helper() call: {f:?}");

    let prop = session
        .indexed_references_to(
            &Name::Property {
                class: Arc::from("App\\Cfg"),
                name: Arc::from("name"),
            },
            &all,
            false,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(prop.len(), 1, "$c->name access: {prop:?}");

    let cnst = session
        .indexed_references_to(
            &Name::ClassConstant {
                class: Arc::from("App\\Cfg"),
                name: Arc::from("MODE"),
            },
            &all,
            false,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(cnst.len(), 1, "Cfg::MODE access: {cnst:?}");
}

#[test]
fn freshness_edit_updates_postings() {
    let files = [
        (
            "base.php",
            "<?php\nclass B { public function m(): void {} }\n",
        ),
        ("caller.php", "<?php\nfunction c(B $b): void { $b->m(); }\n"),
    ];
    let session = session_with(&files);
    let sym = Name::method("B", "m");
    let all = paths(&files);
    let refs = session
        .indexed_references_to(&sym, &all, false, &|| false)
        .expect("not cancelled");
    assert_eq!(refs.len(), 1);

    // Edit the caller: two call sites now. A plain set_file_text write must
    // make the file stale and the next query must reflect the new text.
    session.set_file_text(
        Arc::from("caller.php"),
        Arc::from("<?php\nfunction c(B $b): void { $b->m(); $b->m(); }\n"),
    );
    let refs = session
        .indexed_references_to(&sym, &all, false, &|| false)
        .expect("not cancelled");
    assert_eq!(refs.len(), 2, "postings must follow the edit: {refs:?}");

    // And an edit that removes all call sites empties the result.
    session.set_file_text(
        Arc::from("caller.php"),
        Arc::from("<?php\nfunction c(B $b): void {}\n"),
    );
    let refs = session
        .indexed_references_to(&sym, &all, false, &|| false)
        .expect("not cancelled");
    assert!(refs.is_empty(), "stale postings must not survive: {refs:?}");
}

#[test]
fn subtype_classes_transitive_with_alias_and_fqn_forms() {
    let files = [
        ("animal.php", "<?php\nnamespace Zoo;\ninterface Animal {}\n"),
        (
            "cat.php",
            "<?php\nnamespace Pets;\nuse Zoo\\Animal as Beast;\nclass Cat implements Beast {}\n",
        ),
        (
            "lion.php",
            "<?php\nnamespace Wild;\nclass Lion extends \\Pets\\Cat {}\n",
        ),
        ("rock.php", "<?php\nnamespace Geo;\nclass Rock {}\n"),
    ];
    let session = session_with(&files);
    let subs = session.indexed_subtype_classes("Zoo\\Animal", &paths(&files), false);
    let names: Vec<&str> = subs.iter().map(|s| s.fqcn.as_ref()).collect();
    assert!(
        names.contains(&"Pets\\Cat") && names.contains(&"Wild\\Lion"),
        "aliased implements + FQN extends both resolve: {names:?}"
    );
    assert_eq!(subs.len(), 2, "{names:?}");
    // Declaration name spans for goto-implementation targets.
    let cat = subs
        .iter()
        .find(|s| s.fqcn.as_ref() == "Pets\\Cat")
        .unwrap();
    assert_eq!(cat.range.start.line, 4);
    // `class Cat implements Beast {}` — name token starts at char col 6.
    assert_eq!(cat.range.start.column, 6);
}

#[test]
fn method_implementations_across_subtypes() {
    let files = [
        (
            "shape.php",
            "<?php\ninterface Shape { public function area(): float; }\n",
        ),
        (
            "circle.php",
            "<?php\nclass Circle implements Shape { public function area(): float { return 3.14; } }\n",
        ),
        (
            "abstractbox.php",
            "<?php\nabstract class Box implements Shape { abstract public function area(): float; }\n",
        ),
        (
            "cube.php",
            "<?php\nclass Cube extends Box { public function area(): float { return 6.0; } }\n",
        ),
    ];
    let session = session_with(&files);
    let impls = session.indexed_method_implementations("Shape", "area", &paths(&files));
    let files_hit: Vec<&str> = impls.iter().map(|(_, f, _)| f.as_ref()).collect();
    assert!(
        files_hit.contains(&"circle.php") && files_hit.contains(&"cube.php"),
        "concrete overrides only: {impls:?}"
    );
    assert_eq!(
        impls.len(),
        2,
        "abstract Box::area must be excluded: {impls:?}"
    );
    // Ranges point at the method name token.
    let circle = impls
        .iter()
        .find(|(_, f, _)| f.as_ref() == "circle.php")
        .unwrap();
    assert_eq!(circle.2.start.line, 2);
}

#[test]
fn subtype_index_follows_reparenting_edit() {
    let files = [
        ("a.php", "<?php\nclass Base {}\nclass Other {}\n"),
        ("b.php", "<?php\nclass Kid extends Base {}\n"),
    ];
    let session = session_with(&files);
    let all = paths(&files);
    let subs = session.indexed_subtype_classes("Base", &all, false);
    assert_eq!(subs.len(), 1);

    session.set_file_text(
        Arc::from("b.php"),
        Arc::from("<?php\nclass Kid extends Other {}\n"),
    );
    let subs = session.indexed_subtype_classes("Base", &all, false);
    assert!(
        subs.is_empty(),
        "old edge must not survive the edit: {subs:?}"
    );
    let subs = session.indexed_subtype_classes("Other", &all, false);
    assert_eq!(subs.len(), 1, "new edge must be present: {subs:?}");
}
