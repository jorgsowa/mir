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

    let locs = analyzer
        .codebase()
        .symbol_reference_locations
        .get("greet")
        .expect("greet should be in symbol_reference_locations");

    assert!(
        locs.contains_key(&file_arc),
        "reference location should be recorded for the analyzed file"
    );
    assert!(!locs[&file_arc].is_empty(), "at least one span recorded");
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

    let locs = analyzer
        .codebase()
        .symbol_reference_locations
        .get("greet")
        .expect("greet should be in symbol_reference_locations");

    let spans = &locs[&file_arc];
    assert_eq!(spans.len(), 1);
    let &(start, end) = spans.iter().next().unwrap();
    // The span should cover only the 5-byte identifier "greet", not the full call
    assert_eq!(
        end - start,
        5,
        "span should cover only 'greet' (5 bytes), got start={start} end={end}"
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

    let locs = analyzer
        .codebase()
        .symbol_reference_locations
        .get("Svc::run")
        .expect("Svc::run should be in symbol_reference_locations");

    let spans = &locs[&file_arc];
    assert_eq!(spans.len(), 1);
    let &(start, end) = spans.iter().next().unwrap();
    // The span should cover only the 3-byte identifier "run", not the full call
    assert_eq!(
        end - start,
        3,
        "span should cover only 'run' (3 bytes), got start={start} end={end}"
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

    let locs = analyzer
        .codebase()
        .symbol_reference_locations
        .get("Counter::count")
        .expect("Counter::count should be in symbol_reference_locations");

    let spans = &locs[&file_arc];
    assert_eq!(spans.len(), 1);
    let &(start, end) = spans.iter().next().unwrap();
    // The span should cover only the 5-byte identifier "count", not the full "$c->count"
    assert_eq!(
        end - start,
        5,
        "span should cover only 'count' (5 bytes), got start={start} end={end}"
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

    let locs = analyzer
        .codebase()
        .symbol_reference_locations
        .get("Box::val")
        .expect("Box::val should be in symbol_reference_locations after $b?->val");

    let spans = &locs[&file_arc];
    assert_eq!(spans.len(), 1);
    let &(start, end) = spans.iter().next().unwrap();
    // The span should cover only the 3-byte identifier "val", not "$b?->val"
    assert_eq!(
        end - start,
        3,
        "span should cover only 'val' (3 bytes), got start={start} end={end}"
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
        analyzer
            .codebase()
            .symbol_reference_locations
            .contains_key("Svc::run"),
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

    let locs = analyzer
        .codebase()
        .symbol_reference_locations
        .get("ping")
        .expect("ping should be in symbol_reference_locations");

    assert_eq!(
        locs[&file_arc].len(),
        3,
        "three calls should produce three spans"
    );
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

    let locs = analyzer
        .codebase()
        .symbol_reference_locations
        .get("Widget")
        .expect("Widget should be in symbol_reference_locations after new Widget()");

    assert!(
        locs.contains_key(&file_arc),
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
            .symbol_reference_locations
            .get("helper")
            .map(|m| m.contains_key(&file_arc))
            .unwrap_or(false),
        "initial analysis should record location"
    );

    // Re-analyze with content that no longer calls helper()
    analyzer.re_analyze_file(
        &file_str,
        "<?php\nfunction helper(): void {}\nfunction caller(): void {}\n",
    );

    let stale = analyzer
        .codebase()
        .symbol_reference_locations
        .get("helper")
        .map(|m| m.contains_key(&file_arc))
        .unwrap_or(false);

    assert!(
        !stale,
        "stale reference location should be removed after re-analysis"
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
            analyzer
                .codebase()
                .symbol_reference_locations
                .contains_key("cached_fn"),
            "first run should record reference"
        );
    }

    // Second run — file unchanged, cache hit
    {
        let analyzer = ProjectAnalyzer::with_cache(&cache_dir);
        analyzer.analyze(std::slice::from_ref(&file));

        let locs = analyzer
            .codebase()
            .symbol_reference_locations
            .get("cached_fn")
            .expect("cache hit should replay reference locations");

        assert!(
            locs.contains_key(&file_arc),
            "replayed locations should include the correct file"
        );
    }
}
