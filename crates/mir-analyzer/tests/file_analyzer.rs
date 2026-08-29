//! Smoke tests for the session-based per-file analysis API.
//!
//! These verify the core invariants of `AnalysisSession` + `FileAnalyzer`:
//!   * trait method bodies are analyzed,
//!   * stubs are loaded lazily on first ingest/analyze,
//!   * concurrent reads can take cheap snapshots while edits proceed.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::symbol::ReferenceKind;
use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion};

use self::common::create_temp_dir;

fn parse_and_analyze(source: &str) -> mir_analyzer::FileAnalysis {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("<test>");
    session.ingest_file(file.clone(), Arc::from(source));

    let parsed = php_rs_parser::parse(source);
    assert!(
        parsed.errors.is_empty(),
        "parser errors in test source: {:?}",
        parsed.errors
    );

    FileAnalyzer::new(&session).analyze(file, source, &parsed.program, &parsed.source_map)
}

fn parse_and_analyze_diagnostics(source: &str) -> mir_analyzer::FileAnalysis {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("<test>");
    session.ingest_file(file.clone(), Arc::from(source));

    let parsed = php_rs_parser::parse(source);
    assert!(
        parsed.errors.is_empty(),
        "parser errors in test source: {:?}",
        parsed.errors
    );

    FileAnalyzer::new(&session).analyze_diagnostics_only(
        file,
        source,
        &parsed.program,
        &parsed.source_map,
    )
}

fn session_for_source(path: &str, source: &str) -> (AnalysisSession, Arc<str>) {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from(path);
    session.ingest_file(file.clone(), Arc::from(source));
    (session, file)
}

/// Trait method bodies must be analyzed. `StatementsAnalyzer` (the layer
/// some external consumers were forced to use) skips traits; `FileAnalyzer`
/// goes through `BodyAnalyzer`, which walks them. Regression guard for the
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
    let result = parse_and_analyze_diagnostics(src);
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
    let result = parse_and_analyze_diagnostics(src);
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

#[test]
fn file_analyzer_diagnostics_only_skips_symbols_but_keeps_issues() {
    let src = "<?php
function helper(): void {}
function demo(): void {
    helper();
    totally_undefined_function();
}
";
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/demo.php");
    session.ingest_file(file.clone(), Arc::from(src));

    let parsed = php_rs_parser::parse(src);
    assert!(parsed.errors.is_empty());

    let full =
        FileAnalyzer::new(&session).analyze(file.clone(), src, &parsed.program, &parsed.source_map);
    let diag_only = FileAnalyzer::new(&session).analyze_diagnostics_only(
        file,
        src,
        &parsed.program,
        &parsed.source_map,
    );

    assert!(
        !full.symbols.is_empty(),
        "full open-file analysis should still collect navigation symbols"
    );
    assert!(
        diag_only.symbols.is_empty(),
        "diagnostics-only analysis should skip whole-file symbol retention"
    );
    assert_eq!(
        full.issues.iter().map(|i| &i.kind).collect::<Vec<_>>(),
        diag_only.issues.iter().map(|i| &i.kind).collect::<Vec<_>>(),
        "diagnostics-only analysis must preserve diagnostics"
    );
}

#[test]
fn analysis_session_diagnostics_helper_keeps_issues_without_symbols() {
    let src = "<?php
function helper(): void {}
function demo(): void {
    helper();
    totally_undefined_function();
}
";
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let analysis = session.analyze_file_diagnostics("/proj/helper.php", src);

    assert!(
        analysis.symbols.is_empty(),
        "session diagnostics helper should not retain whole-file symbols"
    );
    assert!(
        analysis
            .issues
            .iter()
            .any(|issue| issue.kind.name() == "UndefinedFunction"),
        "session diagnostics helper should preserve body-analysis issues"
    );
}

#[test]
fn analysis_session_diagnostics_helper_preserves_parse_errors() {
    let src = "<?php
function broken( {
";
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let analysis = session.analyze_file_diagnostics("/proj/broken.php", src);

    assert!(
        analysis.symbols.is_empty(),
        "parse-error diagnostics helper path should not retain symbols"
    );
    assert!(
        analysis
            .issues
            .iter()
            .any(|issue| issue.kind.name() == "ParseError"),
        "session diagnostics helper should preserve parse errors"
    );
}

/// `ensure_all_stubs` is idempotent; calling it many times must be cheap
/// and must not double-load stubs (would corrupt the codebase).
#[test]
fn ensure_all_stubs_is_idempotent() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.ensure_all_stubs();
    session.ensure_all_stubs();

    // After loading, a built-in like strlen() should be known.
    assert!(
        session.contains_function("strlen"),
        "strlen() must be loaded after ensure_all_stubs"
    );
}

/// `ensure_stub_for_function` lazily loads exactly the stub containing the
/// requested function — no more, no less. On a fresh session nothing is loaded
/// yet; requesting `imagecreate` brings in the gd stub on demand.
#[test]
fn ensure_stub_for_function_lazy_loads_extension() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let baseline = session.loaded_stub_count();

    // Nothing loaded yet on a fresh session.
    assert!(
        !session.contains_function("imagecreate"),
        "imagecreate() must not be loaded on a fresh session"
    );

    let was_known = session.ensure_stub_for_function("imagecreate");
    assert!(was_known, "imagecreate() must be a recognized PHP built-in");

    let after = session.loaded_stub_count();
    assert!(
        after > baseline,
        "ensure_stub_for_function must ingest at least one new stub"
    );

    assert!(
        session.contains_function("imagecreate"),
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

    let parsed = php_rs_parser::parse(src);
    assert!(parsed.errors.is_empty());

    let analysis = FileAnalyzer::new(&session).analyze_diagnostics_only(
        file,
        src,
        &parsed.program,
        &parsed.source_map,
    );

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

    // Sanity: at least the gd, Reflection, and json stubs must have been pulled in.
    let count = session.loaded_stub_count();
    assert!(
        count >= 3,
        "expected at least gd, Reflection, and json stubs to be loaded; got {count}"
    );
}

#[test]
fn resolve_at_finds_function_call_without_whole_file_symbol_list() {
    let src = "<?php\nfunction greet(): void {}\nfunction caller(): void { greet(); }\n";
    let (session, file) = session_for_source("/proj/a.php", src);
    assert!(
        session.reference_locations("fn:greet").is_empty(),
        "fixture should start with no committed reference postings"
    );

    let offset = src.find("{ greet").unwrap() as u32 + 2;
    let sym = session
        .resolve_at(file.as_ref(), offset)
        .expect("resolve_at should find greet()");

    assert!(
        matches!(&sym.kind, ReferenceKind::FunctionCall(name) if name.as_ref() == "greet"),
        "expected FunctionCall(greet), got {:?}",
        sym.kind
    );
    assert!(
        session.reference_locations("fn:greet").is_empty(),
        "resolve_at should not commit reference postings as a side effect"
    );
}

#[test]
fn resolve_at_finds_method_call_inside_class_scope() {
    let src = "<?php\nclass Svc { public function helper(): void {}\npublic function run(): void { $this->helper(); } }\n";
    let (session, file) = session_for_source("/proj/this_call.php", src);

    let offset = src.find("->helper").unwrap() as u32 + 2;
    let sym = session
        .resolve_at(file.as_ref(), offset)
        .expect("resolve_at should resolve $this->helper()");

    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { method, .. } if method.as_ref() == "helper"),
        "expected MethodCall(helper), got {:?}",
        sym.kind
    );
}

