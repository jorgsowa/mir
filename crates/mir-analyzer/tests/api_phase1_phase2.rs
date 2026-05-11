//! End-to-end verification of Phase 1 and Phase 2 API improvements.
//!
//! Phase 1 (analyzer's job):
//! - hover() returns real HoverInfo (no longer a stub)
//! - Symbol enum for type-safe identity
//! - Result types for lookups (NotFound vs NoSourceLocation)
//! - Hierarchical DocumentSymbol (classes contain method/property children)
//!
//! Phase 2 (boundary fixes):
//! - ProjectAnalyzer builder pattern
//! - with_cache_dir() avoids Arc wrapping
//! - SymbolKind::Variable uses Arc<str>
//! - mir_codebase types re-exported

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, PhpVersion, Symbol, SymbolLookupError};

#[test]
fn hover_returns_real_info_for_function() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();

    let file: Arc<str> = Arc::from("test.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         /**\n\
          * Adds two integers and returns the sum.\n\
          */\n\
         function add(int $a, int $b): int { return $a + $b; }\n",
    );

    session.ingest_file(file.clone(), source.clone());

    let hover = session
        .hover(&Symbol::function("add"))
        .expect("add() should be resolvable");

    assert!(
        hover.docstring.is_some(),
        "Docstring should be populated from the docblock description"
    );
    assert!(
        hover
            .docstring
            .as_ref()
            .unwrap()
            .contains("Adds two integers"),
        "Docstring should include the description text, got: {:?}",
        hover.docstring
    );
    assert!(
        hover.definition.is_some(),
        "Function should have a source location"
    );
}

#[test]
fn hover_returns_not_found_for_unknown_symbol() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let result = session.hover(&Symbol::function("nonexistent_function_xyz"));
    assert_eq!(result.unwrap_err(), SymbolLookupError::NotFound);
}

#[test]
fn symbol_method_normalizes_case() {
    // PHP methods are case-insensitive — the Symbol enum should normalize.
    let s1 = Symbol::method("Foo", "Bar");
    let s2 = Symbol::method("Foo", "bar");
    let s3 = Symbol::method("Foo", "BAR");

    assert_eq!(s1, s2);
    assert_eq!(s1, s3);
    assert_eq!(s1.codebase_key(), "Foo::bar");
}

#[test]
fn definition_of_returns_result_with_distinct_errors() {
    let session = AnalysisSession::new(PhpVersion::LATEST);

    // Class never registered → NotFound
    let err = session
        .definition_of(&Symbol::class("CompletelyMadeUp"))
        .unwrap_err();
    assert_eq!(err, SymbolLookupError::NotFound);
}

#[test]
fn document_symbols_returns_hierarchical_tree() {
    use mir_analyzer::symbol::DocumentSymbolKind;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();

    let file: Arc<str> = Arc::from("hierarchy.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         class Container {\n\
             public int $count = 0;\n\
             const VERSION = 1;\n\
             public function add(int $n): void {}\n\
             public function reset(): void {}\n\
         }\n",
    );

    session.ingest_file(file.clone(), source.clone());

    let symbols = session.document_symbols(file.as_ref());
    let container = symbols
        .iter()
        .find(|s| s.name.as_ref() == "Container")
        .expect("Container class should be in document symbols");

    assert_eq!(container.kind, DocumentSymbolKind::Class);
    assert!(
        !container.children.is_empty(),
        "Class should have children (methods, properties, constants)"
    );

    // Should contain methods, property, constant
    let kinds: Vec<DocumentSymbolKind> = container.children.iter().map(|c| c.kind).collect();
    assert!(
        kinds.contains(&DocumentSymbolKind::Method),
        "Should have at least one method child, got: {kinds:?}"
    );
}

#[test]
fn references_to_takes_typed_symbol() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();

    let file: Arc<str> = Arc::from("refs.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         function helper(): void {}\n\
         function caller(): void { helper(); helper(); }\n",
    );

    session.ingest_file(file.clone(), source.clone());

    // Now run pass 2 to record references
    use mir_analyzer::FileAnalyzer;
    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, &source);
    let _analysis = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    // New typed API: pass Symbol::function, not &str
    let refs = session.references_to(&Symbol::function("helper"));
    assert!(
        refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
        "Should find references to helper in {}",
        file
    );
}

