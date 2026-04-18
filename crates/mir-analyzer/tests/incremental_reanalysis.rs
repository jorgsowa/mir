// Integration tests for incremental single-file re-analysis (mir#79).

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
fn re_analyze_file_picks_up_new_error() {
    let src_dir = TempDir::new().unwrap();

    // Initial file: valid code, no issues expected for undefined functions
    let file_a = write(
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
    let src_dir = TempDir::new().unwrap();

    // Initial: defines class Foo with method bar()
    let file_a = write(
        &src_dir,
        "A.php",
        "<?php\nclass Foo { public function bar(): void {} }\n",
    );
    let file_b = write(
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
        analyzer.codebase().get_method("Foo", "baz").is_some(),
        "baz() should exist after re-analysis"
    );
    assert!(
        analyzer.codebase().get_method("Foo", "bar").is_none(),
        "bar() should be removed after re-analysis"
    );
}

#[test]
fn re_analyze_file_fixes_error() {
    let src_dir = TempDir::new().unwrap();

    // Initial: code with a call to an undefined function
    let file_a = write(
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

/// Verify that `re_analyze_file` skips `finalize()` when only method bodies change.
///
/// Strategy: after the initial analysis (which populates `all_parents` for every
/// class), we manually insert a new class `C extends A` into the codebase with
/// `all_parents = []`.  A full re-analysis of `A.php` with a body-only edit would
/// call `finalize()`, which would walk the hierarchy and set `C::all_parents = [A]`.
/// The structural-snapshot fast path skips `finalize()`, so `all_parents` stays
/// empty — proving the skip was taken.
#[test]
fn re_analyze_file_skips_finalize_on_body_only_change() {
    let src_dir = TempDir::new().unwrap();

    let file_a = write(
        &src_dir,
        "A.php",
        "<?php\nclass A { public function foo(): void {} }\n",
    );
    let file_path = file_a.to_string_lossy().to_string();

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(std::slice::from_ref(&file_a));

    // Insert class C that extends A, but leave all_parents empty.
    // A slow-path finalize() would populate it to [A]; the fast path skips finalize.
    analyzer.codebase().classes.insert(
        Arc::from("C"),
        mir_codebase::ClassStorage {
            fqcn: Arc::from("C"),
            short_name: Arc::from("C"),
            parent: Some(Arc::from("A")),
            interfaces: vec![],
            traits: vec![],
            own_methods: indexmap::IndexMap::new(),
            own_properties: indexmap::IndexMap::new(),
            own_constants: indexmap::IndexMap::new(),
            template_params: vec![],
            is_abstract: false,
            is_final: false,
            is_readonly: false,
            all_parents: vec![],
            is_deprecated: false,
            is_internal: false,
            location: None,
        },
    );

    // Re-analyze A.php with a body-only change (same class signature, new method body).
    let new_content = "<?php\nclass A { public function foo(): int { return 1; } }\n";
    analyzer.re_analyze_file(&file_path, new_content);

    // Fast path: finalize() was skipped, so C::all_parents is still empty.
    // Slow path: finalize() would have set C::all_parents = [A].
    let c_all_parents = analyzer
        .codebase()
        .classes
        .get("C")
        .map(|c| c.all_parents.clone())
        .unwrap_or_default();
    assert!(
        c_all_parents.is_empty(),
        "finalize() should have been skipped for a body-only change; \
         C::all_parents should still be [] but got {:?}",
        c_all_parents
    );
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
    let src_dir = TempDir::new().unwrap();
    let cache_dir = TempDir::new().unwrap();

    // Content that calls an undefined function → produces UndefinedFunction
    let content = "<?php\nfunction test(): void { ghost_fn(); }\n";
    let file_a = write(&src_dir, "A.php", content);
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

    // Insert ghost_fn() into the codebase so a slow-path re-analysis would
    // find it and produce no issues.
    analyzer.codebase().functions.insert(
        Arc::from("ghost_fn"),
        mir_codebase::FunctionStorage {
            fqn: Arc::from("ghost_fn"),
            short_name: Arc::from("ghost_fn"),
            params: vec![],
            return_type: None,
            inferred_return_type: None,
            template_params: vec![],
            assertions: vec![],
            throws: vec![],
            is_deprecated: false,
            is_pure: false,
            location: None,
        },
    );

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
