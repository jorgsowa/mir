// Cross-file *inferred-type* cache invalidation (follow-up to cache_invalidation.rs).
//
// cache_invalidation.rs covers invalidation through *structural* edges
// (a subclass calling a base method). This file probes the harder class the
// hand-built reverse-dependency graph must also cover: invalidation through
// *inferred* return types that propagate across files via call sites.
//
// Chain under test:  A() calls B(), B() returns C(), and C()'s return type is
// *inferred* (no declared signature). Editing C must re-analyze A, because A's
// view of its own locals depends transitively on C's inferred return type.
//
// The signal is a `@mir-check` directive in A: it emits `TypeCheckMismatch`
// during body analysis, so it rides the normal issue cache/replay path. A stale
// cache hit on A would silently serve the pre-edit (correct) result and the
// expected mismatch would never surface.

mod common;

use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

use self::common::{create_temp_dir, write_file};

fn type_check_mismatches(result: &mir_analyzer::AnalysisResult) -> usize {
    result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "TypeCheckMismatch")
        .count()
}

#[test]
fn transitive_inferred_return_invalidation() {
    let src_dir = create_temp_dir("inferred_invalidation: source");
    let cache_dir = create_temp_dir("inferred_invalidation: cache");

    // C: inferred return is `int` initially.
    let c = write_file(
        &src_dir,
        "C.php",
        "<?php\nfunction c_val() {\n    return 42;\n}\n",
    );
    // B: forwards C's inferred return (no declared return type of its own).
    let b = write_file(
        &src_dir,
        "B.php",
        "<?php\nfunction b_val() {\n    return c_val();\n}\n",
    );
    // A: pins the expected type of the transitively-inferred value.
    let a = write_file(
        &src_dir,
        "A.php",
        "<?php\nfunction a_test(): void {\n    $x = b_val();\n    /** @mir-check $x is int */\n    echo $x;\n}\n",
    );

    let files = [a.clone(), b.clone(), c.clone()];

    // --- Run 1: $x is int, check passes -> no mismatch. -----------------------
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result1 = session.analyze_paths(&files, &BatchOptions::new());
    assert_eq!(
        type_check_mismatches(&result1),
        0,
        "run 1: cross-file inferred return should resolve to int (got mismatches: {:#?})",
        result1
            .issues
            .iter()
            .filter(|i| i.kind.name() == "TypeCheckMismatch")
            .collect::<Vec<_>>()
    );

    // --- Edit ONLY C: inferred return becomes string. ------------------------
    write_file(
        &src_dir,
        "C.php",
        "<?php\nfunction c_val() {\n    return \"str\";\n}\n",
    );

    // --- Run 2: fresh session, same cache (simulates a new CLI invocation). ---
    // A.php is byte-identical, so its content-hash entry is a cache hit unless
    // the reverse-dep graph evicts it as a transitive dependent of C.
    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result2 = session2.analyze_paths(&files, &BatchOptions::new());

    assert_eq!(
        type_check_mismatches(&result2),
        1,
        "run 2: editing C changes b_val()'s inferred return to string; A must be \
         re-analyzed and report the @mir-check mismatch. A count of 0 means A was \
         served a STALE cache hit (reverse-dep graph missed the inferred edge)."
    );
}

#[test]
fn transitive_inferred_return_invalidation_via_methods() {
    // Same chain, but expressed with classes/methods so that B's body contains a
    // strong structural reference to C (`new C()`). This isolates whether the gap
    // is specific to free-function call sites or affects inferred returns generally.
    let src_dir = create_temp_dir("inferred_invalidation_methods: source");
    let cache_dir = create_temp_dir("inferred_invalidation_methods: cache");

    let c = write_file(
        &src_dir,
        "C.php",
        "<?php\nclass C {\n    public function val() {\n        return 42;\n    }\n}\n",
    );
    let b = write_file(
        &src_dir,
        "B.php",
        "<?php\nclass B {\n    public function val() {\n        return (new C())->val();\n    }\n}\n",
    );
    let a = write_file(
        &src_dir,
        "A.php",
        "<?php\nclass A {\n    public function test(): void {\n        $x = (new B())->val();\n        /** @mir-check $x is int */\n        echo $x;\n    }\n}\n",
    );

    let files = [a.clone(), b.clone(), c.clone()];

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result1 = session.analyze_paths(&files, &BatchOptions::new());
    assert_eq!(
        type_check_mismatches(&result1),
        0,
        "run 1 (methods): cross-file inferred method return should resolve to int"
    );

    write_file(
        &src_dir,
        "C.php",
        "<?php\nclass C {\n    public function val() {\n        return \"str\";\n    }\n}\n",
    );

    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result2 = session2.analyze_paths(&files, &BatchOptions::new());
    assert_eq!(
        type_check_mismatches(&result2),
        1,
        "run 2 (methods): editing C must re-analyze A via the structural `new C()` edge"
    );
}

