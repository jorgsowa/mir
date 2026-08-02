//! Regression test: spreading a string-keyed array as a call's sole argument
//! (`f(...$args)`) is valid PHP 8.1+ named-argument syntax and must bind each
//! entry to the parameter of the same name, not merge every value into one
//! union type checked against the first parameter alone.
//!
//! Previously `expand_sole_spread_arg` (`call/args.rs`) only recognized a
//! sequentially int-keyed (0..n-1) shape; a string-keyed shape fell back to
//! `spread_element_type`, which merged `'Bob'|2` into one union and checked
//! it against `$name: string` — a false `InvalidArgument`.

mod common;

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, FileAnalyzer, PhpVersion};

fn issues_for(source: &str) -> Vec<String> {
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let file: Arc<str> = Arc::from("<test>");
    session.ingest_file(file.clone(), Arc::from(source));

    let parsed = php_rs_parser::parse(source);
    assert!(
        parsed.errors.is_empty(),
        "parser errors in test source: {:?}",
        parsed.errors
    );

    FileAnalyzer::new(&session)
        .analyze(file, source, &parsed.program, &parsed.source_map)
        .issues
        .iter()
        .map(|i| i.kind.name().to_string())
        .collect()
}

#[test]
fn string_keyed_spread_as_named_args_binds_by_name_not_flagged() {
    let src = r#"<?php
function greet(string $name, int $times = 1): string {
    return str_repeat($name, $times);
}
function test(): string {
    $args = ['name' => 'Bob', 'times' => 2];
    return greet(...$args);
}
"#;
    let issues = issues_for(src);
    assert!(
        issues.is_empty(),
        "string-keyed spread should bind by parameter name with no diagnostics; got {issues:?}"
    );
}

#[test]
fn string_keyed_spread_with_unknown_param_name_is_flagged() {
    let src = r#"<?php
function greet(string $name, int $times = 1): string {
    return str_repeat($name, $times);
}
function test(): string {
    $args = ['name' => 'Bob', 'nope' => 2];
    return greet(...$args);
}
"#;
    let issues = issues_for(src);
    assert!(
        issues.iter().any(|k| k == "InvalidNamedArgument"),
        "an unknown key in a string-keyed spread must still be flagged; got {issues:?}"
    );
}

#[test]
fn string_keyed_spread_with_wrong_type_for_named_param_is_flagged() {
    let src = r#"<?php
function greet(string $name, int $times = 1): string {
    return str_repeat($name, $times);
}
function test(): string {
    $args = ['name' => 42, 'times' => 2];
    return greet(...$args);
}
"#;
    let issues = issues_for(src);
    // int → string is coercible under weak typing, so this surfaces as
    // ArgumentTypeCoercion rather than a hard InvalidArgument — the key
    // point is that 42 is checked against $name specifically (by name),
    // not merged into a union checked against whichever parameter comes
    // first positionally.
    assert!(
        issues.iter().any(|k| k == "ArgumentTypeCoercion"),
        "binding 'name' => 42 against string $name must still be flagged; got {issues:?}"
    );
}

/// Sequential int-keyed spreads (the pre-existing behavior) must keep working
/// unchanged alongside the new string-keyed path.
#[test]
fn int_keyed_spread_still_binds_positionally() {
    let src = r#"<?php
function needsTwoInts(int $a, int $b): void {}
function test(): void {
    $pair = [1, 'two'];
    needsTwoInts(...$pair);
}
"#;
    let issues = issues_for(src);
    assert!(
        issues.iter().any(|k| k == "InvalidArgument"),
        "positional spread must still check each element against its own parameter; got {issues:?}"
    );
}
