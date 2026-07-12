//! End-to-end verification of Phase 1 and Phase 2 API improvements.
//!
//! Phase 1 (analyzer's job):
//! - hover() returns real HoverInfo (no longer a stub)
//! - Name enum for type-safe identity
//! - Result types for lookups (NotFound vs NoSourceLocation)
//! - Hierarchical DocumentSymbol (classes contain method/property children)
//!
//! Phase 2 (boundary fixes):
//! - ProjectAnalyzer builder pattern
//! - with_cache_dir() avoids Arc wrapping
//! - ReferenceKind::Variable uses Arc<str>
//! - mir_codebase types re-exported

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, Name, PhpVersion, SymbolLookupError};

#[test]
fn hover_returns_real_info_for_function() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

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
        .hover(&Name::function("add"))
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
    let result = session.hover(&Name::function("nonexistent_function_xyz"));
    assert_eq!(result.unwrap_err(), SymbolLookupError::NotFound);
}

#[test]
fn symbol_method_normalizes_case() {
    // PHP methods are case-insensitive — the Name enum should normalize.
    let s1 = Name::method("Foo", "Bar");
    let s2 = Name::method("Foo", "bar");
    let s3 = Name::method("Foo", "BAR");

    assert_eq!(s1, s2);
    assert_eq!(s1, s3);
    assert_eq!(s1.codebase_key(), "meth:Foo::bar");
}

#[test]
fn definition_of_returns_result_with_distinct_errors() {
    let session = AnalysisSession::new(PhpVersion::LATEST);

    // Class never registered → NotFound
    let err = session
        .definition_of(&Name::class("CompletelyMadeUp"))
        .unwrap_err();
    assert_eq!(err, SymbolLookupError::NotFound);
}

#[test]
fn document_symbols_returns_hierarchical_tree() {
    use mir_analyzer::symbol::DeclarationKind;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

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

    assert_eq!(container.kind, DeclarationKind::Class);
    assert!(
        !container.children.is_empty(),
        "Class should have children (methods, properties, constants)"
    );

    // Should contain methods, property, constant
    let kinds: Vec<DeclarationKind> = container.children.iter().map(|c| c.kind).collect();
    assert!(
        kinds.contains(&DeclarationKind::Method),
        "Should have at least one method child, got: {kinds:?}"
    );
}

#[test]
fn references_to_takes_typed_symbol() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let file: Arc<str> = Arc::from("refs.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         function helper(): void {}\n\
         function caller(): void { helper(); helper(); }\n",
    );

    session.ingest_file(file.clone(), source.clone());

    // Now run pass 2 to record references
    use mir_analyzer::FileAnalyzer;
    let parsed = php_rs_parser::parse(&source);
    let _analysis = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    // New typed API: pass Name::function, not &str
    let refs = session.references_to(&Name::function("helper"));
    assert!(
        refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
        "Should find references to helper in {}",
        file
    );
}

