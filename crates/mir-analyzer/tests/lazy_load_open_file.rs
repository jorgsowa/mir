//! Open-file (LSP / `FileAnalyzer`) diagnostic completeness under the eager
//! background-indexing model.
//!
//! `tests/lazy_load.rs` exercises the *batch* path (`analyze_paths`). This file
//! exercises the *open-file* path (`AnalysisSession::ingest_file` +
//! `FileAnalyzer::analyze`), which the LSP uses for the currently-edited buffer.
//!
//! In the eager-static-input model the consumer indexes the project + vendor
//! files up front (`index_batch` + `finalize_index`), so the workspace symbol
//! index is complete and `find_class_like` resolves any class — including types
//! reached only through a method's return type or an inheritance chain. The
//! open-file path then needs no transitive lazy-load fixpoint: a single pass
//! against the complete index produces the same diagnostics the batch path does.
//!
//! Each test is a 0 -> 1 flip proving the relevant type was resolved through the
//! eagerly-built index rather than silently degrading to `mixed`.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, IndexCancel, IndexParallelism, PhpVersion};
use php_rs_parser::parse as php_parse;

use self::common::create_temp_dir;

/// Set up a PSR-4 project (`App\` -> `src/`) with the given library files under
/// `src/`, eagerly index them (as the background indexer would at session
/// start), then analyze `open_src` through the open-file path. After indexing
/// the input set is static, so the open file's references — and the types
/// reachable only through their signatures / inheritance — all resolve.
fn analyze_open_file_with_psr4(
    lib_files: &[(&str, &str)],
    open_name: &str,
    open_src: &str,
) -> mir_analyzer::FileAnalysis {
    let root = create_temp_dir("lazy_load_open_file");
    fs::create_dir_all(root.path().join("src")).unwrap();
    for (name, src) in lib_files {
        fs::write(root.path().join("src").join(name), src).unwrap();
    }
    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map");

    // Enumerate the project's library files before moving the map into the session.
    let index_files: Vec<(Arc<str>, Arc<str>)> = psr4
        .project_files()
        .into_iter()
        .filter_map(|p| {
            let t = fs::read_to_string(&p).ok()?;
            Some((
                Arc::from(p.to_string_lossy().as_ref()),
                Arc::from(t.as_str()),
            ))
        })
        .collect();

    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    // Eager background-index pass over the project's library files.
    let cancel = IndexCancel::new();
    session.index_batch(&index_files, IndexParallelism::Sequential, &cancel);
    session.finalize_index();

    let open_path: Arc<str> = Arc::from(root.path().join(open_name).to_string_lossy().as_ref());
    let open_src_arc: Arc<str> = Arc::from(open_src);
    session.ingest_file(open_path.clone(), open_src_arc.clone());

    let parsed = php_rs_parser::parse(open_src);
    assert!(
        parsed.errors.is_empty(),
        "parser errors in open-file source: {:?}",
        parsed.errors
    );
    FileAnalyzer::new(&session).analyze(open_path, open_src, &parsed.program, &parsed.source_map)
}

fn undefined_method_count(analysis: &mir_analyzer::FileAnalysis) -> usize {
    analysis
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .count()
}

/// Set up a PSR-4 project whose `lib_files` are written to disk (resolvable via
/// the ClassResolver) but deliberately NOT eagerly indexed. The only way the
/// open file's references reach those classes is `priority_index_for_ast`'s
/// pre-load pass — the warm-up window these tests exercise. `lib_files` names
/// may include subdirectories (e.g. `Contract/Runnable.php`).
fn analyze_open_file_unindexed(
    lib_files: &[(&str, &str)],
    open_name: &str,
    open_src: &str,
) -> mir_analyzer::FileAnalysis {
    let root = common::create_temp_dir("priority_index_unindexed");
    for (name, src) in lib_files {
        let p = root.path().join("src").join(name);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, src).unwrap();
    }
    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();

    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map");
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    let open_path: Arc<str> = Arc::from(
        root.path()
            .join("src")
            .join(open_name)
            .to_string_lossy()
            .as_ref(),
    );
    session.ingest_file(open_path.clone(), Arc::from(open_src));

    let parsed = php_parse(open_src);
    assert!(
        parsed.errors.is_empty(),
        "parse errors: {:?}",
        parsed.errors
    );
    FileAnalyzer::new(&session).analyze(open_path, open_src, &parsed.program, &parsed.source_map)
}

