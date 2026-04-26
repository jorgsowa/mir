// Integration tests for user-injectable stubs via ProjectAnalyzer::stub_files/stub_dirs.

use std::fs;
use std::path::PathBuf;

use mir_analyzer::ProjectAnalyzer;
use tempfile::TempDir;

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
    let stubs_dir = TempDir::new().unwrap();
    let src_dir = TempDir::new().unwrap();

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

    let mut analyzer = ProjectAnalyzer::new();
    analyzer.stub_files = vec![stub_file];
    let result = analyzer.analyze(&[src_file]);

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
    let stubs_dir = TempDir::new().unwrap();
    let src_dir = TempDir::new().unwrap();

    write(
        &stubs_dir,
        "framework.php",
        "<?php\nfunction framework_fn(int $x): int { return $x; }\n",
    );
    let src_file = write(&src_dir, "main.php", "<?php\n$v = framework_fn(42);\n");

    let mut analyzer = ProjectAnalyzer::new();
    analyzer.stub_dirs = vec![stubs_dir.path().to_path_buf()];
    let result = analyzer.analyze(&[src_file]);

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