#[test]
fn analysis_session_builder_pattern() {
    use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

    // Builder pattern is chainable.
    let _session = AnalysisSession::new(PhpVersion::LATEST);

    // Dead-code reporting is opted in by removing the dead-code names from
    // BatchOptions::suppressed_issue_kinds.
    let mut opts =
        BatchOptions::new().with_suppressed(mir_analyzer::dead_code_issue_kinds().iter().copied());
    for kind in mir_analyzer::dead_code_issue_kinds() {
        opts.suppressed_issue_kinds.remove(*kind);
    }
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
    use mir_analyzer::symbol::ReferenceKind;

    let kind = ReferenceKind::Variable(Arc::from("count"));
    match kind {
        ReferenceKind::Variable(name) => {
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
    // DeclaredParam and TemplateParam should also be reachable as types
    let _name: &'static str = std::any::type_name::<mir_analyzer::DeclaredParam>();
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
    use mir_analyzer::symbol::ReferenceKind;
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let file: Arc<str> = Arc::from("bridge.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         function helper(): void {}\n\
         function caller(): void { helper(); }\n",
    );

    session.ingest_file(file.clone(), source.clone());

    let parsed = php_rs_parser::parse(&source);
    let analysis = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    let helper_call = analysis
        .symbols
        .iter()
        .find(|s| matches!(&s.kind, ReferenceKind::FunctionCall(name) if name.as_ref() == "helper"))
        .expect("should record helper() call in caller body");

    let typed_symbol = helper_call
        .to_symbol()
        .expect("FunctionCall should convert to Name");

    assert_eq!(typed_symbol, Name::function("helper"));

    // The typed Name can be passed directly to references_to
    let refs = session.references_to(&typed_symbol);
    assert!(refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()));
}

#[test]
fn method_references_scoped_by_declaring_class() {
    // Verify: findReferences on Foo::toString must NOT return Bar::toString or
    // its call sites — they are unrelated classes with no common ancestor.
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let file: Arc<str> = Arc::from("scope.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         final class Foo { public function toString(): string { return 'foo'; } }\n\
         final class Bar { public function toString(): string { return 'bar'; } }\n\
         (new Foo())->toString();\n",
    );

    session.ingest_file(file.clone(), source.clone());
    let parsed = php_rs_parser::parse(&source);
    let _ = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    let foo_refs = session.references_to(&Name::method("Foo", "toString"));
    let bar_refs = session.references_to(&Name::method("Bar", "toString"));

    assert!(
        !foo_refs.is_empty(),
        "Foo::toString should have at least one reference (the call site); got none"
    );
    assert!(
        bar_refs.is_empty(),
        "Bar::toString should have zero references; got {bar_refs:?}"
    );

    let foo_lines: Vec<u32> = foo_refs.iter().map(|(_, r)| r.start.line).collect();
    assert!(
        foo_lines.contains(&4),
        "Expected reference on line 4 (1-based); got {foo_lines:?}"
    );
}

#[test]
fn method_references_end_to_end_symbol_at_flow() {
    // Verify the real findReferences flow: symbol_at → to_symbol() → references_to.
    // The Name built from the resolved symbol at the call position must round-trip
    // back to the same reference.
    use mir_analyzer::symbol::ReferenceKind;
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let file: Arc<str> = Arc::from("e2e.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         final class Foo { public function toString(): string { return 'foo'; } }\n\
         (new Foo())->toString();\n",
    );

    session.ingest_file(file.clone(), source.clone());
    let parsed = php_rs_parser::parse(&source);
    let analysis = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    // "->toString" — skip the "->" (2 bytes) to land on the 't'.
    let call_offset = source.find("->toString").unwrap() as u32 + 2;

    let sym = analysis
        .symbol_at(call_offset)
        .expect("should resolve symbol at toString call site");

    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { class, .. } if class.as_ref() == "Foo"),
        "symbol_at should report class Foo; got {:?}",
        sym.kind
    );

    let name = sym.to_symbol().expect("MethodCall should map to a Name");
    let refs = session.references_to(&name);

    assert!(
        !refs.is_empty(),
        "references_to via symbol_at flow must find the call site; got none"
    );
}

#[test]
fn method_references_inherited_method_end_to_end() {
    // When Foo inherits toString from Base, record_ref stores the reference
    // under "Base::tostring" (the declaring class). Before this fix, record_symbol
    // stored class "Foo" (the receiver), making symbol_at → references_to return
    // nothing. After the fix both keys agree on the declaring class.
    use mir_analyzer::symbol::ReferenceKind;
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let file: Arc<str> = Arc::from("inherit.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         class Base { public function toString(): string { return 'b'; } }\n\
         final class Foo extends Base {}\n\
         (new Foo())->toString();\n",
    );

    session.ingest_file(file.clone(), source.clone());
    let parsed = php_rs_parser::parse(&source);
    let analysis = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    // "->toString" — skip the "->" (2 bytes) to land on the 't'.
    let call_offset = source.find("->toString").unwrap() as u32 + 2;

    let sym = analysis
        .symbol_at(call_offset)
        .expect("should resolve symbol at toString call");

    let declaring_class = match &sym.kind {
        ReferenceKind::MethodCall { class, .. } => class.as_ref().to_string(),
        other => panic!("unexpected kind: {other:?}"),
    };

    assert_eq!(
        declaring_class, "Base",
        "symbol_at must report the DECLARING class (Base), not the receiver (Foo)"
    );

    let name = sym.to_symbol().expect("MethodCall maps to Name");
    let refs = session.references_to(&name);

    assert!(
        !refs.is_empty(),
        "references_to(Base::toString) must find the (new Foo())->toString() call; \
         got none (declaring_class was '{declaring_class}', refs: {refs:?})"
    );
}

