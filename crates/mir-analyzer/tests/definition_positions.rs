// Integration tests for definition position lookups (mir#78).
//
// After analysis, the codebase should store definition locations for all
// top-level symbols and class members, accessible via the typed Symbol API.

mod common;

use mir_analyzer::{ProjectAnalyzer, Symbol, SymbolLookupError};

use self::common::{create_temp_dir, path_to_str, write_file};

#[test]
fn definition_of_finds_class() {
    let dir = create_temp_dir("test");
    let file = write_file(&dir, "Foo.php", "<?php\nclass Foo {}\n");
    let file_str = path_to_str(&file).to_string();

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    let loc = analyzer
        .definition_of(&Symbol::class("Foo"))
        .expect("should find location for class Foo");
    assert_eq!(loc.file.as_ref(), file_str.as_str());
}

#[test]
fn definition_of_finds_function() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "funcs.php",
        "<?php\nfunction my_func(): int { return 1; }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    assert!(
        analyzer.definition_of(&Symbol::function("my_func")).is_ok(),
        "should find location for function my_func"
    );
}

#[test]
fn definition_of_finds_interface() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "Iface.php",
        "<?php\ninterface Renderable { public function render(): string; }\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    assert!(
        analyzer.definition_of(&Symbol::class("Renderable")).is_ok(),
        "should find location for interface"
    );
}

#[test]
fn definition_of_finds_method() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "Bar.php",
        "<?php\nclass Bar {\n    public function baz(): void {}\n}\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    assert!(
        analyzer
            .definition_of(&Symbol::method("Bar", "baz"))
            .is_ok(),
        "should find location for method Bar::baz"
    );
}

#[test]
fn definition_of_finds_property() {
    let dir = create_temp_dir("test");
    let file = write_file(
        &dir,
        "Qux.php",
        "<?php\nclass Qux {\n    public string $name = '';\n}\n",
    );

    let analyzer = ProjectAnalyzer::new();
    analyzer.analyze(&[file]);

    assert!(
        analyzer
            .definition_of(&Symbol::property("Qux", "name"))
            .is_ok(),
        "should find location for property Qux::$name"
    );
}

#[test]
fn definition_of_returns_not_found_for_unknown() {
    let analyzer = ProjectAnalyzer::new();
    assert_eq!(
        analyzer
            .definition_of(&Symbol::class("NonExistent"))
            .unwrap_err(),
        SymbolLookupError::NotFound
    );
    assert_eq!(
        analyzer
            .definition_of(&Symbol::method("NonExistent", "foo"))
            .unwrap_err(),
        SymbolLookupError::NotFound
    );
}
