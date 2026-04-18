// Integration tests for AnalysisResult::symbol_at and ResolvedSymbol::codebase_key (mir#185).
//
// Verifies that after analysis the caller can resolve a byte-offset cursor
// position to a ResolvedSymbol, and that codebase_key() returns the same key
// format used by Codebase::symbol_reference_locations.

use std::fs;
use std::path::PathBuf;

use mir_analyzer::symbol::SymbolKind;
use mir_analyzer::ProjectAnalyzer;
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

// ---------------------------------------------------------------------------
// symbol_at — basic resolution
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_finds_function_call() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "a.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let offset = src.find("{ greet").unwrap() as u32 + 2; // points at 'g' of greet()
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at the greet() call");

    assert!(
        matches!(&sym.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"),
        "expected FunctionCall(greet), got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_returns_none_for_unknown_offset() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction foo(): void {}\n";
    let file = write(&dir, "b.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

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
    let dir = TempDir::new().unwrap();
    // "<?php\n"                               = 6 bytes
    // "function greet(): void {}\n"           = 26 bytes  → total 32
    // "function caller(): void { greet(); }\n"
    //                            ^-- greet starts at 32 + 26 = 58
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "c.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    // Locate the exact span start from the recorded symbol
    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"))
        .expect("FunctionCall(greet) must be recorded");
    let span_start = sym_recorded.span.start;

    // symbol_at with offset == span.start must find the symbol
    let sym = result
        .symbol_at(file_str, span_start)
        .expect("symbol_at should find symbol at span.start");
    assert!(matches!(&sym.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"));
}

#[test]
fn symbol_at_matches_at_last_byte_of_span() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "d.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"))
        .expect("FunctionCall(greet) must be recorded");
    // span.end is exclusive, so span.end - 1 is the last byte inside the span
    let last_byte = sym_recorded.span.end - 1;

    let sym = result
        .symbol_at(file_str, last_byte)
        .expect("symbol_at should find symbol at span.end - 1");
    assert!(matches!(&sym.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"));
}

#[test]
fn symbol_at_returns_none_one_past_span_end() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "e.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"))
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
    let dir = TempDir::new().unwrap();
    // Both files define and call a function named "run" so their symbol spans
    // overlap in byte-offset space. symbol_at must not confuse them.
    let file_a = write(
        &dir,
        "a.php",
        "<?php\nfunction run(): void {}\nfunction a(): void { run(); }\n",
    );
    let file_b = write(
        &dir,
        "b.php",
        "<?php\nfunction run(): void {}\nfunction b(): void { run(); }\n",
    );
    let file_a_str = file_a.to_str().unwrap();
    let file_b_str = file_b.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(&[file_a.clone(), file_b.clone()]);

    // Collect all FunctionCall(run) symbols per file
    let in_a: Vec<_> = result
        .symbols
        .iter()
        .filter(|s| {
            s.file.as_ref() == file_a_str
                && matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "run")
        })
        .collect();
    let in_b: Vec<_> = result
        .symbols
        .iter()
        .filter(|s| {
            s.file.as_ref() == file_b_str
                && matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "run")
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
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Svc { public function helper(): void {}\npublic function run(): void { $this->helper(); } }\n";
    let file = write(&dir, "this_call.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    // Point cursor at 'helper' in '$this->helper()'
    let offset = src.find("->helper").unwrap() as u32 + 2; // +2 skips '->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should resolve $this->helper() (issue #191)");

    assert!(
        matches!(&sym.kind, SymbolKind::MethodCall { method, .. } if method.as_ref() == "helper"),
        "expected MethodCall(helper) for $this->helper(), got {:?}",
        sym.kind
    );
}

#[test]
fn symbol_at_finds_this_property_access() {
    // $this->prop was invisible for the same reason as $this->method() — $this
    // was untyped so the mixed-receiver guard fired before record_symbol.
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Counter { public int $count = 0;\npublic function inc(): void { $this->count++; } }\n";
    let file = write(&dir, "this_prop.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let offset = src.find("->count").unwrap() as u32 + 2; // +2 skips '->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should resolve $this->count");

    assert!(
        matches!(&sym.kind, SymbolKind::PropertyAccess { property, .. } if property.as_ref() == "count"),
        "expected PropertyAccess(count) for $this->count, got {:?}",
        sym.kind
    );

    let key = sym
        .codebase_key()
        .expect("PropertyAccess must have a codebase key");
    assert_eq!(key, "Counter::count");
}

#[test]
fn symbol_at_this_method_call_full_lsp_flow() {
    // Verify the full LSP flow: cursor → codebase_key → get_reference_locations.
    // Two calls to $this->helper() from the same method must both be indexed.
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Svc {\n  public function helper(): void {}\n  public function run(): void { $this->helper(); $this->helper(); }\n}\n";
    let file = write(&dir, "this_flow.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

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

    let locs = analyzer.codebase().get_reference_locations(&key);
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
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Svc {\n  public function helper(): void {}\n  public function run(): void { $fn = function() { $this->helper(); }; $fn(); }\n}\n";
    let file = write(&dir, "closure_this.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let offset = src.find("->helper").unwrap() as u32 + 2;
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should resolve $this->helper() inside a non-static closure");

    assert!(
        matches!(&sym.kind, SymbolKind::MethodCall { method, .. } if method.as_ref() == "helper"),
        "expected MethodCall(helper) inside closure, got {:?}",
        sym.kind
    );
}

// ---------------------------------------------------------------------------
// symbol_at — most-specific span is returned
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_returns_innermost_symbol() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n";
    let file = write(&dir, "f.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    // Find the offset of "run" in "$s->run()"
    let offset = src.rfind("run").unwrap() as u32;

    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at the run() call");

    assert!(
        matches!(&sym.kind, SymbolKind::MethodCall { method, .. } if method.as_ref() == "run"),
        "expected MethodCall(run) as the innermost symbol, got {:?}",
        sym.kind
    );
}

// ---------------------------------------------------------------------------
// codebase_key — key format matches symbol_reference_locations
// ---------------------------------------------------------------------------

#[test]
fn codebase_key_for_function_call_matches_reference_index() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "g.php", src);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"))
        .expect("FunctionCall(greet) must be recorded");

    let key = sym
        .codebase_key()
        .expect("FunctionCall should have a codebase key");
    assert_eq!(key, "greet");

    assert!(
        !analyzer
            .codebase()
            .get_reference_locations(key.as_str())
            .is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_method_call_is_lowercased() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Svc { public function Run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->Run(); }\n";
    let file = write(&dir, "h.php", src);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::MethodCall { method, .. } if method.as_ref() == "Run"))
        .expect("MethodCall(Run) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert!(
        key.ends_with("::run"),
        "method part of key must be lowercased, got: {key}"
    );
    assert!(
        !analyzer
            .codebase()
            .get_reference_locations(key.as_str())
            .is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_static_call_matches_reference_index() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Math { public static function square(int $n): int { return $n * $n; } }\nfunction caller(): void { Math::square(3); }\n";
    let file = write(&dir, "i.php", src);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym = result
        .symbols
        .iter()
        .find(|s| {
            matches!(&s.kind, SymbolKind::StaticCall { method, .. } if method.as_ref() == "square")
        })
        .expect("StaticCall(square) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert_eq!(key, "Math::square");
    assert!(
        !analyzer
            .codebase()
            .get_reference_locations(key.as_str())
            .is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_property_access_matches_reference_index() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Counter { public int $count = 0; }\nfunction read(Counter $c): int { return $c->count; }\n";
    let file = write(&dir, "j.php", src);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym = result
        .symbols
        .iter()
        .find(|s| {
            matches!(&s.kind, SymbolKind::PropertyAccess { property, .. } if property.as_ref() == "count")
        })
        .expect("PropertyAccess(count) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert_eq!(key, "Counter::count");
    assert!(
        !analyzer
            .codebase()
            .get_reference_locations(key.as_str())
            .is_empty(),
        "codebase_key for PropertyAccess should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_class_reference_matches_reference_index() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Widget {}\nfunction make(): void { $w = new Widget(); }\n";
    let file = write(&dir, "k.php", src);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::ClassReference(n) if n.as_ref() == "Widget"))
        .expect("ClassReference(Widget) must be recorded");

    let key = sym.codebase_key().unwrap();
    assert_eq!(key, "Widget");
    assert!(
        !analyzer
            .codebase()
            .get_reference_locations(key.as_str())
            .is_empty(),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_variable_is_none() {
    // Use a parameter that is read so it's recorded as a Variable symbol.
    let src = "<?php\nfunction f(int $n): int { return $n; }\n";
    let result = ProjectAnalyzer::analyze_source(src);

    let sym = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::Variable(n) if n == "n"))
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
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction ping(): void {}\nfunction caller(): void { ping(); ping(); }\n";
    let file = write(&dir, "l.php", src);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));
    let file_str = file.to_str().unwrap();

    let first_call_offset = src.find("{ ping").unwrap() as u32 + 2;

    let sym = result
        .symbol_at(file_str, first_call_offset)
        .expect("symbol_at should find a symbol at the first ping() call");

    let key = sym.codebase_key().expect("FunctionCall must have a key");
    assert_eq!(key, "ping");

    let locs = analyzer.codebase().get_reference_locations(&key);
    assert_eq!(
        locs.len(),
        2,
        "two calls to ping() should produce two reference locations"
    );
}

#[test]
fn symbol_at_function_call_span_is_identifier_only() {
    // Cursor on '(' (one past the identifier) must NOT find the function symbol.
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "span_fn.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"))
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
                && matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet")
        })
        .count();
    assert_eq!(found, 0, "cursor at '(' must not match the function symbol");
}

