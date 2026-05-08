//! Smoke tests for the session-based per-file analysis API.
//!
//! These verify the core invariants of `AnalysisSession` + `FileAnalyzer`:
//!   * trait method bodies are analyzed,
//!   * stubs are loaded lazily on first ingest/analyze,
//!   * concurrent reads can take cheap snapshots while edits proceed.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion};

fn parse_and_analyze(source: &str) -> mir_analyzer::FileAnalysis {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("<test>");
    session.ingest_file(file.clone(), Arc::from(source));

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, source);
    assert!(
        parsed.errors.is_empty(),
        "parser errors in test source: {:?}",
        parsed.errors
    );

    FileAnalyzer::new(&session).analyze(file, source, &parsed.program, &parsed.source_map)
}

/// Trait method bodies must be analyzed. `StatementsAnalyzer` (the layer
/// some external consumers were forced to use) skips traits; `FileAnalyzer`
/// goes through `Pass2Driver`, which walks them. Regression guard for the
/// hidden-trait-bug class.
#[test]
fn file_analyzer_walks_trait_method_bodies() {
    let src = "<?php
trait Greeter {
    public function greet(): string {
        return totally_undefined_function();
    }
}
";
    let result = parse_and_analyze(src);
    let has_undefined_fn = result
        .issues
        .iter()
        .any(|i| i.kind.name() == "UndefinedFunction");
    assert!(
        has_undefined_fn,
        "FileAnalyzer must walk trait method bodies; missed UndefinedFunction in trait. \
         Issues: {:?}",
        result
            .issues
            .iter()
            .map(|i| i.kind.name())
            .collect::<Vec<_>>()
    );
}

/// Plain function bodies in a single file must analyze cleanly.
#[test]
fn file_analyzer_analyzes_function_body() {
    let src = "<?php
function greet(): string {
    return 'hello';
}
";
    let result = parse_and_analyze(src);
    let problem = result
        .issues
        .iter()
        .find(|i| i.severity == mir_analyzer::Severity::Error);
    assert!(
        problem.is_none(),
        "no errors expected for valid code; got: {:?}",
        result
            .issues
            .iter()
            .map(|i| i.kind.name())
            .collect::<Vec<_>>()
    );
}

/// `ensure_stubs_loaded` is idempotent; calling it many times must be cheap
/// and must not double-load stubs (would corrupt the codebase).
#[test]
fn ensure_stubs_loaded_is_idempotent() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_stubs_loaded();
    session.ensure_stubs_loaded();
    session.ensure_stubs_loaded();

    // After loading, a built-in like strlen() should be known.
    let known = session.read(|db| db.lookup_function_node("strlen").is_some());
    assert!(known, "strlen() must be loaded after ensure_stubs_loaded");
}

/// Essential-only loading covers Core / standard / SPL / date but skips
/// less-common extensions like gd. The skipped stubs are loadable on demand.
#[test]
fn essential_stubs_loaded_count_is_smaller_than_full_set() {
    let essential = AnalysisSession::new(PhpVersion::LATEST);
    essential.ensure_essential_stubs_loaded();
    let essential_count = essential.loaded_stub_count();

    let full = AnalysisSession::new(PhpVersion::LATEST);
    full.ensure_all_stubs_loaded();
    let full_count = full.loaded_stub_count();

    assert!(
        essential_count < full_count,
        "essentials ({essential_count}) should be a strict subset of all stubs ({full_count})"
    );
    // The curated essentials list is 25 of ~120 files; sanity-bound the ratio.
    assert!(
        essential_count * 3 < full_count,
        "essentials ({essential_count}) should be a small fraction of all stubs ({full_count}); \
         if this asserts the curated list grew unintentionally"
    );

    // Both must cover universally-used built-ins.
    for name in ["strlen", "array_map", "count"] {
        let has = essential.read(|db| db.lookup_function_node(name).is_some());
        assert!(has, "essentials must define {name}()");
    }
}

