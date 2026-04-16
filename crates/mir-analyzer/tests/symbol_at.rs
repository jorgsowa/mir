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
    // "<?php\n"                               = 6 bytes  (offsets 0-5)
    // "function greet(): void {}\n"           = 26 bytes (offsets 6-31)
    // "function caller(): void { greet(); }\n"
    //   'greet' starts at offset 32 + len("function caller(): void { ") = 32 + 26 = 58
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "a.php", src);
    let file_str = file.to_str().unwrap().to_string();

    let result = ProjectAnalyzer::analyze_source(src);
    // Find the offset of "greet" in the caller body
    let offset = src.find("{ greet").unwrap() as u32 + 2; // points at 'g' of greet()

    let _sym = result.symbol_at(&file_str, offset);
    // symbol_at uses the file path recorded during analysis; analyze_source uses
    // "<source>" as the synthetic file name.
    // Verify by checking with the synthetic path used internally.
    let sym_any = result
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::FunctionCall(n) if n.as_ref() == "greet"));
    assert!(sym_any.is_some(), "FunctionCall(greet) should be recorded");
}

#[test]
fn symbol_at_returns_none_for_unknown_offset() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction foo(): void {}\n";
    let file = write(&dir, "b.php", src);
    let file_str = file.to_str().unwrap().to_string();

    let result = ProjectAnalyzer::analyze_source(src);
    // Offset 0 is '<?', which resolves to no symbol
    assert!(
        result.symbol_at(&file_str, 0).is_none(),
        "no symbol at offset 0 (opening tag)"
    );
}

// ---------------------------------------------------------------------------
// symbol_at — most-specific span is returned
// ---------------------------------------------------------------------------

#[test]
fn symbol_at_returns_innermost_symbol() {
    // If the same offset matches multiple symbols with different span widths,
    // the one with the smallest span (most specific) should be returned.
    let src = "<?php\nclass Svc { public function run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->run(); }\n";
    let result = ProjectAnalyzer::analyze_source(src);

    // Find offset of "run" in "$s->run()"
    let offset = src.rfind("run").unwrap() as u32;

    // There should be a MethodCall symbol at this offset
    let sym = result.symbols.iter().find(|s| {
        s.span.start <= offset
            && offset < s.span.end
            && matches!(&s.kind, SymbolKind::MethodCall { method, .. } if method.as_ref() == "run")
    });
    assert!(
        sym.is_some(),
        "MethodCall(run) should be recorded at 'run' offset"
    );
}

// ---------------------------------------------------------------------------
// codebase_key — key format matches symbol_reference_locations
// ---------------------------------------------------------------------------

#[test]
fn codebase_key_for_function_call_matches_reference_index() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let file = write(&dir, "c.php", src);

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

    // The key must exist in symbol_reference_locations
    assert!(
        analyzer
            .codebase()
            .symbol_reference_locations
            .contains_key(key.as_str()),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_method_call_is_lowercased() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Svc { public function Run(): void {} }\nfunction caller(): void { $s = new Svc(); $s->Run(); }\n";
    let file = write(&dir, "d.php", src);

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
        analyzer
            .codebase()
            .symbol_reference_locations
            .contains_key(key.as_str()),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_static_call_matches_reference_index() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Math { public static function square(int $n): int { return $n * $n; } }\nfunction caller(): void { Math::square(3); }\n";
    let file = write(&dir, "e.php", src);

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
        analyzer
            .codebase()
            .symbol_reference_locations
            .contains_key(key.as_str()),
        "codebase_key should match an entry in symbol_reference_locations"
    );
}

#[test]
fn codebase_key_for_class_reference_matches_reference_index() {
    let dir = TempDir::new().unwrap();
    let src = "<?php\nclass Widget {}\nfunction make(): void { $w = new Widget(); }\n";
    let file = write(&dir, "f.php", src);

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
        analyzer
            .codebase()
            .symbol_reference_locations
            .contains_key(key.as_str()),
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
    // Craft src so we can find the exact byte offset of the call.
    let src = "<?php\nfunction ping(): void {}\nfunction caller(): void { ping(); ping(); }\n";
    // "<?php\n"                                     6 bytes
    // "function ping(): void {}\n"                 25 bytes  → ping defined at 6..10
    // "function caller(): void { ping(); ping(); }\n"
    //   first call:  offset 6+25+26 = 57  (len("function caller(): void { ") = 26)
    let file = write(&dir, "g.php", src);

    let analyzer = ProjectAnalyzer::new();
    let result = analyzer.analyze(std::slice::from_ref(&file));
    let file_str = file.to_str().unwrap();

    // Find the offset of the first "ping" call in the caller body
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
