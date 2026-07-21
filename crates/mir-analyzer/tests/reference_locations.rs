// Integration tests for symbol_reference_locations (mir#184).

mod common;

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

use self::common::{create_temp_dir, pathbuf_to_arc_str, write_file};

#[test]
fn function_call_records_reference_location() {
    let dir = create_temp_dir("test");
    // The call must be inside a function body — analyze_bodies only processes declarations.
    let file = write_file(
        &dir,
        "a.php",
        "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs = analyzer.reference_locations("fn:greet");
    assert!(
        locs.iter().any(|(f, ..)| f == &file_arc),
        "reference location should be recorded for the analyzed file"
    );
    assert!(!locs.is_empty(), "at least one span recorded");
}

#[test]
fn function_call_span_covers_only_name() {
    let dir = create_temp_dir("test");
    //                  0123456789...
    // "<?php\n"        = 6 bytes
    // "function greet(): void {}\n"
    // "function caller(): void { greet(); }\n"
    //                            ^-- 'greet' starts here
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write_file(&dir, "b.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("fn:greet")
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
    let dir = create_temp_dir("test");
    // "<?php\n"                                          = 6 bytes
    // "class Svc { public function run(): void {} }\n"   = 45 bytes  (offset 6)
    // "function caller(): void { $s = new Svc(); $s->run(); }\n"
    //                                             ^-- 'run' starts at offset 97
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n";
    let file = write_file(&dir, "h.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("meth:Svc::run")
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
fn static_method_call_via_class_string_variable_records_reference() {
    // `$cls::method()` where `$cls` holds a class-string variable must record
    // a reference, matching the plain `Math::sq()` form.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Math { public static function sq(int $n): int { return $n * $n; } }\nfunction caller(): void { $cls = Math::class; $cls::sq(3); }\n";
    let file = write_file(&dir, "dyn_static.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("meth:Math::sq")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(
        locs.len(),
        1,
        "expected exactly one reference to Math::sq from $cls::sq(3)"
    );
}

#[test]
fn dynamic_invoke_call_records_reference_to_invoke_method() {
    // `$obj(...)` invoking an object's __invoke() must record a reference,
    // matching every other call form ($obj->run(), Svc::run(), etc.).
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc { public function __invoke(): void {} }\nfunction caller(Svc $s): void { $s(); }\n";
    let file = write_file(&dir, "i.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("meth:Svc::__invoke")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(
        locs.len(),
        1,
        "expected exactly one reference to Svc::__invoke from $s()"
    );
}

#[test]
fn property_access_span_covers_only_name() {
    let dir = create_temp_dir("test");
    // "<?php\n"                                          = 6 bytes
    // "class Counter { public int $count = 0; }\n"      = 41 bytes  (offset 6)
    // "function read(Counter $c): int { return $c->count; }\n"
    //                                              ^-- 'count' starts at offset 91
    let src = "<?php\nclass Counter { public int $count = 0; }\nfunction read(Counter $c): int { return $c->count; }\n";
    let file = write_file(&dir, "i.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("prop:Counter::count")
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
    let dir = create_temp_dir("test");
    // "<?php\n"                                     = 6 bytes
    // "class Box { public int $val = 0; }\n"        = 35 bytes  (offset 6)
    // "function read(?Box $b): void { $b?->val; }\n"
    //                                        ^-- 'val' starts at offset 77
    let src =
        "<?php\nclass Box { public int $val = 0; }\nfunction read(?Box $b): void { $b?->val; }\n";
    let file = write_file(&dir, "j.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("prop:Box::val")
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
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "c.php",
        "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("meth:Svc::run").is_empty(),
        "Svc::run should be in symbol_reference_locations"
    );
}

#[test]
fn multiple_calls_in_same_file_produce_multiple_spans() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "d.php",
        "<?php\nfunction ping(): void {}\nfunction caller(): void { ping(); ping(); ping(); }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let count = analyzer
        .reference_locations("fn:ping")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .count();

    assert_eq!(count, 3, "three calls should produce three spans");
}

#[test]
fn new_expression_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "e.php",
        "<?php\nclass Widget {}\nfunction make(): void { $w = new Widget(); }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs = analyzer.reference_locations("cls:Widget");
    assert!(
        locs.iter().any(|(f, ..)| f == &file_arc),
        "new Widget() should record a reference to Widget"
    );
}

#[test]
fn new_expression_via_class_string_variable_records_class_reference() {
    // `new $cls()` where `$cls` holds a known class-string must record a
    // reference, matching the plain `new Widget()` form.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "e2.php",
        "<?php\nclass Widget {}\nfunction make(): void { $cls = Widget::class; $w = new $cls(); }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs = analyzer.reference_locations("cls:Widget");
    assert!(
        locs.iter().any(|(f, ..)| f == &file_arc),
        "new $cls() should record a reference to Widget"
    );
}

#[test]
fn instanceof_via_class_string_variable_records_class_reference() {
    // `$x instanceof $cls` where `$cls` holds a known class-string must
    // record a reference, matching the plain `$x instanceof Widget` form.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "e3.php",
        "<?php\nclass Widget {}\nfunction check(object $o): bool { $cls = Widget::class; return $o instanceof $cls; }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs = analyzer.reference_locations("cls:Widget");
    assert!(
        locs.iter().any(|(f, ..)| f == &file_arc),
        "$o instanceof $cls should record a reference to Widget"
    );
}

#[test]
fn re_analyze_removes_stale_reference_locations() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "f.php",
        "<?php\nfunction helper(): void {}\nfunction caller(): void { helper(); }\n",
    );
    let file_str = file.to_str().unwrap().to_string();
    let file_arc: Arc<str> = Arc::from(file_str.as_str());

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        analyzer
            .reference_locations("fn:helper")
            .iter()
            .any(|(f, ..)| f == &file_arc),
        "initial analysis should record location"
    );

    // Re-analyze with content that no longer calls helper()
    analyzer.re_analyze_file(
        &file_str,
        "<?php\nfunction helper(): void {}\nfunction caller(): void {}\n",
        &BatchOptions::new(),
    );

    let stale = analyzer
        .reference_locations("fn:helper")
        .iter()
        .any(|(f, ..)| f == &file_arc);

    assert!(
        !stale,
        "stale reference location should be removed after re-analysis"
    );
}