#[test]
fn resolve_at_finds_use_import_symbol() {
    let dir = create_temp_dir("resolve_at_use_import");
    let dep = self::common::write_file(&dir, "Dep.php", "<?php\nnamespace App;\nclass Dep {}\n");
    let main_src = "<?php\nuse App\\Dep;\nfunction run(): Dep { return new Dep(); }\n";
    let main = self::common::write_file(&dir, "Main.php", main_src);
    let main_str = self::common::path_to_str(&main).to_string();

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(
        Arc::from(self::common::path_to_str(&dep)),
        Arc::from(fs::read_to_string(&dep).unwrap()),
    );
    session.ingest_file(Arc::from(main_str.as_str()), Arc::from(main_src));

    let offset = main_src.find("App\\Dep").unwrap() as u32 + "App\\".len() as u32;
    let sym = session
        .resolve_at(&main_str, offset)
        .expect("resolve_at should find use-import symbol");

    assert!(
        matches!(&sym.kind, ReferenceKind::UseImport(inner) if matches!(inner.as_ref(), ReferenceKind::ClassReference(name) if name.as_ref() == "App\\Dep")),
        "expected UseImport(ClassReference(App\\Dep)), got {:?}",
        sym.kind
    );
}

#[test]
fn resolve_at_finds_top_level_exec_symbol() {
    let src =
        "<?php\nclass Svc { public function run(): void {} }\n$svc = new Svc();\n$svc->run();\n";
    let (session, file) = session_for_source("/proj/top_level_exec.php", src);

    let offset = src.rfind("->run").unwrap() as u32 + 2;
    let sym = session
        .resolve_at(file.as_ref(), offset)
        .expect("resolve_at should find top-level method call");

    assert!(
        matches!(&sym.kind, ReferenceKind::MethodCall { class, method } if class.as_ref() == "Svc" && method.as_ref() == "run"),
        "expected top-level MethodCall(Svc::run), got {:?}",
        sym.kind
    );
}

#[test]
fn resolve_at_finds_native_type_hint_symbol() {
    let src = "<?php\nclass Dep {}\nfunction run(Dep $d): Dep { return $d; }\n";
    let (session, file) = session_for_source("/proj/resolve_type_hint.php", src);

    let offset = src.find("run(Dep").unwrap() as u32 + "run(".len() as u32;
    let sym = session
        .resolve_at(file.as_ref(), offset)
        .expect("resolve_at should resolve the parameter type-hint class name");

    assert!(
        matches!(&sym.kind, ReferenceKind::ClassReference(name) if name.as_ref() == "Dep"),
        "expected ClassReference(Dep), got {:?}",
        sym.kind
    );
    assert_eq!(sym.resolved_type.to_string(), "class-string");
}

#[test]
fn resolve_at_finds_variable_symbol() {
    let src = "<?php\nfunction run(int $value): int { return $value; }\n";
    let (session, file) = session_for_source("/proj/resolve_var.php", src);

    let offset = src.rfind("$value").unwrap() as u32 + 1;
    let sym = session
        .resolve_at(file.as_ref(), offset)
        .expect("resolve_at should resolve the variable read");

    assert!(
        matches!(&sym.kind, ReferenceKind::Variable(name) if name.as_ref() == "value"),
        "expected Variable(value), got {:?}",
        sym.kind
    );
    assert_eq!(sym.resolved_type.to_string(), "int");
}

#[test]
fn resolve_at_finds_receiver_gap_symbol() {
    let src =
        "<?php\nclass Foo { public string $name = ''; }\nfunction read(Foo $obj): void { $obj->name; }\n";
    let (session, file) = session_for_source("/proj/resolve_receiver_gap.php", src);

    let gap_off = src.find("$obj->name").unwrap() as u32 + "$obj".len() as u32;
    let sym = session
        .resolve_at(file.as_ref(), gap_off)
        .expect("resolve_at should resolve the receiver-gap symbol");

    assert!(
        matches!(&sym.kind, ReferenceKind::Receiver),
        "expected Receiver, got {:?}",
        sym.kind
    );
    assert_eq!(sym.resolved_type.to_string(), "Foo");
}

#[test]
fn hover_at_uses_targeted_navigation_path() {
    let src = "<?php\nfunction helper(): int { return 1; }\nfunction caller(): int { return helper(); }\n";
    let (session, file) = session_for_source("/proj/hover.php", src);

    let offset = src.rfind("helper()").unwrap() as u32;
    let hover = session
        .hover_at(file.as_ref(), offset)
        .expect("hover_at should resolve helper()");

    assert_eq!(hover.ty.to_string(), "int");
    assert!(
        hover.definition.is_some(),
        "hover_at should surface the helper() definition location"
    );
}

#[test]
fn hover_at_returns_type_for_variable_without_definition_lookup() {
    let src = "<?php\nfunction run(int $value): int { return $value; }\n";
    let (session, file) = session_for_source("/proj/hover_var.php", src);

    let offset = src.rfind("$value").unwrap() as u32 + 1;
    let hover = session
        .hover_at(file.as_ref(), offset)
        .expect("hover_at should resolve variable reads");

    assert_eq!(hover.ty.to_string(), "int");
    assert!(
        hover.definition.is_none(),
        "variable hover should not require a codebase definition location"
    );
    assert!(hover.docstring.is_none());
}