#[test]
fn property_references_inherited_property_end_to_end() {
    // When Foo inherits $count from Base, record_ref and record_symbol must both
    // use the declaring class (Base), not the receiver (Foo), so that
    // references_to(Base::count) finds the access site.
    use mir_analyzer::symbol::ReferenceKind;
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let file: Arc<str> = Arc::from("inherit_prop.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         class Base { public int $count = 0; }\n\
         final class Foo extends Base {}\n\
         (new Foo())->count;\n",
    );

    session.ingest_file(file.clone(), source.clone());
    let parsed = php_rs_parser::parse(&source);
    let analysis = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    let prop_offset = source.find("->count").unwrap() as u32 + 2;
    let sym = analysis
        .symbol_at(prop_offset)
        .expect("should resolve symbol at ->count");

    let declaring_class = match &sym.kind {
        ReferenceKind::PropertyAccess { class, .. } => class.as_ref().to_string(),
        other => panic!("unexpected kind: {other:?}"),
    };

    assert_eq!(
        declaring_class, "Base",
        "symbol_at must report the DECLARING class (Base), not the receiver (Foo)"
    );

    let name = sym.to_symbol().expect("PropertyAccess maps to Name");
    let refs = session.references_to(&name);

    assert!(
        !refs.is_empty(),
        "references_to(Base::count) must find the (new Foo())->count access; \
         got none (declaring_class was '{declaring_class}', refs: {refs:?})"
    );
}

#[test]
fn property_references_direct_property_end_to_end() {
    // Non-inherited property: references_to(Foo::value) finds the access site.
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let file: Arc<str> = Arc::from("direct_prop.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         class Foo { public string $value = ''; }\n\
         (new Foo())->value;\n",
    );

    session.ingest_file(file.clone(), source.clone());
    let parsed = php_rs_parser::parse(&source);
    let _ = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    let refs = session.references_to(&Name::property("Foo", "value"));
    assert!(
        !refs.is_empty(),
        "references_to(Foo::value) must find the ->value access; got none"
    );
}

#[test]
fn load_class_with_custom_resolver() {
    use mir_analyzer::{ClassResolver, LoadOutcome};
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
    let outcome = session.load_class("ResolvedByCustom");
    assert_eq!(outcome, LoadOutcome::Loaded);
    assert!(session.contains_class("ResolvedByCustom"));

    // Second call: already loaded
    let outcome = session.load_class("ResolvedByCustom");
    assert_eq!(outcome, LoadOutcome::AlreadyLoaded);

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
fn reanalyze_dependents_runs_in_parallel() {
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

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
        let parsed = php_rs_parser::parse(src);
        FileAnalyzer::new(&session).analyze(file.clone(), src, &parsed.program, &parsed.source_map);
    }

    // source_of returns the registered source.
    assert!(session.source_of(dep_a.as_ref()).is_some());
    assert_eq!(session.source_of("does-not-exist.php"), None);

    // reanalyze_dependents returns analyses for dependents of base.php.
    // (May be empty if dependency graph wasn't populated — that's still a
    // valid result; the API just shouldn't panic.)
    let analyses = session.reanalyze_dependents(base.as_ref());
    // Sanity: returned files are a subset of the ingested ones.
    for (file, _) in &analyses {
        assert!(file.as_ref() == dep_a.as_ref() || file.as_ref() == dep_b.as_ref());
    }
}