#[test]
fn static_method_call_span_covers_only_name() {
    let dir = create_temp_dir("test");
    // "<?php\n"                                                                    = 6 bytes
    // "class Math { public static function sq(int $n): int { return $n * $n; } }\n" = 74 bytes
    // "function caller(): void { Math::sq(3); }\n"
    //                                    ^-- 'sq' starts at byte 6+74+32 = 112
    let src = "<?php\nclass Math { public static function sq(int $n): int { return $n * $n; } }\nfunction caller(): void { Math::sq(3); }\n";
    let file = write_file(&dir, "static_span.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("meth:Math::sq")
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
    let dir = create_temp_dir("test");
    let cache_dir = dir.path().join("cache");
    let file = write_file(
        &dir,
        "g.php",
        "<?php\nfunction cached_fn(): void {}\nfunction caller(): void { cached_fn(); }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    // First run — populates cache
    {
        let analyzer = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(&cache_dir);
        analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());
        assert!(
            !analyzer.reference_locations("fn:cached_fn").is_empty(),
            "first run should record reference"
        );
    }

    // Second run — file unchanged, cache hit
    {
        let analyzer = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(&cache_dir);
        analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

        let locs = analyzer.reference_locations("fn:cached_fn");
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
    let dir = create_temp_dir("test");
    let src = "<?php\nfunction ping(): void {}\nfunction caller(): void { ping(); ping(); }\n";
    let file = write_file(&dir, "compact.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // After analyze(), the reference index must hold both call sites.
    let locs: Vec<_> = analyzer
        .reference_locations("fn:ping")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert_eq!(
        locs.len(),
        2,
        "two calls → two spans in the reference index"
    );
}

#[test]
fn compact_index_survives_re_analyze() {
    // re_analyze_file() must work correctly even when the index was compacted
    // by a preceding full analyze() call.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "reanalyze.php",
        "<?php\nfunction helper(): void {}\nfunction caller(): void { helper(); }\n",
    );
    let file_str = file.to_str().unwrap().to_string();
    let file_arc: Arc<str> = Arc::from(file_str.as_str());

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Index is now compact; verify initial state.
    assert!(
        analyzer
            .reference_locations("fn:helper")
            .iter()
            .any(|(f, ..)| f == &file_arc),
        "initial reference should be recorded"
    );

    // Re-analyze without the call — compact index must be expanded, stale entry removed.
    analyzer.re_analyze_file(
        &file_str,
        "<?php\nfunction helper(): void {}\nfunction caller(): void {}\n",
        &BatchOptions::new(),
    );

    let stale = analyzer
        .reference_locations("fn:helper")
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
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "this_ref.php",
        "<?php\nclass Svc { public function helper(): void {}\npublic function run(): void { $this->helper(); } }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("meth:Svc::helper").is_empty(),
        "$this->helper() should record a reference to Svc::helper"
    );
}

#[test]
fn this_method_call_span_covers_only_name() {
    // The recorded span for $this->helper() must cover only the method name
    // identifier, matching the behaviour for non-$this receivers.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc { public function helper(): void {}\npublic function run(): void { $this->helper(); } }\n";
    let file = write_file(&dir, "this_span.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("meth:Svc::helper")
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
    let dir = create_temp_dir("test");
    // "<?php\n"                                       = 6 bytes
    // "class Svc { public function run(): void {} }\n" = 45 bytes  (offset 6)
    // "function caller(?Svc $s): void { $s?->run(); }\n"
    //                                          ^-- 'run' starts at offset 88
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(?Svc $s): void { $s?->run(); }\n";
    let file = write_file(&dir, "nullsafe_method.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("meth:Svc::run")
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

#[test]
fn instanceof_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "instanceof_ref.php",
        "<?php\nclass Widget {}\nfunction check(mixed $v): bool { return $v instanceof Widget; }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("cls:Widget").is_empty(),
        "instanceof Widget should record a reference to Widget"
    );
}

#[test]
fn catch_type_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "catch_ref.php",
        "<?php\nclass AppEx extends \\Exception {}\nfunction run(): void { try {} catch (AppEx $e) { echo $e->getMessage(); } }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("cls:AppEx").is_empty(),
        "catch (AppEx $e) should record a reference to AppEx"
    );
}