#[test]
fn hover_at_returns_type_for_receiver_gap_without_definition_lookup() {
    let src =
        "<?php\nclass Foo { public string $name = ''; }\nfunction read(Foo $obj): void { $obj->name; }\n";
    let (session, file) = session_for_source("/proj/hover_receiver.php", src);

    let gap_off = src.find("$obj->name").unwrap() as u32 + "$obj".len() as u32;
    let hover = session
        .hover_at(file.as_ref(), gap_off)
        .expect("hover_at should resolve receiver-gap positions");

    assert_eq!(hover.ty.to_string(), "Foo");
    assert!(
        hover.definition.is_none(),
        "receiver-gap hover should surface the inferred receiver type without a definition lookup"
    );
    assert!(hover.docstring.is_none());
}

#[test]
fn name_at_uses_compact_navigation_fact_path() {
    let src = "<?php\nfunction helper(): void {}\nfunction caller(): void { helper(); }\n";
    let (session, file) = session_for_source("/proj/name_at.php", src);
    assert!(
        session.reference_locations("fn:helper").is_empty(),
        "fixture should start with no committed reference postings"
    );

    let offset = src.rfind("helper();").unwrap() as u32;
    let name = session
        .name_at(file.as_ref(), offset)
        .expect("name_at should resolve helper()");

    assert_eq!(name, mir_analyzer::Name::function("helper"));
    assert!(
        session.reference_locations("fn:helper").is_empty(),
        "name_at should not commit reference postings as a side effect"
    );
}

#[test]
fn name_at_resolves_use_import_via_navigation_facts() {
    let dir = create_temp_dir("name_at_use_import");
    let dep = self::common::write_file(&dir, "Dep.php", "<?php\nnamespace App;\nclass Dep {}\n");
    let main_src = "<?php\nuse App\\Dep;\nfunction run(): Dep { return new Dep(); }\n";
    let main = self::common::write_file(&dir, "Main.php", main_src);
    let main_str = self::common::path_to_str(&main).to_string();

    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(
        Arc::from(self::common::path_to_str(&dep)),
        Arc::from(fs::read_to_string(&dep).unwrap()),
    );
    session.ingest_file(Arc::from(main_str.as_str()), Arc::from(main_src));

    let offset = main_src.find("App\\Dep").unwrap() as u32 + "App\\".len() as u32;
    let name = session
        .name_at(&main_str, offset)
        .expect("name_at should resolve the use-import class name");

    assert_eq!(name, mir_analyzer::Name::class("App\\Dep"));
    assert!(
        session.reference_locations("cls:App\\Dep").is_empty(),
        "name_at on a use import should not commit reference postings"
    );
}

#[test]
fn name_at_resolves_native_type_hint_via_navigation_facts() {
    let src = "<?php\nclass Dep {}\nfunction run(Dep $d): Dep { return $d; }\n";
    let (session, file) = session_for_source("/proj/type_hint_name_at.php", src);

    let offset = src.find("run(Dep").unwrap() as u32 + "run(".len() as u32;
    let name = session
        .name_at(file.as_ref(), offset)
        .expect("name_at should resolve the parameter type-hint class name");

    assert_eq!(name, mir_analyzer::Name::class("Dep"));
    assert!(
        session.reference_locations("cls:Dep").is_empty(),
        "name_at on a type hint should not commit reference postings"
    );
}

#[test]
fn references_at_uses_compact_navigation_fact_path() {
    let src = "<?php
function helper(): void {}
function caller(): void { helper(); }
";
    let (session, file) = session_for_source("/proj/references_at.php", src);
    let offset = src.find("helper();").unwrap() as u32 + 1;

    let refs = session
        .references_at(
            file.as_ref(),
            offset,
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
        )
        .expect("references_at should resolve helper()");

    assert!(
        refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
        "references_at should include the helper() call site; got {refs:?}"
    );
}

#[test]
fn references_at_cancellable_reports_not_found_without_running_query() {
    let src = "<?php
function helper(): void {}
";
    let (session, file) = session_for_source("/proj/references_at_missing.php", src);
    let offset = src.find("function").unwrap() as u32;

    let result = session.references_at_cancellable(
        file.as_ref(),
        offset,
        std::slice::from_ref(&file),
        false,
        mir_analyzer::ReferenceIncludes::Plain,
        &|| true,
    );

    assert_eq!(result, Err(mir_analyzer::SymbolLookupError::NotFound));
}

// ── Version-filtering helpers ────────────────────────────────────────────────

/// Run `FileAnalyzer` on `src` inside `session` and return all issue-kind
/// names. A fresh file path is used each time so there is no cross-test
/// ingestion state.
fn version_test_issues(session: &AnalysisSession, src: &str) -> Vec<String> {
    let file: Arc<str> = Arc::from("<version-test>");
    session.ingest_file(file.clone(), Arc::from(src));
    let parsed = php_rs_parser::parse(src);
    FileAnalyzer::new(session)
        .analyze_diagnostics_only(file, src, &parsed.program, &parsed.source_map)
        .issues
        .iter()
        .map(|i| i.kind.name().to_string())
        .collect()
}

// ── @since filtering ─────────────────────────────────────────────────────────

/// PHP 7.4 session must reject `str_contains` (`@since 8.0`) in the
/// FileAnalyzer (LSP / incremental) path.
///
/// Discriminator: `strlen` has no `@since` tag and lives in the same
/// `Core/Core.php` stub file as `str_contains`. It must be present on PHP 7.4,
/// proving that Core.php was loaded AND that filtering was selective rather
/// than a blanket load failure.
#[test]
fn version_filter_since_php74_rejects_php80_function() {
    let session = AnalysisSession::new(PhpVersion::new(7, 4));
    session.ensure_all_stubs();

    assert!(
        session.contains_function("strlen"),
        "strlen (no @since) must be present on PHP 7.4 — Core.php must have been loaded"
    );
    assert!(
        !session.contains_function("str_contains"),
        "str_contains (@since 8.0) must be absent on PHP 7.4"
    );

    let issues = version_test_issues(&session, "<?php\nstr_contains('hello', 'x');\n");
    assert!(
        issues.iter().any(|n| n == "UndefinedFunction"),
        "FileAnalyzer must emit UndefinedFunction for str_contains on PHP 7.4; got: {issues:?}"
    );
}

