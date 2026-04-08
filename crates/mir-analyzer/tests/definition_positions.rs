// Integration tests for definition position lookups (mir#78).
//
// After analysis, the codebase should store definition locations for all
// top-level symbols and class members, and the get_symbol_location /
// get_member_location APIs should return them.

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
fn get_symbol_location_finds_class() {
    let dir = TempDir::new().unwrap();
    let file = write(&dir, "Foo.php", "<?php\nclass Foo {}\n");
    let file_str = file.to_str().unwrap().to_string();

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    let loc = analyzer.codebase.get_symbol_location("Foo");
    assert!(loc.is_some(), "should find location for class Foo");
    let loc = loc.unwrap();
    assert_eq!(loc.file.as_ref(), file_str.as_str());
}

#[test]
fn get_symbol_location_finds_function() {
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "funcs.php",
        "<?php\nfunction my_func(): int { return 1; }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    let loc = analyzer.codebase.get_symbol_location("my_func");
    assert!(loc.is_some(), "should find location for function my_func");
}

#[test]
fn get_symbol_location_finds_interface() {
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "Iface.php",
        "<?php\ninterface Renderable { public function render(): string; }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    let loc = analyzer.codebase.get_symbol_location("Renderable");
    assert!(loc.is_some(), "should find location for interface");
}

#[test]
fn get_member_location_finds_method() {
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "Bar.php",
        "<?php\nclass Bar {\n    public function baz(): void {}\n}\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    let loc = analyzer.codebase.get_member_location("Bar", "baz");
    assert!(loc.is_some(), "should find location for method Bar::baz");
}

#[test]
fn get_member_location_finds_property() {
    let dir = TempDir::new().unwrap();
    let file = write(
        &dir,
        "Qux.php",
        "<?php\nclass Qux {\n    public string $name = '';\n}\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    let loc = analyzer.codebase.get_member_location("Qux", "name");
    assert!(
        loc.is_some(),
        "should find location for property Qux::$name"
    );
}

#[test]
fn get_symbol_location_returns_none_for_unknown() {
    let analyzer = ProjectAnalyzer::new();
    assert!(analyzer
        .codebase
        .get_symbol_location("NonExistent")
        .is_none());
    assert!(analyzer
        .codebase
        .get_member_location("NonExistent", "foo")
        .is_none());
}