/// `ensure_stub_for_function` lazily loads exactly the stub containing the
/// requested function — no more, no less. After essentials, the gd extension
/// is unloaded; requesting `imagecreate` brings in the gd stub on demand.
#[test]
fn ensure_stub_for_function_lazy_loads_extension() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();
    let baseline = session.loaded_stub_count();

    // gd is not part of the essentials.
    let imagecreate_known = session.read(|db| db.lookup_function_node("imagecreate").is_some());
    assert!(
        !imagecreate_known,
        "imagecreate() must not be loaded after essentials-only init"
    );

    let was_known = session.ensure_stub_for_function("imagecreate");
    assert!(was_known, "imagecreate() must be a recognized PHP built-in");

    let after = session.loaded_stub_count();
    assert!(
        after > baseline,
        "ensure_stub_for_function must ingest at least one new stub"
    );

    let imagecreate_now_known = session.read(|db| db.lookup_function_node("imagecreate").is_some());
    assert!(
        imagecreate_now_known,
        "imagecreate() must be loaded after ensure_stub_for_function"
    );
}

/// `FileAnalyzer::analyze` must auto-discover and lazy-load the extension
/// stubs that the file references — without callers having to enumerate them.
/// Headline test for the lazy-stub UX: a fresh session that touches gd, json,
/// and Reflection should not produce false `UndefinedFunction` /
/// `UndefinedClass` errors.
#[test]
fn file_analyzer_auto_discovers_extension_stubs() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/uses_extensions.php");
    let src = "<?php
function pixel(): int {
    $img = imagecreate(10, 10);
    return imagecolorat($img, 0, 0);
}
function ref(string $cls): \\ReflectionClass {
    return new \\ReflectionClass($cls);
}
function encode(array $data): string {
    return json_encode($data);
}
";
    session.ingest_file(file.clone(), Arc::from(src));

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, src);
    assert!(parsed.errors.is_empty());

    let analysis =
        FileAnalyzer::new(&session).analyze(file, src, &parsed.program, &parsed.source_map);

    let undefined: Vec<_> = analysis
        .issues
        .iter()
        .filter(|i| {
            matches!(
                i.kind.name(),
                "UndefinedFunction" | "UndefinedClass" | "UndefinedConstant"
            )
        })
        .map(|i| i.kind.name())
        .collect();
    assert!(
        undefined.is_empty(),
        "auto-discovery must lazy-load extension stubs (gd, Reflection, json) so no \
         Undefined* diagnostics fire; got: {undefined:?}"
    );

    // Sanity: stubs beyond the curated essentials must have been pulled in.
    // Essentials are 25 stub files; auto-discovery here loads at least gd,
    // Reflection, and json on top.
    let count = session.loaded_stub_count();
    assert!(
        count > 25,
        "expected more than just essentials (25) to be loaded; got {count}"
    );
}

/// Go-to-definition flow: find a symbol at the cursor, then resolve its
/// declaration location. Verifies that `FileAnalysis::symbol_at` and
/// `AnalysisSession::definition_of` compose into the expected end-to-end
/// behavior.
#[test]
fn definition_of_resolves_class_declaration_via_session() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/decls.php");
    let src = "<?php
class Greeter {
    public function greet(): string { return 'hi'; }
}
function build(): Greeter { return new Greeter(); }
";
    session.ingest_file(file.clone(), Arc::from(src));

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, src);
    let analysis =
        FileAnalyzer::new(&session).analyze(file.clone(), src, &parsed.program, &parsed.source_map);

    // Resolve "Greeter" by name — caller doesn't need to know its position.
    let loc = session
        .definition_of("Greeter")
        .expect("Greeter must resolve");
    assert_eq!(loc.file.as_ref(), file.as_ref());
    assert!(loc.line >= 1, "expected a real source line; got {loc:?}");

    // Member resolution.
    let greet_loc = session.member_definition("Greeter", "greet");
    assert!(greet_loc.is_some(), "Greeter::greet() must resolve");

    // Sanity: at least one ClassReference symbol got recorded so symbol_at
    // is wired through the pipeline.
    let any_class_ref = analysis.symbols.iter().any(|s| {
        matches!(
            s.kind,
            mir_analyzer::SymbolKind::ClassReference(_) | mir_analyzer::SymbolKind::FunctionCall(_)
        )
    });
    assert!(any_class_ref, "expected at least one resolved symbol");
}