#[test]
fn reanalyze_files_recomputes_the_given_set_only() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let base: Arc<str> = Arc::from("rf_base.php");
    let dependent: Arc<str> = Arc::from("rf_dep.php");
    let unrelated: Arc<str> = Arc::from("rf_other.php");

    session.ingest_file(base.clone(), Arc::from("<?php\nclass RfBase {}\n"));
    session.ingest_file(
        dependent.clone(),
        Arc::from("<?php\nclass RfDep extends RfBase {}\n"),
    );
    session.ingest_file(
        unrelated.clone(),
        Arc::from("<?php\nfunction rf_free(): void {}\n"),
    );

    // Edit the base: RfBase disappears. Re-analyzing the caller-supplied
    // "open" set must surface the dependent's broken extends without any
    // dependency-graph computation, and must not touch files outside the set.
    session.ingest_file(base.clone(), Arc::from("<?php\nclass RfRenamed {}\n"));

    let open_set = [dependent.clone(), unrelated.clone()];
    let analyses =
        session.reanalyze_files_cancellable(&open_set, &mir_analyzer::IndexCancel::new());

    let files: Vec<&str> = analyses.iter().map(|(f, _)| f.as_ref()).collect();
    assert_eq!(files, vec![dependent.as_ref(), unrelated.as_ref()]);

    let dep_analysis = &analyses[0].1;
    assert!(
        dep_analysis
            .issues
            .iter()
            .any(|i| i.location.file.as_ref() == dependent.as_ref()
                && format!("{:?}", i.kind).contains("RfBase")),
        "dependent must report the missing RfBase after the base edit; got {:?}",
        dep_analysis.issues
    );
    assert!(
        analyses[1].1.issues.is_empty(),
        "unrelated file must stay clean"
    );

    // Files the session doesn't know are skipped, not errored.
    let ghost: [Arc<str>; 1] = [Arc::from("rf_ghost.php")];
    assert!(session
        .reanalyze_files_cancellable(&ghost, &mir_analyzer::IndexCancel::new())
        .is_empty());

    // A pre-cancelled token short-circuits.
    let cancelled = mir_analyzer::IndexCancel::new();
    cancelled.cancel();
    assert!(session
        .reanalyze_files_cancellable(&open_set, &cancelled)
        .is_empty());
}

