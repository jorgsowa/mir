//! `AnalysisSession::outgoing_calls`: resolved callees inside a function or method.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, Name, PhpVersion};

const FILE: &str = "calls.php";

fn callees(src: &str, marker: &str) -> Vec<Name> {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.set_file_text(Arc::from(FILE), Arc::from(src));
    let offset = src.find(marker).expect("marker present") as u32;
    session
        .outgoing_calls(FILE, offset)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[test]
fn method_calls_resolve_to_declaring_class() {
    let src = "<?php\n\
        class Base { public function save(): void {} }\n\
        class Child extends Base {\n\
            public function run(): void { $this->save(); parent::save(); strlen('x'); }\n\
        }\n";
    assert_eq!(
        callees(src, "run()"),
        vec![
            Name::method("Base", "save"),
            Name::method("Base", "save"),
            Name::function("strlen"),
        ]
    );
}

#[test]
fn trait_alias_reports_the_original_method() {
    let src = "<?php\n\
        trait Auditable { public function record(): void {} }\n\
        class Post {\n\
            use Auditable { record as audit; }\n\
            public function save(): void { $this->audit(); }\n\
        }\n";
    assert_eq!(
        callees(src, "save()"),
        vec![Name::method("Auditable", "record")]
    );
}

#[test]
fn undeclared_parent_reports_literal_extends_class() {
    let src = "<?php\n\
        class Child extends Missing {\n\
            public function __construct() { parent::__construct(); }\n\
        }\n";
    assert_eq!(
        callees(src, "__construct()"),
        vec![Name::method("Missing", "__construct")]
    );
}

#[test]
fn excludes_other_methods_and_the_declaration_itself() {
    let src = "<?php\n\
        function helper(): void {}\n\
        class Svc {\n\
            public function a(): void { helper(); }\n\
            public function b(): void { $this->a(); }\n\
        }\n";
    assert_eq!(callees(src, "a()"), vec![Name::function("helper")]);
    assert_eq!(callees(src, "b()"), vec![Name::method("Svc", "a")]);
}

#[test]
fn free_function_and_closure_calls() {
    let src = "<?php\n\
        function helper(): void {}\n\
        function outer(): void { $f = function () { helper(); }; $f(); }\n";
    assert_eq!(callees(src, "outer()"), vec![Name::function("helper")]);
}

#[test]
fn offset_outside_a_body_yields_nothing() {
    let src = "<?php\nfunction helper(): void {}\nhelper();\n";
    assert!(callees(src, "helper();").is_empty());
}

#[test]
fn ranges_cover_the_callee_name_token_in_source_order() {
    let src = "<?php\nfunction a(): void {}\nfunction b(): void {}\nfunction run(): void {\n    b(); a();\n}\n";
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.set_file_text(Arc::from(FILE), Arc::from(src));
    let calls = session.outgoing_calls(FILE, src.find("run()").unwrap() as u32);
    let at = |name: &str, line: u32, start: u32| {
        (
            Name::function(name),
            mir_analyzer::Range {
                start: mir_analyzer::Position {
                    line,
                    column: start,
                },
                end: mir_analyzer::Position {
                    line,
                    column: start + 1,
                },
            },
        )
    };
    assert_eq!(calls, vec![at("b", 5, 4), at("a", 5, 9)]);
}