/// PHP 8.0 session must accept `str_contains` (introduced in 8.0).
///
/// Same discriminator: both `strlen` and `str_contains` must be present,
/// proving Core.php was loaded and the symbol passed the version filter.
#[test]
fn version_filter_since_php80_accepts_php80_function() {
    let session = AnalysisSession::new(PhpVersion::new(8, 0));
    session.ensure_all_stubs();

    assert!(
        session.contains_function("strlen"),
        "strlen must be present on PHP 8.0"
    );
    assert!(
        session.contains_function("str_contains"),
        "str_contains (@since 8.0) must be present on PHP 8.0"
    );

    let issues = version_test_issues(&session, "<?php\nstr_contains('hello', 'x');\n");
    assert!(
        !issues.iter().any(|n| n == "UndefinedFunction"),
        "str_contains must be defined on PHP 8.0; got: {issues:?}"
    );
}

// ── @removed filtering ───────────────────────────────────────────────────────

/// `hebrevc` is `@removed 8.0`. It must be resolvable on PHP 7.4 …
#[test]
fn version_filter_removed_php74_accepts_hebrevc() {
    let session = AnalysisSession::new(PhpVersion::new(7, 4));
    session.ensure_all_stubs();

    assert!(
        session.contains_function("hebrevc"),
        "hebrevc (@removed 8.0) must be present on PHP 7.4"
    );

    let issues = version_test_issues(&session, "<?php\nhebrevc('hello');\n");
    assert!(
        !issues.iter().any(|n| n == "UndefinedFunction"),
        "hebrevc must be defined on PHP 7.4; got: {issues:?}"
    );
}

/// … and must be absent (and raise `UndefinedFunction`) on PHP 8.0.
#[test]
fn version_filter_removed_php80_rejects_hebrevc() {
    let session = AnalysisSession::new(PhpVersion::new(8, 0));
    session.ensure_all_stubs();

    assert!(
        !session.contains_function("hebrevc"),
        "hebrevc (@removed 8.0) must be absent on PHP 8.0"
    );

    let issues = version_test_issues(&session, "<?php\nhebrevc('hello');\n");
    assert!(
        issues.iter().any(|n| n == "UndefinedFunction"),
        "FileAnalyzer must emit UndefinedFunction for hebrevc on PHP 8.0; got: {issues:?}"
    );
}

// ── @since filtering applies to enum stubs too ───────────────────────────────

/// collect_enum never called version_allows, unlike class/trait/interface —
/// an @since-gated enum stub (e.g. PropertyHookType, @since 8.4) was collected
/// regardless of the configured target version.
#[test]
fn version_filter_since_php83_rejects_php84_enum() {
    let session = AnalysisSession::new(PhpVersion::new(8, 3));
    session.ensure_all_stubs();

    assert!(
        !session.contains_class("PropertyHookType"),
        "PropertyHookType (@since 8.4) must be absent on PHP 8.3"
    );
}

/// Sibling of version_filter_since_php83_rejects_php84_enum: the same enum
/// must be collected once the target version reaches @since.
#[test]
fn version_filter_since_php84_accepts_php84_enum() {
    let session = AnalysisSession::new(PhpVersion::new(8, 4));
    session.ensure_all_stubs();

    assert!(
        session.contains_class("PropertyHookType"),
        "PropertyHookType (@since 8.4) must be present on PHP 8.4"
    );
}

// ── Secondary regression guards: with_cache_dir / with_cache paths ───────────

/// `with_cache_dir` rebuilds `self.db`; the fix must re-apply `php_version`
/// after the rebuild so version filtering is not silently reset to the "8.2"
/// default.
#[test]
fn version_filter_with_cache_dir_preserves_version() {
    let cache_dir = create_temp_dir("ver_cache_dir");
    let session = AnalysisSession::new(PhpVersion::new(7, 4)).with_cache_dir(cache_dir.path());
    session.ensure_all_stubs();

    assert!(
        session.contains_function("strlen"),
        "strlen must be present after with_cache_dir on PHP 7.4"
    );
    assert!(
        !session.contains_function("str_contains"),
        "str_contains must be filtered after with_cache_dir on PHP 7.4"
    );

    let issues = version_test_issues(&session, "<?php\nstr_contains('hello', 'x');\n");
    assert!(
        issues.iter().any(|n| n == "UndefinedFunction"),
        "with_cache_dir must not silently reset php_version to 8.2; got: {issues:?}"
    );
}

/// `with_cache` also rebuilds `self.db`; the same fix must apply.
#[test]
fn version_filter_with_cache_preserves_version() {
    use mir_analyzer::cache::AnalysisCache;

    let cache_dir = create_temp_dir("ver_cache");
    let cache = Arc::new(AnalysisCache::open(
        cache_dir.path(),
        PhpVersion::LATEST.cache_byte(),
        0,
    ));
    let session = AnalysisSession::new(PhpVersion::new(7, 4)).with_cache(cache);
    session.ensure_all_stubs();

    assert!(
        session.contains_function("strlen"),
        "strlen must be present after with_cache on PHP 7.4"
    );
    assert!(
        !session.contains_function("str_contains"),
        "str_contains must be filtered after with_cache on PHP 7.4"
    );

    let issues = version_test_issues(&session, "<?php\nstr_contains('hello', 'x');\n");
    assert!(
        issues.iter().any(|n| n == "UndefinedFunction"),
        "with_cache must not silently reset php_version to 8.2; got: {issues:?}"
    );
}

// ── Session isolation ─────────────────────────────────────────────────────────

