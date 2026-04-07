// Integration tests for incremental single-file re-analysis (mir#79).

use std::fs;
use std::path::PathBuf;

use mir_analyzer::ProjectAnalyzer;
use tempfile::TempDir;

fn write(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, content).unwrap();
    path
}

#[test]
fn re_analyze_file_picks_up_new_error() {
    let src_dir = TempDir::new().unwrap();

    // Initial file: valid code, no issues expected for undefined functions
    let file_a = write(
        &src_dir,
        "A.php",
        "<?php\nfunction greet(): string { return 'hello'; }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    let result1 = analyzer.analyze(std::slice::from_ref(&file_a));

    // The initial code should have no UndefinedFunction issues
    let undef_fn_count = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert_eq!(
        undef_fn_count, 0,
        "initial code should have no UndefinedFunction"
    );

    // Now re-analyze the same file with content that calls an undefined function
    let file_path = file_a.to_string_lossy().to_string();
    let new_content = "<?php\nfunction test(): void { nonexistent_func(); }\n";
    let result2 = analyzer.re_analyze_file(&file_path, new_content);

    let undef_fn_count2 = result2
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert!(
        undef_fn_count2 > 0,
        "re-analyzed code should report UndefinedFunction, got issues: {:?}",
        result2
            .issues
            .iter()
            .map(|i| i.kind.name())
            .collect::<Vec<_>>()
    );
}

#[test]
fn re_analyze_file_removes_old_definitions() {
    let src_dir = TempDir::new().unwrap();

    // Initial: defines class Foo with method bar()
    let file_a = write(
        &src_dir,
        "A.php",
        "<?php\nclass Foo { public function bar(): void {} }\n",
    );
    let file_b = write(
        &src_dir,
        "B.php",
        "<?php\nfunction test(): void { $f = new Foo(); $f->bar(); }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    let result1 = analyzer.analyze(&[file_a.clone(), file_b.clone()]);

    // bar() exists, so no UndefinedMethod on file B
    let undef_method = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedMethod")
        .count();
    assert_eq!(undef_method, 0, "bar() should be found");

    // Now change A.php: rename the method from bar() to baz()
    let file_path_a = file_a.to_string_lossy().to_string();
    let new_content_a = "<?php\nclass Foo { public function baz(): void {} }\n";
    let _result2 = analyzer.re_analyze_file(&file_path_a, new_content_a);

    // Verify the old method bar() is gone and baz() exists
    assert!(
        analyzer.codebase().get_method("Foo", "baz").is_some(),
        "baz() should exist after re-analysis"
    );
    assert!(
        analyzer.codebase().get_method("Foo", "bar").is_none(),
        "bar() should be removed after re-analysis"
    );
}

#[test]
fn re_analyze_file_fixes_error() {
    let src_dir = TempDir::new().unwrap();

    // Initial: code with a call to an undefined function
    let file_a = write(
        &src_dir,
        "A.php",
        "<?php\nfunction test(): void { missing_fn(); }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    let result1 = analyzer.analyze(std::slice::from_ref(&file_a));

    let undef_count = result1
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert!(undef_count > 0, "should have UndefinedFunction initially");

    // Fix the file: define the function and call it
    let file_path = file_a.to_string_lossy().to_string();
    let new_content =
        "<?php\nfunction missing_fn(): void {}\nfunction test(): void { missing_fn(); }\n";
    let result2 = analyzer.re_analyze_file(&file_path, new_content);

    let undef_count2 = result2
        .issues
        .iter()
        .filter(|i| i.kind.name() == "UndefinedFunction")
        .count();
    assert_eq!(undef_count2, 0, "after fix, no UndefinedFunction expected");
}
