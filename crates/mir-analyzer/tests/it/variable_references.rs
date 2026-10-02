//! `AnalysisSession::variable_references`: occurrences of a local variable or parameter.

use std::sync::Arc;

use mir_analyzer::{AnalysisSession, PhpVersion, Severity};

const FILE: &str = "vars.php";
const CURSOR: char = '‸';

/// `(line, column)` of every occurrence for the variable under the `‸` marker.
/// Also fails if the fixture's `@mir-check` annotations do not hold.
fn refs(marked: &str) -> Vec<(u32, u32)> {
    let offset = marked.find(CURSOR).expect("cursor marker present");
    let src = marked.replacen(CURSOR, "", 1);
    let mut session = AnalysisSession::new(PhpVersion::LATEST);
    session.ensure_all_stubs();
    session.set_file_text(Arc::from(FILE), Arc::from(src.as_str()));

    let issues: Vec<_> = session
        .analyze_file_diagnostics(FILE, &src)
        .issues
        .into_iter()
        .filter(|i| i.severity != Severity::Info)
        .collect();
    assert!(issues.is_empty(), "{issues:?}");

    let lines: Vec<&str> = src.lines().collect();
    session
        .variable_references(FILE, offset as u32)
        .into_iter()
        .map(|range| {
            let line = lines[range.start.line as usize - 1];
            let text: String = line
                .chars()
                .skip(range.start.column as usize)
                .take((range.end.column - range.start.column) as usize)
                .collect();
            assert!(text.starts_with('$'), "range covers {text:?}");
            (range.start.line, range.start.column)
        })
        .collect()
}

#[test]
fn parameter_and_locals_in_a_function() {
    let found = refs(
        "<?php
function f(int $a, int $b): int {
    $t = $a + $b;
    /** @mir-check $t is int */
    $_ = $t;
    return $t + $‸a;
}
",
    );
    assert_eq!(found, [(2, 15), (3, 9), (6, 16)]);
}

#[test]
fn cursor_on_the_parameter_declaration() {
    let found = refs(
        "<?php
function f(int $‸a): int {
    return $a * 2;
}
",
    );
    assert_eq!(found, [(2, 15), (3, 11)]);
}

#[test]
fn same_name_in_other_methods_is_a_different_variable() {
    let found = refs(
        "<?php
class C {
    public function one(int $x): int { return $x; }
    public function two(int $x): int { return $‸x + 1; }
}
",
    );
    assert_eq!(found, [(4, 28), (4, 46)]);
}

#[test]
fn closure_without_capture_has_its_own_scope() {
    let found = refs(
        "<?php
function f(int $v): void {
    $f = function (int $v): int { return $v; };
    $_ = $‸v;
    $_ = $f(1);
}
",
    );
    assert_eq!(found, [(2, 15), (4, 9)]);
}

#[test]
fn captured_variable_spans_the_closure() {
    let found = refs(
        "<?php
function f(): void {
    $n = 1;
    $g = function () use ($n): int { return $n + 1; };
    $_ = $g() + $‸n;
}
",
    );
    assert_eq!(found, [(3, 4), (4, 26), (4, 44), (5, 16)]);
}

#[test]
fn cursor_inside_closure_resolves_the_captured_outer_variable() {
    let found = refs(
        "<?php
function f(): void {
    $n = 1;
    $g = function () use ($n): int { return $‸n + 1; };
    $_ = $g();
}
",
    );
    assert_eq!(found, [(3, 4), (4, 26), (4, 44)]);
}

#[test]
fn arrow_function_shares_parent_scope_but_not_its_params() {
    let shared = refs(
        "<?php
function f(int $k): array {
    return array_map(fn(int $i): int => $i + $‸k, [1, 2]);
}
",
    );
    assert_eq!(shared, [(2, 15), (3, 45)]);

    let shadowed = refs(
        "<?php
function f(int $k): array {
    return array_map(fn(int $k): int => $‸k + 1, [1, 2]) + [$k];
}
",
    );
    assert_eq!(shadowed, [(3, 28), (3, 40)]);
}

#[test]
fn top_level_script_scope_excludes_functions() {
    let found = refs(
        "<?php
$total = 1;
function f(): int { $total = 2; return $total; }
$_ = $‸total;
",
    );
    assert_eq!(found, [(2, 0), (4, 5)]);
}

#[test]
fn foreach_catch_and_reassignment() {
    let found = refs(
        "<?php
function f(array $items): void {
    foreach ($items as $‸item) {
        $_ = $item;
    }
    try {
        throw new Exception('x');
    } catch (Exception $item) {
        $_ = $item->getMessage();
    }
}
",
    );
    assert_eq!(found, [(3, 23), (4, 13), (8, 23), (9, 13)]);
}

#[test]
fn not_on_a_variable_is_empty() {
    let found = refs(
        "<?php
function f(int $a): int {
    return str‸len('x') + $a;
}
",
    );
    assert!(found.is_empty());
}

#[test]
fn static_declaration_and_uses() {
    let found = refs(
        "<?php
function bump(): int {
    static $calls = 0;
    $calls++;
    return $‸calls;
}
",
    );
    assert_eq!(found, [(3, 11), (4, 4), (5, 11)]);
}

#[test]
fn global_declaration_excludes_the_top_level_variable() {
    let found = refs(
        "<?php
$counter = 0;
function bump(): int {
    global $counter;
    $counter += 1;
    return $‸counter;
}
bump();
$_ = $counter;
",
    );
    assert_eq!(found, [(4, 11), (5, 4), (6, 11)]);
}

#[test]
fn cursor_on_a_use_clause_variable() {
    let found = refs(
        "<?php
function f(): void {
    $n = 1;
    $g = function () use ($‸n): int { return $n + 1; };
    $_ = $g();
}
",
    );
    assert_eq!(found, [(3, 4), (4, 26), (4, 44)]);
}