#[test]
fn symbol_at_method_call_span_is_identifier_only() {
    // Cursor on '(' (one past the method identifier) must NOT find the method symbol.
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n";
    let file = write(&dir, "span_method.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym_recorded = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::MethodCall { method, .. } if method.as_ref() == "run"))
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
                && matches!(&s.kind, SymbolKind::MethodCall { method, .. } if method.as_ref() == "run")
        })
        .count();
    assert_eq!(found, 0, "cursor at '(' must not match the method symbol");
}

#[test]
fn symbol_at_static_call_span_is_identifier_only() {
    // Cursor on '(' (one past the static method identifier) must NOT find the symbol.
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Math { public static function sq(int $n): int { return $n * $n; } }\nfunction caller(): void { Math::sq(3); }\n";
    let file = write(&dir, "span_static.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    let sym_recorded = result
        .symbols
        .iter()
        .find(
            |s| matches!(&s.kind, SymbolKind::StaticCall { method, .. } if method.as_ref() == "sq"),
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
                && matches!(&s.kind, SymbolKind::StaticCall { method, .. } if method.as_ref() == "sq")
        })
        .count();
    assert_eq!(
        found, 0,
        "cursor at '(' must not match the static call symbol"
    );
}

#[test]
fn symbol_at_finds_property_access() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Counter { public int $count = 0; }\nfunction read(Counter $c): int { return $c->count; }\n";
    let file = write(&dir, "m.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    // Point cursor at 'count' in '$c->count'
    let offset = src.find("->count").unwrap() as u32 + 2; // +2 skips '->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at $c->count");

    assert!(
        matches!(&sym.kind, SymbolKind::PropertyAccess { property, .. } if property.as_ref() == "count"),
        "expected PropertyAccess(count), got {:?}",
        sym.kind
    );

    // Verify the full LSP flow: cursor → key → reference locations
    let key = sym.codebase_key().expect("PropertyAccess must have a key");
    assert_eq!(key, "Counter::count");
    let locs = analyzer.codebase().get_reference_locations(&key);
    assert_eq!(
        locs.len(),
        1,
        "one property access should produce one reference location"
    );
}

#[test]
fn symbol_at_finds_nullsafe_property_access() {
    let dir = TempDir::new().unwrap();
    let src =
        "<?php\nclass Box { public int $val = 0; }\nfunction read(?Box $b): void { $b?->val; }\n";
    let file = write(&dir, "n.php", src);
    let file_str = file.to_str().unwrap();

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));

    // Point cursor at 'val' in '$b?->val'
    let offset = src.find("?->val").unwrap() as u32 + 3; // +3 skips '?->'
    let sym = result
        .symbol_at(file_str, offset)
        .expect("symbol_at should find a symbol at $b?->val");

    assert!(
        matches!(&sym.kind, SymbolKind::PropertyAccess { property, .. } if property.as_ref() == "val"),
        "expected PropertyAccess(val) for nullsafe access, got {:?}",
        sym.kind
    );

    // Verify the full LSP flow: cursor → key → reference locations
    let key = sym.codebase_key().expect("PropertyAccess must have a key");
    assert_eq!(key, "Box::val");
    let locs = analyzer.codebase().get_reference_locations(&key);
    assert_eq!(
        locs.len(),
        1,
        "one nullsafe property access should produce one reference location"
    );
}
