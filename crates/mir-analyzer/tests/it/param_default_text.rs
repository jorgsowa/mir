//! `DeclaredParam::default_text`: parameter default value source text.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, PhpVersion};

const FILE: &str = "defaults.php";

// `@mir-check` mismatches surface as issues, so an empty issue list verifies the annotations.
const SRC: &str = "<?php
const LIMIT = 5;
function plain(int $a, int $b = 10, ?string $c = null, string $d = 'x y', array $e = [1, 2]): int {
    return $a + $b;
}
function spread(string $first, string ...$rest): string { return $first; }
function consts(int $n = LIMIT, int $m = PHP_INT_MAX): int { return $n + $m; }
class Box {
    const SIZE = 3;
    public function __construct(public int $w = self::SIZE, public ?Box $next = null) {}
    public function put(int $x = -1, bool $flag = true): int { return $x; }
    public static function make(string $name = \"n\"): string { return $name; }
}
interface Sink { public function write(string $s = ''): void; }
trait Counts { public function bump(int $by = 1 + 1): int { return $by; } }
enum Level: int {
    case Low = 1;
    public function next(int $step = self::Low->value): int { return $step; }
}
function check(Box $box): void {
    $a = plain(1);
    /** @mir-check $a is int */
    $_ = $a;
    $b = $box->put();
    /** @mir-check $b is int */
    $_ = $b;
    $c = Box::make();
    /** @mir-check $c is string */
    $_ = $c;
    $d = spread('a', 'b', 'c');
    /** @mir-check $d is string */
    $_ = $d;
}
";

fn session() -> AnalysisSession {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.set_file_text(Arc::from(FILE), Arc::from(SRC));
    session
}

fn function_defaults(session: &AnalysisSession, fqn: &str) -> Vec<Option<String>> {
    let f = session
        .snapshot()
        .find_function(fqn)
        .expect("not cancelled")
        .expect("function exists");
    f.params
        .iter()
        .map(|p| p.default_text.as_deref().map(str::to_string))
        .collect()
}

fn method_defaults(session: &AnalysisSession, class: &str, method: &str) -> Vec<Option<String>> {
    let (_, m) = session
        .snapshot()
        .find_method_in_chain(class, method)
        .expect("not cancelled")
        .expect("method exists");
    m.params
        .iter()
        .map(|p| p.default_text.as_deref().map(str::to_string))
        .collect()
}

fn texts(items: &[Option<&str>]) -> Vec<Option<String>> {
    items.iter().map(|t| t.map(str::to_string)).collect()
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
fn function_literals() {
    assert_eq!(
        function_defaults(&session(), "plain"),
        texts(&[
            None,
            Some("10"),
            Some("null"),
            Some("'x y'"),
            Some("[1, 2]")
        ])
    );
}

#[test]
fn function_without_defaults_and_variadic() {
    assert_eq!(
        function_defaults(&session(), "spread"),
        texts(&[None, None])
    );
}

#[test]
fn function_constant_defaults() {
    assert_eq!(
        function_defaults(&session(), "consts"),
        texts(&[Some("LIMIT"), Some("PHP_INT_MAX")])
    );
}

#[test]
fn constructor_promoted_params() {
    assert_eq!(
        method_defaults(&session(), "Box", "__construct"),
        texts(&[Some("self::SIZE"), Some("null")])
    );
}

#[test]
fn instance_and_static_methods() {
    let s = session();
    assert_eq!(
        method_defaults(&s, "Box", "put"),
        texts(&[Some("-1"), Some("true")])
    );
    assert_eq!(method_defaults(&s, "Box", "make"), texts(&[Some("\"n\"")]));
}

#[test]
fn interface_trait_and_enum_methods() {
    let s = session();
    assert_eq!(method_defaults(&s, "Sink", "write"), texts(&[Some("''")]));
    assert_eq!(
        method_defaults(&s, "Counts", "bump"),
        texts(&[Some("1 + 1")])
    );
    assert_eq!(
        method_defaults(&s, "Level", "next"),
        texts(&[Some("self::Low->value")])
    );
}