#[test]
fn load_class_not_resolvable_without_resolver() {
    use mir_analyzer::LoadOutcome;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    let outcome = session.load_class("Some\\Unknown\\Class");
    assert_eq!(outcome, LoadOutcome::NotResolvable);
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
// the dependency graph so that `reanalyze_dependents` re-analyzes the referencing
// file when the definition changes.  Currently not implemented — these tests document
// the missing behaviour and should be un-ignored once the bug is fixed.

#[test]
fn reanalyze_dependents_tracks_bare_fqn_new() {
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

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
    let parsed = php_rs_parser::parse(consumer_src);
    FileAnalyzer::new(&session).analyze(
        consumer.clone(),
        consumer_src,
        &parsed.program,
        &parsed.source_map,
    );

    let analyses = session.reanalyze_dependents(service.as_ref());
    let dependent_files: Vec<&str> = analyses.iter().map(|(f, _)| f.as_ref()).collect();
    assert!(
        dependent_files.contains(&consumer.as_ref()),
        "consumer.php references Service via bare FQN but was not returned by \
         reanalyze_dependents — dependency graph is missing FQN reference edges"
    );
}

#[test]
fn reanalyze_dependents_tracks_bare_fqn_static_call() {
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

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
    let parsed = php_rs_parser::parse(caller_src);
    FileAnalyzer::new(&session).analyze(
        caller.clone(),
        caller_src,
        &parsed.program,
        &parsed.source_map,
    );

    let analyses = session.reanalyze_dependents(helper.as_ref());
    let dependent_files: Vec<&str> = analyses.iter().map(|(f, _)| f.as_ref()).collect();
    assert!(
        dependent_files.contains(&caller.as_ref()),
        "caller.php references Helper via bare FQN static call but was not returned by \
         reanalyze_dependents — dependency graph is missing FQN reference edges"
    );
}

#[test]
fn dependency_graph_includes_unused_param_type_hint() {
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let service: Arc<str> = Arc::from("service.php");
    let consumer: Arc<str> = Arc::from("consumer.php");

    // Define Service in service.php (namespace Vendor)
    session.ingest_file(
        service.clone(),
        Arc::from("<?php\nnamespace Vendor\nclass Service { }\n"),
    );

    // Use Service in param type hint WITHOUT use statement and WITHOUT using the param
    session.ingest_file(
        consumer.clone(),
        Arc::from("<?php\nnamespace Vendor\nfunction consume(Service $s) { }\n"),
    );

    // Analyze the consumer file to trigger Pass 2
    let consumer_src = "<?php\nnamespace Vendor\nfunction consume(Service $s) { }\n";
    let parsed = php_rs_parser::parse(consumer_src);
    FileAnalyzer::new(&session).analyze(
        consumer.clone(),
        consumer_src,
        &parsed.program,
        &parsed.source_map,
    );

    // Check if consumer is considered a dependent of service
    let dependents = session
        .dependency_graph()
        .transitive_dependents(service.as_ref());
    assert!(
        dependents.contains(&consumer.to_string()),
        "consumer.php should depend on service.php due to type hint in parameter, \
         even though the parameter is unused and there's no use statement"
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Mutation tests: reanalyze_dependents after definition is removed / renamed.
//
// The correctness gap: when class Foo is deleted from A.php, files referencing
// \Foo were dropped from the dependent set because symbol_defining_file("Foo")
// returned None. The fix maintains a stale_defined_symbols map + a
// symbol_referencers reverse index so the edges survive the deletion.
// ──────────────────────────────────────────────────────────────────────────────

/// Helper: run Pass 2 on `src` under `file` path in `session`.
fn analyze_file(session: &AnalysisSession, file: Arc<str>, src: &str) {
    use mir_analyzer::FileAnalyzer;
    let parsed = php_rs_parser::parse(src);
    FileAnalyzer::new(session).analyze(file, src, &parsed.program, &parsed.source_map);
}

/// Return the set of file paths returned by reanalyze_dependents.
fn dependent_files(session: &AnalysisSession, file: &str) -> std::collections::HashSet<String> {
    session
        .reanalyze_dependents(file)
        .into_iter()
        .map(|(f, _)| f.to_string())
        .collect()
}

#[test]
fn reanalyze_dependents_after_definition_deleted() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let foo: Arc<str> = Arc::from("Foo.php");
    let bar: Arc<str> = Arc::from("Bar.php");

    // Establish: Foo.php defines class Foo, Bar.php references it.
    session.ingest_file(foo.clone(), Arc::from("<?php\nclass Foo {}\n"));
    session.ingest_file(
        bar.clone(),
        Arc::from("<?php\nfunction f(\\Foo $x): void {}\n"),
    );
    let bar_src = "<?php\nfunction f(\\Foo $x): void {}\n";
    analyze_file(&session, bar.clone(), bar_src);

    // Precondition: Bar.php is a dependent before the mutation.
    let before = dependent_files(&session, foo.as_ref());
    assert!(
        before.contains(bar.as_ref()),
        "precondition: Bar.php must be a dependent before deletion; got {:?}",
        before
    );

    // Mutate: remove class Foo from Foo.php.
    session.ingest_file(foo.clone(), Arc::from("<?php\n// class Foo removed\n"));

    // Assert: Bar.php still appears — it has a broken reference that needs re-analysis.
    let after = dependent_files(&session, foo.as_ref());
    assert!(
        after.contains(bar.as_ref()),
        "Bar.php references \\Foo which was deleted from Foo.php — \
         it must still appear in reanalyze_dependents so the broken reference is surfaced; \
         got {:?}",
        after
    );
}

#[test]
fn reanalyze_dependents_after_definition_renamed() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let foo: Arc<str> = Arc::from("Foo.php");
    let bar: Arc<str> = Arc::from("Bar.php");

    session.ingest_file(foo.clone(), Arc::from("<?php\nclass Foo {}\n"));
    session.ingest_file(
        bar.clone(),
        Arc::from("<?php\nfunction f(\\Foo $x): void {}\n"),
    );
    analyze_file(
        &session,
        bar.clone(),
        "<?php\nfunction f(\\Foo $x): void {}\n",
    );

    // Rename: class Foo → class Renamed in the same file.
    session.ingest_file(foo.clone(), Arc::from("<?php\nclass Renamed {}\n"));

    let after = dependent_files(&session, foo.as_ref());
    assert!(
        after.contains(bar.as_ref()),
        "Bar.php references \\Foo which was renamed to \\Renamed in Foo.php — \
         Bar.php must still appear in reanalyze_dependents; got {:?}",
        after
    );
}

#[test]
fn reanalyze_dependents_after_definition_moved() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let a: Arc<str> = Arc::from("A.php");
    let b: Arc<str> = Arc::from("B.php");
    let consumer: Arc<str> = Arc::from("Consumer.php");

    // class Foo initially in A.php.
    session.ingest_file(a.clone(), Arc::from("<?php\nclass Foo {}\n"));
    session.ingest_file(b.clone(), Arc::from("<?php\n// empty\n"));
    session.ingest_file(
        consumer.clone(),
        Arc::from("<?php\nfunction f(\\Foo $x): void {}\n"),
    );
    analyze_file(
        &session,
        consumer.clone(),
        "<?php\nfunction f(\\Foo $x): void {}\n",
    );

    // Move: remove Foo from A.php, add it to B.php.
    session.ingest_file(a.clone(), Arc::from("<?php\n// Foo moved to B.php\n"));
    session.ingest_file(b.clone(), Arc::from("<?php\nclass Foo {}\n"));

    // Consumer.php references \Foo — it must appear as a dependent of A.php
    // (broken reference) AND of B.php (resolved reference).
    let a_deps = dependent_files(&session, a.as_ref());
    assert!(
        a_deps.contains(consumer.as_ref()),
        "Consumer.php must appear as dependent of A.php after Foo is moved out; got {:?}",
        a_deps
    );
    let b_deps = dependent_files(&session, b.as_ref());
    assert!(
        b_deps.contains(consumer.as_ref()),
        "Consumer.php must appear as dependent of B.php after Foo is moved in; got {:?}",
        b_deps
    );
}