#[test]
fn project_analyzer_builder_pattern() {
    use mir_analyzer::ProjectAnalyzer;

    // The builder pattern is chainable
    let _analyzer = ProjectAnalyzer::new()
        .with_php_version(PhpVersion::LATEST)
        .with_dead_code(true);

    // Old mutable-field style still works for backward compat
    let mut legacy = ProjectAnalyzer::new();
    legacy.php_version = Some(PhpVersion::LATEST);
    legacy.find_dead_code = true;
}

#[test]
fn analysis_session_with_cache_dir() {
    // New convenience constructor avoids Arc::new wrapping at call site
    let temp = std::env::temp_dir().join("mir_test_cache_xyz");
    let _session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(&temp);
    let _ = std::fs::remove_dir_all(&temp);
}

#[test]
fn symbol_kind_variable_uses_arc_str() {
    use mir_analyzer::symbol::SymbolKind;

    let kind = SymbolKind::Variable(Arc::from("count"));
    match kind {
        SymbolKind::Variable(name) => {
            // Arc<str> can be compared via as_ref()
            assert_eq!(name.as_ref(), "count");
        }
        _ => panic!("expected Variable"),
    }
}

#[test]
fn re_exports_available_at_crate_root() {
    // Should not require depending on mir_codebase
    let _: mir_analyzer::Visibility = mir_analyzer::Visibility::Public;
    // FnParam and TemplateParam should also be reachable as types
    let _name: &'static str = std::any::type_name::<mir_analyzer::FnParam>();
    let _name: &'static str = std::any::type_name::<mir_analyzer::TemplateParam>();
}

#[test]
fn contains_function_class_method_typed_queries() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(
        Arc::from("typed.php"),
        Arc::from(
            "<?php\n\
             class Worker { public function run(): void {} }\n\
             function helper(): void {}\n",
        ),
    );

    // Class / function / method are checkable without poking at internals
    assert!(session.contains_class("Worker"));
    assert!(session.contains_function("helper"));
    assert!(session.contains_method("Worker", "run"));
    // PHP method case insensitivity
    assert!(session.contains_method("Worker", "RUN"));
    assert!(session.contains_method("Worker", "Run"));

    assert!(!session.contains_class("DoesNotExist"));
    assert!(!session.contains_function("does_not_exist_xyz"));
    assert!(!session.contains_method("Worker", "missing"));
}

#[test]
fn resolved_symbol_to_symbol_bridges_pass2_with_queries() {
    use mir_analyzer::symbol::SymbolKind;
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();

    let file: Arc<str> = Arc::from("bridge.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         function helper(): void {}\n\
         function caller(): void { helper(); }\n",
    );

    session.ingest_file(file.clone(), source.clone());

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, &source);
    let analysis = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    let helper_call = analysis
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, SymbolKind::FunctionCall(name) if name.as_ref() == "helper"))
        .expect("should record helper() call in caller body");

    let typed_symbol = helper_call
        .to_symbol()
        .expect("FunctionCall should convert to Symbol");

    assert_eq!(typed_symbol, Symbol::function("helper"));

    // The typed Symbol can be passed directly to references_to
    let refs = session.references_to(&typed_symbol);
    assert!(refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()));
}

#[test]
fn all_classes_and_all_functions_workspace_iteration() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(
        Arc::from("ws.php"),
        Arc::from(
            "<?php\n\
             class Alpha {}\n\
             class Beta {}\n\
             function gamma(): void {}\n",
        ),
    );

    let classes = session.all_classes();
    let class_names: Vec<&str> = classes.iter().map(|(f, _)| f.as_ref()).collect();
    assert!(class_names.contains(&"Alpha"));
    assert!(class_names.contains(&"Beta"));

    let functions = session.all_functions();
    let fn_names: Vec<&str> = functions.iter().map(|(f, _)| f.as_ref()).collect();
    assert!(fn_names.contains(&"gamma"));
}