#[test]
fn multi_type_catch_records_all_class_references() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "multi_catch_ref.php",
        "<?php\nclass ErrA extends \\Exception {}\nclass ErrB extends \\Exception {}\nfunction run(): void { try {} catch (ErrA | ErrB $e) { echo $e->getMessage(); } }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs_a: Vec<_> = analyzer
        .reference_locations("cls:ErrA")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();
    let locs_b: Vec<_> = analyzer
        .reference_locations("cls:ErrB")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs_a.is_empty(),
        "catch (ErrA | ErrB $e) should record a reference to ErrA"
    );
    assert!(
        !locs_b.is_empty(),
        "catch (ErrA | ErrB $e) should record a reference to ErrB"
    );
}

#[test]
fn class_const_syntax_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "classconst_ref.php",
        "<?php\nclass Router {}\nfunction getClass(): string { return Router::class; }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("cls:Router").is_empty(),
        "Router::class should record a reference to Router"
    );
}

#[test]
fn static_const_access_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "static_const_ref.php",
        "<?php\nclass Config { const VERSION = '1.0'; }\nfunction ver(): string { return Config::VERSION; }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("cls:Config").is_empty(),
        "Config::VERSION should record a reference to Config"
    );
}

#[test]
fn function_param_type_hint_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "param_hint_ref.php",
        "<?php\nclass Service {}\nfunction process(Service $svc): void { echo get_class($svc); }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Service")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "function param type hint Service $svc should record a reference to Service"
    );
}