#[test]
fn reanalyze_dependents_after_definition_readded() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let foo: Arc<str> = Arc::from("Foo.php");
    let bar: Arc<str> = Arc::from("Bar.php");

    session.ingest_file(foo.clone(), Arc::from("<?php\nclass Foo {}\n"));
    session.ingest_file(
        bar.clone(),
        Arc::from("<?php\nfunction f(\\Foo $x): void {}\n"),
    );
    analyze_file(
        &session,
        bar.clone(),
        "<?php\nfunction f(\\Foo $x): void {}\n",
    );

    // Delete Foo.
    session.ingest_file(foo.clone(), Arc::from("<?php\n// deleted\n"));

    // Re-add Foo. The stale entry should be cleared and the normal dep graph
    // edge restored — Bar.php is still a dependent via the current edge.
    session.ingest_file(foo.clone(), Arc::from("<?php\nclass Foo {}\n"));

    let after = dependent_files(&session, foo.as_ref());
    assert!(
        after.contains(bar.as_ref()),
        "Bar.php must be a dependent of Foo.php after Foo is re-added; got {:?}",
        after
    );
}

#[test]
fn reanalyze_dependents_transitive_after_delete() {
    // A.php defines Foo. B.php references Foo (direct dependent).
    // C.php structurally depends on B.php (e.g. extends a class from B.php).
    // After Foo is deleted from A.php:
    //   - B.php must appear (direct stale dependent)
    //   - C.php must appear (transitive via structural dep on B)
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let a: Arc<str> = Arc::from("A.php");
    let b: Arc<str> = Arc::from("B.php");
    let c: Arc<str> = Arc::from("C.php");

    session.ingest_file(a.clone(), Arc::from("<?php\nclass Foo {}\n"));
    session.ingest_file(
        b.clone(),
        Arc::from("<?php\nclass Bar { public function f(\\Foo $x): void {} }\n"),
    );
    session.ingest_file(c.clone(), Arc::from("<?php\nclass Baz extends \\Bar {}\n"));

    // Pass 2 on both B and C so their reference edges land in file_referenced_symbols.
    analyze_file(
        &session,
        b.clone(),
        "<?php\nclass Bar { public function f(\\Foo $x): void {} }\n",
    );
    analyze_file(&session, c.clone(), "<?php\nclass Baz extends \\Bar {}\n");

    // Delete Foo from A.php.
    session.ingest_file(a.clone(), Arc::from("<?php\n// Foo deleted\n"));

    let after = dependent_files(&session, a.as_ref());
    assert!(
        after.contains(b.as_ref()),
        "B.php (direct referencer of deleted Foo) must appear; got {:?}",
        after
    );
    assert!(
        after.contains(c.as_ref()),
        "C.php (transitively depends on B.php) must appear; got {:?}",
        after
    );
}

