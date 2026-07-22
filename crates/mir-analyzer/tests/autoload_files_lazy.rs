//! Regression tests: Composer `autoload.files` functions lazy-loaded automatically.
//!
//! Composer's `autoload.files` lists vendor files that define global functions
//! and constants.  Unlike PSR-4 classes there is no name → path mapping, so
//! the class resolver cannot discover them on demand.  Instead, `with_psr4`
//! registers their paths and `prepare_ast_for_analysis` (called from every
//! `FileAnalyzer::analyze`) indexes them on the first analysis call with no
//! action required from the consumer.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, IndexCancel, IndexParallelism, PhpVersion};

use self::common::create_temp_dir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal Composer vendor tree under `root` with one `autoload.files`
/// entry pointing to `vendor/helpers/functions.php`.
///
/// The generated `autoload_files.php` uses the Composer-generated variable
/// format (`$vendorDir . '/helpers/functions.php'`); mir's composer parser
/// resolves `$vendorDir` to the real vendor directory at read time.
fn write_vendor_autoload_files(root: &std::path::Path, function_src: &str) {
    fs::create_dir_all(root.join("vendor/composer")).unwrap();
    fs::create_dir_all(root.join("vendor/helpers")).unwrap();

    // Composer-generated autoload_files.php format.
    fs::write(
        root.join("vendor/composer/autoload_files.php"),
        "<?php\n\
         $vendorDir = dirname(__DIR__);\n\
         $baseDir = dirname($vendorDir);\n\
         return array(\n\
             'abc123' => $vendorDir . '/helpers/functions.php',\n\
         );\n",
    )
    .unwrap();
    fs::write(
        root.join("vendor/composer/autoload_psr4.php"),
        "<?php\nreturn [];\n",
    )
    .unwrap();
    fs::write(
        root.join("vendor/composer/autoload_classmap.php"),
        "<?php\nreturn [];\n",
    )
    .unwrap();
    fs::write(
        root.join("vendor/composer/autoload_namespaces.php"),
        "<?php\nreturn [];\n",
    )
    .unwrap();
    fs::write(root.join("vendor/helpers/functions.php"), function_src).unwrap();
}