#[test]
fn return_type_hint_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "return_hint_ref.php",
        "<?php\nclass Repo {}\nfunction make(): Repo { return new Repo(); }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Repo")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "return type hint Repo should record a reference to Repo"
    );
}

#[test]
fn property_type_hint_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "prop_hint_ref.php",
        "<?php\nclass Logger {}\nclass App { public Logger $logger; }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Logger")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "property type hint Logger should record a reference to Logger"
    );
}

#[test]
fn self_const_access_records_constant_reference() {
    // self::CONST inside the declaring class was silently dropped — no record_ref
    // was emitted for the constant key, so findReferences returned nothing.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "self_const.php",
        "<?php\nfinal class Foo {\n    public const string SEP = ':';\n    public function build(string $a, string $b): string {\n        return $a . self::SEP . $b;\n    }\n}\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("cnst:Foo::SEP").is_empty(),
        "self::SEP should record a reference to Foo::SEP"
    );
}

#[test]
fn static_const_access_records_constant_reference() {
    // static::CONST (late static binding) should also record the constant reference.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "static_const.php",
        "<?php\nclass Bar {\n    public const string PREFIX = 'x';\n    public function go(): string {\n        return static::PREFIX;\n    }\n}\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("cnst:Bar::PREFIX").is_empty(),
        "static::PREFIX should record a reference to Bar::PREFIX"
    );
}

#[test]
fn parent_const_access_records_constant_reference() {
    // parent::CONST should record a reference to the parent class's constant.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "parent_const.php",
        "<?php\nclass Base {\n    public const string TAG = 'base';\n}\nclass Child extends Base {\n    public function tag(): string {\n        return parent::TAG;\n    }\n}\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("cnst:Base::TAG").is_empty(),
        "parent::TAG should record a reference to Base::TAG"
    );
}

#[test]
fn trait_constant_accessed_via_consuming_class_records_reference_to_trait() {
    // `ClassUsingTrait::CONST` where the constant is declared in a used trait
    // must record a reference against the trait itself (Trait::CONST is
    // never a legal access target, so this is the only way to ever record a
    // usage of the trait's own constant declaration).
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "trait_const.php",
        "<?php\ntrait HasVersion {\n    public const string VERSION = '1.0';\n}\nclass Config {\n    use HasVersion;\n}\nfunction ver(): string { return Config::VERSION; }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer
            .reference_locations("cnst:HasVersion::VERSION")
            .is_empty(),
        "Config::VERSION should record a reference to HasVersion::VERSION, not Config::VERSION"
    );
}

#[test]
fn explicit_class_const_access_records_constant_reference() {
    // ClassName::CONST should record a reference to the constant, not just the class.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "explicit_const.php",
        "<?php\nclass Config {\n    public const string VERSION = '1.0';\n}\nfunction ver(): string { return Config::VERSION; }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer
            .reference_locations("cnst:Config::VERSION")
            .is_empty(),
        "Config::VERSION should record a reference to Config::VERSION"
    );
}

#[test]
fn inherited_static_method_call_keys_by_declaring_class() {
    // Child::foo() where foo is declared on Base must record "Base::foo",
    // not "Child::foo", so that reference_locations("meth:Base::foo") finds the call.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "inherited_static.php",
        "<?php\nclass Base { public static function foo(): void {} }\nclass Child extends Base {}\nfunction caller(): void { Child::foo(); }\n",
    );

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    assert!(
        !analyzer.reference_locations("meth:Base::foo").is_empty(),
        "Child::foo() should record a reference to the declaring class Base::foo"
    );
    assert!(
        analyzer.reference_locations("meth:Child::foo").is_empty(),
        "Child::foo() must not be recorded under the called subclass key Child::foo"
    );
}