/// `document_symbols` powers the editor outline view. Must list every top-
/// level declaration in the file with its kind.
#[test]
fn document_symbols_lists_file_declarations() {
    use mir_analyzer::DocumentSymbolKind;

    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/outline.php");
    let src = "<?php
class Cat { public function meow(): void {} }
interface Animal { public function name(): string; }
trait Furry { public function shed(): void {} }
function pet_count(): int { return 0; }
";
    session.ingest_file(file.clone(), Arc::from(src));

    let symbols = session.document_symbols(file.as_ref());

    let by_name: std::collections::HashMap<&str, DocumentSymbolKind> =
        symbols.iter().map(|s| (s.name.as_ref(), s.kind)).collect();

    assert_eq!(by_name.get("Cat"), Some(&DocumentSymbolKind::Class));
    assert_eq!(by_name.get("Animal"), Some(&DocumentSymbolKind::Interface));
    assert_eq!(by_name.get("Furry"), Some(&DocumentSymbolKind::Trait));
    assert_eq!(
        by_name.get("pet_count"),
        Some(&DocumentSymbolKind::Function)
    );
}

/// `references_to` returns every recorded use of a symbol after Pass 2.
#[test]
fn references_to_returns_recorded_call_sites() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/refs.php");
    let src = "<?php
function helper(): string { return 'a'; }
function caller(): string { return helper(); }
";
    session.ingest_file(file.clone(), Arc::from(src));

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, src);
    let _ =
        FileAnalyzer::new(&session).analyze(file.clone(), src, &parsed.program, &parsed.source_map);

    let refs = session.references_to("helper");
    assert!(
        refs.iter().any(|(f, _, _, _)| f.as_ref() == file.as_ref()),
        "helper() must have at least one reference recorded in {file}; got {refs:?}"
    );
}

/// `FileAnalysis::symbol_at` finds the symbol at a cursor byte offset.
/// Used by editors to map (line, column) → resolved symbol → definition /
/// hover info.
#[test]
fn file_analysis_symbol_at_finds_call_site() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/sym_at.php");
    // The call to `target()` is at byte offset 26 in the source (within the
    // `target()` identifier).
    let src = "<?php
target(); function target(): void {}
";
    session.ingest_file(file.clone(), Arc::from(src));

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, src);
    let analysis =
        FileAnalyzer::new(&session).analyze(file, src, &parsed.program, &parsed.source_map);

    // Find an offset inside the `target` call. The call is on line 2, before
    // the `function` keyword.
    let call_offset = src.find("target()").unwrap() as u32 + 1;
    let resolved = analysis
        .symbol_at(call_offset)
        .expect("expected a resolved symbol at the call site");
    assert!(
        matches!(resolved.kind, mir_analyzer::SymbolKind::FunctionCall(_)),
        "expected FunctionCall kind; got {:?}",
        resolved.kind
    );
}

/// Unknown names return `false` and do not spuriously ingest anything.
#[test]
fn ensure_stub_for_unknown_symbol_returns_false() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_essential_stubs_loaded();
    let before = session.loaded_stub_count();

    assert!(!session.ensure_stub_for_function("definitely_not_a_php_builtin_xyz123"));
    assert!(!session.ensure_stub_for_class("\\Not\\A\\Real\\Class"));

    assert_eq!(
        session.loaded_stub_count(),
        before,
        "unknown lookups must not ingest any stubs"
    );
}