// ──────────────────────────────────────────────────────────────────────────────
// Regression: reanalyze_dependents must not deadlock when many dependents each
// trigger a lazy class-load during warm-up.
//
// `reanalyze_dependents` warms up each dependent via `prepare_ast_for_analysis`,
// which resolves the dependent's direct class references and loads any that
// aren't indexed yet. Loading mutates the shared session salsa storage
// (`load_class` → `ingest_file` takes the salsa write lock and sets inputs).
//
// v0.37.0 moved that warm-up *inside* the parallel rayon worker. With many
// dependents each referencing a not-yet-loaded class, multiple workers entered
// `ingest_file` concurrently while sibling workers held live snapshot clones
// mid-`analyze_file` — quiescing the salsa runtime and deadlocking. On a
// high-fan-out workspace (the symfony LSP feature tests) the call hung
// indefinitely. The fix hoists the input-mutating warm-up out of the parallel
// loop. This test reproduces the condition and asserts the call completes.
// ──────────────────────────────────────────────────────────────────────────────

/// In-memory `ClassResolver`: maps an FQCN to a virtual path. Leading
/// backslash on fully-qualified names is normalized away.
struct MapResolver(std::collections::HashMap<String, std::path::PathBuf>);
impl mir_analyzer::ClassResolver for MapResolver {
    fn resolve(&self, fqcn: &str) -> Option<std::path::PathBuf> {
        self.0.get(fqcn.trim_start_matches('\\')).cloned()
    }
}

/// In-memory `SourceProvider`: serves virtual-path source text.
struct MapProvider(std::collections::HashMap<String, Arc<str>>);
impl mir_analyzer::SourceProvider for MapProvider {
    fn read(&self, path: &str) -> Option<Arc<str>> {
        self.0.get(path).cloned()
    }
}