/// Two independent sessions at different PHP versions must not share salsa db
/// state. A PHP 8.0 session created first must not contaminate a PHP 7.4
/// session created afterwards.
#[test]
fn version_filter_independent_sessions_do_not_share_state() {
    let session_80 = AnalysisSession::new(PhpVersion::new(8, 0));
    let session_74 = AnalysisSession::new(PhpVersion::new(7, 4));

    session_80.ensure_all_stubs();
    session_74.ensure_all_stubs();

    assert!(
        session_80.contains_function("str_contains"),
        "str_contains must be present in the PHP 8.0 session"
    );
    assert!(
        !session_74.contains_function("str_contains"),
        "str_contains must be absent in the PHP 7.4 session even when a PHP 8.0 session exists"
    );

    let issues_74 = version_test_issues(&session_74, "<?php\nstr_contains('a', 'b');\n");
    assert!(
        issues_74.iter().any(|n| n == "UndefinedFunction"),
        "PHP 7.4 session must produce UndefinedFunction for str_contains even with a PHP 8.0 session alive; got: {issues_74:?}"
    );

    let issues_80 = version_test_issues(&session_80, "<?php\nstr_contains('a', 'b');\n");
    assert!(
        !issues_80.iter().any(|n| n == "UndefinedFunction"),
        "PHP 8.0 session must not produce UndefinedFunction for str_contains; got: {issues_80:?}"
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

    let parsed = php_rs_parser::parse(src);
    let _analysis = FileAnalyzer::new(&session).analyze_diagnostics_only(
        file.clone(),
        src,
        &parsed.program,
        &parsed.source_map,
    );

    // Resolve "Greeter" by name — caller doesn't need to know its position.
    let loc = session
        .definition_of(&mir_analyzer::Name::class("Greeter"))
        .expect("Greeter must resolve");
    assert_eq!(loc.file.as_ref(), file.as_ref());
    assert!(loc.line >= 1, "expected a real source line; got {loc:?}");

    // Member resolution.
    let greet_loc = session.definition_of(&mir_analyzer::Name::method("Greeter", "greet"));
    assert!(greet_loc.is_ok(), "Greeter::greet() must resolve");

    let class_offset = src.rfind("new Greeter").unwrap() as u32 + "new ".len() as u32;
    let class_ref = session
        .symbol_at(file.as_ref(), class_offset)
        .expect("expected targeted symbol resolution for new Greeter()");
    assert!(matches!(
        class_ref.kind,
        mir_analyzer::ReferenceKind::ClassReference(_)
            | mir_analyzer::ReferenceKind::FunctionCall(_)
    ));
}

/// `document_symbols` powers the editor outline view. Must list every top-
/// level declaration in the file with its kind.
#[test]
fn document_symbols_lists_file_declarations() {
    use mir_analyzer::DeclarationKind;

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

    let by_name: std::collections::HashMap<&str, DeclarationKind> =
        symbols.iter().map(|s| (s.name.as_ref(), s.kind)).collect();

    assert_eq!(by_name.get("Cat"), Some(&DeclarationKind::Class));
    assert_eq!(by_name.get("Animal"), Some(&DeclarationKind::Interface));
    assert_eq!(by_name.get("Furry"), Some(&DeclarationKind::Trait));
    assert_eq!(by_name.get("pet_count"), Some(&DeclarationKind::Function));
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

    let parsed = php_rs_parser::parse(src);
    let _ = FileAnalyzer::new(&session).analyze_diagnostics_only(
        file.clone(),
        src,
        &parsed.program,
        &parsed.source_map,
    );

    let refs = session
        .indexed_references_to(
            &mir_analyzer::Name::function("helper"),
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        refs.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
        "helper() must have at least one reference recorded in {file}; got {refs:?}"
    );
}

/// `FileAnalysis::symbol_at` finds the symbol at a cursor byte offset.
/// Used by editors to map (line, column) → resolved symbol → definition /
/// hover info.
#[test]
fn file_analysis_symbol_at_finds_call_site() {
    let src = "<?php
target(); function target(): void {}
";
    let analysis = parse_and_analyze(src);

    // Find an offset inside the `target` call. The call is on line 2, before
    // the `function` keyword.
    let call_offset = src.find("target()").unwrap() as u32 + 1;
    let resolved = analysis
        .symbol_at(call_offset)
        .expect("expected a resolved symbol at the call site");
    assert!(
        matches!(resolved.kind, mir_analyzer::ReferenceKind::FunctionCall(_)),
        "expected FunctionCall kind; got {:?}",
        resolved.kind
    );
}

/// `FileAnalysis::symbol_at` must fall back to a call's full `expr_span` when
/// the offset misses every identifier span — e.g. a cursor sitting on the
/// closing `)` of `$f->bar()` in a chain `$f->bar()->bar()`, which an editor
/// hits when resolving the receiver type for completion right after typing
/// `->` following a call. `BatchAnalysis::symbol_at` (batch/mod.rs) already
/// has this fallback; this is the single-file `FileAnalyzer` path exercised
/// by `AnalysisSession`'s open-file/interactive queries.
#[test]
fn file_analysis_symbol_at_falls_back_to_call_expr_span_in_chain() {
    let src = "<?php
class Foo {
    public function bar(): Foo { return $this; }
}
function test(Foo $f): void {
    $f->bar()->bar();
}
";
    let result = parse_and_analyze(src);

    // Offset of the closing ')' of the *first* `bar()` call: inside that
    // call's expr_span, but outside every recorded identifier span (the
    // method-name span covers only `bar`).
    let first_call = src
        .find("$f->bar()")
        .expect("fixture must contain $f->bar()");
    let close_paren_offset = (first_call + "$f->bar(".len()) as u32;

    let resolved = result
        .symbol_at(close_paren_offset)
        .expect("expected the chained call's expr_span to cover its own closing paren");
    assert!(
        matches!(&resolved.kind, mir_analyzer::ReferenceKind::MethodCall { method, .. } if method.as_ref() == "bar"),
        "expected MethodCall(bar) via expr_span fallback; got {:?}",
        resolved.kind
    );
    assert!(
        resolved.resolved_type.to_string().contains("Foo"),
        "expected the first bar() call's return type (Foo) to be recorded; got {}",
        resolved.resolved_type
    );
}

/// `location_from_span` translates a parser span to a `Location` using the
/// crate's own conventions. Round-trip sanity check: spans from a parsed
/// program convert to lines/columns that match the source text.
#[test]
fn location_from_span_translates_pass2_spans_to_source_locations() {
    let src = "<?php
function helper(): string { return 'x'; }
function caller(): string { return helper(); }
";

    let parsed = php_rs_parser::parse(src);
    assert!(parsed.errors.is_empty());

    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/loc.php");
    session.ingest_file(file.clone(), Arc::from(src));
    let _analysis = FileAnalyzer::new(&session).analyze_diagnostics_only(
        file.clone(),
        src,
        &parsed.program,
        &parsed.source_map,
    );

    let call_offset = src.rfind("helper();").unwrap() as u32;
    let call = session
        .symbol_at(file.as_ref(), call_offset)
        .expect("expected a FunctionCall symbol for helper()");
    let loc = mir_analyzer::location_from_span(call.span, file.clone(), src, &parsed.source_map);

    assert_eq!(loc.file.as_ref(), file.as_ref());
    assert_eq!(
        loc.line, 3,
        "helper() is called on the 3rd line; got {loc:?}"
    );
    assert!(loc.line_end >= loc.line);
    assert!(
        loc.col_end > loc.col_start,
        "non-empty span must produce a non-empty column range: {loc:?}"
    );
}

/// Soft-stub-fallback regression guard: a name that the build-time stub
/// index does *not* know about must still trigger `UndefinedFunction`. The
/// fallback should only suppress diagnostics for names mir is confident are
/// real PHP built-ins.
#[test]
fn truly_unknown_function_still_emits_undefined_function() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/unknown_fn.php");
    let src = "<?php
function caller(): void {
    definitely_not_a_real_php_function_xyz123();
}
";
    session.ingest_file(file.clone(), Arc::from(src));

    let parsed = php_rs_parser::parse(src);
    let analysis = FileAnalyzer::new(&session).analyze_diagnostics_only(
        file,
        src,
        &parsed.program,
        &parsed.source_map,
    );

    let undefined: Vec<_> = analysis
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .collect();
    assert_eq!(
        undefined.len(),
        1,
        "user-defined unknown function must still emit UndefinedFunction; got: {:?}",
        analysis
            .issues
            .iter()
            .map(|i| i.kind.name())
            .collect::<Vec<_>>()
    );
}

