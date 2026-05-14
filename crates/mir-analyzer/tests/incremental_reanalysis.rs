// Integration tests for incremental single-file re-analysis (mir#79).

mod common;

use std::sync::Arc;

use mir_analyzer::ProjectAnalyzer;

use self::common::{create_temp_dir, write_file};

#[test]
fn re_analyze_file_picks_up_new_error() {
    let src_dir = create_temp_dir("test");

    // Initial file: valid code, no issues expected for undefined functions
    let file_a = write_file(
        &src_dir,
        "A.php",
        "<?php\nfunction greet(): string { return 'hello'; }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    let result1 = analyzer.analyze(std::slice::from_ref(&file_a));

    // The initial code should have no UndefinedFunction issues
    let undef_fn_count = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert_eq!(
        undef_fn_count, 0,
        "initial code should have no UndefinedFunction"
    );

    // Now re-analyze the same file with content that calls an undefined function
    let file_path = file_a.to_string_lossy().to_string();
    let new_content = "<?php\nfunction test(): void { nonexistent_func(); }\n";
    let result2 = analyzer.re_analyze_file(&file_path, new_content);

    let undef_fn_count2 = result2
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert!(
        undef_fn_count2 > 0,
        "re-analyzed code should report UndefinedFunction, got issues: {:?}",
        result2
            .issues
            .iter()
            .map(|i| i.kind.name())
            .collect::<Vec<_>>()
    );
}

#[test]
fn re_analyze_file_removes_old_definitions() {
    let src_dir = create_temp_dir("test");

    // Initial: defines class Foo with method bar()
    let file_a = write_file(
        &src_dir,
        "A.php",
        "<?php\nclass Foo { public function bar(): void {} }\n",
    );
    let file_b = write_file(
        &src_dir,
        "B.php",
        "<?php\nfunction test(): void { $f = new Foo(); $f->bar(); }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    let result1 = analyzer.analyze(&[file_a.clone(), file_b.clone()]);

    // bar() exists, so no UndefinedMethod on file B
    let undef_method = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .count();
    assert_eq!(undef_method, 0, "bar() should be found");

    // Now change A.php: rename the method from bar() to baz()
    let file_path_a = file_a.to_string_lossy().to_string();
    let new_content_a = "<?php\nclass Foo { public function baz(): void {} }\n";
    let _result2 = analyzer.re_analyze_file(&file_path_a, new_content_a);

    // Verify the old method bar() is gone and baz() exists
    assert!(
        analyzer.contains_method("Foo", "baz"),
        "baz() should exist after re-analysis"
    );
    assert!(
        !analyzer.contains_method("Foo", "bar"),
        "bar() should be removed after re-analysis"
    );
}

#[test]
fn re_analyze_file_fixes_error() {
    let src_dir = create_temp_dir("test");

    // Initial: code with a call to an undefined function
    let file_a = write_file(
        &src_dir,
        "A.php",
        "<?php\nfunction test(): void { missing_fn(); }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    let result1 = analyzer.analyze(std::slice::from_ref(&file_a));

    let undef_count = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert!(undef_count > 0, "should have UndefinedFunction initially");

    // Fix the file: define the function and call it
    let file_path = file_a.to_string_lossy().to_string();
    let new_content =
        "<?php\nfunction missing_fn(): void {}\nfunction test(): void { missing_fn(); }\n";
    let result2 = analyzer.re_analyze_file(&file_path, new_content);

    let undef_count2 = result2
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert_eq!(undef_count2, 0, "after fix, no UndefinedFunction expected");
}

/// Verify that `re_analyze_file` takes the content-hash fast path when the
/// cache already holds a valid entry for the unchanged content.
///
/// Strategy: after the initial analysis caches an `UndefinedFunction` issue,
/// we manually insert the "missing" function into the codebase so that a slow-
/// path re-analysis would find it and return *no* issues.  Re-analyzing with
/// the same content then lets us distinguish the two paths:
/// - fast path (cache hit)  → cached `UndefinedFunction` issue still returned
/// - slow path (re-analyze) → no issue (function now exists in codebase)
#[test]
fn re_analyze_file_uses_cache_on_unchanged_content() {
    let src_dir = create_temp_dir("test");
    let cache_dir = create_temp_dir("cache");

    // Content that calls an undefined function → produces UndefinedFunction
    let content = "<?php\nfunction test(): void { ghost_fn(); }\n";
    let file_a = write_file(&src_dir, "A.php", content);
    let file_path = file_a.to_string_lossy().to_string();

    let analyzer = ProjectAnalyzer::with_cache(cache_dir.path());
    let result1 = analyzer.analyze(std::slice::from_ref(&file_a));

    let undef_count = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert!(
        undef_count > 0,
        "initial analysis should report UndefinedFunction"
    );

    // Insert ghost_fn() into the salsa db so a slow-path re-analysis would
    // find it and produce no issues.
    {
        let mut guard = analyzer.salsa_db_for_test().lock();
        let db = &mut *guard;
        db.upsert_function_node(&mir_codebase::FunctionStorage {
            fqn: Arc::from("ghost_fn"),
            short_name: Arc::from("ghost_fn"),
            params: Arc::from([].as_slice()),
            return_type: None,
            inferred_return_type: None,
            template_params: vec![],
            assertions: vec![],
            throws: vec![],
            deprecated: None,
            is_pure: false,
            location: None,
            docstring: None,
        });
    }

    // Re-analyze with identical content — must hit the cache.
    let result2 = analyzer.re_analyze_file(&file_path, content);

    let undef_count2 = result2
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert_eq!(
        undef_count2, undef_count,
        "cache hit should return the same cached issues; slow-path would return 0 \
         because ghost_fn was inserted into the codebase"
    );
}

/// After re_analyze_file, file_imports and file_namespaces must be restored so
/// that use-alias resolution still works on the re-analyzed file.
///
/// Mechanism: re_analyze_file calls remove_file_definitions, which clears both
/// maps for the re-analyzed file, then calls DefinitionCollector::collect →
/// inject_stub_slice. inject_stub_slice is now the sole write path that
/// repopulates file_namespaces and file_imports (via StubSlice::namespace and
/// StubSlice::imports). If either field is missing from the slice, the maps stay
/// empty after re-analysis and StatementsAnalyzer emits false UndefinedClass
/// diagnostics for `use`-aliased classes (`new Entity()`, `catch (Entity $e)`,
/// type hints, etc.).
#[test]
fn re_analyze_preserves_namespace_and_use_alias_resolution() {
    let src_dir = create_temp_dir("test");

    // Entity lives in App\Model.
    let _entity = write_file(
        &src_dir,
        "Entity.php",
        "<?php\nnamespace App\\Model;\nclass Entity {}\n",
    );

    // Handler is in App\Service and imports Entity via `use`.
    let handler_src = "<?php\nnamespace App\\Service;\nuse App\\Model\\Entity;\n\
        function handle(): void { $e = new Entity(); }\n";
    let handler = write_file(&src_dir, "Handler.php", handler_src);

    let analyzer = ProjectAnalyzer::new();
    let result1 = analyzer.analyze(&[src_dir.path().join("Entity.php"), handler.clone()]);

    let undef1: Vec<_> = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .collect();
    assert!(
        undef1.is_empty(),
        "initial analysis must not report UndefinedClass; got: {undef1:?}"
    );

    // Re-analyze Handler.php with a trivial body change (adds a comment).
    let handler_src2 = "<?php\nnamespace App\\Service;\nuse App\\Model\\Entity;\n\
        function handle(): void { $e = new Entity(); /* re-analyzed */ }\n";
    let result2 = analyzer.re_analyze_file(handler.to_string_lossy().as_ref(), handler_src2);

    let undef2: Vec<_> = result2
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .collect();
    assert!(
        undef2.is_empty(),
        "re_analyze_file must not produce false UndefinedClass after namespace/import restoration; \
         got: {undef2:?}"
    );
}

/// `re_analyze_file` must prime inferred return types before the issue-emitting
/// pass so that within-file cross-function calls see the correct return type.
///
/// Without the priming sweep, `bar()` (no return type hint) gets
/// `inferred_return_type = None` after `inject_stub_slice` replaces the
/// definition. The call site then falls back to `mixed`, causing a false
/// `InvalidReturnType` for `foo(): string { return bar(); }`.
#[test]
fn re_analyze_file_primes_inferred_return_type_for_same_file_calls() {
    let src_dir = create_temp_dir("test");

    // bar() has no return type hint; its return type must be inferred.
    // foo() has an explicit `: string` return type and delegates to bar().
    let content =
        "<?php\nfunction bar() { return 'hello'; }\nfunction foo(): string { return bar(); }\n";
    let file = write_file(&src_dir, "A.php", content);
    let file_path = file.to_string_lossy().to_string();

    let analyzer = ProjectAnalyzer::new();
    let result1 = analyzer.analyze(std::slice::from_ref(&file));

    let issues1: Vec<_> = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "InvalidReturnType")
        .collect();
    assert!(
        issues1.is_empty(),
        "initial analysis must not report InvalidReturnType; got: {issues1:?}"
    );

    // Re-analyze the same file with a trivial body change.  The priming sweep
    // must repopulate bar.inferred_return_type before foo is analyzed.
    let content2 = "<?php\nfunction bar() { return 'hello'; }\nfunction foo(): string { return bar(); /* re-analyzed */ }\n";
    let result2 = analyzer.re_analyze_file(&file_path, content2);

    let issues2: Vec<_> = result2
        .issues
        .iter()
        .filter(|i| i.kind.name() == "InvalidReturnType")
        .collect();
    assert!(
        issues2.is_empty(),
        "re_analyze_file must not report false InvalidReturnType after body-only change; \
         got: {issues2:?}"
    );
}