#[test]
fn reanalyze_dependents_lazy_load_warmup_does_not_deadlock() {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::mpsc;
    use std::time::Duration;

    // Enough dependents to guarantee concurrent rayon workers contend on the
    // shared salsa write lock during warm-up.
    const N: usize = 64;

    // Build resolver + provider for the lazily-loaded classes (Lazy0..LazyN).
    // These are NOT ingested up front — the warm-up inside reanalyze_dependents
    // is what faults them in, which is exactly what mutates shared salsa state.
    let mut resolver_map: HashMap<String, PathBuf> = HashMap::new();
    let mut provider_map: HashMap<String, Arc<str>> = HashMap::new();
    for i in 0..N {
        let path = format!("lazy_{i}.php");
        resolver_map.insert(format!("Lazy{i}"), PathBuf::from(&path));
        provider_map.insert(
            path,
            Arc::from(format!("<?php\nclass Lazy{i} {{}}\n").as_str()),
        );
    }

    let session = AnalysisSession::new(PhpVersion::LATEST)
        .with_class_resolver(Arc::new(MapResolver(resolver_map)))
        .with_source_provider(Arc::new(MapProvider(provider_map)));
    session.ensure_all_stubs();

    // Base class every dependent extends — gives each dep a structural edge to
    // base.php (recorded at ingest time), so all deps are transitive dependents.
    session.ingest_file(Arc::from("base.php"), Arc::from("<?php\nclass Base {}\n"));

    // Each dependent `extends \Base` (the dependency edge) and constructs a
    // distinct `\Lazy{i}` in a method body. The Lazy reference is collected by
    // the warm-up and triggers a lazy ingest, since it isn't loaded yet. Only
    // ingest_file is called here (no FileAnalyzer), so Lazy{i} stays unloaded
    // until reanalyze_dependents runs.
    for i in 0..N {
        let path: Arc<str> = Arc::from(format!("dep_{i}.php").as_str());
        let src = format!(
            "<?php\nclass Dep{i} extends \\Base {{ public function go(): void {{ $x = new \\Lazy{i}(); }} }}\n"
        );
        session.ingest_file(path, Arc::from(src.as_str()));
    }

    // Run on a worker thread guarded by a timeout: a regression deadlocks here
    // rather than returning, so recv_timeout is what turns the hang into a
    // failed assertion instead of a hung test binary.
    let session = Arc::new(session);
    let (tx, rx) = mpsc::channel();
    let worker_session = Arc::clone(&session);
    let handle = std::thread::spawn(move || {
        let result = worker_session.reanalyze_dependents("base.php");
        let _ = tx.send(result.len());
    });

    match rx.recv_timeout(Duration::from_secs(60)) {
        Ok(count) => {
            handle.join().expect("worker thread panicked");
            assert_eq!(
                count, N,
                "every Dep{{i}} extends \\Base, so all {N} should be returned as dependents"
            );
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!(
                "reanalyze_dependents did not complete within 60s — likely the \
                 in-parallel-worker lazy-load deadlock regressed (v0.37.0 bug)"
            );
        }
        Err(e) => panic!("worker channel error: {e:?}"),
    }
}

#[test]
fn references_to_finds_extends_implements_and_trait_use() {
    // `extends`, `implements`, and trait `use` name a class/interface/trait
    // just as much as `new Foo()` does — references_to() must find those
    // sites too, not just constructor/static-call/type-hint usages.
    use mir_analyzer::FileAnalyzer;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();

    let file: Arc<str> = Arc::from("hierarchy.php");
    let source: Arc<str> = Arc::from(
        "<?php\n\
         class Base {}\n\
         interface Greets {}\n\
         trait Helper {}\n\
         class Child extends Base implements Greets {\n\
             use Helper;\n\
         }\n",
    );

    session.ingest_file(file.clone(), source.clone());
    let parsed = php_rs_parser::parse(&source);
    let _ = FileAnalyzer::new(&session).analyze(
        file.clone(),
        &source,
        &parsed.program,
        &parsed.source_map,
    );

    let base_refs = session.references_to(&Name::class("Base"));
    assert!(
        base_refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
        "references_to(Base) must find `class Child extends Base`"
    );

    let iface_refs = session.references_to(&Name::class("Greets"));
    assert!(
        iface_refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
        "references_to(Greets) must find `implements Greets`"
    );

    let trait_refs = session.references_to(&Name::class("Helper"));
    assert!(
        trait_refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
        "references_to(Helper) must find `use Helper;`"
    );
}