/// `snapshot_db()` must return a usable clone after Pass 1 ingest.
#[test]
fn snapshot_db_observes_ingested_definitions() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(Arc::from("<test>"), Arc::from("<?php\nclass Foo {}\n"));

    let class_known = session.read(|db| db.lookup_class_node("Foo").is_some());
    assert!(
        class_known,
        "snapshot_db() must observe definitions ingested via ingest_file"
    );
}

/// `FileAnalyzer::analyze` deliberately skips inference, so calls to a
/// no-hint function fall back to `mixed` until `run_inference_sweep` runs.
/// This documents and verifies the explicit two-step incremental flow:
///   1. analyze on edit → fast, may flag false-positive InvalidReturnType
///   2. inference sweep on idle → primes inferred return types
///   3. re-analyze → false positive disappears
#[test]
fn run_inference_sweep_primes_return_types_for_subsequent_analysis() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/A.php");
    let src = "<?php
function bar() { return 'hello'; }
function foo(): string { return bar(); }
";
    session.ingest_file(file.clone(), Arc::from(src));

    // Run sweep so bar()'s inferred return type lands in the canonical db.
    session.run_inference_sweep(&[(file.clone(), Arc::from(src))]);

    let arena = bumpalo::Bump::new();
    let parsed = php_rs_parser::parse(&arena, src);
    assert!(parsed.errors.is_empty());

    let analysis =
        FileAnalyzer::new(&session).analyze(file, src, &parsed.program, &parsed.source_map);

    let invalid_return = analysis
        .issues
        .iter()
        .filter(|i| i.kind.name() == "InvalidReturnType")
        .count();
    assert_eq!(
        invalid_return,
        0,
        "inference sweep must prime bar()'s return type so foo(): string is OK; got issues: {:?}",
        analysis
            .issues
            .iter()
            .map(|i| i.kind.name())
            .collect::<Vec<_>>()
    );
}

/// `invalidate_file` must fully drop the file's contributions: salsa input
/// handle, codebase definitions, reference locations, and reverse-dep
/// outgoing edges. Long-running sessions rely on this for bounded memory
/// when files are closed.
#[test]
fn invalidate_file_releases_all_per_file_state() {
    use mir_analyzer::cache::AnalysisCache;
    use tempfile::TempDir;

    let cache_dir = TempDir::new().unwrap();
    let cache = Arc::new(AnalysisCache::open(cache_dir.path()));
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache.clone());

    let base: Arc<str> = Arc::from("/proj/Base.php");
    let child: Arc<str> = Arc::from("/proj/Child.php");

    session.ingest_file(base.clone(), Arc::from("<?php\nclass Base {}\n"));
    session.ingest_file(
        child.clone(),
        Arc::from("<?php\nclass Child extends Base {}\n"),
    );
    cache.put(base.as_ref(), "h1".to_string(), Vec::new(), Vec::new());
    cache.put(child.as_ref(), "h2".to_string(), Vec::new(), Vec::new());
    assert_eq!(session.tracked_file_count(), 2);

    session.invalidate_file(child.as_ref());

    assert_eq!(
        session.tracked_file_count(),
        1,
        "salsa input handle for Child must be released after invalidate"
    );
    let child_active =
        session.read(|db| db.lookup_class_node("Child").is_some_and(|n| n.active(db)));
    assert!(
        !child_active,
        "Child class node must be inactive after invalidate"
    );

    // Re-evict from Base to confirm Child is no longer a dependent of Base
    // (its outgoing edge to Base must have been dropped on invalidate).
    cache.put(child.as_ref(), "h3".to_string(), Vec::new(), Vec::new());
    let evicted = cache.evict_with_dependents(&[base.as_ref().to_string()]);
    assert_eq!(
        evicted, 0,
        "after invalidate, Child must no longer be a dependent of Base; got {evicted} evictions"
    );
}

