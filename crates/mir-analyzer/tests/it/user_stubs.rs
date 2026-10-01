// Integration tests for user-injectable stubs via ProjectAnalyzer::stub_files/stub_dirs.

use std::fs;
use std::path::PathBuf;

use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};
use tempfile::TempDir;

use crate::common::create_temp_dir;

fn write(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn stub_file_function_resolves_without_undefined_function_error() {
    let stubs_dir = create_temp_dir("stubs");
    let src_dir = create_temp_dir("source");

    let stub_file = write(
        &stubs_dir,
        "helpers.php",
        "<?php\nfunction my_helper(string $s): string { return $s; }\n",
    );
    let src_file = write(
        &src_dir,
        "main.php",
        "<?php\n$result = my_helper('hello');\n",
    );

    let mut analyzer =
        AnalysisSession::new(PhpVersion::LATEST).with_user_stubs(vec![stub_file], Vec::new());
    let result = analyzer.analyze_paths(&[src_file], &BatchOptions::new().without_symbols());

    let undefined: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .collect();

    assert!(
        undefined.is_empty(),
        "my_helper should be defined via stub file; got: {undefined:?}"
    );
}

#[test]
fn stub_directory_function_resolves_without_undefined_function_error() {
    let stubs_dir = create_temp_dir("stubs");
    let src_dir = create_temp_dir("source");

    write(
        &stubs_dir,
        "framework.php",
        "<?php\nfunction framework_fn(int $x): int { return $x; }\n",
    );
    let src_file = write(&src_dir, "main.php", "<?php\n$v = framework_fn(42);\n");

    let mut analyzer = AnalysisSession::new(PhpVersion::LATEST)
        .with_user_stubs(Vec::new(), vec![stubs_dir.path().to_path_buf()]);
    let result = analyzer.analyze_paths(&[src_file], &BatchOptions::new().without_symbols());

    let undefined: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .collect();

    assert!(
        undefined.is_empty(),
        "framework_fn should be defined via stub directory; got: {undefined:?}"
    );
}

#[test]
fn partial_stub_class_keeps_members_of_the_real_class() {
    let root = create_temp_dir("merge");
    fs::create_dir_all(root.path().join("src")).unwrap();
    fs::write(
        root.path().join("composer.json"),
        r#"{"autoload":{"psr-4":{"App\\":"src/"}}}"#,
    )
    .unwrap();
    fs::write(
        root.path().join("src/Base.php"),
        "<?php\nnamespace App;\nclass Base {\n    public function real(): int { return 1; }\n    public function stubbed(): string { return ''; }\n}\n",
    )
    .unwrap();
    let stub = write(
        &root,
        "stubs/Base.php",
        "<?php\nnamespace App;\nclass Base {\n    /** @return non-empty-string */\n    public function stubbed(): string {}\n}\n",
    );
    let consumer = write(
        &root,
        "main.php",
        "<?php\nfunction go(\\App\\Base $b): void {\n    $b->real();\n    $b->stubbed();\n}\n",
    );

    let psr4 = std::sync::Arc::new(
        mir_analyzer::composer::Psr4Map::from_composer(root.path()).expect("psr4 map"),
    );
    let mut analyzer = AnalysisSession::new(PhpVersion::LATEST)
        .with_psr4(psr4)
        .with_user_stubs(vec![stub], Vec::new());
    let result = analyzer.analyze_paths(&[consumer], &BatchOptions::new().without_symbols());

    let undefined: Vec<_> = result
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .collect();
    assert!(
        undefined.is_empty(),
        "real() lives only on the real class; got: {undefined:?}"
    );
}
