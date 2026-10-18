//! Taint issues carry the path the data took from its source to the sink.

use mir_analyzer::test_utils::check;
use mir_issues::IssueKind;

fn traces(src: &str) -> Vec<Vec<(String, u32)>> {
    check(src)
        .into_iter()
        .filter(|issue| matches!(issue.kind, IssueKind::TaintedHtml | IssueKind::TaintedShell))
        .map(|issue| {
            issue
                .taint_trace
                .unwrap_or_default()
                .iter()
                .map(|step| (step.label.clone(), step.location.line))
                .collect()
        })
        .collect()
}

#[test]
fn trace_follows_assignments_from_the_source() {
    let src = "<?php\nfunction page(): void {\n    $q = $_GET['q'];\n    $label = 'Search: ' . trim($q);\n    echo \"<b>$label</b>\";\n}\n";
    assert_eq!(
        traces(src),
        vec![vec![
            ("$_GET['q']".to_string(), 3),
            ("$q".to_string(), 3),
            ("trim($q)".to_string(), 4),
            ("$label".to_string(), 4),
        ]]
    );
}

#[test]
fn trace_names_the_call_that_returns_tainted_data() {
    let src = "<?php\nfunction readQuery(): string { return (string) $_GET['q']; }\nfunction page(): void {\n    echo readQuery();\n}\n";
    assert_eq!(traces(src), vec![vec![("readQuery()".to_string(), 4)]]);
}

#[test]
fn trace_follows_foreach_values() {
    let src = "<?php\nfunction run(): void {\n    foreach ($_POST['tags'] as $tag) {\n        system('tag ' . $tag);\n    }\n}\n";
    assert_eq!(
        traces(src),
        vec![vec![
            ("$_POST['tags']".to_string(), 3),
            ("$tag".to_string(), 3),
        ]]
    );
}

#[test]
fn long_trails_keep_the_source_and_the_latest_steps() {
    let src = "<?php\nfunction page(): void {\n    $a = $_GET['q'];\n    $b = $a;\n    $c = $b;\n    $d = $c;\n    $e = $d;\n    $f = $e;\n    $g = $f;\n    echo $g;\n}\n";
    let trace = traces(src).remove(0);
    assert_eq!(trace.len(), 6);
    assert_eq!(trace[0], ("$_GET['q']".to_string(), 3));
    assert_eq!(trace[5], ("$g".to_string(), 9));
}