/// Long-running sessions must not accumulate stale reference locations
/// when a file is re-ingested with different content. Re-ingesting `f.php`
/// with a body that references `bar()` instead of `foo()` must leave no
/// trace of the original `foo()` reference in `f.php`.
#[test]
fn re_ingesting_a_file_drops_its_stale_reference_locations() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/use_funcs.php");

    let v1 = "<?php
function foo() {}
function bar() {}
function caller_v1() { foo(); }
";
    session.ingest_file(file.clone(), Arc::from(v1));
    {
        let arena = bumpalo::Bump::new();
        let parsed = php_rs_parser::parse(&arena, v1);
        FileAnalyzer::new(&session).analyze(file.clone(), v1, &parsed.program, &parsed.source_map);
    }

    let foo_refs_v1 = session.read(|db| db.reference_locations("foo"));
    assert!(
        foo_refs_v1
            .iter()
            .any(|(f, _, _, _)| f.as_ref() == file.as_ref()),
        "v1 must record a foo() call from {file}; got {foo_refs_v1:?}"
    );

    // Re-ingest with foo() call removed; bar() called instead.
    let v2 = "<?php
function foo() {}
function bar() {}
function caller_v2() { bar(); }
";
    session.ingest_file(file.clone(), Arc::from(v2));
    {
        let arena = bumpalo::Bump::new();
        let parsed = php_rs_parser::parse(&arena, v2);
        FileAnalyzer::new(&session).analyze(file.clone(), v2, &parsed.program, &parsed.source_map);
    }

    let foo_refs_v2 = session.read(|db| db.reference_locations("foo"));
    assert!(
        !foo_refs_v2.iter().any(|(f, _, _, _)| f.as_ref() == file.as_ref()),
        "after re-ingest without foo(), no foo-reference should remain from {file}; got {foo_refs_v2:?}"
    );
    let bar_refs_v2 = session.read(|db| db.reference_locations("bar"));
    assert!(
        bar_refs_v2
            .iter()
            .any(|(f, _, _, _)| f.as_ref() == file.as_ref()),
        "after re-ingest with bar(), bar-reference must be present in {file}; got {bar_refs_v2:?}"
    );
}

/// Cross-file invalidation must work for session-based callers without
/// requiring a full `ProjectAnalyzer::analyze()` pass to seed the reverse-dep
/// graph. After ingesting a base + a dependent, evicting the base must also
/// evict the dependent.
#[test]
fn ingest_file_maintains_reverse_dep_graph_for_session_callers() {
    use mir_analyzer::cache::AnalysisCache;
    use tempfile::TempDir;

    let cache_dir = TempDir::new().unwrap();
    let cache = Arc::new(AnalysisCache::open(cache_dir.path()));
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache.clone());

    let base_path: Arc<str> = Arc::from("/proj/Base.php");
    let child_path: Arc<str> = Arc::from("/proj/Child.php");

    session.ingest_file(base_path.clone(), Arc::from("<?php\nclass Base {}\n"));
    session.ingest_file(
        child_path.clone(),
        Arc::from("<?php\nuse Base;\nclass Child extends Base {}\n"),
    );

    // Seed dummy cache entries so eviction is observable.
    cache.put(base_path.as_ref(), "h1".to_string(), Vec::new(), Vec::new());
    cache.put(
        child_path.as_ref(),
        "h2".to_string(),
        Vec::new(),
        Vec::new(),
    );
    assert!(cache.get(base_path.as_ref(), "h1").is_some());
    assert!(cache.get(child_path.as_ref(), "h2").is_some());

    // Editing Base must cascade-evict Child via the reverse-dep graph that
    // was incrementally built by ingest_file (no full analyze() ever ran).
    let evicted = cache.evict_with_dependents(&[base_path.as_ref().to_string()]);
    assert!(
        evicted >= 1,
        "session-built reverse-dep graph must yield >= 1 evicted dependent; got {evicted}"
    );
    assert!(
        cache.get(child_path.as_ref(), "h2").is_none(),
        "Child.php cache entry must have been evicted as a dependent of Base.php"
    );
}
