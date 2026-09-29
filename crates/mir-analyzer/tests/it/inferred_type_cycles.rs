// Inferred return types across a cross-file inference cycle must not depend on
// which file a session happens to analyze first.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, PhpVersion};

const A: &str = "<?php\nclass A { function f() { return B::g(); } function h() { return 1; } }\n";
const B: &str = "<?php\nclass B { static function g() { return (new A)->h(); } }\n";
const Z: &str = "<?php\nfunction test(): string { return B::g(); }\n";

fn z_issues_after_analyzing(order: &[&str]) -> Vec<String> {
    let files = [("a.php", A), ("b.php", B), ("z.php", Z)];
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    for (path, src) in files {
        session.ingest_file(Arc::from(path), Arc::from(src));
    }
    for path in order {
        let src = files.iter().find(|(p, _)| p == path).unwrap().1;
        session.analyze_file_diagnostics(path, src);
    }
    let mut issues: Vec<String> = session
        .analyze_file_diagnostics("z.php", Z)
        .issues
        .iter()
        .map(|i| format!("{} {}", i.kind.name(), i.kind.message()))
        .collect();
    issues.sort();
    issues
}

#[test]
fn inferred_return_through_cycle_is_order_independent() {
    let expected = z_issues_after_analyzing(&[]);
    assert!(
        expected.iter().any(|i| i.starts_with("InvalidReturnType")),
        "B::g() infers `1` through A::h(): {expected:?}"
    );
    for order in [
        &["a.php"][..],
        &["b.php"],
        &["a.php", "b.php"],
        &["b.php", "a.php"],
    ] {
        assert_eq!(z_issues_after_analyzing(order), expected, "order {order:?}");
    }
}
