// Integration tests for symbol_reference_locations (mir#184).

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use mir_analyzer::ProjectAnalyzer;
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn function_call_records_reference_location() {
    let dir = TempDir::new().unwrap();
    // The call must be inside a function body — analyze_bodies only processes declarations.
    let file = write(
        &dir,
        "a.php",
        "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n",
    );
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs = analyzer.codebase().get_reference_locations("greet");
    assert!(
        locs.iter().any(|(f, ..)| f == &file_arc),
        "reference location should be recorded for the analyzed file"
    );
    assert!(!locs.is_empty(), "at least one span recorded");
}

#[test]
fn function_call_span_covers_only_name() {
    let dir = TempDir::new().unwrap();
    //                  0123456789...
    // "<?php\n"        = 6 bytes
    // "function greet(): void {}\n"
    // "function caller(): void { greet(); }\n"
    //                            ^-- 'greet' starts here
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "b.php", src);
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs: Vec<_> = analyzer
        .codebase()
        .get_reference_locations("greet")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(locs.len(), 1);
    let (_, _line, col_start, col_end) = locs[0];
    // The span should cover only the 5-byte identifier "greet", not the full call
    assert_eq!(
        col_end - col_start,
        5,
        "span should cover only 'greet' (5 bytes), got col_start={col_start} col_end={col_end}"
    );
}

#[test]
fn method_call_span_covers_only_name() {
    let dir = TempDir::new().unwrap();
    // "<?php\n"                                          = 6 bytes
    // "class Svc { public function run(): void {} }\n"   = 45 bytes  (offset 6)
    // "function caller(): void { $s = new Svc(); $s->run(); }\n"
    //                                             ^-- 'run' starts at offset 97
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n";
    let file = write(&dir, "h.php", src);
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs: Vec<_> = analyzer
        .codebase()
        .get_reference_locations("Svc::run")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(locs.len(), 1);
    let (_, _line, col_start, col_end) = locs[0];
    // The span should cover only the 3-byte identifier "run", not the full call
    assert_eq!(
        col_end - col_start,
        3,
        "span should cover only 'run' (3 bytes), got col_start={col_start} col_end={col_end}"
    );
}

#[test]
fn property_access_span_covers_only_name() {
    let dir = TempDir::new().unwrap();
    // "<?php\n"                                          = 6 bytes
    // "class Counter { public int $count = 0; }\n"      = 41 bytes  (offset 6)
    // "function read(Counter $c): int { return $c->count; }\n"
    //                                              ^-- 'count' starts at offset 91
    let src = "<?php\nclass Counter { public int $count = 0; }\nfunction read(Counter $c): int { return $c->count; }\n";
    let file = write(&dir, "i.php", src);
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs: Vec<_> = analyzer
        .codebase()
        .get_reference_locations("Counter::count")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(locs.len(), 1);
    let (_, _line, col_start, col_end) = locs[0];
    // The span should cover only the 5-byte identifier "count", not the full "$c->count"
    assert_eq!(
        col_end - col_start,
        5,
        "span should cover only 'count' (5 bytes), got col_start={col_start} col_end={col_end}"
    );
}

#[test]
fn nullsafe_property_access_records_reference_location() {
    let dir = TempDir::new().unwrap();
    // "<?php\n"                                     = 6 bytes
    // "class Box { public int $val = 0; }\n"        = 35 bytes  (offset 6)
    // "function read(?Box $b): void { $b?->val; }\n"
    //                                        ^-- 'val' starts at offset 77
    let src =
        "<?php\nclass Box { public int $val = 0; }\nfunction read(?Box $b): void { $b?->val; }\n";
    let file = write(&dir, "j.php", src);
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs: Vec<_> = analyzer
        .codebase()
        .get_reference_locations("Box::val")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(locs.len(), 1);
    let (_, _line, col_start, col_end) = locs[0];
    // The span should cover only the 3-byte identifier "val", not "$b?->val"
    assert_eq!(
        col_end - col_start,
        3,
        "span should cover only 'val' (3 bytes), got col_start={col_start} col_end={col_end}"
    );
}