#[test]
fn transitive_inferred_return_invalidation_via_trait() {
    // Mixed edge chain: A -> (trait use) -> T -> (call ref-loc) -> C. Exercises the
    // structural trait edge composed with an inferred-return ref-loc edge.
    let src_dir = create_temp_dir("inferred_invalidation_trait: source");
    let cache_dir = create_temp_dir("inferred_invalidation_trait: cache");

    let c = write_file(
        &src_dir,
        "C.php",
        "<?php\nfunction c_v() {\n    return 42;\n}\n",
    );
    let t = write_file(
        &src_dir,
        "T.php",
        "<?php\ntrait T {\n    public function tv() {\n        return c_v();\n    }\n}\n",
    );
    let a = write_file(
        &src_dir,
        "A.php",
        "<?php\nclass A {\n    use T;\n    public function test(): void {\n        $x = $this->tv();\n        /** @mir-check $x is int */\n        echo $x;\n    }\n}\n",
    );

    let files = [a.clone(), t.clone(), c.clone()];

    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result1 = session.analyze_paths(&files, &BatchOptions::new());
    assert_eq!(
        type_check_mismatches(&result1),
        0,
        "run 1 (trait): inferred return through a trait method should resolve to int"
    );

    write_file(
        &src_dir,
        "C.php",
        "<?php\nfunction c_v() {\n    return \"str\";\n}\n",
    );

    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result2 = session2.analyze_paths(&files, &BatchOptions::new());
    assert_eq!(
        type_check_mismatches(&result2),
        1,
        "run 2 (trait): editing C must re-analyze A through the T trait + c_v() edges"
    );
}

#[test]
fn deleting_a_dependency_file_invalidates_dependents() {
    // A calls c_val() (defined in C) and pins the inferred type of the result.
    // After the first run, C.php is DELETED from disk and the second run is
    // invoked with only [A] (C is no longer globbed). A is byte-identical, so it
    // is a cache hit unless the now-missing C is treated as a change and its
    // dependents are evicted. With C gone, c_val() is unresolved, A's `$x` is no
    // longer `int`, and the `@mir-check` must flip to a mismatch.
    let src_dir = create_temp_dir("delete_dependency: source");
    let cache_dir = create_temp_dir("delete_dependency: cache");

    let c = write_file(
        &src_dir,
        "C.php",
        "<?php\nfunction c_val() {\n    return 42;\n}\n",
    );
    let a = write_file(
        &src_dir,
        "A.php",
        "<?php\nfunction a_test(): void {\n    $x = c_val();\n    /** @mir-check $x is int */\n    echo $x;\n}\n",
    );

    // Baseline: analyzing A *alone* (C absent), with a brand-new cache, must
    // produce the mismatch. This proves the signal fires absent any caching, so
    // a count of 0 in run 2 can only mean a stale cache hit.
    {
        let baseline_cache = create_temp_dir("delete_dependency: baseline cache");
        let baseline = AnalysisSession::new(PhpVersion::LATEST)
            .with_cache_dir(baseline_cache.path())
            .analyze_paths(std::slice::from_ref(&a), &BatchOptions::new());
        assert!(
            type_check_mismatches(&baseline) >= 1,
            "baseline: with C absent, c_val() is unresolved and the @mir-check must \
             flag a mismatch (signal sanity check)"
        );
    }

    // Run 1: C present, $x is int, check passes.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result1 = session.analyze_paths(&[a.clone(), c.clone()], &BatchOptions::new());
    assert_eq!(
        type_check_mismatches(&result1),
        0,
        "run 1: C present, $x resolves to int"
    );

    // Delete C.php and re-run with only A in the path set.
    std::fs::remove_file(&c).unwrap();

    let session2 = AnalysisSession::new(PhpVersion::LATEST).with_cache_dir(cache_dir.path());
    let result2 = session2.analyze_paths(std::slice::from_ref(&a), &BatchOptions::new());
    assert_eq!(
        type_check_mismatches(&result2),
        1,
        "run 2: C was deleted; A must be re-analyzed. A count of 0 means A was \
         served a STALE cache hit (deleted dependency not invalidated)."
    );
}
