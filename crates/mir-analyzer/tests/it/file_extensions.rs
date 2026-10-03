//! Non-`.php` source extensions (`.module`, `.inc`) are collected and
//! followed through `include_once` when configured.

use std::fs;
use std::path::PathBuf;

use mir_analyzer::{
    composer::Psr4Map, discover_files, discover_files_with_extensions, include_closure,
    AnalysisSession, BatchOptions, PhpFileExtensions, PhpVersion, VendorTargets,
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
fn include_closure_reaches_inc_target_with_extra_extensions() {
    let f = module_and_inc_fixture();
    let exts = PhpFileExtensions::new([".php", "Module", "INC"]);

    let files = include_closure(vec![f.module.clone()], VendorTargets::Follow, &exts);

    assert!(files.contains(&f.root.join("b.inc")), "got {files:?}");
    assert!(undefined_functions(&files).is_empty());
}

#[test]
fn include_closure_ignores_inc_target_by_default() {
    let f = module_and_inc_fixture();

    let files = include_closure(
        vec![f.module.clone()],
        VendorTargets::Follow,
        &PhpFileExtensions::default(),
    );

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

fn write(root: &std::path::Path, rel: &str, content: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn walk_recurses_and_skips_excluded_dirs_with_custom_extensions() {
    let dir = create_temp_dir("walk_skips");
    let root = dir.path();
    write(root, "a/b/deep.inc", "<?php\n");
    write(root, "top.module", "<?php\n");
    write(root, "vendor/skip.inc", "<?php\n");
    write(root, ".git/skip.inc", "<?php\n");
    write(root, "node_modules/skip.inc", "<?php\n");
    write(root, "a/ignored.txt", "x");

    let exts = PhpFileExtensions::new(["module", "inc"]);
    let mut found = discover_files_with_extensions(root, &exts);
    found.sort();

    assert_eq!(
        found,
        vec![root.join("a/b/deep.inc"), root.join("top.module")]
    );
}

#[cfg(unix)]
#[test]
fn walk_skips_symlinks_with_custom_extensions() {
    let dir = create_temp_dir("walk_symlinks");
    let root = dir.path();
    write(root, "real/x.inc", "<?php\n");
    write(root, "real.inc", "<?php\n");
    std::os::unix::fs::symlink(root.join("real"), root.join("linked_dir")).unwrap();
    std::os::unix::fs::symlink(root.join("real.inc"), root.join("linked.inc")).unwrap();

    let exts = PhpFileExtensions::new(["inc"]);
    let mut found = discover_files_with_extensions(root, &exts);
    found.sort();

    assert_eq!(found, vec![root.join("real/x.inc"), root.join("real.inc")]);
}

fn write_vendor_package(root: &std::path::Path, autoload: &str) -> PathBuf {
    write(
        root,
        "composer.json",
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    );
    write(
        root,
        "vendor/composer/installed.json",
        &format!(r#"{{"packages":[{{"name":"v/pkg","autoload":{autoload}}}]}}"#),
    );
    root.join("vendor/v/pkg")
}

#[test]
fn vendor_files_use_configured_extensions() {
    let dir = create_temp_dir("vendor_files_ext");
    let root = dir.path();
    let pkg = write_vendor_package(root, r#"{"classmap":["lib/"]}"#);
    write(&pkg, "lib/a.inc", "<?php\n");
    write(&pkg, "lib/b.php", "<?php\n");

    let default = Psr4Map::from_composer(root).unwrap().vendor_files();
    assert_eq!(default, vec![pkg.join("lib/b.php")]);

    let exts = PhpFileExtensions::new(["php", "inc"]);
    let mut files = Psr4Map::from_composer_with_extensions(root, exts)
        .unwrap()
        .vendor_files();
    files.sort();
    assert_eq!(files, vec![pkg.join("lib/a.inc"), pkg.join("lib/b.php")]);
}

#[test]
fn vendor_eager_files_follow_sibling_inc_when_configured() {
    let dir = create_temp_dir("vendor_eager_ext");
    let root = dir.path();
    let pkg = write_vendor_package(root, r#"{"files":["bootstrap.php"]}"#);
    write(
        &pkg,
        "bootstrap.php",
        "<?php require __DIR__ . '/impl.inc';\n",
    );
    write(&pkg, "impl.inc", "<?php function pkg_impl() {}\n");

    let default = Psr4Map::from_composer(root).unwrap().vendor_eager_files();
    assert_eq!(default, vec![pkg.join("bootstrap.php")]);

    let exts = PhpFileExtensions::new(["php", "inc"]);
    let eager = Psr4Map::from_composer_with_extensions(root, exts)
        .unwrap()
        .vendor_eager_files();
    assert!(eager.contains(&pkg.join("impl.inc")), "got {eager:?}");
}

#[test]
fn include_closure_follows_absolute_path_literal() {
    let f = module_and_inc_fixture();
    let abs = f.root.join("b.inc");
    write(
        &f.root,
        "abs.module",
        &format!("<?php\nrequire '{}';\n", abs.display()),
    );
    let exts = PhpFileExtensions::new(["php", "module", "inc"]);

    let files = include_closure(vec![f.root.join("abs.module")], VendorTargets::Skip, &exts);

    assert!(files.contains(&abs), "got {files:?}");
}

#[test]
fn include_closure_skips_vendor_targets_only_when_asked() {
    let dir = create_temp_dir("closure_vendor");
    let root = dir.path();
    write(
        root,
        "a.module",
        "<?php require __DIR__ . '/vendor/h.inc';\n",
    );
    write(root, "vendor/h.inc", "<?php\n");
    let exts = PhpFileExtensions::new(["module", "inc"]);
    let seed = vec![root.join("a.module")];

    let skipped = include_closure(seed.clone(), VendorTargets::Skip, &exts);
    let followed = include_closure(seed, VendorTargets::Follow, &exts);

    assert_eq!(skipped, vec![root.join("a.module")]);
    assert!(followed.contains(&root.join("vendor/h.inc")));
}
