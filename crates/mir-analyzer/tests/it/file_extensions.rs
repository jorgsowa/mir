//! Non-`.php` source extensions (`.module`, `.inc`) are collected and
//! followed through `include_once` when configured.

use std::fs;
use std::path::PathBuf;

use mir_analyzer::{
    composer::{follow_includes, Psr4Map},
    discover_files, discover_files_with_extensions, AnalysisSession, BatchOptions,
    PhpFileExtensions, PhpVersion,
};
use mir_issues::IssueKind;

use crate::common::create_temp_dir;

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    module: PathBuf,
}

fn module_and_inc_fixture() -> Fixture {
    let dir = create_temp_dir("file_extensions");
    let root = dir.path().to_path_buf();
    fs::create_dir_all(root.join("sub")).unwrap();
    let module = root.join("a.module");
    fs::write(
        &module,
        "<?php\ninclude_once __DIR__ . '/b.inc';\nfunction a_hook(): string { return b_helper(); }\n",
    )
    .unwrap();
    fs::write(
        root.join("b.inc"),
        "<?php\nfunction b_helper(): string { return 'b'; }\n",
    )
    .unwrap();
    fs::write(root.join("sub/c.php"), "<?php\nfunction c() {}\n").unwrap();
    Fixture {
        _dir: dir,
        root,
        module,
    }
}

fn undefined_functions(files: &[PathBuf]) -> Vec<String> {
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    let result = session.analyze_paths(files, &BatchOptions::new().without_symbols());
    result
        .issues
        .iter()
        .filter_map(|i| match &i.kind {
            IssueKind::UndefinedFunction { name } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn follow_includes_reaches_inc_target_with_extra_extensions() {
    let f = module_and_inc_fixture();
    let exts = PhpFileExtensions::new([".php", "Module", "INC"]);

    let files = follow_includes(vec![f.module.clone()], false, &exts);

    assert!(files.contains(&f.root.join("b.inc")), "got {files:?}");
    assert!(undefined_functions(&files).is_empty());
}

#[test]
fn follow_includes_ignores_inc_target_by_default() {
    let f = module_and_inc_fixture();

    let files = follow_includes(vec![f.module.clone()], false, &PhpFileExtensions::default());

    assert_eq!(files, vec![f.module.clone()]);
    assert_eq!(undefined_functions(&files), vec!["b_helper".to_string()]);
}

#[test]
fn discover_files_collects_only_configured_extensions() {
    let f = module_and_inc_fixture();

    let default = discover_files(&f.root);
    assert_eq!(default, vec![f.root.join("sub/c.php")]);

    let exts = PhpFileExtensions::new(["php", "module", "inc"]);
    let mut found = discover_files_with_extensions(&f.root, &exts);
    found.sort();
    assert_eq!(
        found,
        vec![
            f.root.join("a.module"),
            f.root.join("b.inc"),
            f.root.join("sub/c.php"),
        ]
    );
}

#[test]
fn psr4_project_files_use_configured_extensions() {
    let f = module_and_inc_fixture();
    fs::write(
        f.root.join("composer.json"),
        r#"{ "autoload": { "classmap": ["a.module"], "psr-4": { "App\\": "sub/" } } }"#,
    )
    .unwrap();

    let default = Psr4Map::from_composer(&f.root).unwrap().project_files();
    assert_eq!(default, vec![f.root.join("sub/c.php")]);

    let exts = PhpFileExtensions::new(["php", "module", "inc"]);
    let map = Psr4Map::from_composer_with_extensions(&f.root, exts).unwrap();
    let files = map.project_files();
    assert!(files.contains(&f.root.join("a.module")), "got {files:?}");
    assert!(files.contains(&f.root.join("b.inc")), "got {files:?}");
}