/// Unknown names return `false` and do not spuriously ingest anything.
#[test]
fn ensure_stub_for_unknown_symbol_returns_false() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let before = session.loaded_stub_count();

    assert!(!session.ensure_stub_for_function("definitely_not_a_php_builtin_xyz123"));
    assert!(!session.ensure_stub_for_class("\\Not\\A\\Real\\Class"));

    assert_eq!(
        session.loaded_stub_count(),
        before,
        "unknown lookups must not ingest any stubs"
    );
}

/// Ingested definitions must be observable via the public query API.
#[test]
fn ingested_definitions_are_observable() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    session.ingest_file(Arc::from("<test>"), Arc::from("<?php\nclass Foo {}\n"));

    assert!(
        session.contains_class("Foo"),
        "ingest_file definitions must be observable via the public API"
    );
}

/// `FileAnalyzer::analyze` infers cross-file return types on demand via salsa.
/// No explicit inference sweep is needed — the demand path calls
/// `infer_file_return_types` lazily when Pass 2 encounters a call to a
/// function without an explicit return-type hint.
#[test]
fn analyze_infers_return_types_without_prior_sweep() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/A.php");
    let src = "<?php
function bar() { return 'hello'; }
function foo(): string { return bar(); }
";
    session.ingest_file(file.clone(), Arc::from(src));

    let parsed = php_rs_parser::parse(src);
    assert!(parsed.errors.is_empty());

    let analysis = FileAnalyzer::new(&session).analyze_diagnostics_only(
        file,
        src,
        &parsed.program,
        &parsed.source_map,
    );

    let invalid_return = analysis
        .issues
        .iter()
        .filter(|i| i.kind.name() == "InvalidReturnType")
        .count();
    assert_eq!(
        invalid_return,
        0,
        "demand-driven inference must resolve bar()'s return type so foo(): string is OK; got issues: {:?}",
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

    let cache_dir = create_temp_dir("cache");
    let cache = Arc::new(AnalysisCache::open(
        cache_dir.path(),
        PhpVersion::LATEST.cache_byte(),
        0,
    ));
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache.clone());

    let base: Arc<str> = Arc::from("/proj/Base.php");
    let child: Arc<str> = Arc::from("/proj/Child.php");

    // Stubs are now registered as SourceFiles too (so the pull path can
    // see PHP built-ins). Count the stub baseline up front and assert
    // against the delta rather than absolute count.
    session.ensure_all_stubs();
    let stub_count = session.tracked_file_count();

    session.ingest_file(base.clone(), Arc::from("<?php\nclass Base {}\n"));
    session.ingest_file(
        child.clone(),
        Arc::from("<?php\nclass Child extends Base {}\n"),
    );
    cache.put(
        base.as_ref(),
        "h1".to_string(),
        String::new(),
        [].into(),
        [].into(),
    );
    cache.put(
        child.as_ref(),
        "h2".to_string(),
        String::new(),
        [].into(),
        [].into(),
    );
    assert_eq!(session.tracked_file_count(), stub_count + 2);

    session.invalidate_file(child.as_ref());

    assert_eq!(
        session.tracked_file_count(),
        stub_count + 1,
        "salsa input handle for Child must be released after invalidate"
    );
    assert!(
        !session.contains_class("Child"),
        "Child class must be inactive after invalidate"
    );

    // Re-evict from Base to confirm Child is no longer a dependent of Base
    // (its outgoing edge to Base must have been dropped on invalidate).
    cache.put(
        child.as_ref(),
        "h3".to_string(),
        String::new(),
        [].into(),
        [].into(),
    );
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
        let parsed = php_rs_parser::parse(v1);
        FileAnalyzer::new(&session).analyze_diagnostics_only(
            file.clone(),
            v1,
            &parsed.program,
            &parsed.source_map,
        );
    }

    let foo_refs_v1 = session
        .indexed_references_to(
            &mir_analyzer::Name::function("foo"),
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        foo_refs_v1.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
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
        let parsed = php_rs_parser::parse(v2);
        FileAnalyzer::new(&session).analyze_diagnostics_only(
            file.clone(),
            v2,
            &parsed.program,
            &parsed.source_map,
        );
    }

    let foo_refs_v2 = session
        .indexed_references_to(
            &mir_analyzer::Name::function("foo"),
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        !foo_refs_v2.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
        "after re-ingest without foo(), no foo-reference should remain from {file}; got {foo_refs_v2:?}"
    );
    let bar_refs_v2 = session
        .indexed_references_to(
            &mir_analyzer::Name::function("bar"),
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        bar_refs_v2.iter().any(|(f, _)| f.as_ref() == file.as_ref()),
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

    let cache_dir = create_temp_dir("cache");
    let cache = Arc::new(AnalysisCache::open(
        cache_dir.path(),
        PhpVersion::LATEST.cache_byte(),
        0,
    ));
    let session = AnalysisSession::new(PhpVersion::LATEST).with_cache(cache.clone());

    let base_path: Arc<str> = Arc::from("/proj/Base.php");
    let child_path: Arc<str> = Arc::from("/proj/Child.php");

    session.ingest_file(base_path.clone(), Arc::from("<?php\nclass Base {}\n"));
    session.ingest_file(
        child_path.clone(),
        Arc::from("<?php\nuse Base;\nclass Child extends Base {}\n"),
    );

    // Seed dummy cache entries so eviction is observable.
    cache.put(
        base_path.as_ref(),
        "h1".to_string(),
        String::new(),
        [].into(),
        [].into(),
    );
    cache.put(
        child_path.as_ref(),
        "h2".to_string(),
        String::new(),
        [].into(),
        [].into(),
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

/// Phase 2.4: `FileAnalyzer::analyze` self-loads referenced classes via the
/// configured `ClassResolver`. The caller no longer has to enumerate class
/// references and pre-load them before analysis — the post-Pass-2 lazy-load
/// loop runs internally.
///
/// Setup: PSR-4 maps `App\` to a `src/` dir. `Lib.php` defines `App\Lib` and
/// is **not** ingested; `Consumer.php` uses `App\Lib` and is analyzed
/// directly. Pre-Phase-2.4 behaviour: `UndefinedClass: App\Lib`. After
/// Phase 2.4: clean.
#[test]
fn file_analyzer_self_loads_psr4_classes_without_pre_enumeration() {
    use std::fs;

    let root = create_temp_dir("self_load");
    fs::create_dir_all(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("src/Lib.php"),
        "<?php\nnamespace App;\nclass Lib {\n    public function go(): void {}\n}\n",
    )
    .unwrap();
    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    let psr4 =
        mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map creation");
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    // Consumer file references App\Lib without `use`. The session is told
    // about *only* this file — Lib.php is never explicitly ingested.
    let consumer_src =
        "<?php\nfunction probe(): void {\n    $x = new \\App\\Lib();\n    $x->go();\n}\n";
    let consumer_path: Arc<str> =
        Arc::from(root.path().join("Consumer.php").to_string_lossy().as_ref());
    session.ingest_file(consumer_path.clone(), Arc::from(consumer_src));

    let parsed = php_rs_parser::parse(consumer_src);
    let analyzer = FileAnalyzer::new(&session);
    let result = analyzer.analyze_diagnostics_only(
        consumer_path,
        consumer_src,
        &parsed.program,
        &parsed.source_map,
    );

    let undefined: Vec<_> = result
        .issues
        .iter()
        .filter(|i| matches!(i.kind.name(), "UndefinedClass" | "UndefinedMethod"))
        .map(|i| (i.kind.name(), format!("{:?}", i.kind)))
        .collect();
    assert!(
        undefined.is_empty(),
        "FileAnalyzer must self-load App\\Lib via PSR-4 and resolve ->go(); got: {undefined:?}"
    );
}

/// Contract: the analyzer always reports `UndefinedClass` when it sees one;
/// it has no concept of "workspace scan in progress". Filtering during a
/// pending scan is the consumer's responsibility (LSPs decide what to
/// publish; the analyzer reports the facts).
#[test]
fn file_analyzer_reports_undefined_class_unconditionally() {
    let session = AnalysisSession::new(PhpVersion::LATEST);

    let src = "<?php\nfunction probe(): void { new NotDefined(); }\n";
    let file: Arc<str> = Arc::from("<scan-test>");
    session.ingest_file(file.clone(), Arc::from(src));

    let parsed = php_rs_parser::parse(src);
    let analyzer = FileAnalyzer::new(&session);
    let result = analyzer.analyze_diagnostics_only(file, src, &parsed.program, &parsed.source_map);

    let undefined = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedClass")
        .count();
    assert!(
        undefined > 0,
        "expected an UndefinedClass for NotDefined; got: {:?}",
        result
            .issues
            .iter()
            .map(|i| i.kind.name())
            .collect::<Vec<_>>()
    );
}

/// Vendor `autoload.files` global functions are lazy-loaded automatically
/// by `FileAnalyzer::analyze` — no manual indexing call required.
///
/// `with_psr4` registers the vendor eager-file paths; the first
/// `FileAnalyzer::analyze` call indexes them via `ensure_vendor_eager_functions`
/// inside `prepare_ast_for_analysis`.  Subsequent calls are no-ops.
#[test]
fn vendor_autoload_files_functions_lazy_loaded_automatically() {
    let root = create_temp_dir("autoload_files_lazy");

    fs::create_dir_all(root.path().join("vendor/composer")).unwrap();
    // Composer-generated format: $vendorDir resolves to the vendor/ directory.
    fs::write(
        root.path().join("vendor/composer/autoload_files.php"),
        "<?php\n\
         $vendorDir = dirname(__DIR__);\n\
         $baseDir = dirname($vendorDir);\n\
         return array(\n\
             'abc123' => $vendorDir . '/helpers/functions.php',\n\
         );\n",
    )
    .unwrap();
    fs::write(
        root.path().join("vendor/composer/autoload_psr4.php"),
        "<?php\nreturn [];\n",
    )
    .unwrap();
    fs::write(
        root.path().join("vendor/composer/autoload_classmap.php"),
        "<?php\nreturn [];\n",
    )
    .unwrap();
    fs::write(
        root.path().join("vendor/composer/autoload_namespaces.php"),
        "<?php\nreturn [];\n",
    )
    .unwrap();
    fs::create_dir_all(root.path().join("vendor/helpers")).unwrap();
    fs::write(
        root.path().join("vendor/helpers/functions.php"),
        "<?php\nfunction helper_greet(string $name): string { return 'hi ' . $name; }\n",
    )
    .unwrap();
    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();

    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map");
    // No manual indexing — `with_psr4` registers the eager files and
    // `FileAnalyzer::analyze` lazy-loads them on first call.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    let open_src = "<?php\nhelper_greet('world');\n";
    let open_path: Arc<str> = Arc::from("open.php");
    let parsed = php_rs_parser::parse(open_src);
    session.ingest_file(open_path.clone(), Arc::from(open_src));

    let result = FileAnalyzer::new(&session).analyze_diagnostics_only(
        open_path,
        open_src,
        &parsed.program,
        &parsed.source_map,
    );
    let undefined = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert_eq!(
        undefined, 0,
        "vendor autoload.files functions must be lazy-loaded automatically \
         by FileAnalyzer::analyze; got issues: {:?}",
        result.issues
    );
}

/// Cancellable references: a pre-cancelled request aborts with `None` before
/// doing any warm-up or analysis work.
#[test]
fn indexed_references_to_aborts_when_cancelled() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/cancel_refs.php");
    let src = "<?php
function helper(): string { return 'a'; }
function caller(): string { return helper(); }
";
    session.ingest_file(file.clone(), Arc::from(src));

    let refs = session.indexed_references_to(
        &mir_analyzer::Name::function("helper"),
        std::slice::from_ref(&file),
        false,
        mir_analyzer::ReferenceIncludes::Plain,
        &|| true,
    );
    assert!(refs.is_none(), "cancelled request must return None");
}

/// A never-cancelling query, and repeated queries (now served through the
/// warm-up skip set) keep returning the same locations.
#[test]
fn indexed_references_to_warm_repeat_is_stable() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/warm_refs.php");
    let src = "<?php
function helper(): string { return 'a'; }
function caller(): string { return helper(); }
";
    session.ingest_file(file.clone(), Arc::from(src));

    let name = mir_analyzer::Name::function("helper");
    let first = session
        .indexed_references_to(
            &name,
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        !first.is_empty(),
        "helper() must have a recorded reference; got none"
    );

    let cancellable = session
        .indexed_references_to(
            &name,
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("uncancelled request must return Some");
    assert_eq!(first, cancellable);

    // Second call takes the prepared-files skip path; results must not change.
    let second = session
        .indexed_references_to(
            &name,
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(first, second);
}

/// Editing a file invalidates its warm-up skip entry: references added by the
/// new text are found by the next query.
#[test]
fn indexed_references_to_sees_new_refs_after_edit() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("/proj/edit_refs.php");
    let v1 = "<?php
function helper(): string { return 'a'; }
function caller(): string { return helper(); }
";
    session.ingest_file(file.clone(), Arc::from(v1));

    let name = mir_analyzer::Name::function("helper");
    let refs_v1 = session
        .indexed_references_to(
            &name,
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");

    let v2 = "<?php
function helper(): string { return 'a'; }
function caller(): string { return helper(); }
function caller2(): string { return helper(); }
";
    session.ingest_file(file.clone(), Arc::from(v2));

    let refs_v2 = session
        .indexed_references_to(
            &name,
            std::slice::from_ref(&file),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        refs_v2.len() > refs_v1.len(),
        "after adding a call site the query must find more references \
         (v1: {refs_v1:?}, v2: {refs_v2:?})"
    );
}

/// Cold indexed references must not analyze stale candidates in parallel.
/// Real workspaces can contain files whose inferred return types recurse
/// through other files; fanning those cold `analyze_file` queries across
/// worker snapshots can make salsa merge the same cycle at different
/// iterations and panic. The API should settle the postings without exposing
/// that internal cycle handling to callers.
#[test]
fn indexed_references_to_handles_recursive_inference_candidates() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let mut files = Vec::new();
    for i in 0..8 {
        let next = (i + 1) % 8;
        let file: Arc<str> = Arc::from(format!("/proj/cycle{i}.php"));
        let src = format!(
            "<?php
namespace App;
class C{i} {{
    public function save(): void {{}}
    public function peer() {{ return (new C{next}())->peer(); }}
    public function go(): void {{ $this->peer()->save(); }}
}}
"
        );
        session.ingest_file(file.clone(), Arc::from(src));
        files.push(file);
    }

    let refs = session
        .indexed_references_to(
            &mir_analyzer::Name::method("App\\C0", "save"),
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("uncancelled recursive candidate query should complete");
    let repeat = session
        .indexed_references_to(
            &mir_analyzer::Name::method("App\\C0", "save"),
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("warm repeat should also complete");
    assert_eq!(refs, repeat);
}

/// A file committed via `FileAnalyzer::analyze` before a class it
/// references gets registered elsewhere must be re-verified once that class
/// exists, even though its own text never changes.
#[test]
fn indexed_references_to_recovers_from_commit_before_dependency_registered() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let caller_path: Arc<str> = Arc::from("/proj/caller.php");
    let caller_src = "<?php
namespace App;
class Caller {
    private Svc $svc;
    public function go(): void { $this->svc->run(); }
}
";
    session.ingest_file(caller_path.clone(), Arc::from(caller_src));

    // Commit caller.php's postings before Svc exists.
    let parsed = php_rs_parser::parse(caller_src);
    FileAnalyzer::new(&session).analyze_diagnostics_only(
        caller_path.clone(),
        caller_src,
        &parsed.program,
        &parsed.source_map,
    );

    let name = mir_analyzer::Name::method("App\\Svc", "run");
    let before = session
        .indexed_references_to(
            &name,
            std::slice::from_ref(&caller_path),
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        before.is_empty(),
        "Svc is not registered yet, so the typed key must not resolve: {before:?}"
    );

    // A new file bumps the workspace generation; caller.php's text is untouched.
    let svc_path: Arc<str> = Arc::from("/proj/svc.php");
    let svc_src = "<?php
namespace App;
class Svc { public function run(): void {} }
";
    session.ingest_file(svc_path.clone(), Arc::from(svc_src));

    // No re-analysis of caller.php in between — staleness must catch this alone.
    let files: Vec<Arc<str>> = vec![caller_path.clone(), svc_path.clone()];
    let after = session
        .indexed_references_to(
            &name,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        after.len(),
        1,
        "caller.php's stale pre-registration commit must be re-verified now \
         that Svc exists, not served from the incomplete cached posting: {after:?}"
    );
}

/// Same staleness class through the edit path: the missing class appears in
/// an *already-registered* file, so no new-file registration is involved —
/// the ingest of a newly-defined symbol must advance the generation itself.
#[test]
fn indexed_references_to_recovers_when_dependency_appears_in_existing_file() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let caller_path: Arc<str> = Arc::from("/proj/caller.php");
    let caller_src = "<?php
namespace App;
class Caller {
    private Svc $svc;
    public function go(): void { $this->svc->run(); }
}
";
    session.ingest_file(caller_path.clone(), Arc::from(caller_src));

    // svc.php is registered from the start — but without Svc.
    let svc_path: Arc<str> = Arc::from("/proj/svc.php");
    session.ingest_file(svc_path.clone(), Arc::from("<?php\nnamespace App;\n"));

    // Commit both files' postings while Svc is still undefined.
    let name = mir_analyzer::Name::method("App\\Svc", "run");
    let files: Vec<Arc<str>> = vec![caller_path.clone(), svc_path.clone()];
    let before = session
        .indexed_references_to(
            &name,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert!(
        before.is_empty(),
        "Svc is not defined yet, so the typed key must not resolve: {before:?}"
    );

    // Svc appears via an edit to the existing file — no file add, so this
    // relies on the newly-defined-symbol generation bump in ingest_file.
    session.ingest_file(
        svc_path.clone(),
        Arc::from("<?php\nnamespace App;\nclass Svc { public function run(): void {} }\n"),
    );

    let after = session
        .indexed_references_to(
            &name,
            &files,
            false,
            mir_analyzer::ReferenceIncludes::Plain,
            &|| false,
        )
        .expect("not cancelled");
    assert_eq!(
        after.len(),
        1,
        "caller.php's commit predates Svc's definition and must be \
         re-verified after the defining edit: {after:?}"
    );
}
