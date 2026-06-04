//! Regression: batch `analyze_paths` must resolve classes when the workspace
//! symbol index singleton was already built (from vendor) *before* the project
//! files were registered.
//!
//! The eager-indexing rework made `workspace_index` read a maintained singleton
//! and stopped nulling it when files are added. The CLI builds that singleton
//! from vendor (`index_vendor_chunked` / `collect_definitions`) BEFORE calling
//! `analyze_paths`, which then registers project files and lazy-loads referenced
//! classes — but nothing refreshed the singleton, so every project class and
//! every lazy-loaded class was absent from it and reported as a false
//! `UndefinedClass`. On the real Laravel tree this produced ~34k bogus
//! `UndefinedClass` (e.g. `Illuminate\Support\Str`, all real project classes).
//!
//! Precondition that triggers the bug: the singleton must be SET before
//! `analyze_paths` runs. A run that never pre-builds it falls back to the
//! always-correct tracked query and hides the bug — which is why small fixtures
//! and `run_plain_flow` never caught it.

mod common;

use std::fs;
use std::sync::Arc;

use mir_analyzer::{AnalysisSession, BatchOptions, IndexCancel, IndexParallelism, PhpVersion};
use mir_issues::IssueKind;

use self::common::create_temp_dir;

fn arc_pair(path: &std::path::Path) -> (Arc<str>, Arc<str>) {
    let src = fs::read_to_string(path).unwrap();
    (
        Arc::from(path.to_string_lossy().as_ref()),
        Arc::from(src.as_str()),
    )
}

#[test]
fn batch_resolves_project_and_lazy_classes_with_prebuilt_index() {
    let dir = create_temp_dir("batch_index_resolution");
    let root = dir.path();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("vendor/acme/lib/src")).unwrap();
    fs::create_dir_all(root.join("vendor/composer")).unwrap();

    // Project: App\ -> src/. Vendor packages come from installed.json.
    fs::write(
        root.join("composer.json"),
        r#"{ "name": "t/app", "autoload": { "psr-4": { "App\\": "src/" } } }"#,
    )
    .unwrap();
    fs::write(
        root.join("vendor/composer/installed.json"),
        r#"{ "packages": [ { "name": "acme/lib", "autoload": { "psr-4": { "Acme\\": "src/" } } } ] }"#,
    )
    .unwrap();

    // A vendor class used to PRE-BUILD the singleton (mirrors the CLI indexing
    // vendor before analyze_paths).
    fs::write(
        root.join("vendor/acme/lib/src/Bootstrap.php"),
        "<?php\nnamespace Acme;\nclass Bootstrap {}\n",
    )
    .unwrap();
    // A vendor class that is NOT pre-indexed — it must be lazy-loaded AND merged
    // into the singleton during analyze_paths.
    fs::write(
        root.join("vendor/acme/lib/src/Widget.php"),
        "<?php\nnamespace Acme;\nclass Widget { public function name(): string { return \"w\"; } }\n",
    )
    .unwrap();

    // A project class referenced by another project file (both are in the
    // analyzed set, but absent from the vendor-only singleton until refreshed).
    fs::write(
        root.join("src/Helper.php"),
        "<?php\nnamespace App;\nclass Helper { public function help(): string { return \"h\"; } }\n",
    )
    .unwrap();
    let service = root.join("src/Service.php");
    fs::write(
        &service,
        "<?php\nnamespace App;\nuse Acme\\Widget;\nclass Service {\n    public function run(Widget $w, Helper $h): string { return $w->name() . $h->help(); }\n}\n",
    )
    .unwrap();

    let psr4 = mir_analyzer::composer::Psr4Map::from_composer(root).expect("psr4");
    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));
    session.ensure_all_stubs();

    // PRE-BUILD the workspace index singleton from a vendor file, exactly as the
    // CLI does before analyze_paths. This is the precondition that triggers the
    // regression.
    let cancel = IndexCancel::new();
    let boot = arc_pair(&root.join("vendor/acme/lib/src/Bootstrap.php"));
    session.index_batch(&[boot], IndexParallelism::Sequential, &cancel);

    // Now analyze the project. `Acme\Widget` (lazy-loaded vendor) and `App\Helper`
    // (sibling project class) are both real and must resolve.
    let helper = root.join("src/Helper.php");
    let result = session.analyze_paths(&[service.clone(), helper], &BatchOptions::new());

    let undefined: Vec<&str> = result
        .issues
        .iter()
        .filter_map(|i| match &i.kind {
            IssueKind::UndefinedClass { name } => Some(name.as_str()),
            _ => None,
        })
        .collect();

    assert!(
        undefined.is_empty(),
        "real classes wrongly reported UndefinedClass after pre-built index: {:?}",
        undefined
    );
}
