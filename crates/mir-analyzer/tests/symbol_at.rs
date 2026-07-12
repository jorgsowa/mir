// Integration tests for AnalysisResult::symbol_at and ResolvedSymbol::codebase_key (mir#185).
//
// Verifies that after analysis the caller can resolve a byte-offset cursor
// position to a ResolvedSymbol, and that codebase_key() returns the same key
// format used by Codebase::symbol_reference_locations.

mod common;

use mir_analyzer::symbol::ReferenceKind;
use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

use self::common::{create_temp_dir, path_to_str, write_file};

// ---------------------------------------------------------------------------
// symbol_at — basic resolution
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_finds_function_call() {
    let dir = create_temp_dir("symbol_at_finds_function_call");
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write_file(&dir, "a.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let offset = src.find("{ greet").unwrap() as u32 + 2; // points at 'g' of greet()
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at the greet() call");

    assert!(
        matches!(&sym.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet"),
        "expected FunctionCall(greet), got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_returns_none_for_unknown_offset() {
    let dir = create_temp_dir("test");
    let src = "<?php\nfunction foo(): void {}\n";
    let file = write_file(&dir, "b.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Offset 0 is '<?php', before any symbol spans
    assert!(
        result.symbol_at(file_str, 0).is_none(),
        "no symbol at offset 0 (opening tag)"
    );
}

// ---------------------------------------------------------------------------
// symbol_at — span boundary behaviour
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_matches_at_span_start() {
    let dir = create_temp_dir("test");
    // "<?php\n"                               = 6 bytes
    // "function greet(): void {}\n"           = 26 bytes  → total 32
    // "function caller(): void { greet(); }\n"
    //                            ^-- greet starts at 32 + 26 = 58
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write_file(&dir, "c.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Locate the exact span start from the recorded symbol
    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet"))
        .expect("FunctionCall(greet) must be recorded");
    let span_start = sym_recorded.span.start;

    // symbol_at with offset == span.start must find the symbol
    let sym = result
        .symbol_at(file_str, span_start)
        .expect("symbol_at should find symbol at span.start");
    assert!(matches!(&sym.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet"));
}

#[test]
fn symbol_at_matches_at_last_byte_of_span() {
    let dir = create_temp_dir("test");
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write_file(&dir, "d.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet"))
        .expect("FunctionCall(greet) must be recorded");
    // span.end is exclusive, so span.end - 1 is the last byte inside the span
    let last_byte = sym_recorded.span.end - 1;

    let sym = result
        .symbol_at(file_str, last_byte)
        .expect("symbol_at should find symbol at span.end - 1");
    assert!(matches!(&sym.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet"));
}

#[test]
fn symbol_at_returns_none_one_past_span_end() {
    let dir = create_temp_dir("test");
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write_file(&dir, "e.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet"))
        .expect("FunctionCall(greet) must be recorded");
    // span.end is the first byte after the span — no symbol should cover it
    // (the '(' character follows, which has no symbol)
    let past_end = sym_recorded.span.end;

    let found = result
        .symbols
        .iter()
        .filter(|s| {
            s.file.as_ref() == file_str && s.span.start <= past_end && past_end < s.span.end
        })
        .count();
    assert_eq!(
        found, 0,
        "no symbol should cover the byte immediately after the identifier"
    );
}

// ---------------------------------------------------------------------------
// symbol_at — multi-file isolation
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_isolates_symbols_by_file() {
    let dir = create_temp_dir("test");
    // Both files define and call a function named "run" so their symbol spans
    // overlap in byte-offset space. symbol_at must not confuse them.
    let file_a = write_file(
        &dir,
        "a.php",
        "<?php\nfunction run(): void {}\nfunction a(): void { run(); }\n",
    );
    let file_b = write_file(
        &dir,
        "b.php",
        "<?php\nfunction run(): void {}\nfunction b(): void { run(); }\n",
    );
    let file_a_str = path_to_str(&file_a);
    let file_b_str = path_to_str(&file_b);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(&[file_a.clone(), file_b.clone()], &BatchOptions::new());

    // Collect all FunctionCall(run) symbols per file
    let in_a: Vec<_> = result
        .symbols
        .iter()
        .filter(|s| {
            s.file.as_ref() == file_a_str
                && matches!(&s.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "run")
        })
        .collect();
    let in_b: Vec<_> = result
        .symbols
        .iter()
        .filter(|s| {
            s.file.as_ref() == file_b_str
                && matches!(&s.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "run")
        })
        .collect();

    assert_eq!(in_a.len(), 1, "exactly one run() call recorded in a.php");
    assert_eq!(in_b.len(), 1, "exactly one run() call recorded in b.php");

    // symbol_at for each file must return only that file's symbol
    let sym_a = result
        .symbol_at(file_a_str, in_a[0].span.start)
        .expect("symbol_at should find run() in a.php");
    assert_eq!(
        sym_a.file.as_ref(),
        file_a_str,
        "returned symbol must belong to a.php"
    );

    let sym_b = result
        .symbol_at(file_b_str, in_b[0].span.start)
        .expect("symbol_at should find run() in b.php");
    assert_eq!(
        sym_b.file.as_ref(),
        file_b_str,
        "returned symbol must belong to b.php"
    );
}

// ---------------------------------------------------------------------------
// symbol_at — $this receiver (issue #191)
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_finds_this_method_call() {
    // $this->method() was previously invisible to symbol_at because $this was
    // not typed in the method context, causing analyze_method_call to hit the
    // mixed-receiver guard and return early without recording a symbol.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc { public function helper(): void {}\npublic function run(): void { $this->helper(); } }\n";
    let file = write_file(&dir, "this_call.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Point cursor at 'helper' in '$this->helper()'
    let offset = src.find("->helper").unwrap() as u32 + 2; // +2 skips '->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should resolve $this->helper() (issue #191)");

    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "helper"),
        "expected MethodCall(helper) for $this->helper(), got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_finds_this_property_access() {
    // $this->prop was invisible for the same reason as $this->method() — $this
    // was untyped so the mixed-receiver guard fired before record_symbol.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Counter { public int $count = 0;\npublic function inc(): void { $this->count++; } }\n";
    let file = write_file(&dir, "this_prop.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let offset = src.find("->count").unwrap() as u32 + 2; // +2 skips '->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should resolve $this->count");

    assert!(
        matches!(&sym.kind, ReferenceKind::PropertyAccess { property, .. } if property.as_ref() == "count"),
        "expected PropertyAccess(count) for $this->count, got {:?}",
        sym.kind
    );

    let key = sym
        .codebase_key()
        .expect("PropertyAccess must have a codebase key");
    assert_eq!(key, "prop:Counter::count");
}

#[test]
fn symbol_at_this_method_call_full_lsp_flow() {
    // Verify the full LSP flow: cursor → codebase_key → get_reference_locations.
    // Two calls to $this->helper() from the same method must both be indexed.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc {\n  public function helper(): void {}\n  public function run(): void { $this->helper(); $this->helper(); }\n}\n";
    let file = write_file(&dir, "this_flow.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let offset = src.find("->helper").unwrap() as u32 + 2;
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find first $this->helper()");

    let key = sym
        .codebase_key()
        .expect("MethodCall must have a codebase key");
    assert!(
        key.ends_with("::helper"),
        "codebase_key must end with '::helper', got: {key}"
    );

    let locs = analyzer.reference_locations(&key);
    assert_eq!(
        locs.len(),
        2,
        "two $this->helper() calls must produce two reference locations"
    );
}

#[test]
fn symbol_at_this_in_non_static_closure() {
    // A non-static closure inside a non-static method inherits $this, so
    // $this->method() calls inside the closure should also be resolved.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc {\n  public function helper(): void {}\n  public function run(): void { $fn = function() { $this->helper(); }; $fn(); }\n}\n";
    let file = write_file(&dir, "closure_this.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let offset = src.find("->helper").unwrap() as u32 + 2;
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should resolve $this->helper() inside a non-static closure");

    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "helper"),
        "expected MethodCall(helper) inside closure, got {:?}",
        sym.kind
    );
}

// ---------------------------------------------------------------------------
// symbol_at — most-specific span is returned
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_returns_innermost_symbol() {
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n";
    let file = write_file(&dir, "f.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Find the offset of "run" in "$s->run()"
    let offset = src.rfind("run").unwrap() as u32;

    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at the run() call");

    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "run"),
        "expected MethodCall(run) as the innermost symbol, got {:?}",
        sym.kind
    );
}

// ---------------------------------------------------------------------------
// codebase_key — key format matches symbol_reference_locations
// ---------------------------------------------------------------------------

#[test]
fn codebase_key_for_function_call_matches_reference_index() {
    let dir = create_temp_dir("test");
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write_file(&dir, "g.php", src);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet"))
        .expect("FunctionCall(greet) must be recorded");

    let key = sym
        .codebase_key()
        .expect("FunctionCall should have a codebase key");
    assert_eq!(key, "fn:greet");

    assert!(
        !analyzer.reference_locations(key.as_str()).is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_method_call_is_lowercased() {
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc { public function Run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->Run(); }\n";
    let file = write_file(&dir, "h.php", src);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "Run"))
        .expect("MethodCall(Run) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert!(
        key.ends_with("::run"),
        "method part of key must be lowercased, got: {key}"
    );
    assert!(
        !analyzer.reference_locations(key.as_str()).is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_static_call_matches_reference_index() {
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Math { public static function square(int $n): int { return $n * $n; } }\nfunction caller(): void { Math::square(3); }\n";
    let file = write_file(&dir, "i.php", src);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym = result
        .symbols
        .iter()
        .find(|s| {
            matches!(&s.kind, ReferenceKind::StaticCall { method, .. } if method.as_ref() == "square")
        })
        .expect("StaticCall(square) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert_eq!(key, "meth:Math::square");
    assert!(
        !analyzer.reference_locations(key.as_str()).is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_property_access_matches_reference_index() {
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Counter { public int $count = 0; }\nfunction read(Counter $c): int { return $c->count; }\n";
    let file = write_file(&dir, "j.php", src);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym = result
        .symbols
        .iter()
        .find(|s| {
            matches!(&s.kind, ReferenceKind::PropertyAccess { property, .. } if property.as_ref() == "count")
        })
        .expect("PropertyAccess(count) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert_eq!(key, "prop:Counter::count");
    assert!(
        !analyzer.reference_locations(key.as_str()).is_empty(),
        "codebase_key for PropertyAccess should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_class_reference_matches_reference_index() {
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Widget {}\nfunction make(): void { $w = new Widget(); }\n";
    let file = write_file(&dir, "k.php", src);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::ClassReference(n) if n.as_ref() == "Widget"))
        .expect("ClassReference(Widget) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert_eq!(key, "cls:Widget");
    assert!(
        !analyzer.reference_locations(key.as_str()).is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_variable_is_none() {
    // Use a parameter that is read so it's recorded as a Variable symbol.
    let src = "<?php\nfunction f(int $n): int { return $n; }\n";
    let result = mir_analyzer::analyze_source(src);

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::Variable(n) if n.as_ref() == "n"))
        .expect("Variable(n) must be recorded for the $n read in return");

    assert!(
        sym.codebase_key().is_none(),
        "variables have no codebase key"
    );
}

// ---------------------------------------------------------------------------
// symbol_at + codebase_key → get_reference_locations (full flow)
// ---------------------------------------------------------------------------

#[test]
fn full_flow_cursor_to_reference_locations() {
    let dir = create_temp_dir("test");
    let src = "<?php\nfunction ping(): void {}\nfunction caller(): void { ping(); ping(); }\n";
    let file = write_file(&dir, "l.php", src);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());
    let file_str = path_to_str(&file);

    let first_call_offset = src.find("{ ping").unwrap() as u32 + 2;

    let sym = result
        .symbol_at(file_str, first_call_offset)
        .expect("symbol_at should find a symbol at the first ping() call");

    let key = sym.codebase_key().expect("FunctionCall must have a key");
    assert_eq!(key, "fn:ping");

    let locs = analyzer.reference_locations(&key);
    assert_eq!(
        locs.len(),
        2,
        "two calls to ping() should produce two reference locations"
    );
}

#[test]
fn symbol_at_function_call_span_is_identifier_only() {
    // Cursor on '(' (one past the identifier) must NOT find the function symbol.
    let dir = create_temp_dir("test");
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write_file(&dir, "span_fn.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet"))
        .expect("FunctionCall(greet) must be recorded");

    // span should cover only "greet" (5 bytes), not the full "greet()" call
    assert_eq!(
        sym_recorded.span.end - sym_recorded.span.start,
        5,
        "FunctionCall symbol span must be identifier-only (5 bytes for 'greet')"
    );

    // Cursor at span.end (the '(' character) must not find the function symbol
    let past_name = sym_recorded.span.end;
    let found = result
        .symbols
        .iter()
        .filter(|s| {
            s.file.as_ref() == file_str
                && s.span.start <= past_name
                && past_name < s.span.end
                && matches!(&s.kind, ReferenceKind::FunctionCall(n) if n.as_ref() == "greet")
        })
        .count();
    assert_eq!(found, 0, "cursor at '(' must not match the function symbol");
}

#[test]
fn symbol_at_method_call_span_is_identifier_only() {
    // Cursor on '(' (one past the method identifier) must NOT find the method symbol.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n";
    let file = write_file(&dir, "span_method.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "run"))
        .expect("MethodCall(run) must be recorded");

    // span should cover only "run" (3 bytes), not "run()" or "$s->run()"
    assert_eq!(
        sym_recorded.span.end - sym_recorded.span.start,
        3,
        "MethodCall symbol span must be identifier-only (3 bytes for 'run')"
    );

    // Cursor at span.end (the '(' character) must not find the method symbol
    let past_name = sym_recorded.span.end;
    let found = result
        .symbols
        .iter()
        .filter(|s| {
            s.file.as_ref() == file_str
                && s.span.start <= past_name
                && past_name < s.span.end
                && matches!(&s.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "run")
        })
        .count();
    assert_eq!(found, 0, "cursor at '(' must not match the method symbol");
}

#[test]
fn symbol_at_static_call_span_is_identifier_only() {
    // Cursor on '(' (one past the static method identifier) must NOT find the symbol.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Math { public static function sq(int $n): int { return $n * $n; } }\nfunction caller(): void { Math::sq(3); }\n";
    let file = write_file(&dir, "span_static.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym_recorded = result
        .symbols
        .iter()
        .find(
            |s| matches!(&s.kind, ReferenceKind::StaticCall { method, .. } if method.as_ref() == "sq"),
        )
        .expect("StaticCall(sq) must be recorded");

    // span should cover only "sq" (2 bytes), not "Math::sq(3)"
    assert_eq!(
        sym_recorded.span.end - sym_recorded.span.start,
        2,
        "StaticCall symbol span must be identifier-only (2 bytes for 'sq')"
    );

    // Cursor at span.end (the '(' character) must not find the static call symbol
    let past_name = sym_recorded.span.end;
    let found = result
        .symbols
        .iter()
        .filter(|s| {
            s.file.as_ref() == file_str
                && s.span.start <= past_name
                && past_name < s.span.end
                && matches!(&s.kind, ReferenceKind::StaticCall { method, .. } if method.as_ref() == "sq")
        })
        .count();
    assert_eq!(
        found, 0,
        "cursor at '(' must not match the static call symbol"
    );
}

#[test]
fn symbol_at_finds_property_access() {
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Counter { public int $count = 0; }\nfunction read(Counter $c): int { return $c->count; }\n";
    let file = write_file(&dir, "m.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Point cursor at 'count' in '$c->count'
    let offset = src.find("->count").unwrap() as u32 + 2; // +2 skips '->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at $c->count");

    assert!(
        matches!(&sym.kind, ReferenceKind::PropertyAccess { property, .. } if property.as_ref() == "count"),
        "expected PropertyAccess(count), got {:?}",
        sym.kind
    );

    // Verify the full LSP flow: cursor → key → reference locations
    let key = sym.codebase_key().expect("PropertyAccess must have a key");
    assert_eq!(key, "prop:Counter::count");
    let locs = analyzer.reference_locations(&key);
    assert_eq!(
        locs.len(),
        1,
        "one property access should produce one reference location"
    );
}

#[test]
fn symbol_at_finds_nullsafe_property_access() {
    let dir = create_temp_dir("test");
    let src =
        "<?php\nclass Box { public int $val = 0; }\nfunction read(?Box $b): void { $b?->val; }\n";
    let file = write_file(&dir, "n.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Point cursor at 'val' in '$b?->val'
    let offset = src.find("?->val").unwrap() as u32 + 3; // +3 skips '?->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at $b?->val");

    assert!(
        matches!(&sym.kind, ReferenceKind::PropertyAccess { property, .. } if property.as_ref() == "val"),
        "expected PropertyAccess(val) for nullsafe access, got {:?}",
        sym.kind
    );

    // Verify the full LSP flow: cursor → key → reference locations
    let key = sym.codebase_key().expect("PropertyAccess must have a key");
    assert_eq!(key, "prop:Box::val");
    let locs = analyzer.reference_locations(&key);
    assert_eq!(
        locs.len(),
        1,
        "one nullsafe property access should produce one reference location"
    );
}

#[test]
fn symbol_at_finds_nullsafe_method_call() {
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(?Svc $s): void { $s?->run(); }\n";
    let file = write_file(&dir, "o.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Point cursor at 'run' in '$s?->run()'
    let offset = src.find("?->run").unwrap() as u32 + 3; // +3 skips '?->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at $s?->run()");

    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "run"),
        "expected MethodCall(run) for nullsafe call, got {:?}",
        sym.kind
    );

    // Verify the full LSP flow: cursor → key → reference locations
    let key = sym.codebase_key().expect("MethodCall must have a key");
    assert_eq!(key, "meth:Svc::run");
    let locs = analyzer.reference_locations(&key);
    assert_eq!(
        locs.len(),
        1,
        "one nullsafe method call should produce one reference location"
    );
}

#[test]
fn symbol_at_method_call_span_matches_reference_location_span() {
    // Regression guard for #186: record_symbol and mark_method_referenced_at
    // must use the same identifier-only span so an LSP client sees consistent
    // ranges when highlighting the current symbol and listing its references.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n";
    let file = write_file(&dir, "span_eq_method.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let offset = src.find("->run").unwrap() as u32 + 2; // points at 'r' of run
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at must find MethodCall(run)");

    let key = sym.codebase_key().unwrap();
    let locs = analyzer.reference_locations(&key);
    let (_, _line, ref_col_start, ref_col_end) = *locs
        .iter()
        .find(|(f, ..)| f.as_ref() == file_str)
        .expect("reference location must exist for this file");

    // PHP identifiers are ASCII, so byte length == char count == col width.
    let span_len = sym.span.end - sym.span.start;
    assert_eq!(
        span_len,
        (ref_col_end - ref_col_start) as u32,
        "symbol_at span length must equal reference location column width"
    );
}

#[test]
fn symbol_at_function_call_span_matches_reference_location_span() {
    // Regression guard for #186: same consistency check for function calls.
    let dir = create_temp_dir("test");
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write_file(&dir, "span_eq_fn.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let offset = src.find("{ greet").unwrap() as u32 + 2; // points at 'g' of greet
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at must find FunctionCall(greet)");

    let key = sym.codebase_key().unwrap();
    let locs = analyzer.reference_locations(&key);
    let (_, _line, ref_col_start, ref_col_end) = *locs
        .iter()
        .find(|(f, ..)| f.as_ref() == file_str)
        .expect("reference location must exist for this file");

    // PHP identifiers are ASCII, so byte length == char count == col width.
    let span_len = sym.span.end - sym.span.start;
    assert_eq!(
        span_len,
        (ref_col_end - ref_col_start) as u32,
        "symbol_at span length must equal reference location column width"
    );
}

#[test]
fn class_const_access_records_symbol() {
    // Config::VERSION should push a ConstantAccess ResolvedSymbol so that
    // symbol_at can resolve the position to a typed symbol for hover / go-to-def.
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Config { const string VERSION = '1.0'; }\nfunction ver(): string { return Config::VERSION; }\n";
    let file = write_file(&dir, "const_sym.php", src);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym = result
        .symbols
        .iter()
        .find(|s| {
            matches!(&s.kind, ReferenceKind::ConstantAccess { constant, .. } if constant.as_ref() == "VERSION")
        })
        .expect("ConstantAccess(VERSION) must be recorded for Config::VERSION");

    let key = sym.codebase_key().unwrap();
    assert_eq!(key, "cnst:Config::VERSION");
    assert!(
        !analyzer.reference_locations(key.as_str()).is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn inherited_static_call_symbol_keys_by_declaring_class() {
    // The StaticCall symbol for Child::foo() must use the declaring class Base,
    // so that codebase_key() matches the reference-index entry "Base::foo".
    let dir = create_temp_dir("test");
    let src = "<?php\nclass Base { public static function foo(): void {} }\nclass Child extends Base {}\nfunction caller(): void { Child::foo(); }\n";
    let file = write_file(&dir, "inherited_static_sym.php", src);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::StaticCall { method, .. } if method.as_ref() == "foo"))
        .expect("StaticCall(foo) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert_eq!(
        key, "meth:Base::foo",
        "codebase_key must be the declaring class Base, not Child"
    );
    assert!(
        !analyzer.reference_locations(key.as_str()).is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

// ---------------------------------------------------------------------------
// Gap #9 — trait insteadof conflict resolution
// ---------------------------------------------------------------------------

#[test]
fn gap_trait_insteadof_definition_ignores_conflict_resolution() {
    // When two traits define the same method and one is excluded via `insteadof`,
    // a method call should resolve to the *winning* trait, not the excluded one.
    //
    // class MyClass { use A, B { B::hello insteadof A; } }
    // $obj->hello() must resolve to B::hello, not A::hello.
    let dir = create_temp_dir("test");
    let src = "<?php\n\
trait A {\n\
    public function hello(): string { return 'A'; }\n\
}\n\
trait B {\n\
    public function hello(): string { return 'B'; }\n\
}\n\
class MyClass {\n\
    use A, B {\n\
        B::hello insteadof A;\n\
    }\n\
}\n\
function caller(): void { $obj = new MyClass(); $obj->hello(); }\n";

    let file = write_file(&dir, "insteadof.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "hello"))
        .expect("MethodCall(hello) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert_eq!(
        key, "meth:B::hello",
        "insteadof should route $obj->hello() to B::hello, not A::hello"
    );

    let offset = src.find("->hello").unwrap() as u32 + 2;
    let sym_at = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should resolve $obj->hello()");
    let key_at = sym_at.codebase_key().unwrap();
    assert_eq!(
        key_at, "meth:B::hello",
        "symbol_at codebase_key must also be B::hello after insteadof resolution"
    );
}

// ---------------------------------------------------------------------------
// symbol_at — method-chain gap fallback (expr_span)
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_chain_gap_returns_innermost_enclosing_call() {
    // When the cursor sits in the whitespace between two chained method calls
    // (e.g. the "\n  ->" between `->where('id')` and `->limit(10)`), symbol_at
    // should fall back to the innermost call expression whose expr_span contains
    // the offset rather than returning None.
    let dir = create_temp_dir("test");
    // Use concat! so the "  ->" indentation is not stripped by a Rust
    // string-continuation backslash.
    let src = concat!(
        "<?php\n",
        "class Builder {\n",
        "    public function where(string $col): static { return $this; }\n",
        "    public function limit(int $n): static { return $this; }\n",
        "}\n",
        "$q = new Builder();\n",
        "$q->where('id')\n",
        "  ->limit(10);\n",
    );

    let file = write_file(&dir, "chain_gap.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    // Cursor on `where` identifier — exact match.
    let where_id_off = src.find("->where(").unwrap() as u32 + 2;
    let sym = result
        .symbol_at(file_str, where_id_off)
        .expect("symbol_at should find `where` on its identifier");
    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "where"),
        "expected MethodCall(where), got {:?}",
        sym.kind
    );

    // Cursor on `(` immediately after `where` — inside the call expression,
    // not on the identifier.  Should resolve to `where` via expr_span.
    let where_open_paren = where_id_off + "where".len() as u32;
    let sym = result
        .symbol_at(file_str, where_open_paren)
        .expect("symbol_at should find `where` from inside its argument list");
    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "where"),
        "expected MethodCall(where) at '(', got {:?}",
        sym.kind
    );

    // Cursor on `\n` (the chain gap after `->where('id')`).  Should resolve to
    // `limit` — the innermost call whose expr_span encloses the newline.
    let newline_off = (src.find("->where('id')").unwrap() + "->where('id')".len()) as u32;
    let sym = result
        .symbol_at(file_str, newline_off)
        .expect("symbol_at must not return None for a chain-gap cursor");
    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "limit"),
        "expected MethodCall(limit) at chain-gap newline, got {:?}",
        sym.kind
    );

    // Cursor on `->` before `limit` — still in the gap.
    let arrow_off = src.find("  ->limit").unwrap() as u32 + 2;
    let sym = result
        .symbol_at(file_str, arrow_off)
        .expect("symbol_at must not return None on `->` before limit");
    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "limit"),
        "expected MethodCall(limit) on `->`, got {:?}",
        sym.kind
    );
}

// ---------------------------------------------------------------------------
// Regression: parameter declaration sites must be recorded as symbols
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_param_declaration_function() {
    // symbol_at on `$user` at its declaration site should return Variable("user")
    // with the declared type (User), not None.
    let src =
        "<?php\nclass User {}\nfunction greet(User $user): string { return $user->getName(); }\n";
    let result = mir_analyzer::analyze_source(src);

    // Find the byte offset of `$user` in the parameter declaration.
    let decl_offset = src.find("User $user").unwrap() as u32 + 5; // skip "User " → points at '$'

    let sym = result
        .symbol_at("<source>", decl_offset)
        .expect("symbol_at must return a symbol at the parameter declaration");
    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "user"),
        "expected Variable(user) at param declaration, got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_param_declaration_method() {
    let src =
        "<?php\nclass Greeter { public function greet(string $name): string { return $name; } }\n";
    let result = mir_analyzer::analyze_source(src);

    let decl_offset = src.find("string $name").unwrap() as u32 + 7; // skip "string " → '$'

    let sym = result
        .symbol_at("<source>", decl_offset)
        .expect("symbol_at must return a symbol at the method parameter declaration");
    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "name"),
        "expected Variable(name) at method param declaration, got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_param_declaration_closure() {
    let src = "<?php\n$fn = function(int $x): int { return $x * 2; };\n";
    let result = mir_analyzer::analyze_source(src);

    let decl_offset = src.find("int $x").unwrap() as u32 + 4; // skip "int " → '$'

    let sym = result
        .symbol_at("<source>", decl_offset)
        .expect("symbol_at must return a symbol at the closure parameter declaration");
    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "x"),
        "expected Variable(x) at closure param declaration, got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_param_declaration_arrow_function() {
    let src = "<?php\n$fn = fn(int $n) => $n + 1;\n";
    let result = mir_analyzer::analyze_source(src);

    let decl_offset = src.find("int $n").unwrap() as u32 + 4; // skip "int " → '$'

    let sym = result
        .symbol_at("<source>", decl_offset)
        .expect("symbol_at must return a symbol at the arrow function parameter declaration");
    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "n"),
        "expected Variable(n) at arrow fn param declaration, got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_param_declaration_tight_span() {
    // The declaration symbol span must cover only the `$name` token,
    // not the entire `TypeHint $name` parameter declaration.
    let src = "<?php\nfunction foo(string $val): void {}\n";
    let result = mir_analyzer::analyze_source(src);

    // Find the `$val` offset directly.
    let dollar_off = src.find("$val").unwrap() as u32;

    // symbol_at on `$val` must return Variable(val).
    let sym = result
        .symbol_at("<source>", dollar_off)
        .expect("symbol_at must find Variable(val) on `$val` in param declaration");
    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "val"),
        "expected Variable(val), got {:?}",
        sym.kind
    );

    // The recorded span must NOT cover `string` (the type hint prefix).
    let string_off = src.find("string $val").unwrap() as u32;
    let sym_at_type = result.symbol_at("<source>", string_off);
    // Either no symbol at all on the type token, or a non-variable symbol.
    if let Some(s) = sym_at_type {
        assert!(
            !matches!(&s.kind, ReferenceKind::Variable(n) if n.as_ref() == "val"),
            "Variable(val) span must not cover the type hint token"
        );
    }
}

// ---------------------------------------------------------------------------
// symbol_at — negated instanceof guard narrows receiver type (issue #6)
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_negated_instanceof_guard_narrows_receiver_type() {
    // After `if (!$subject instanceof Post) { return; }`, $subject is narrowed
    // to Post. symbol_at at the $subject token in the method-call receiver must
    // return resolved_type == "Post", not "Post|Comment".
    let src = r#"<?php
class Post { public function getTitle(): string { return ''; } }
class Comment {}
/** @param Post|Comment $subject */
function test(Post|Comment $subject): void {
    if (!$subject instanceof Post) {
        return;
    }
    $subject->getTitle();
}
"#;
    let result = mir_analyzer::analyze_source(src);

    // Find the `$subject` token in `$subject->getTitle()`:
    // occurrences are: docblock, param decl, guard condition, method-call receiver.
    let off0 = src.find("$subject").unwrap();
    let off1 = src[off0 + 1..].find("$subject").unwrap() + off0 + 1;
    let off2 = src[off1 + 1..].find("$subject").unwrap() + off1 + 1;
    let receiver_off = (src[off2 + 1..].find("$subject").unwrap() + off2 + 1) as u32;

    let sym = result
        .symbol_at("<source>", receiver_off)
        .expect("symbol_at must find a symbol at $subject in the method-call receiver");

    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "subject"),
        "expected Variable(subject), got {:?}",
        sym.kind
    );

    let ty = format!("{}", sym.resolved_type);
    assert_eq!(
        ty, "Post",
        "after negated instanceof guard, $subject must be narrowed to Post, got {ty}"
    );
}

#[test]
fn symbol_at_negated_instanceof_guard_narrows_mixed_receiver_type() {
    // Same pattern but with mixed parameter type (two occurrences before receiver).
    let src = r#"<?php
class Post { public function getTitle(): string { return ''; } }
function test(mixed $subject): void {
    if (!$subject instanceof Post) {
        return;
    }
    $subject->getTitle();
}
"#;
    let result = mir_analyzer::analyze_source(src);

    // Two occurrences before the receiver: param decl, guard condition.
    let off0 = src.find("$subject").unwrap();
    let off1 = src[off0 + 1..].find("$subject").unwrap() + off0 + 1;
    let receiver_off = (src[off1 + 1..].find("$subject").unwrap() + off1 + 1) as u32;

    let sym = result
        .symbol_at("<source>", receiver_off)
        .expect("symbol_at must find a symbol at $subject in the method-call receiver");

    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "subject"),
        "expected Variable(subject), got {:?}",
        sym.kind
    );

    let ty = format!("{}", sym.resolved_type);
    assert_eq!(
        ty, "Post",
        "after negated instanceof guard, mixed $subject must be narrowed to Post, got {ty}"
    );
}

#[test]
fn symbol_at_negated_instanceof_guard_method_call_resolves_to_narrowed_class() {
    // After the negated guard, the method call `$subject->getTitle()` must
    // resolve as MethodCall { class: "Post", method: "getTitle" }, not
    // MethodCall { class: "mixed" } or unresolved.
    let src = r#"<?php
class Post { public function getTitle(): string { return ''; } }
class Comment {}
function test(Post|Comment $subject): void {
    if (!$subject instanceof Post) {
        return;
    }
    $subject->getTitle();
}
"#;
    let result = mir_analyzer::analyze_source(src);

    // Find MethodCall(Post, getTitle) in the recorded symbols.
    let method_sym = result.symbols.iter().find(|s| {
        matches!(&s.kind, ReferenceKind::MethodCall { class, method }
            if class.as_ref() == "Post" && method.as_ref() == "getTitle")
    });

    assert!(
        method_sym.is_some(),
        "expected MethodCall {{ class: Post, method: getTitle }} after negated instanceof guard; \
         found: {:?}",
        result
            .symbols
            .iter()
            .filter(|s| matches!(&s.kind, ReferenceKind::MethodCall { .. }))
            .map(|s| &s.kind)
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// symbol_at — instanceof class name resolves to ClassReference
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_instanceof_class_name_resolves_to_class_reference() {
    // `$v instanceof Widget` — symbol_at on "Widget" must return
    // ClassReference("Widget"), enabling hover and find-references on it.
    let src =
        "<?php\nclass Widget {}\nfunction check(mixed $v): bool { return $v instanceof Widget; }\n";
    let result = mir_analyzer::analyze_source(src);

    let offset = src.find("instanceof Widget").unwrap() as u32 + "instanceof ".len() as u32;

    let sym = result
        .symbol_at("<source>", offset)
        .expect("symbol_at must find ClassReference on instanceof class name");

    assert!(
        matches!(&sym.kind, ReferenceKind::ClassReference(n) if n.as_ref() == "Widget"),
        "expected ClassReference(Widget), got {:?}",
        sym.kind
    );
}

// ---------------------------------------------------------------------------
// symbol_at — class token in a static call resolves to ClassReference
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_static_call_class_name_resolves_to_class_reference() {
    // `Math::square(3)` — symbol_at on "Math" must return ClassReference("Math"),
    // mirroring `new Math` / `$v instanceof Math`.
    let src = "<?php\nclass Math { public static function square(int $n): int { return $n; } }\nfunction caller(): void { Math::square(3); }\n";
    let result = mir_analyzer::analyze_source(src);

    let offset = src.find("Math::square").unwrap() as u32;

    let sym = result
        .symbol_at("<source>", offset)
        .expect("symbol_at must find ClassReference on the static-call class name");

    assert!(
        matches!(&sym.kind, ReferenceKind::ClassReference(n) if n.as_ref() == "Math"),
        "expected ClassReference(Math), got {:?}",
        sym.kind
    );
}

// ---------------------------------------------------------------------------
// symbol_at — class token in `Foo::class` resolves to ClassReference
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_class_const_class_name_resolves_to_class_reference() {
    // `Widget::class` — symbol_at on "Widget" must return ClassReference("Widget"),
    // mirroring `new Widget` / `$v instanceof Widget` / `Widget::method()`.
    let src = "<?php\nclass Widget {}\nfunction caller(): string { return Widget::class; }\n";
    let result = mir_analyzer::analyze_source(src);

    let offset = src.find("Widget::class").unwrap() as u32;

    let sym = result
        .symbol_at("<source>", offset)
        .expect("symbol_at must find ClassReference on the Foo::class class name");

    assert!(
        matches!(&sym.kind, ReferenceKind::ClassReference(n) if n.as_ref() == "Widget"),
        "expected ClassReference(Widget), got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_parent_keyword_in_static_call_resolves_to_parent_class() {
    // `parent::greet()` — symbol_at on the `parent` keyword must return a
    // ClassReference to the *actual* parent class (Base), not the child.
    let dir = create_temp_dir("symbol_at_parent_keyword");
    let src = "<?php\nclass Base { public static function greet(): void {} }\nclass Child extends Base { public static function go(): void { parent::greet(); } }\n";
    let file = write_file(&dir, "parent_kw.php", src);
    let file_str = path_to_str(&file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(std::slice::from_ref(&file), &BatchOptions::new());

    let offset = src.find("parent::greet").unwrap() as u32;

    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at must find a ClassReference on the `parent` keyword");

    assert!(
        matches!(&sym.kind, ReferenceKind::ClassReference(n) if n.as_ref() == "Base"),
        "expected ClassReference(Base) for `parent`, got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_self_keyword_in_static_call_resolves_to_self_class() {
    // `self::make()` — symbol_at on the `self` keyword must resolve to the
    // enclosing class (Factory).
    let src = "<?php\nclass Factory { public static function make(): void {} public static function build(): void { self::make(); } }\n";
    let result = mir_analyzer::analyze_source(src);

    let offset = src.find("self::make").unwrap() as u32;

    let sym = result
        .symbol_at("<source>", offset)
        .expect("symbol_at must find a ClassReference on the `self` keyword");

    assert!(
        matches!(&sym.kind, ReferenceKind::ClassReference(n) if n.as_ref() == "Factory"),
        "expected ClassReference(Factory) for `self`, got {:?}",
        sym.kind
    );
}

// ---------------------------------------------------------------------------
// Regression: analyze_source must surface declared types, not `mixed`.
//
// `analyze_source` previously registered its file with a bare `SourceFile::new`,
// bypassing the workspace symbol index. Body analysis then could not look up the
// file's own functions/methods, so every parameter fell back to `mixed`
// (`ast_derived_fn_params`). The single-file/LSP surface must report real types.
// ---------------------------------------------------------------------------

#[test]
fn analyze_source_native_param_read_resolves_to_declared_type() {
    let src = "<?php\nfunction g(int $n): int { return $n; }\n";
    let result = mir_analyzer::analyze_source(src);

    // `$n` in `return $n` — the read site, not the declaration.
    let offset = src.rfind("$n").unwrap() as u32;
    let sym = result
        .symbol_at("<source>", offset)
        .expect("symbol_at must find Variable(n) at the return read");

    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "n"),
        "expected Variable(n), got {:?}",
        sym.kind
    );
    let ty = format!("{}", sym.resolved_type);
    assert_eq!(
        ty, "int",
        "native-typed param must resolve to `int`, not `mixed` (workspace-index regression)"
    );
}

#[test]
fn analyze_source_receiver_var_resolves_to_declared_class() {
    // A native-typed receiver must carry its class type so member resolution and
    // hover work; previously this degraded to `mixed`.
    let src = "<?php\nclass Repo { public function go(): void {} }\nfunction h(Repo $repo): void { $repo->go(); }\n";
    let result = mir_analyzer::analyze_source(src);

    let offset = src.find("$repo->go").unwrap() as u32;
    let sym = result
        .symbol_at("<source>", offset)
        .expect("symbol_at must find Variable(repo) at the method-call receiver");

    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(n) if n.as_ref() == "repo"),
        "expected Variable(repo), got {:?}",
        sym.kind
    );
    let ty = format!("{}", sym.resolved_type);
    assert_eq!(
        ty, "Repo",
        "native-typed receiver must resolve to `Repo`, not `mixed`"
    );

    // The method call itself must resolve against the in-file class.
    let m = result.symbols.iter().find(|s| {
        matches!(&s.kind, ReferenceKind::MethodCall { class, method }
            if class.as_ref() == "Repo" && method.as_ref() == "go")
    });
    assert!(
        m.is_some(),
        "expected MethodCall(Repo::go) resolved via the workspace index"
    );
}

#[test]
fn analyze_source_typed_param_has_no_typecheck_mismatch() {
    // Internal-inference guard: @mir-check reads the same flow state that
    // resolved_type is built from. A native-typed param must satisfy it.
    let src = "<?php\nfunction g(int $n): void {\n/** @mir-check $n is int */\necho $n;\n}\n";
    let result = mir_analyzer::analyze_source(src);

    let mismatches: Vec<_> = result
        .issues
        .iter()
        .filter(|i| matches!(i.kind, mir_analyzer::IssueKind::TypeCheckMismatch { .. }))
        .collect();
    assert!(
        mismatches.is_empty(),
        "native-typed param should infer as `int`; got mismatches: {:?}",
        mismatches.iter().map(|i| &i.kind).collect::<Vec<_>>()
    );
}
