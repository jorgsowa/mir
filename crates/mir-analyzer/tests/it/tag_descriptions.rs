//! `@param`/`@return`/`@throws` prose on `FunctionDef` and `MethodDef`.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, PhpVersion};

const FILE: &str = "tags.php";

// `@mir-check` mismatches surface as issues, so an empty issue list verifies the annotations.
const SRC: &str = "<?php
namespace App;
use OutOfBoundsException as Failure;
/**
 * Adds numbers.
 *
 * @param int $a First operand
 * @param int $b Second operand,
 *               wrapped over lines
 * @param int ...$rest
 * @return int The sum
 * @throws \\InvalidArgumentException When negative
 * @throws Failure|\\RuntimeException Either failure
 */
function add(int $a, int $b, int ...$rest): int { return $a + $b; }
/** @return string */
function bare(string $s): string { return $s; }
/** No tags at all. */
function none(): void {}
/** @param mixed &$out Filled in */
function refs(&$out): void {}
class Calc {
    /**
     * @param int $x The input
     * @return int Doubled
     * @throws \\LogicException Never really
     */
    public function twice(int $x): int { return $x * 2; }
    /** @param $untyped Loose description */
    public function loose($untyped): void {}
}
function check(Calc $c): void {
    $a = add(1, 2);
    /** @mir-check $a is int */
    $_ = $a;
    $b = $c->twice(2);
    /** @mir-check $b is int */
    $_ = $b;
}
";

fn session() -> AnalysisSession {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.set_file_text(Arc::from(FILE), Arc::from(SRC));
    session
}

fn pairs(items: &[(Arc<str>, Arc<str>)]) -> Vec<(&str, &str)> {
    items
        .iter()
        .map(|(k, v)| (k.as_ref(), v.as_ref()))
        .collect()
}

fn function_tags(fqn: &str) -> Option<Arc<mir_analyzer::TagDescriptions>> {
    session()
        .snapshot()
        .find_function(fqn)
        .expect("not cancelled")
        .expect("function exists")
        .tag_descriptions
        .clone()
}

fn method_tags(class: &str, method: &str) -> Option<Arc<mir_analyzer::TagDescriptions>> {
    session()
        .snapshot()
        .find_method_in_chain(class, method)
        .expect("not cancelled")
        .expect("method exists")
        .1
        .tag_descriptions
        .clone()
}

#[test]
fn annotations_hold() {
    let analysis = session().analyze_file_diagnostics(FILE, SRC);
    let issues: Vec<_> = analysis
        .issues
        .iter()
        .filter(|i| i.severity != mir_analyzer::Severity::Info)
        .collect();
    assert!(issues.is_empty(), "{:?}", issues);
}

#[test]
fn function_param_return_and_throws() {
    let tags = function_tags("App\\add").expect("descriptions recorded");
    assert_eq!(
        pairs(&tags.params),
        [
            ("a", "First operand"),
            ("b", "Second operand, wrapped over lines")
        ]
    );
    assert_eq!(tags.returns.as_deref(), Some("The sum"));
    assert_eq!(
        pairs(&tags.throws),
        [
            ("InvalidArgumentException", "When negative"),
            ("OutOfBoundsException", "Either failure"),
            ("RuntimeException", "Either failure"),
        ]
    );
}

#[test]
fn tags_without_prose_are_omitted() {
    assert!(function_tags("App\\bare").is_none());
    assert!(function_tags("App\\none").is_none());
}

#[test]
fn byref_and_untyped_params() {
    let refs = function_tags("App\\refs").expect("descriptions recorded");
    assert_eq!(pairs(&refs.params), [("out", "Filled in")]);
    let loose = method_tags("App\\Calc", "loose").expect("descriptions recorded");
    assert_eq!(pairs(&loose.params), [("untyped", "Loose description")]);
}

#[test]
fn method_tags_recorded() {
    let tags = method_tags("App\\Calc", "twice").expect("descriptions recorded");
    assert_eq!(pairs(&tags.params), [("x", "The input")]);
    assert_eq!(tags.returns.as_deref(), Some("Doubled"));
    assert_eq!(pairs(&tags.throws), [("LogicException", "Never really")]);
}
