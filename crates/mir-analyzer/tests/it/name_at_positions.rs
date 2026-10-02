//! `AnalysisSession::name_at` on declaration sites and docblock type names.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, Name, PhpVersion};

const FILE: &str = "/proj/names.php";
const CURSOR: char = '‸';

fn name_at(marked: &str) -> Option<Name> {
    let offset = marked.find(CURSOR).expect("cursor marker present");
    let src = marked.replacen(CURSOR, "", 1);
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(Arc::from(FILE), Arc::from(src.as_str()));
    session.name_at(FILE, offset as u32)
}

#[test]
fn promoted_constructor_property_declaration() {
    let src = "<?php\nclass A { public function __construct(public int $va‸l) {} }\n";
    assert_eq!(name_at(src), Some(Name::property("A", "val")));
}

#[test]
fn readonly_only_promoted_property_declaration() {
    let src = "<?php\nclass A { public function __construct(readonly int $va‸l) {} }\n";
    assert_eq!(name_at(src), Some(Name::property("A", "val")));
}

#[test]
fn plain_constructor_parameter_is_not_a_property() {
    let src = "<?php\nclass A { public function __construct(int $va‸l) {} }\n";
    assert_eq!(name_at(src), None);
}

#[test]
fn define_name_string() {
    let src = "<?php\ndefine('APP_VE‸RSION', 1);\n";
    assert_eq!(name_at(src), Some(Name::global_constant("APP_VERSION")));
}

#[test]
fn define_inside_function_body() {
    let src = "<?php\nfunction boot() { define('APP_VE‸RSION', 1); }\n";
    assert_eq!(name_at(src), Some(Name::global_constant("APP_VERSION")));
}

#[test]
fn docblock_param_type() {
    let src = "<?php\nclass Widget {}\n/** @param Widg‸et $w */\nfunction f($w) {}\n";
    assert_eq!(name_at(src), Some(Name::class("Widget")));
}

#[test]
fn docblock_multiline_param_type_in_union() {
    let src = "<?php\nclass Widget {}\n/**\n * @param int|Widg‸et|null $w the widget\n */\nfunction f($w) {}\n";
    assert_eq!(name_at(src), Some(Name::class("Widget")));
}

#[test]
fn docblock_see_resolves_through_use_alias() {
    let src = "<?php\nnamespace Lib { class Widget {} }\nnamespace App {\nuse Lib\\Widget as W;\n/** @see ‸W */\nfunction f() {}\n}\n";
    assert_eq!(name_at(src), Some(Name::class("Lib\\Widget")));
}

#[test]
fn docblock_see_method_member() {
    let src = "<?php\nclass Base { public function run(): void {} }\nclass Widget extends Base {}\n/** @see Widget::r‸un() */\nfunction f() {}\n";
    assert_eq!(name_at(src), Some(Name::method("Base", "run")));
}

#[test]
fn docblock_see_class_part_of_member_reference() {
    let src =
        "<?php\nclass Widget { const SIZE = 1; }\n/** @see Wid‸get::SIZE */\nfunction f() {}\n";
    assert_eq!(name_at(src), Some(Name::class("Widget")));
}

#[test]
fn docblock_see_class_constant_member() {
    let src =
        "<?php\nclass Widget { const SIZE = 1; }\n/** @see Widget::SI‸ZE */\nfunction f() {}\n";
    assert_eq!(name_at(src), Some(Name::class_constant("Widget", "SIZE")));
}

#[test]
fn docblock_template_bound() {
    let src = "<?php\nclass X {}\n/** @template T of ‸X */\nfunction f() {}\n";
    assert_eq!(name_at(src), Some(Name::class("X")));
}

#[test]
fn docblock_template_parameter_itself_is_not_a_class() {
    let src = "<?php\nclass T {}\nclass X {}\n/** @template ‸T of X */\nfunction f() {}\n";
    assert_eq!(name_at(src), None);
}

#[test]
fn docblock_inline_var() {
    let src =
        "<?php\nclass Widget {}\nfunction f($x) {\n    /** @var Wid‸get $y */\n    $y = $x;\n}\n";
    assert_eq!(name_at(src), Some(Name::class("Widget")));
}

#[test]
fn docblock_description_words_are_not_names() {
    let src =
        "<?php\nclass Widget {}\n/** @param int $w the Wid‸get to use */\nfunction f($w) {}\n";
    assert_eq!(name_at(src), None);
}

#[test]
fn docblock_variable_name_is_not_a_class() {
    let src = "<?php\nclass Widget {}\n/** @param int $Wid‸get */\nfunction f($Widget) {}\n";
    assert_eq!(name_at(src), None);
}

#[test]
fn docblock_keyword_types_are_not_classes() {
    let src = "<?php\n/** @return arr‸ay<string> */\nfunction f() { return []; }\n";
    assert_eq!(name_at(src), None);
}

#[test]
fn docblock_unknown_class_is_none() {
    let src = "<?php\n/** @param Mis‸sing $w */\nfunction f($w) {}\n";
    assert_eq!(name_at(src), None);
}

#[test]
fn code_after_a_closed_docblock_still_resolves() {
    let src = "<?php\nfunction helper(): void {}\n/** @return void */\nfunction a(): void {}\nfunction caller(): void { help‸er(); }\n";
    assert_eq!(name_at(src), Some(Name::function("helper")));
}
