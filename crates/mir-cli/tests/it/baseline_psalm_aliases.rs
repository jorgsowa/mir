use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

const SOURCE: &str = "<?php\n\
if (rand() > 1) {\n    $maybe = 1;\n}\n\
echo $maybe;\n\
function f(int $n): void { if ($n === null) { echo 1; } }\n";

fn fixture(baseline_entries: &str) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(dir.path().join("a.php"), SOURCE).expect("failed to write PHP fixture");
    std::fs::write(
        dir.path().join("psalm-baseline.xml"),
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<files>\n  <file src=\"./a.php\">\n{baseline_entries}  </file>\n</files>\n"),
    )
    .expect("failed to write baseline");
    dir
}

fn run(dir: &Path, extra_args: &[&str]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear();
    cmd.current_dir(dir);
    cmd.args([
        "--no-cache",
        "--no-progress",
        "--baseline",
        "psalm-baseline.xml",
    ]);
    cmd.args(extra_args);
    cmd.arg(".");
    cmd.output().expect("failed to run mir binary")
}

fn entry(kind: &str, snippet: &str) -> String {
    format!("    <{kind}>\n      <code><![CDATA[{snippet}]]></code>\n    </{kind}>\n")
}

fn emitted_kinds(out: &Output) -> Vec<String> {
    let issues: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout should be JSON");
    issues
        .as_array()
        .expect("JSON array")
        .iter()
        .map(|i| {
            i["kind"]
                .as_object()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .clone()
        })
        .collect()
}

#[test]
fn psalm_named_entries_consume_equivalent_mir_issues() {
    let dir = fixture(&format!(
        "{}{}",
        entry("PossiblyUndefinedGlobalVariable", "$maybe"),
        entry("RedundantCondition", "$n === null"),
    ));

    let out = run(dir.path(), &["--format", "json", "--report-stale-baseline"]);

    assert_eq!(emitted_kinds(&out), Vec::<String>::new());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn unrelated_kind_does_not_consume_the_issue() {
    let dir = fixture(&format!(
        "{}{}",
        entry("UndefinedVariable", "$maybe"),
        entry("TypeDoesNotContainNull", "$n === null"),
    ));

    let out = run(dir.path(), &["--format", "json"]);

    assert_eq!(emitted_kinds(&out), vec!["PossiblyUndefinedVariable"]);
}

#[test]
fn update_baseline_keeps_the_psalm_kind_name() {
    let dir = fixture(&format!(
        "{}{}",
        entry("PossiblyUndefinedGlobalVariable", "$maybe"),
        entry("TypeDoesNotContainType", "$n === null"),
    ));

    run(dir.path(), &["--update-baseline"]);

    let updated = std::fs::read_to_string(dir.path().join("psalm-baseline.xml")).unwrap();
    assert!(
        updated.contains("<PossiblyUndefinedGlobalVariable>"),
        "{updated}"
    );
    assert!(updated.contains("<TypeDoesNotContainType>"), "{updated}");
    assert!(
        !updated.contains("<ImpossibleIdenticalComparison>"),
        "{updated}"
    );
}

#[test]
fn composer_root_run_matches_baseline_keys_relative_to_the_baseline_dir() {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(
        dir.path().join("composer.json"),
        r#"{"autoload": {"files": ["src/a.php"]}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.path().join("src/a.php"),
        "<?php\nfunction f(int $n): void { if ($n === null) { echo 1; } }\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("psalm-baseline.xml"),
        format!(
            "<files>\n  <file src=\"src/a.php\">\n{}  </file>\n</files>\n",
            entry("RedundantCondition", "$n === null")
        ),
    )
    .unwrap();

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_mir"));
    cmd.env_clear().current_dir(dir.path());
    cmd.args([
        "--no-cache",
        "--no-progress",
        "--format",
        "json",
        "--report-stale-baseline",
    ]);
    let out = cmd.output().expect("failed to run mir binary");

    assert_eq!(emitted_kinds(&out), Vec::<String>::new());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn generic_tainted_input_entries_consume_dedicated_sink_kinds() {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(
        dir.path().join("a.php"),
        "<?php\n\
         function f(): void {\n\
         \x20   $a = file_get_contents((string) $_GET['a']);\n\
         \x20   include (string) $_GET['b'];\n\
         \x20   eval((string) $_GET['c']);\n\
         \x20   $d = unserialize((string) $_GET['d']);\n\
         }\n",
    )
    .unwrap();
    run(dir.path(), &["--set-baseline", "psalm-baseline.xml"]);
    let written = std::fs::read_to_string(dir.path().join("psalm-baseline.xml")).unwrap();
    for kind in [
        "TaintedFile",
        "TaintedInclude",
        "TaintedEval",
        "TaintedUnserialize",
    ] {
        assert!(written.contains(&format!("<{kind}>")), "{written}");
    }
    let legacy = [
        "TaintedFile",
        "TaintedInclude",
        "TaintedEval",
        "TaintedUnserialize",
    ]
    .iter()
    .fold(written, |xml, kind| xml.replace(kind, "TaintedInput"));
    std::fs::write(dir.path().join("psalm-baseline.xml"), legacy).unwrap();

    let out = run(dir.path(), &["--format", "json", "--report-stale-baseline"]);

    let tainted: Vec<String> = emitted_kinds(&out)
        .into_iter()
        .filter(|k| k.starts_with("Tainted"))
        .collect();
    assert_eq!(tainted, Vec::<String>::new());
}

#[test]
fn invalid_property_assignment_entries_consume_partly_valid_assignments() {
    let dir = TempDir::new().expect("failed to create temp dir");
    std::fs::write(
        dir.path().join("a.php"),
        "<?php\nclass C {\n    public int $n = 0;\n    public function f(?int $x): void { $this->n = $x; }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("psalm-baseline.xml"),
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<files>\n  <file src=\"a.php\">\n{}  </file>\n</files>\n",
            entry("InvalidPropertyAssignment", "$this->n = $x")
        ),
    )
    .unwrap();

    let out = run(
        dir.path(),
        &["--format", "json", "--show-info", "--report-stale-baseline"],
    );

    assert_eq!(emitted_kinds(&out), Vec::<String>::new());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