/// `extends` with a bare, same-namespace name (no `use`): `Child extends Base`
/// resolves to `App\Base` purely through the namespace. The parent is reachable
/// only via the resolver, so `priority_index_for_ast` must collect the `extends`
/// name and pre-load it; otherwise Child's ancestor chain stays incomplete and
/// the genuinely-undefined `$c->nope()` is suppressed.
#[test]
fn priority_index_pre_loads_same_namespace_extends() {
    let analysis = analyze_open_file_unindexed(
        &[(
            "Base.php",
            "<?php\nnamespace App;\nclass Base {\n    public function fromBase(): void {}\n}\n",
        )],
        "Child.php",
        "<?php\nnamespace App;\nclass Child extends Base {}\n$c = new Child();\n$c->nope();\n",
    );

    assert_eq!(
        undefined_method_count(&analysis),
        1,
        "Child extends the bare same-namespace name `Base` (App\\Base), loadable only via the \
         resolver. Once `extends` is collected and Base pre-loaded, the ancestor chain is \
         complete and $c->nope() surfaces. A count of 0 means `extends` was never collected. \
         Got issues: {:?}",
        analysis.issues
    );
}

/// `implements` with a `use`-imported alias: `Worker implements Runnable` where
/// `use App\Contract\Runnable` aliases the interface. Until the alias is
/// collected and the interface pre-loaded, Worker has an unknown ancestor and
/// member checks against it are suppressed; once loaded, the hierarchy is
/// complete and the genuinely-undefined `$w->nope()` surfaces.
#[test]
fn priority_index_pre_loads_use_imported_implements() {
    let analysis = analyze_open_file_unindexed(
        &[(
            "Contract/Runnable.php",
            "<?php\nnamespace App\\Contract;\ninterface Runnable {\n    public function run(): void;\n}\n",
        )],
        "Worker.php",
        "<?php\nnamespace App;\nuse App\\Contract\\Runnable;\nclass Worker implements Runnable {}\n$w = new Worker();\n$w->nope();\n",
    );

    assert_eq!(
        undefined_method_count(&analysis),
        1,
        "Worker implements the use-imported `Runnable` (App\\Contract\\Runnable). Until the \
         alias is collected and the interface pre-loaded, Worker has an unknown ancestor and \
         $w->nope() is suppressed. Once loaded, the hierarchy is complete and nope() surfaces. \
         A count of 0 means `implements` was never collected. Got issues: {:?}",
        analysis.issues
    );
}

/// The open file calls a method that only exists on the *return type* of a
/// lazily-loaded class. `App\Widget` is never named directly in the open file —
/// it is reachable only through `App\Service::make()`'s declared return type.
/// To know that `nope()` is undefined on the returned `Widget`, the loader must
/// follow the return-type edge and load `Widget`.
#[test]
fn open_file_resolves_method_on_lazily_loaded_return_type() {
    let analysis = analyze_open_file_with_psr4(
        &[
            (
                "Widget.php",
                "<?php\nnamespace App;\nclass Widget {\n    public function render(): void {}\n}\n",
            ),
            (
                "Service.php",
                "<?php\nnamespace App;\nclass Service {\n    public function make(): \\App\\Widget {\n        return new \\App\\Widget();\n    }\n}\n",
            ),
        ],
        "consumer.php",
        "<?php\n$s = new \\App\\Service();\n$w = $s->make();\n$w->nope();\n",
    );

    assert_eq!(
        undefined_method_count(&analysis),
        1,
        "Widget is reachable only via Service::make()'s return type. Once that closure \
         is loaded, $w->nope() is a known-class/unknown-method -> UndefinedMethod. A count \
         of 0 means Widget was never loaded and the value silently degraded to mixed."
    );
}

