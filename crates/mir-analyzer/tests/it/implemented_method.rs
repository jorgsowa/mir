//! `AnalysisSession::implemented_method`: the interface or abstract method a method implements.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, Name, PhpVersion};

const FILE: &str = "impl.php";

// `@mir-check` mismatches surface as issues, so an empty issue list verifies the annotations.
const SRC: &str = "<?php
interface Shape { public function area(): float; }
interface Named { public function label(): string; }
abstract class Base implements Shape {
    abstract public function perimeter(): float;
    public function describe(): string { return 'base'; }
}
class Square extends Base implements Named {
    public function area(): float { return 1.0; }
    public function perimeter(): float { return 4.0; }
    public function label(): string { return 'sq'; }
    public function describe(): string { return 'sq'; }
    public function own(): void {}
}
enum Level: string implements Named {
    case Low = 'low';
    public function label(): string { return $this->value; }
}
class Child extends Square {
    public function area(): float { return 2.0; }
}
function check(Square $s, Child $c, Level $l): void {
    $a = $s->area();
    /** @mir-check $a is float */
    $_ = $a;
    $b = $c->label();
    /** @mir-check $b is string */
    $_ = $b;
    $d = $l->label();
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

#[test]
fn annotations_hold() {
    let analysis = session().analyze_file_diagnostics(FILE, SRC);
    let issues: Vec<_> = analysis
        .issues
        .iter()
        .filter(|i| !matches!(i.kind, mir_analyzer::IssueKind::UnusedVariable { .. }))
        .collect();
    assert!(issues.is_empty(), "{:?}", issues);
}

#[test]
fn resolves_interface_and_abstract_methods() {
    let s = session();
    let of = |class: &str, m: &str| s.implemented_method(&Name::method(class, m));
    assert_eq!(of("Square", "area"), Some(Name::method("Shape", "area")));
    assert_eq!(
        of("Square", "perimeter"),
        Some(Name::method("Base", "perimeter"))
    );
    assert_eq!(of("Square", "label"), Some(Name::method("Named", "label")));
    assert_eq!(of("Level", "label"), Some(Name::method("Named", "label")));
    // Skips the concrete override in Square and finds the contract.
    assert_eq!(of("Child", "area"), Some(Name::method("Shape", "area")));
}

#[test]
fn none_without_a_contract() {
    let s = session();
    let of = |class: &str, m: &str| s.implemented_method(&Name::method(class, m));
    assert_eq!(of("Square", "describe"), None);
    assert_eq!(of("Square", "own"), None);
    assert_eq!(of("Shape", "area"), None);
    assert_eq!(of("Base", "perimeter"), None);
    assert_eq!(of("Missing", "area"), None);
    assert_eq!(s.implemented_method(&Name::class("Square")), None);
}
