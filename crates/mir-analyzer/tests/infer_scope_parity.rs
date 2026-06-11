//! Parity gate for per-scope tracked inference: merging all of a file's
//! `infer_scope` results in the defined order must reproduce
//! `BodyAnalyzer::analyze_bodies` output *exactly* — including issue
//! emission order (fixture assertions are set-based, so order drift would
//! otherwise go unnoticed until an UPDATE_FIXTURES run churns every file).

use std::sync::Arc;

use mir_analyzer::db::{analyze_file_per_scope, MirDatabase, RefLoc};
use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

/// Run both paths over `source` and assert exact equality.
fn assert_scope_parity(name: &str, source: &str) {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(format!("{name}.php"));
    std::fs::write(&path, source).unwrap();
    let _ = session.analyze_paths(std::slice::from_ref(&path), &BatchOptions::new());

    let db = session.snapshot_db();
    let path_str: Arc<str> = Arc::from(path.to_string_lossy().as_ref());
    let file = db.lookup_source_file(path_str.as_ref()).unwrap();

    // Reference path: whole-file walk, refs drained from the staging buffer.
    let parsed = mir_analyzer::db::parse_file(&db, file);
    let (expected_issues, _symbols) = {
        // BodyAnalyzer is crate-private; go through analyze_file's components:
        // re-run the whole-file walk via a second analyze on a fresh frame.
        // FileAnalyzer wraps analyze_bodies 1:1 for a parsed program.
        let analyzer = mir_analyzer::FileAnalyzer::new(&session);
        let analysis = analyzer.analyze(
            path_str.clone(),
            source,
            &parsed.0.program,
            &parsed.0.source_map,
        );
        (analysis.issues, analysis.symbols)
    };
    let mut expected_refs: Vec<RefLoc> = db
        .extract_file_reference_locations(path_str.as_ref())
        .into_iter()
        .map(|(symbol_key, line, col_start, col_end)| RefLoc {
            symbol_key,
            file: path_str.clone(),
            line,
            col_start,
            col_end,
        })
        .collect();
    expected_refs.sort();
    expected_refs.dedup();

    // Per-scope path.
    let (scope_issues, mut scope_refs) = analyze_file_per_scope(&db, file);
    scope_refs.sort();
    scope_refs.dedup();
    // analyze_bodies output excludes parse errors (none in these fixtures);
    // analyze_file_per_scope likewise excludes them by construction.

    assert_eq!(
        expected_issues, scope_issues,
        "issue stream mismatch (order included) for fixture `{name}`"
    );
    // The whole-file reference set may include refs recorded outside body
    // analysis (e.g. during definition collection); the per-scope set must
    // be a subset-equal on the body-analysis-produced refs. Compare via the
    // per-scope refs all being present in the committed set, and the
    // committed per-file set not containing body refs the scopes missed.
    for r in &scope_refs {
        assert!(
            expected_refs.contains(r),
            "per-scope ref {r:?} missing from whole-file set for `{name}`"
        );
    }
    for r in &expected_refs {
        assert!(
            scope_refs.contains(r),
            "whole-file ref {r:?} missing from per-scope set for `{name}`"
        );
    }
}

#[test]
fn parity_functions_and_classes() {
    assert_scope_parity(
        "mixed",
        r#"<?php
use Foo\Bar;

function plain(): string {
    return "hello";
}

class Greeter {
    public string $name;
    private $untypedProp;

    public function greet(int $times): string {
        $out = "";
        for ($i = 0; $i < $times; $i++) {
            $out .= "hi";
        }
        return $out;
    }

    public function broken(): int {
        return $undefined_var;
    }
}

function returns_str(): string {
    return 42;
}
"#,
    );
}

#[test]
fn parity_trait_and_enum() {
    assert_scope_parity(
        "trait_enum",
        r#"<?php
trait Helper {
    public function help(): string {
        return "help";
    }
}

enum Suit: string {
    case Hearts = 'H';
    case Spades = 'S';

    public function color(): string {
        return match($this) {
            Suit::Hearts => 'Red',
            Suit::Spades => 'Black',
        };
    }
}

interface Shape {
    public function area(): float;
}
"#,
    );
}

#[test]
fn parity_namespaced_and_guarded() {
    assert_scope_parity(
        "ns_guarded",
        r#"<?php
namespace App;

class Service {
    public function run(): void {
        $unused = 1;
    }
}

if (! function_exists('App\helper')) {
    function helper(): string {
        return 42;
    }
}

$x = new Service();
$x->run();
"#,
    );
}

#[test]
fn parity_duplicate_declarations() {
    assert_scope_parity(
        "dups",
        r#"<?php
function twice(): string {
    return "a";
}

function twice(): string {
    return 42;
}
"#,
    );
}

#[test]
fn parity_top_level_exec_only() {
    assert_scope_parity(
        "exec",
        r#"<?php
$a = 1;
$b = $a + $undefined_thing;
echo $b;
"#,
    );
}