/// The open file references `App\Mid` directly, but the member it (in)correctly
/// calls would be inherited from `App\Base`, which is reachable only through
/// Mid's `extends` edge. Until Base is loaded, Mid has an unknown ancestor and
/// member checks against it are suppressed (to avoid false positives), so
/// `nope()` is missed. Once the inheritance closure is loaded, the ancestor
/// chain is complete and the genuinely-undefined `nope()` surfaces.
#[test]
fn open_file_resolves_inherited_chain_of_lazily_loaded_parent() {
    let analysis = analyze_open_file_with_psr4(
        &[
            (
                "Base.php",
                "<?php\nnamespace App;\nclass Base {\n    public function fromBase(): void {}\n}\n",
            ),
            (
                "Mid.php",
                "<?php\nnamespace App;\nclass Mid extends \\App\\Base {}\n",
            ),
        ],
        "consumer.php",
        "<?php\n$m = new \\App\\Mid();\n$m->nope();\n",
    );

    assert_eq!(
        undefined_method_count(&analysis),
        1,
        "Mid extends the lazily-loadable Base. Once the inheritance closure is loaded the \
         ancestor chain is complete and $m->nope() (absent on Mid and Base) is reported. A \
         count of 0 means Base was never loaded, Mid had an unknown ancestor, and the check \
         was suppressed."
    );
}

/// Control: a member that genuinely exists on the lazily-loaded parent must NOT
/// be flagged once the closure is loaded — guards against the fix over-reporting.
#[test]
fn open_file_does_not_flag_valid_inherited_member() {
    let analysis = analyze_open_file_with_psr4(
        &[
            (
                "Base.php",
                "<?php\nnamespace App;\nclass Base {\n    public function fromBase(): void {}\n}\n",
            ),
            (
                "Mid.php",
                "<?php\nnamespace App;\nclass Mid extends \\App\\Base {}\n",
            ),
        ],
        "consumer.php",
        "<?php\n$m = new \\App\\Mid();\n$m->fromBase();\n",
    );

    assert_eq!(
        undefined_method_count(&analysis),
        0,
        "fromBase() is inherited from the lazily-loaded Base and must resolve cleanly"
    );
}

/// Verify that `priority_index_for_ast` pre-loads classes referenced *only*
/// inside `@param` / `@return` docblock annotations (no PHP type hint).
///
/// Before the docblock-scanning extension, `collect_class_refs_from_ast` only
/// walked PHP type hints and expression sites (`new X`, `X::method()`, etc.).
/// A class that appeared purely as a docblock `@param` type would be invisible
/// to the pre-loader.  If the background indexer hadn't yet reached that class
/// and it was only resolvable via the ClassResolver, the type would degrade to
/// `mixed` and method checks on it would be silently skipped.
///
/// This test uses `FileAnalyzer` (the open-file / LSP path) with a PSR-4
/// ClassResolver but WITHOUT eagerly indexing `Widget.php`.  The only way
/// `Widget` can be discovered is through the `@param Widget $w` docblock scanned
/// by the pre-loader — no PHP hint, no `new Widget`, no `Widget::`. Once loaded,
/// `$w->nope()` must resolve to an `UndefinedMethod` diagnostic.
#[test]
fn priority_index_pre_loads_docblock_only_param_class() {
    let root = common::create_temp_dir("docblock_preload");
    fs::create_dir_all(root.path().join("src")).unwrap();

    fs::write(
        root.path().join("src/Widget.php"),
        "<?php\nnamespace App;\nclass Widget {\n    public function render(): void {}\n}\n",
    )
    .unwrap();
    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();

    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map");
    // Widget.php is NOT eagerly indexed — only the ClassResolver can find it.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    let open_src = "<?php\n\
        namespace App;\n\
        class Service {\n\
            /**\n\
             * @param Widget $w\n\
             */\n\
            public function process($w): void {\n\
                $w->nope();\n\
            }\n\
        }\n";

    let open_path: Arc<str> = Arc::from(
        root.path()
            .join("src/Service.php")
            .to_string_lossy()
            .as_ref(),
    );
    session.ingest_file(open_path.clone(), Arc::from(open_src));

    let parsed = php_parse(open_src);
    assert!(
        parsed.errors.is_empty(),
        "parse errors: {:?}",
        parsed.errors
    );

    let analysis = FileAnalyzer::new(&session).analyze(
        open_path,
        open_src,
        &parsed.program,
        &parsed.source_map,
    );

    assert_eq!(
        undefined_method_count(&analysis),
        1,
        "Widget should be pre-loaded via @param docblock scanning so that \
         $w->nope() is type-checked. A count of 0 means Widget was not loaded \
         and the call silently degraded to mixed. Got issues: {:?}",
        analysis.issues
    );
}