#[test]
fn static_property_access_records_class_reference() {
    // Foo::$bar should record a class reference to Foo so that
    // reference_locations("cls:Foo") includes the static property access.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "static_prop_class_ref.php",
        "<?php\nclass Config { public static int $timeout = 30; }\nfunction read(): int { return Config::$timeout; }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let class_locs: Vec<_> = analyzer
        .reference_locations("cls:Config")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !class_locs.is_empty(),
        "Config::$timeout should record a class reference to Config"
    );
}

#[test]
fn static_property_access_records_property_reference() {
    // Foo::$bar should also record a property reference so that
    // reference_locations("prop:Foo::timeout") finds the static property access.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "static_prop_ref.php",
        "<?php\nclass Config { public static int $timeout = 30; }\nfunction read(): int { return Config::$timeout; }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let prop_locs: Vec<_> = analyzer
        .reference_locations("prop:Config::timeout")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !prop_locs.is_empty(),
        "Config::$timeout should record a property reference to Config::timeout"
    );
}

#[test]
fn static_method_call_records_class_reference() {
    // Widget::foo() should record a class reference to Widget, consistent with
    // Widget::VERSION, Widget::$bar, and new Widget() all recording one.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "static_method_class_ref.php",
        "<?php\nclass Widget { public static function make(): void {} }\nfunction caller(): void { Widget::make(); }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Widget")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "Widget::make() should record a class reference to Widget"
    );
}

#[test]
fn closure_param_type_hint_records_class_reference() {
    // Type hints in closure parameters (`function(Foo $x)`) should record a
    // ClassReference so that find-references on Foo includes closure usages.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "closure_hint.php",
        "<?php\nclass Logger {}\n$fn = function(Logger $l): void {};\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Logger")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "closure param type hint Logger should record a reference to Logger"
    );
}

#[test]
fn arrow_function_param_type_hint_records_class_reference() {
    // Same requirement for arrow functions (`fn(Foo $x) => $x`).
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "arrow_hint.php",
        "<?php\nclass Formatter {}\n$fn = fn(Formatter $f) => $f;\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Formatter")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "arrow function param type hint Formatter should record a reference to Formatter"
    );
}

#[test]
fn anonymous_class_extends_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "anon_extends.php",
        "<?php\nclass Base {}\nfunction make(): object { return new class extends Base {}; }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Base")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "anonymous class `extends Base` should record a reference to Base"
    );
}

#[test]
fn anonymous_class_implements_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "anon_implements.php",
        "<?php\ninterface Greets {}\nfunction make(): object { return new class implements Greets {}; }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Greets")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "anonymous class `implements Greets` should record a reference to Greets"
    );
}

#[test]
fn anonymous_class_use_trait_records_class_reference() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "anon_use_trait.php",
        "<?php\ntrait Helper {}\nfunction make(): object { return new class { use Helper; }; }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Helper")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "anonymous class `use Helper;` should record a reference to Helper"
    );
}

#[test]
fn interface_declared_property_access_records_reference_location() {
    let dir = create_temp_dir("test");
    // Accessing a `@property`-declared interface property through an
    // interface-typed receiver must record a reference the same way a
    // class/trait-declared property access already does.
    let src = "<?php\n/**\n * @property string $name\n */\ninterface HasName {}\nfunction show(HasName $x): string { return $x->name; }\n";
    let file = write_file(&dir, "k.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("prop:HasName::name")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "property access through an interface-typed receiver should record a reference"
    );
}