#[test]
fn method_call_records_reference_location() {
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "c.php",
        "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    assert!(
        !analyzer
            .codebase()
            .get_reference_locations("Svc::run")
            .is_empty(),
        "Svc::run should be in symbol_reference_locations"
    );
}

#[test]
fn multiple_calls_in_same_file_produce_multiple_spans() {
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "d.php",
        "<?php\nfunction ping(): void {}\nfunction caller(): void { ping(); ping(); ping(); }\n",
    );
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let count = analyzer
        .codebase()
        .get_reference_locations("ping")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .count();

    assert_eq!(count, 3, "three calls should produce three spans");
}

#[test]
fn new_expression_records_class_reference() {
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "e.php",
        "<?php\nclass Widget {}\nfunction make(): void { $w = new Widget(); }\n",
    );
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs = analyzer.codebase().get_reference_locations("Widget");
    assert!(
        locs.iter().any(|(f, ..)| f == &file_arc),
        "new Widget() should record a reference to Widget"
    );
}

#[test]
fn re_analyze_removes_stale_reference_locations() {
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "f.php",
        "<?php\nfunction helper(): void {}\nfunction caller(): void { helper(); }\n",
    );
    let file_str = file.to_str().unwrap().to_string();
    let file_arc: Arc<str> = Arc::from(file_str.as_str());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    assert!(
        analyzer
            .codebase()
            .get_reference_locations("helper")
            .iter()
            .any(|(f, ..)| f == &file_arc),
        "initial analysis should record location"
    );

    // Re-analyze with content that no longer calls helper()
    analyzer.re_analyze_file(
        &file_str,
        "<?php\nfunction helper(): void {}\nfunction caller(): void {}\n",
    );

    let stale = analyzer
        .codebase()
        .get_reference_locations("helper")
        .iter()
        .any(|(f, ..)| f == &file_arc);

    assert!(
        !stale,
        "stale reference location should be removed after re-analysis"
    );
}

#[test]
fn static_method_call_span_covers_only_name() {
    let dir = TempDir::new().unwrap();
    // "<?php\n"                                                                    = 6 bytes
    // "class Math { public static function sq(int $n): int { return $n * $n; } }\n" = 74 bytes
    // "function caller(): void { Math::sq(3); }\n"
    //                                    ^-- 'sq' starts at byte 6+74+32 = 112
    let src = "<?php\nclass Math { public static function sq(int $n): int { return $n * $n; } }\nfunction caller(): void { Math::sq(3); }\n";
    let file = write(&dir, "static_span.php", src);
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs: Vec<_> = analyzer
        .codebase()
        .get_reference_locations("Math::sq")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(locs.len(), 1);
    let (_, _line, col_start, col_end) = locs[0];
    // The span should cover only the 2-byte identifier "sq", not the full call
    assert_eq!(
        col_end - col_start,
        2,
        "span should cover only 'sq' (2 bytes), got col_start={col_start} col_end={col_end}"
    );
}

#[test]
fn cache_hit_replays_reference_locations() {
    let dir = TempDir::new().unwrap();
    let cache_dir = dir.path().join("cache");
    let file = write(
        &dir,
        "g.php",
        "<?php\nfunction cached_fn(): void {}\nfunction caller(): void { cached_fn(); }\n",
    );
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    // First run — populates cache
    {
        let analyzer = ProjectAnalyzer::with_cache(&cache_dir);
        analyzer.analyze(std::slice::from_ref(&file));
        assert!(
            !analyzer
                .codebase()
                .get_reference_locations("cached_fn")
                .is_empty(),
            "first run should record reference"
        );
    }

    // Second run — file unchanged, cache hit
    {
        let analyzer = ProjectAnalyzer::with_cache(&cache_dir);
        analyzer.analyze(std::slice::from_ref(&file));

        let locs = analyzer.codebase().get_reference_locations("cached_fn");
        assert!(
            !locs.is_empty(),
            "cache hit should replay reference locations"
        );
        assert!(
            locs.iter().any(|(f, ..)| f == &file_arc),
            "replayed locations should include the correct file"
        );
    }
}