fn write_composer_json(root: &std::path::Path) {
    fs::write(
        root.join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
}

fn undefined_function_count(issues: &[mir_analyzer::Issue]) -> usize {
    issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count()
}

// ---------------------------------------------------------------------------
// FileAnalyzer (open-file / LSP) path
// ---------------------------------------------------------------------------

/// No manual indexing call. `FileAnalyzer::analyze` must lazy-load vendor
/// `autoload.files` functions automatically on the first call.
#[test]
fn file_analyzer_lazy_loads_vendor_autoload_files_functions() {
    let root = create_temp_dir("autoload_lazy_file");
    write_vendor_autoload_files(
        root.path(),
        "<?php\nfunction vendor_helper(string $s): string { return $s; }\n",
    );
    write_composer_json(root.path());

    let psr4 =
        mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 from composer");
    // No manual index_batch / index_vendor_eager_files call.
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    let src = "<?php\nvendor_helper('test');\n";
    let path: Arc<str> = Arc::from("consumer.php");
    session.ingest_file(path.clone(), Arc::from(src));

    let parsed = php_rs_parser::parse(src);
    let result =
        FileAnalyzer::new(&session).analyze(path, src, &parsed.program, &parsed.source_map);

    assert_eq!(
        undefined_function_count(&result.issues),
        0,
        "vendor_helper() must be found without a manual indexing call; \
         got issues: {:?}",
        result.issues
    );
}

/// A second `FileAnalyzer::analyze` call on the same session must not
/// re-index the eager files (idempotency / double-load guard).
#[test]
fn file_analyzer_lazy_load_is_idempotent_across_calls() {
    let root = create_temp_dir("autoload_lazy_idempotent");
    write_vendor_autoload_files(
        root.path(),
        "<?php\nfunction idem_fn(): int { return 1; }\n",
    );
    write_composer_json(root.path());

    let psr4 =
        mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 from composer");
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    let src = "<?php\nidem_fn();\n";
    let path: Arc<str> = Arc::from("a.php");

    for _ in 0..3 {
        session.ingest_file(path.clone(), Arc::from(src));
        let parsed = php_rs_parser::parse(src);
        let result = FileAnalyzer::new(&session).analyze(
            path.clone(),
            src,
            &parsed.program,
            &parsed.source_map,
        );
        assert_eq!(
            undefined_function_count(&result.issues),
            0,
            "idem_fn() must resolve on every call, not just the first"
        );
    }
}

/// Calling `FileAnalyzer::analyze` on a session *without* a PSR-4 map must
/// not panic or regress — `ensure_vendor_eager_functions` is a no-op when
/// no psr4 map is attached.
#[test]
fn file_analyzer_no_psr4_does_not_panic() {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let src = "<?php\necho 1 + 1;\n";
    let path: Arc<str> = Arc::from("plain.php");
    session.ingest_file(path.clone(), Arc::from(src));
    let parsed = php_rs_parser::parse(src);
    let result =
        FileAnalyzer::new(&session).analyze(path, src, &parsed.program, &parsed.source_map);
    assert!(
        result.issues.is_empty(),
        "no issues expected for trivial PHP; got: {:?}",
        result.issues
    );
}

// ---------------------------------------------------------------------------
// Project autoload.files (via normal project_files() indexing)
// ---------------------------------------------------------------------------

/// Project-level `autoload.files` entries are included in `project_files()`.
/// When indexed via `index_batch(&psr4.project_files(), ...)` as any LSP or
/// CLI would do, the functions they define are visible to analysis.
#[test]
fn project_autoload_files_indexed_via_project_files() {
    let root = create_temp_dir("project_autoload");
    fs::create_dir_all(root.path().join("src")).unwrap();

    // A project-level helper file (not in vendor/).
    fs::write(
        root.path().join("src/helpers.php"),
        "<?php\nfunction project_helper(string $x): bool { return strlen($x) > 0; }\n",
    )
    .unwrap();

    // composer.json with project autoload.files entry.
    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"},"files":["src/helpers.php"]}}"#,
    )
    .unwrap();

    let psr4 =
        mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 from composer");

    // Index all project files (project_files() includes autoload.files entries).
    let index_files: Vec<(Arc<str>, Arc<str>)> = psr4
        .project_files()
        .into_iter()
        .filter_map(|p| {
            let text = fs::read_to_string(&p).ok()?;
            Some((
                Arc::from(p.to_string_lossy().as_ref()),
                Arc::from(text.as_str()),
            ))
        })
        .collect();

    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));
    let cancel = IndexCancel::new();
    session.index_batch(&index_files, IndexParallelism::Sequential, &cancel);
    session.finalize_index();

    let src = "<?php\nproject_helper('hello');\n";
    let path: Arc<str> = Arc::from("consumer.php");
    session.ingest_file(path.clone(), Arc::from(src));
    let parsed = php_rs_parser::parse(src);
    let result =
        FileAnalyzer::new(&session).analyze(path, src, &parsed.program, &parsed.source_map);

    assert_eq!(
        undefined_function_count(&result.issues),
        0,
        "project_helper() must be found after indexing project_files(); \
         got issues: {:?}",
        result.issues
    );
}

/// A vendor package that defines global functions guarded by `function_exists`
/// (the common Laravel pattern) must still be found after lazy loading.
#[test]
fn vendor_autoload_files_function_exists_guard_is_transparent() {
    let root = create_temp_dir("autoload_lazy_guard");
    write_vendor_autoload_files(
        root.path(),
        "<?php\nif (! function_exists('guarded_fn')) {\n\
             function guarded_fn(string $s): string { return $s; }\n\
         }\n",
    );
    write_composer_json(root.path());

    let psr4 =
        mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 from composer");
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));

    let src = "<?php\nguarded_fn('hello');\n";
    let path: Arc<str> = Arc::from("consumer.php");
    session.ingest_file(path.clone(), Arc::from(src));
    let parsed = php_rs_parser::parse(src);
    let result =
        FileAnalyzer::new(&session).analyze(path, src, &parsed.program, &parsed.source_map);

    assert_eq!(
        undefined_function_count(&result.issues),
        0,
        "guarded_fn() is defined inside an if-block — the collector must recurse into it; \
         got issues: {:?}",
        result.issues
    );
}