#[test]
fn inherited_static_property_access_via_subclass_name_keys_by_declaring_class() {
    // Child::$shared for a $shared declared on ParentC must record
    // prop:ParentC::shared (the declaring owner), not prop:Child::shared —
    // otherwise find-references from the declaration never sees usages
    // reached only through a subclass name, mirroring the analogous fix
    // already applied to constants and instance properties.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "inherited_static_prop.php",
        "<?php\nclass ParentC { protected static ?string $shared = null; }\nclass Child extends ParentC {}\nfunction read(): ?string { return Child::$shared; }\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let owner_locs: Vec<_> = analyzer
        .reference_locations("prop:ParentC::shared")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();
    assert!(
        !owner_locs.is_empty(),
        "Child::$shared should record a reference keyed by the declaring class ParentC"
    );

    let subclass_locs: Vec<_> = analyzer
        .reference_locations("prop:Child::shared")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();
    assert!(
        subclass_locs.is_empty(),
        "Child::$shared must not also record a reference keyed by the accessed-through class"
    );
}

#[test]
fn inherited_static_property_access_via_self_keys_by_declaring_class() {
    // self::$shared inside a Child method, for a $shared declared on
    // ParentC, must also key by the declaring owner ParentC.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "inherited_static_prop_self.php",
        "<?php\nclass ParentC { protected static ?string $shared = null; }\nclass Child extends ParentC {\n    public static function read(): ?string { return self::$shared; }\n}\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let owner_locs: Vec<_> = analyzer
        .reference_locations("prop:ParentC::shared")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();
    assert!(
        !owner_locs.is_empty(),
        "self::$shared inside Child should record a reference keyed by the declaring class ParentC"
    );
}

#[test]
fn attribute_argument_class_constant_records_constant_reference() {
    // #[Route(Target::VERSION)] must record a reference to the specific
    // constant Target::VERSION, not just the class Target — otherwise
    // find-references from the constant declaration misses this usage.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "attr_const_ref.php",
        "<?php\nclass Target {\n    const VERSION = 'v1';\n}\n#[Attribute]\nclass Route {\n    public function __construct(public string $v) {}\n}\n#[Route(Target::VERSION)]\nclass Consumer {}\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cnst:Target::VERSION")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "attribute argument Target::VERSION should record a constant reference"
    );
}

#[test]
fn attribute_argument_enum_case_records_constant_reference() {
    // #[Route(Status::Active)] must record a reference to the specific enum
    // case Status::Active, not just the enum class Status.
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "attr_enum_case_ref.php",
        "<?php\nenum Status {\n    case Active;\n    case Inactive;\n}\n#[Attribute]\nclass Route {\n    public function __construct(public Status $s) {}\n}\n#[Route(Status::Active)]\nclass Consumer {}\n",
    );
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cnst:Status::Active")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "attribute argument Status::Active should record a reference to the specific case"
    );

    let inactive_locs: Vec<_> = analyzer
        .reference_locations("cnst:Status::Inactive")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();
    assert!(
        inactive_locs.is_empty(),
        "Status::Inactive was never referenced and must not show up"
    );
}

#[test]
fn trait_composing_trait_records_reference_at_use_site() {
    // A trait consuming another trait (`trait A { use B; }`) previously had no
    // location storage for its own `use` tokens, so the reference fell back to
    // a dummy line-1/col-0 location instead of the real `use B;` line.
    let dir = create_temp_dir("test");
    let src = "<?php\ntrait Greets {\n    public function greet(): void {}\n}\ntrait Person {\n    use Greets;\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Greets")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "use Greets; inside trait Person should record a reference to Greets"
    );
    let (_, line, col_start, _) = locs[0];
    assert_eq!(
        (line, col_start),
        (6, 8),
        "reference should point at the real `use Greets;` line/column, not the line-1/col-0 fallback"
    );
}

#[test]
fn trait_method_parameter_attribute_records_class_reference() {
    // check_trait_attributes only walked method attributes, not each method's
    // own parameter attributes, so `new Foo()` inside a trait method param's
    // attribute args was never recorded as a reference to Foo.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Foo {}\n#[Attribute]\nclass Route {\n    public function __construct(public $v) {}\n}\ntrait Handles {\n    public function handle(#[Route(new Foo())] $x): void {}\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Foo")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "new Foo() inside a trait method parameter's attribute args should record a reference to Foo"
    );
}

