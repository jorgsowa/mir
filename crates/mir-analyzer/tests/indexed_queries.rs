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
fn method_implementations_trait_composed_are_found() {
    // Circle implements Shape::area() only via a `use`d trait — the method is
    // never literally declared in Circle's own body. find_method_in_class
    // (own-declarations-only) misses this entirely; find_method_in_chain
    // walks into the trait and finds it.
    let files = [
        (
            "shape.php",
            "<?php\ninterface Shape { public function area(): float; }\n",
        ),
        (
            "area_trait.php",
            "<?php\ntrait AreaFromRadius { public function area(): float { return 3.14159 * $this->radius ** 2; } }\n",
        ),
        (
            "circle.php",
            "<?php\nclass Circle implements Shape {\n    use AreaFromRadius;\n    public function __construct(public float $radius) {}\n}\n",
        ),
    ];
    let session = session_with(&files);
    let impls = session.indexed_method_implementations("Shape", "area", &paths(&files));
    assert_eq!(
        impls.len(),
        1,
        "Circle's trait-composed area() must be found: {impls:?}"
    );
    assert_eq!(impls[0].0.as_ref(), "Circle");
    assert_eq!(impls[0].1.as_ref(), "area_trait.php");
    // Range must point at the method name token inside the trait, not Circle.
    assert_eq!(impls[0].2.start.line, 2);
}

#[test]
fn method_implementations_multilevel_inheritance_dedups_to_declaring_site() {
    // Square inherits ConcreteBox::area() without overriding it. It's a real
    // concrete subtype of Shape and must not be silently dropped — but it
    // resolves to the same declaring location as ConcreteBox, so it collapses
    // into a single deduped entry rather than producing a visible duplicate.
    let files = [
        (
            "shape.php",
            "<?php\ninterface Shape { public function area(): float; }\n",
        ),
        (
            "box.php",
            "<?php\nabstract class Box implements Shape { abstract public function area(): float; }\n",
        ),
        (
            "concretebox.php",
            "<?php\nclass ConcreteBox extends Box { public function area(): float { return 1.0; } }\n",
        ),
        ("square.php", "<?php\nclass Square extends ConcreteBox {}\n"),
    ];
    let session = session_with(&files);
    let impls = session.indexed_method_implementations("Shape", "area", &paths(&files));
    assert_eq!(
        impls.len(),
        1,
        "Square inherits ConcreteBox::area(); both resolve to the same site and must collapse: {impls:?}"
    );
    assert_eq!(impls[0].1.as_ref(), "concretebox.php");
}

#[test]
fn method_implementations_trait_shared_by_siblings_dedups_to_one_entry() {
    // Two sibling classes composing the same trait both resolve to the same
    // trait method location and must collapse to a single entry, exactly
    // like the plain-inheritance dedup case above.
    let files = [
        (
            "shape.php",
            "<?php\ninterface Shape { public function area(): float; }\n",
        ),
        (
            "area_trait.php",
            "<?php\ntrait AreaFromSide { public function area(): float { return $this->side ** 2; } }\n",
        ),
        (
            "smallsquare.php",
            "<?php\nclass SmallSquare implements Shape {\n    use AreaFromSide;\n    public function __construct(public float $side = 1.0) {}\n}\n",
        ),
        (
            "bigsquare.php",
            "<?php\nclass BigSquare implements Shape {\n    use AreaFromSide;\n    public function __construct(public float $side = 10.0) {}\n}\n",
        ),
    ];
    let session = session_with(&files);
    let impls = session.indexed_method_implementations("Shape", "area", &paths(&files));
    assert_eq!(
        impls.len(),
        1,
        "both siblings resolve to the same trait method and must collapse: {impls:?}"
    );
    assert_eq!(impls[0].1.as_ref(), "area_trait.php");
}

#[test]
fn static_call_name_fallback_on_unresolved_class() {
    let files = [(
        "caller.php",
        "<?php\nfunction c(): void { UnknownClass::doThing(); }\n",
    )];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(&Name::method("", "doThing"), &paths(&files), false, &|| {
            false
        })
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "UnknownClass::doThing() must record a methname: fallback: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "caller.php");
}

#[test]
fn static_call_name_fallback_on_undefined_method() {
    let files = [(
        "caller.php",
        "<?php\nclass Known {}\nfunction c(): void { Known::doThing(); }\n",
    )];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(&Name::method("", "doThing"), &paths(&files), false, &|| {
            false
        })
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "Known::doThing() with no such method must still record a methname: fallback: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "caller.php");
}

#[test]
fn unknown_owner_property_declaration_reachable() {
    let files = [(
        "widget.php",
        "<?php\nclass Widget { public string $label = ''; }\n",
    )];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(&Name::property("", "label"), &paths(&files), true, &|| {
            false
        })
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "propdecl: posting must surface the declaration for an unknown owner: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "widget.php");
    assert_eq!(refs[0].1.start.line, 2);
}

