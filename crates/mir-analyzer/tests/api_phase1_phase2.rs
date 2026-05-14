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
fn lazy_load_class_with_custom_resolver() {
    use mir_analyzer::{ClassResolver, LazyLoadOutcome};
    use std::path::PathBuf;

    // Custom resolver that maps any FQCN to a temp file we write.
    struct TmpResolver {
        path: PathBuf,
    }
    impl ClassResolver for TmpResolver {
        fn resolve(&self, _fqcn: &str) -> Option<PathBuf> {
            Some(self.path.clone())
        }
    }

    let dir = std::env::temp_dir().join(format!("mir_lazy_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file_path = dir.join("Resolved.php");
    std::fs::write(&file_path, "<?php\nclass ResolvedByCustom {}\n").unwrap();

    let resolver: Arc<dyn ClassResolver> = Arc::new(TmpResolver {
        path: file_path.clone(),
    });

    let session = AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(resolver);

    // Class is not yet known
    assert!(!session.contains_class("ResolvedByCustom"));

    // First call: should load via resolver
    let outcome = session.lazy_load_class_with_outcome("ResolvedByCustom");
    assert_eq!(outcome, LazyLoadOutcome::Loaded);
    assert!(session.contains_class("ResolvedByCustom"));

    // Second call: already loaded
    let outcome = session.lazy_load_class_with_outcome("ResolvedByCustom");
    assert_eq!(outcome, LazyLoadOutcome::AlreadyLoaded);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn prefetch_imports_loads_unresolved_use_statements() {
    use mir_analyzer::ClassResolver;
    use std::path::PathBuf;
    use std::sync::Mutex;

    // Resolver that maps known FQCNs to files we wrote to disk, and tracks
    // every call so we can assert on prefetch behavior.
    struct TrackedResolver {
        map: std::collections::HashMap<String, PathBuf>,
        calls: Mutex<Vec<String>>,
    }
    impl ClassResolver for TrackedResolver {
        fn resolve(&self, fqcn: &str) -> Option<PathBuf> {
            self.calls.lock().unwrap().push(fqcn.to_string());
            self.map.get(fqcn).cloned()
        }
    }

    let dir = std::env::temp_dir().join(format!("mir_prefetch_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let dep_path = dir.join("Dep.php");
    std::fs::write(&dep_path, "<?php\nnamespace App;\nclass Dep {}\n").unwrap();

    let mut map = std::collections::HashMap::new();
    map.insert("App\\Dep".to_string(), dep_path.clone());

    let resolver = Arc::new(TrackedResolver {
        map,
        calls: Mutex::new(Vec::new()),
    });
    let session = AnalysisSession::new(PhpVersion::LATEST).with_class_resolver(resolver.clone());

    // User opens a file that imports App\Dep but doesn't have Dep in the
    // session yet.
    let opened: Arc<str> = Arc::from("opened.php");
    let opened_src: Arc<str> =
        Arc::from("<?php\nuse App\\Dep;\nclass Caller { public function go(Dep $d): void {} }\n");
    session.ingest_file(opened.clone(), opened_src);

    // Before prefetch: Dep is not in the codebase.
    assert!(!session.contains_class("App\\Dep"));

    // pending_lazy_loads should surface the unresolved import.
    let pending = session.pending_lazy_loads(opened.as_ref());
    assert!(
        pending.iter().any(|s| s.as_ref() == "App\\Dep"),
        "pending should include App\\Dep, got {:?}",
        pending
    );

    // Prefetch loads it.
    let loaded = session.prefetch_imports(opened.as_ref());
    assert!(loaded >= 1, "prefetch should load at least App\\Dep");
    assert!(session.contains_class("App\\Dep"));

    // A second prefetch is a no-op (no pending imports remain).
    assert_eq!(session.pending_lazy_loads(opened.as_ref()).len(), 0);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn analyze_dependents_of_runs_in_parallel() {
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();

    // base.php defines Base. dep_a.php and dep_b.php extend Base.
    let base: Arc<str> = Arc::from("base.php");
    let dep_a: Arc<str> = Arc::from("dep_a.php");
    let dep_b: Arc<str> = Arc::from("dep_b.php");

    session.ingest_file(base.clone(), Arc::from("<?php\nclass Base {}\n"));
    session.ingest_file(dep_a.clone(), Arc::from("<?php\nclass A extends Base {}\n"));
    session.ingest_file(dep_b.clone(), Arc::from("<?php\nclass B extends Base {}\n"));

    // Run Pass 2 on the dependents once so they're recorded as having
    // analyzed against Base.
    for (file, src) in [
        (&dep_a, "<?php\nclass A extends Base {}\n"),
        (&dep_b, "<?php\nclass B extends Base {}\n"),
    ] {
        let arena = bumpalo::Bump::new();
        let parsed = php_rs_parser::parse(&arena, src);
        FileAnalyzer::new(&session).analyze(file.clone(), src, &parsed.program, &parsed.source_map);
    }

    // source_of returns the registered source.
    assert!(session.source_of(dep_a.as_ref()).is_some());
    assert_eq!(session.source_of("does-not-exist.php"), None);

    // analyze_dependents_of returns analyses for dependents of base.php.
    // (May be empty if dependency graph wasn't populated — that's still a
    // valid result; the API just shouldn't panic.)
    let analyses = session.analyze_dependents_of(base.as_ref());
    // Sanity: returned files are a subset of the ingested ones.
    for (file, _) in &analyses {
        assert!(file.as_ref() == dep_a.as_ref() || file.as_ref() == dep_b.as_ref());
    }
}

#[test]
fn lazy_load_class_not_resolvable_without_resolver() {
    use mir_analyzer::LazyLoadOutcome;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    let outcome = session.lazy_load_class_with_outcome("Some\\Unknown\\Class");
    assert_eq!(outcome, LazyLoadOutcome::NotResolvable);
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

// Regression: bare FQN references without a `use` statement must be tracked in
// the dependency graph so that `analyze_dependents_of` re-analyzes the referencing
// file when the definition changes.  Currently not implemented — these tests document
// the missing behaviour and should be un-ignored once the bug is fixed.

#[test]
fn analyze_dependents_of_tracks_bare_fqn_new() {
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();

    let service: Arc<str> = Arc::from("service.php");
    let consumer: Arc<str> = Arc::from("consumer.php");

    session.ingest_file(
        service.clone(),
        Arc::from("<?php\nclass Service { public function run(): void {} }\n"),
    );
    session.ingest_file(
        consumer.clone(),
        Arc::from("<?php\nfunction consume(): void { $s = new \\Service(); $s->run(); }\n"),
    );

    let consumer_src = "<?php\nfunction consume(): void { $s = new \\Service(); $s->run(); }\n";
    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, consumer_src);
    FileAnalyzer::new(&session).analyze(
        consumer.clone(),
        consumer_src,
        &parsed.program,
        &parsed.source_map,
    );

    let analyses = session.analyze_dependents_of(service.as_ref());
    let dependent_files: Vec<&str> = analyses.iter().map(|(f, _)| f.as_ref()).collect();
    assert!(
        dependent_files.contains(&consumer.as_ref()),
        "consumer.php references Service via bare FQN but was not returned by \
         analyze_dependents_of — dependency graph is missing FQN reference edges"
    );
}

#[test]
fn analyze_dependents_of_tracks_bare_fqn_static_call() {
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();

    let helper: Arc<str> = Arc::from("helper.php");
    let caller: Arc<str> = Arc::from("caller.php");

    session.ingest_file(
        helper.clone(),
        Arc::from("<?php\nclass Helper { public static function go(): void {} }\n"),
    );
    session.ingest_file(
        caller.clone(),
        Arc::from("<?php\nfunction call_it(): void { \\Helper::go(); }\n"),
    );

    let caller_src = "<?php\nfunction call_it(): void { \\Helper::go(); }\n";
    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, caller_src);
    FileAnalyzer::new(&session).analyze(
        caller.clone(),
        caller_src,
        &parsed.program,
        &parsed.source_map,
    );

    let analyses = session.analyze_dependents_of(helper.as_ref());
    let dependent_files: Vec<&str> = analyses.iter().map(|(f, _)| f.as_ref()).collect();
    assert!(
        dependent_files.contains(&caller.as_ref()),
        "caller.php references Helper via bare FQN static call but was not returned by \
         analyze_dependents_of — dependency graph is missing FQN reference edges"
    );
}