#[test]
fn interface_method_parameter_attribute_records_class_reference() {
    // check_interface_attributes only walked method attributes, not each
    // method's own parameter attributes.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Foo {}\n#[Attribute]\nclass Route {\n    public function __construct(public $v) {}\n}\ninterface Handles {\n    public function handle(#[Route(new Foo())] $x): void;\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Foo")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "new Foo() inside an interface method parameter's attribute args should record a reference to Foo"
    );
}

#[test]
fn interface_class_const_attribute_records_class_reference() {
    // check_interface_attributes never matched ClassMemberKind::ClassConst at
    // all, so an interface constant's attributes were skipped entirely.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Foo {}\n#[Attribute]\nclass Route {\n    public function __construct(public $v) {}\n}\ninterface HasDefault {\n    #[Route(Foo::class)]\n    const DEFAULT = 1;\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Foo")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "Foo::class inside an interface constant's attribute args should record a reference to Foo"
    );
}

#[test]
fn enum_method_parameter_attribute_records_class_reference() {
    // check_enum_attributes only walked method attributes, not each method's
    // own parameter attributes.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Foo {}\n#[Attribute]\nclass Route {\n    public function __construct(public $v) {}\n}\nenum Status {\n    case Active;\n    public function handle(#[Route(new Foo())] $x): void {}\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Foo")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "new Foo() inside an enum method parameter's attribute args should record a reference to Foo"
    );
}

#[test]
fn enum_class_const_attribute_records_class_reference() {
    // check_enum_attributes never matched EnumMemberKind::ClassConst, so an
    // enum constant's attributes were skipped entirely.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Foo {}\n#[Attribute]\nclass Route {\n    public function __construct(public $v) {}\n}\nenum Status {\n    case Active;\n    #[Route(Foo::class)]\n    const DEFAULT = 1;\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Foo")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "Foo::class inside an enum constant's attribute args should record a reference to Foo"
    );
}

#[test]
fn property_hook_attribute_records_class_reference() {
    // PHP 8.4 property hooks (`get`/`set`) were never visited by any of the
    // attribute-checking functions, so `new Foo()` inside a hook's own
    // attribute args was never recorded as a reference to Foo.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Foo {}\n#[Attribute]\nclass Route {\n    public function __construct(public $v = null) {}\n}\nclass Widget {\n    public string $name {\n        #[Route(new Foo())]\n        get => $this->name;\n    }\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Foo")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "new Foo() inside a property hook's attribute args should record a reference to Foo"
    );
}

#[test]
fn property_hook_parameter_attribute_records_class_reference() {
    // The `set` hook's own parameter can carry attributes (e.g. `set(#[Route]
    // string $value) {...}`); those were unreachable too since nothing ever
    // walked a hook's params.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Bar {}\n#[Attribute]\nclass Route {\n    public function __construct(public $v = null) {}\n}\nclass Widget {\n    private string $raw;\n    public string $name {\n        set(#[Route(new Bar())] string $value) {\n            $this->raw = $value;\n        }\n    }\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Bar")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "new Bar() inside a set-hook parameter's attribute args should record a reference to Bar"
    );
}

#[test]
fn trait_property_hook_attribute_records_class_reference() {
    // Traits can declare properties with hooks too; check_trait_attributes
    // needed the same hook-walking as check_class_attributes.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Foo {}\n#[Attribute]\nclass Route {\n    public function __construct(public $v = null) {}\n}\ntrait HasName {\n    public string $name {\n        #[Route(new Foo())]\n        get => $this->name;\n    }\n}\n";
    let file = write_file(&dir, "a.php", src);
    let file_arc = pathbuf_to_arc_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let locs: Vec<_> = analyzer
        .reference_locations("cls:Foo")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();

    assert!(
        !locs.is_empty(),
        "new Foo() inside a trait property hook's attribute args should record a reference to Foo"
    );
}