#[test]
fn unknown_owner_constant_declaration_reachable() {
    let files = [("cfg.php", "<?php\nclass Cfg { public const MODE = 'x'; }\n")];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(
            &Name::class_constant("", "MODE"),
            &paths(&files),
            true,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "cnstdecl: posting must surface the declaration for an unknown owner: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "cfg.php");
}

#[test]
fn interface_method_declaration_reachable_with_unknown_owner() {
    let files = [(
        "shape.php",
        "<?php\ninterface Shape { public function area(): float; }\n",
    )];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(&Name::method("", "area"), &paths(&files), true, &|| false)
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "interface method declarations must reach methdecl: too: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "shape.php");
}

#[test]
fn enum_constant_declaration_reachable_with_unknown_owner() {
    let files = [(
        "suit.php",
        "<?php\nenum Suit { case Hearts; const DEFAULT = self::Hearts; }\n",
    )];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(
            &Name::class_constant("", "DEFAULT"),
            &paths(&files),
            true,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        refs.len(),
        1,
        "enum-declared class constant must reach cnstdecl: too: {refs:?}"
    );
    assert_eq!(refs[0].0.as_ref(), "suit.php");
}

#[test]
fn use_import_locations_reachable_but_excluded_from_plain_references() {
    let files = [
        (
            "lib.php",
            "<?php\nnamespace App;\nclass Widget {}\nfunction helper(): void {}\nconst LIMIT = 10;\n",
        ),
        (
            "main.php",
            "<?php\nnamespace Other;\nuse App\\Widget;\nuse function App\\helper;\nuse const App\\LIMIT;\nfunction go(): void {}\n",
        ),
    ];
    let session = session_with(&files);
    let all = paths(&files);

    // `indexed_use_import_locations` is a pure posting read with no freshness
    // pass (see its doc comment) — force analysis + commit via
    // `indexed_references_to` first, same as any indexed-query consumer would
    // for a file it hasn't already touched.
    //
    // A bare import must NOT show up as a plain find-references hit — there is
    // deliberately no `UnusedImport` check, and counting an import as usage
    // would hide genuinely dead classes/functions/constants.
    let cls_refs = session
        .indexed_references_to(&Name::class("App\\Widget"), &all, false, &|| false)
        .expect("not cancelled");
    assert!(
        cls_refs.is_empty(),
        "a bare import must not count as a usage: {cls_refs:?}"
    );

    let cls_use = session.indexed_use_import_locations(&Name::class("App\\Widget"), &all);
    assert_eq!(
        cls_use.len(),
        1,
        "class import must be indexed: {cls_use:?}"
    );
    assert_eq!(cls_use[0].0.as_ref(), "main.php");

    let fn_use = session.indexed_use_import_locations(&Name::function("App\\helper"), &all);
    assert_eq!(
        fn_use.len(),
        1,
        "function import must be indexed: {fn_use:?}"
    );

    let const_use =
        session.indexed_use_import_locations(&Name::global_constant("App\\LIMIT"), &all);
    assert_eq!(
        const_use.len(),
        1,
        "const import must be indexed: {const_use:?}"
    );
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

/// `indexed_subtype_classes`'s on-demand completeness pass prefilters
/// uncommitted files by scanning their raw text for the frontier class's
/// short name (`mentions_identifier`). When that name is a multibyte PHP
/// identifier, an uncommitted file containing it embedded as a substring of
/// a longer identifier must not crash the scan — it must simply be
/// correctly rejected as a non-match — and a real subtype declared
/// elsewhere in the same uncommitted file must still be found.
#[test]
fn subtype_scan_survives_multibyte_identifier_false_match() {
    let files = [
        ("animal.php", "<?php\nnamespace Zoo;\ninterface Éclair {}\n"),
        (
            "cat.php",
            // `xÉclairFoo` is a substring false-match for the short name
            // `Éclair` that the prefilter must skip over without panicking,
            // before it reaches the real `implements Éclair` below.
            "<?php\nnamespace Pets;\nfunction xÉclairFoo(): void {}\nclass Cat implements \\Zoo\\Éclair {}\n",
        ),
    ];
    let session = session_with(&files);
    let subs = session.indexed_subtype_classes("Zoo\\Éclair", &paths(&files), false);
    let names: Vec<&str> = subs.iter().map(|s| s.fqcn.as_ref()).collect();
    assert_eq!(names, vec!["Pets\\Cat"], "{names:?}");
}

/// Same hazard as `subtype_scan_survives_multibyte_identifier_false_match`,
/// but the scanned file contains *only* the false substring match and no
/// real subtype at all — the prefilter must reject the whole file cleanly
/// (empty result, no panic) rather than needing a later real match to
/// terminate the scan loop correctly.
#[test]
fn subtype_scan_multibyte_identifier_with_only_false_match_does_not_panic() {
    let files = [
        ("animal.php", "<?php\nnamespace Zoo;\ninterface Éclair {}\n"),
        (
            "unrelated.php",
            "<?php\nnamespace Pets;\nfunction xÉclairFoo(): void {}\n",
        ),
    ];
    let session = session_with(&files);
    let subs = session.indexed_subtype_classes("Zoo\\Éclair", &paths(&files), false);
    assert!(subs.is_empty(), "{subs:?}");
}

/// Same hazard again, using leading characters of different UTF-8 encoded
/// lengths (2-byte, 3-byte, 4-byte) as the first character of the class
/// short name, so the prefilter's skip-forward must correctly step past
/// codepoints of every width PHP identifiers can contain.
#[test]
fn subtype_scan_survives_three_and_four_byte_leading_char_identifiers() {
    let files = [
        (
            "shapes.php",
            "<?php\nnamespace Zoo;\ninterface 书Shape {}\ninterface 😀Shape {}\n",
        ),
        (
            "impls.php",
            // `x书ShapeFoo` / `x😀ShapeFoo` are substring false-matches the
            // prefilter must skip over before reaching the real implements.
            "<?php\nnamespace Pets;\nfunction x书ShapeFoo(): void {}\nfunction x😀ShapeFoo(): void {}\nclass Box implements \\Zoo\\书Shape, \\Zoo\\😀Shape {}\n",
        ),
    ];
    let session = session_with(&files);
    let three_byte = session.indexed_subtype_classes("Zoo\\书Shape", &paths(&files), false);
    let four_byte = session.indexed_subtype_classes("Zoo\\😀Shape", &paths(&files), false);
    assert_eq!(
        three_byte
            .iter()
            .map(|s| s.fqcn.as_ref())
            .collect::<Vec<_>>(),
        vec!["Pets\\Box"]
    );
    assert_eq!(
        four_byte
            .iter()
            .map(|s| s.fqcn.as_ref())
            .collect::<Vec<_>>(),
        vec!["Pets\\Box"]
    );
}

/// An import whose target never resolves (not yet loaded, vendor-only, or
/// genuinely missing) still gets a `use:` posting keyed by the written FQN —
/// an index-based rename must find/update the import line even when the
/// class it names isn't in the workspace.
#[test]
fn use_import_postings_recorded_for_unresolvable_targets() {
    let files = [(
        "main.php",
        "<?php\nuse App\\Ghost;\nuse function App\\phantom;\nuse const App\\SPOOKY;\n$g = new Ghost();\n",
    )];
    let session = session_with(&files);
    let all = paths(&files);

    // Freshness pass commits main.php's postings (the class never resolves).
    let _ = session
        .indexed_references_to(&Name::class("App\\Ghost"), &all, false, &|| false)
        .expect("not cancelled");

    let cls_use = session.indexed_use_import_locations(&Name::class("App\\Ghost"), &all);
    assert_eq!(
        cls_use.len(),
        1,
        "unresolvable class import must still be indexed: {cls_use:?}"
    );
    assert_eq!(cls_use[0].0.as_ref(), "main.php");

    let fn_use = session.indexed_use_import_locations(&Name::function("App\\phantom"), &all);
    assert_eq!(
        fn_use.len(),
        1,
        "unresolvable function import must still be indexed: {fn_use:?}"
    );

    let const_use =
        session.indexed_use_import_locations(&Name::global_constant("App\\SPOOKY"), &all);
    assert_eq!(
        const_use.len(),
        1,
        "unresolvable const import must still be indexed: {const_use:?}"
    );
}

/// `__construct` postings live at `new Cls(` sites, which never spell the
/// member name — the gate must therefore also admit files mentioning the
/// owner class's short name, or a cold constructor-references query would
/// skip every instantiation site.
#[test]
fn cold_constructor_query_admits_files_naming_only_the_class() {
    let files = [
        (
            "job.php",
            "<?php\nnamespace App;\nclass Job { public function __construct() {} }\n",
        ),
        (
            "spawn.php",
            "<?php\nnamespace App;\nfunction spawn(): Job { return new Job(); }\n",
        ),
    ];
    let session = session_with(&files);
    let refs = session
        .indexed_references_to(
            &Name::method("App\\Job", "__construct"),
            &paths(&files),
            false,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(refs.len(), 1, "{refs:?}");
    assert_eq!(refs[0].0.as_ref(), "spawn.php");
}

/// A gated-out file (cold, never mentions the symbol) must stay invisible to
/// the query result, and a later edit that *adds* a mention must be picked
/// up — the gate skips analysis, it must not freeze the file's state.
#[test]
fn gated_file_participates_after_edit_introduces_mention() {
    let files = [
        (
            "svc.php",
            "<?php\nnamespace App;\nclass Service { public function run(): void {} }\n",
        ),
        ("idle.php", "<?php\nnamespace App;\nclass Idle {}\n"),
    ];
    let session = session_with(&files);
    let sym = Name::method("App\\Service", "run");
    let refs = session
        .indexed_references_to(&sym, &paths(&files), false, &|| false)
        .expect("not cancelled");
    assert!(refs.is_empty(), "{refs:?}");

    session.set_file_text(
        Arc::from("idle.php"),
        Arc::from(
            "<?php\nnamespace App;\nclass Idle { public function go(Service $s): void { $s->run(); } }\n",
        ),
    );
    let refs = session
        .indexed_references_to(&sym, &paths(&files), false, &|| false)
        .expect("not cancelled");
    assert_eq!(refs.len(), 1, "{refs:?}");
    assert_eq!(refs[0].0.as_ref(), "idle.php");
}
