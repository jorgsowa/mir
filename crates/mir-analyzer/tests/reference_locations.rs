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

    let locs = analyzer.reference_locations("greet");
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
        .reference_locations("greet")
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
        .reference_locations("Svc::run")
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
        .reference_locations("Counter::count")
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
        .reference_locations("Box::val")
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
        !analyzer.reference_locations("Svc::run").is_empty(),
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
        .reference_locations("ping")
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

    let locs = analyzer.reference_locations("Widget");
    assert!(
        locs.iter().any(|(f, ..)| f == &file_arc),
        "new Widget() should record a reference to Widget"
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
            .reference_locations("helper")
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
        .reference_locations("helper")
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
        .reference_locations("Math::sq")
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
            !analyzer.reference_locations("cached_fn").is_empty(),
            "first run should record reference"
        );
    }

    // Second run — file unchanged, cache hit
    {
        let analyzer = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(&cache_dir);
        analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

        let locs = analyzer.reference_locations("cached_fn");
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
        .reference_locations("ping")
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
            .reference_locations("helper")
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
        .reference_locations("helper")
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
        !analyzer.reference_locations("Svc::helper").is_empty(),
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
        .reference_locations("Svc::helper")
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
        .reference_locations("Svc::run")
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
        !analyzer.reference_locations("Widget").is_empty(),
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
        !analyzer.reference_locations("AppEx").is_empty(),
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
        .reference_locations("ErrA")
        .into_iter()
        .filter(|(f, ..)| f == &file_arc)
        .collect();
    let locs_b: Vec<_> = analyzer
        .reference_locations("ErrB")
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
        !analyzer.reference_locations("Router").is_empty(),
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
        !analyzer.reference_locations("Config").is_empty(),
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
        .reference_locations("Service")
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
        .reference_locations("Repo")
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
        .reference_locations("Logger")
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
        !analyzer.reference_locations("Foo::SEP").is_empty(),
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
        !analyzer.reference_locations("Bar::PREFIX").is_empty(),
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
        !analyzer.reference_locations("Base::TAG").is_empty(),
        "parent::TAG should record a reference to Base::TAG"
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
        !analyzer.reference_locations("Config::VERSION").is_empty(),
        "Config::VERSION should record a reference to Config::VERSION"
    );
}
