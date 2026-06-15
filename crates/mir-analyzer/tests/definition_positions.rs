// Integration tests for definition position lookups (mir#78).
//
// After analysis, the codebase should store definition locations for all
// top-level symbols and class members, accessible via the typed Name API.

mod common;

use mir_analyzer::symbol::ReferenceKind;
use mir_analyzer::{AnalysisSession, BatchOptions, Name, PhpVersion, SymbolLookupError};

use self::common::{create_temp_dir, path_to_str, write_file};

#[test]
fn definition_of_finds_class() {
    let dir = create_temp_dir("test");
    let file = write_file(&dir, "Foo.php", "<?php\nclass Foo {}\n");
    let file_str = path_to_str(&file).to_string();

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(&[file], &BatchOptions::new());

    let loc = analyzer
        .definition_of(&Name::class("Foo"))
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

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(&[file], &BatchOptions::new());

    assert!(
        analyzer.definition_of(&Name::function("my_func")).is_ok(),
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

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(&[file], &BatchOptions::new());

    assert!(
        analyzer.definition_of(&Name::class("Renderable")).is_ok(),
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

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(&[file], &BatchOptions::new());

    assert!(
        analyzer.definition_of(&Name::method("Bar", "baz")).is_ok(),
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

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(&[file], &BatchOptions::new());

    assert!(
        analyzer
            .definition_of(&Name::property("Qux", "name"))
            .is_ok(),
        "should find location for property Qux::$name"
    );
}

#[test]
fn definition_of_returns_not_found_for_unknown() {
    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    assert_eq!(
        analyzer
            .definition_of(&Name::class("NonExistent"))
            .unwrap_err(),
        SymbolLookupError::NotFound
    );
    assert_eq!(
        analyzer
            .definition_of(&Name::method("NonExistent", "foo"))
            .unwrap_err(),
        SymbolLookupError::NotFound
    );
}

// ---------------------------------------------------------------------------
// laravel_definition_on_new_expression — full flow: symbol_at → definition_of
// ---------------------------------------------------------------------------

#[test]
fn laravel_definition_on_new_expression() {
    // Simulate: AuthManager.php uses `new RequestGuard(...)` where RequestGuard
    // is imported via `use Illuminate\Auth\RequestGuard`.
    //
    // GoToDef on RequestGuard must navigate to RequestGuard.php, not stay in
    // AuthManager.php.
    let dir = create_temp_dir("laravel_def_new");

    let guard_src = "<?php\nnamespace Illuminate\\Auth;\nclass RequestGuard {}\n";
    let guard_file = write_file(&dir, "RequestGuard.php", guard_src);
    let guard_file_str = path_to_str(&guard_file).to_string();

    let auth_src = "<?php\nuse Illuminate\\Auth\\RequestGuard;\nfunction make(): void { $g = new RequestGuard(); }\n";
    let auth_file = write_file(&dir, "AuthManager.php", auth_src);
    let auth_file_str = path_to_str(&auth_file).to_string();

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result = analyzer.analyze_paths(&[guard_file, auth_file], &BatchOptions::new());

    let offset = auth_src.find("new RequestGuard").unwrap() as u32 + "new ".len() as u32;
    let sym = result
        .symbol_at(&auth_file_str, offset)
        .expect("symbol_at must find ClassReference on RequestGuard");

    assert!(
        matches!(&sym.kind, ReferenceKind::ClassReference(n) if n.as_ref() == "Illuminate\\Auth\\RequestGuard"),
        "ClassReference must carry the FQN, got {:?}",
        sym.kind
    );

    let name = sym
        .to_symbol()
        .expect("ClassReference must convert to Name");
    let loc = analyzer
        .definition_of(&name)
        .expect("definition_of must find RequestGuard");

    assert_eq!(
        loc.file.as_ref(),
        guard_file_str.as_str(),
        "definition_of must navigate to RequestGuard.php, not AuthManager.php"
    );
}

// ---------------------------------------------------------------------------
// laravel_completion_static_members — class_imports API for Gap 3
// ---------------------------------------------------------------------------

#[test]
fn class_imports_returns_alias_to_fqn_map() {
    // Verify that class_imports() exposes the file's use-import aliases so
    // that a completion handler can expand a short name (e.g. "Str") to its
    // FQN ("Illuminate\Support\Str") before looking up static members.
    let dir = create_temp_dir("laravel_completion_imports");

    let str_src = "<?php\nnamespace Illuminate\\Support;\nclass Str { public static function camel(string $value): string { return $value; } }\n";
    let str_file = write_file(&dir, "Str.php", str_src);

    let gate_src = "<?php\nuse Illuminate\\Support\\Str;\nfunction test(string $ability): string { return Str::camel($ability); }\n";
    let gate_file = write_file(&dir, "Gate.php", gate_src);
    let gate_file_str = path_to_str(&gate_file).to_string();

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(&[str_file, gate_file], &BatchOptions::new());

    let imports = analyzer.class_imports(&gate_file_str);
    assert!(
        imports
            .iter()
            .any(|(alias, fqcn)| alias.as_ref() == "Str"
                && fqcn.as_ref() == "Illuminate\\Support\\Str"),
        "class_imports must return Str → Illuminate\\Support\\Str for Gate.php, got {:?}",
        imports
    );
}

#[test]
fn class_imports_handles_renamed_alias() {
    // A `use Foo\Bar as Baz` import must appear as alias="Baz", fqcn="Foo\Bar".
    let dir = create_temp_dir("renamed_alias_imports");

    let bar_src = "<?php\nnamespace Foo;\nclass Bar {}\n";
    let bar_file = write_file(&dir, "Bar.php", bar_src);

    let caller_src = "<?php\nuse Foo\\Bar as Baz;\nfunction make(): void { $b = new Baz(); }\n";
    let caller_file = write_file(&dir, "Caller.php", caller_src);
    let caller_file_str = path_to_str(&caller_file).to_string();

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    analyzer.analyze_paths(&[bar_file, caller_file], &BatchOptions::new());

    let imports = analyzer.class_imports(&caller_file_str);
    assert!(
        imports
            .iter()
            .any(|(alias, fqcn)| alias.as_ref() == "Baz" && fqcn.as_ref() == "Foo\\Bar"),
        "class_imports must return Baz → Foo\\Bar for renamed alias, got {:?}",
        imports
    );
}