#[test]
fn compact_index_preserves_reference_locations() {
    // After analyze() calls compact_reference_index(), queries must return the
    // same results as before compaction.
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction ping(): void {}\nfunction caller(): void { ping(); ping(); }\n";
    let file = write(&dir, "compact.php", src);
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    // analyze() calls compact_reference_index() internally; verify results are intact.
    let locs: Vec<_> = analyzer
        .codebase()
        .get_reference_locations("ping")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(locs.len(), 2, "two calls → two spans in compact index");
    assert!(
        analyzer
            .codebase()
            .file_has_symbol_references(file.to_str().unwrap()),
        "file_has_symbol_references must return true after compaction"
    );
}

#[test]
fn compact_index_survives_re_analyze() {
    // re_analyze_file() must work correctly even when the index was compacted
    // by a preceding full analyze() call.
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "reanalyze.php",
        "<?php\nfunction helper(): void {}\nfunction caller(): void { helper(); }\n",
    );
    let file_str = file.to_str().unwrap().to_string();
    let file_arc: Arc<str> = Arc::from(file_str.as_str());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    // Index is now compact; verify initial state.
    assert!(
        analyzer
            .codebase()
            .get_reference_locations("helper")
            .iter()
            .any(|(f, ..)| f == &file_arc),
        "initial reference should be recorded"
    );

    // Re-analyze without the call — compact index must be expanded, stale entry removed.
    analyzer.re_analyze_file(
        &file_str,
        "<?php\nfunction helper(): void {}\nfunction caller(): void {}\n",
    );

    let stale = analyzer
        .codebase()
        .get_reference_locations("helper")
        .iter()
        .any(|(f, ..)| f == &file_arc);
    assert!(
        !stale,
        "stale span must be removed after re-analysis through compact index"
    );
}

#[test]
fn this_method_call_records_reference_location() {
    // $this->method() calls were previously invisible to the reference index
    // because $this was untyped and the mixed-receiver guard fired before
    // record_symbol could be called (issue #191).
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "this_ref.php",
        "<?php\nclass Svc { public function helper(): void {}\npublic function run(): void { $this->helper(); } }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    assert!(
        !analyzer
            .codebase()
            .get_reference_locations("Svc::helper")
            .is_empty(),
        "$this->helper() should record a reference to Svc::helper"
    );
}

#[test]
fn this_method_call_span_covers_only_name() {
    // The recorded span for $this->helper() must cover only the method name
    // identifier, matching the behaviour for non-$this receivers.
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Svc { public function helper(): void {}\npublic function run(): void { $this->helper(); } }\n";
    let file = write(&dir, "this_span.php", src);
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs: Vec<_> = analyzer
        .codebase()
        .get_reference_locations("Svc::helper")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(locs.len(), 1, "one $this->helper() call → one span");

    let (_, _line, col_start, col_end) = locs[0];
    assert_eq!(
        col_end - col_start,
        6, // "helper" = 6 bytes
        "span must cover only the identifier 'helper' (6 bytes), got col_start={col_start} col_end={col_end}"
    );
}

#[test]
fn nullsafe_method_call_records_reference_location() {
    let dir = TempDir::new().unwrap();
    // "<?php\n"                                       = 6 bytes
    // "class Svc { public function run(): void {} }\n" = 45 bytes  (offset 6)
    // "function caller(?Svc $s): void { $s?->run(); }\n"
    //                                          ^-- 'run' starts at offset 88
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(?Svc $s): void { $s?->run(); }\n";
    let file = write(&dir, "nullsafe_method.php", src);
    let file_arc: Arc<str> = Arc::from(file.to_str().unwrap());

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file));

    let locs: Vec<_> = analyzer
        .codebase()
        .get_reference_locations("Svc::run")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(locs.len(), 1);
    let (_, _line, col_start, col_end) = locs[0];
    // The span should cover only the 3-byte identifier "run", not "$s?->run()"
    assert_eq!(
        col_end - col_start,
        3,
        "span should cover only 'run' (3 bytes), got col_start={col_start} col_end={col_end}"
    );
}
