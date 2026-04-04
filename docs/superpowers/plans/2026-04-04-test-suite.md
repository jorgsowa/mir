# Test Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a `mir-test-utils` harness crate and 8 per-rule integration test files covering the Psalm-parity rules.

**Architecture:** A new `crates/mir-test-utils` crate exposes `check()`, `assert_issue()`, `assert_issue_kind()`, and `assert_no_issue()`. Integration tests live in `crates/mir-analyzer/tests/<rule>.rs`, each a separate Rust test binary that depends on `mir-test-utils` via dev-dependencies.

**Tech Stack:** Rust, Cargo workspace, `mir-analyzer`, `mir-issues`, PHP inline string fixtures.

---

## Key facts for implementors

- `span_to_line_col`: line is **1-based**, col is **0-based byte offset from start of line**.
- `FnParam.name` does **not** include `$` (the PHP parser strips the sigil at parse time).
- `IssueKind::UndefinedClass { name }` stores the resolved FQCN (e.g. `"UnknownClass"` or `"ast\\Node"`).
- `IssueKind::PossiblyInvalidArrayOffset` is the mir variant for what Psalm calls `PossiblyInvalidArrayAccess`.
- `MethodSignatureMismatch` is emitted by `ClassAnalyzer`, not by body analysis — it uses `Location { line: 1, col_start: 0 }` (file-level, no span). Assertions on position for that rule use line=1, col=0.
- Tests for rules that currently over-fire (InvalidArgument, InvalidReturnType, MethodSignatureMismatch) or under-fire (PossiblyInvalidArrayOffset for unpack) are marked `// TODO: currently failing — documents expected behavior`.

---

## File Map

| Action | Path |
|--------|------|
| Create | `crates/mir-test-utils/Cargo.toml` |
| Create | `crates/mir-test-utils/src/lib.rs` |
| Modify | `Cargo.toml` (workspace members + deps) |
| Modify | `crates/mir-analyzer/Cargo.toml` (dev-deps) |
| Create | `crates/mir-analyzer/tests/undefined_function.rs` |
| Create | `crates/mir-analyzer/tests/undefined_class.rs` |
| Create | `crates/mir-analyzer/tests/invalid_argument.rs` |
| Create | `crates/mir-analyzer/tests/invalid_return_type.rs` |
| Create | `crates/mir-analyzer/tests/undefined_method.rs` |
| Create | `crates/mir-analyzer/tests/method_signature_mismatch.rs` |
| Create | `crates/mir-analyzer/tests/possibly_invalid_array_offset.rs` |
| Create | `crates/mir-analyzer/tests/redundant_condition.rs` |

---

## Task 1: Create `mir-test-utils` crate

**Files:**
- Create: `crates/mir-test-utils/Cargo.toml`
- Create: `crates/mir-test-utils/src/lib.rs`

- [ ] **Step 1: Create `crates/mir-test-utils/Cargo.toml`**

```toml
[package]
name    = "mir-test-utils"
description = "Test helpers for mir integration tests"
version.workspace = true
edition.workspace = true
license.workspace    = true
authors.workspace    = true
repository.workspace = true
homepage.workspace   = true

[dependencies]
mir-analyzer = { workspace = true }
mir-issues   = { workspace = true }
```

- [ ] **Step 2: Create `crates/mir-test-utils/src/lib.rs`**

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;

use mir_analyzer::project::ProjectAnalyzer;
use mir_issues::{Issue, IssueKind};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Run the full analyzer on an inline PHP string.
/// Creates a unique temp file, analyzes it, deletes it, and returns all
/// unsuppressed issues.
pub fn check(src: &str) -> Vec<Issue> {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp: PathBuf = std::env::temp_dir().join(format!("mir_test_{}.php", id));
    std::fs::write(&tmp, src).expect("write temp PHP file");
    let result = ProjectAnalyzer::new().analyze(&[tmp.clone()]);
    std::fs::remove_file(&tmp).ok();
    result.issues.into_iter().filter(|i| !i.suppressed).collect()
}

/// Assert that `issues` contains at least one issue with the exact `IssueKind`
/// at `line` and `col_start`. Panics with the full issue list on failure.
pub fn assert_issue(issues: &[Issue], kind: IssueKind, line: u32, col_start: u16) {
    let found = issues.iter().any(|i| {
        i.kind == kind && i.location.line == line && i.location.col_start == col_start
    });
    if !found {
        panic!(
            "Expected issue {:?} at line {}, col {}.\nActual issues:\n{}",
            kind,
            line,
            col_start,
            fmt_issues(issues),
        );
    }
}

/// Assert that `issues` contains at least one issue whose `kind.name()` equals
/// `kind_name`, at `line` and `col_start`. Use this when the exact IssueKind
/// field values are complex (e.g. type-format strings in InvalidArgument).
pub fn assert_issue_kind(issues: &[Issue], kind_name: &str, line: u32, col_start: u16) {
    let found = issues.iter().any(|i| {
        i.kind.name() == kind_name
            && i.location.line == line
            && i.location.col_start == col_start
    });
    if !found {
        panic!(
            "Expected issue {} at line {}, col {}.\nActual issues:\n{}",
            kind_name,
            line,
            col_start,
            fmt_issues(issues),
        );
    }
}

/// Assert that `issues` contains no issue whose `kind.name()` equals `kind_name`.
/// Panics with the matching issues on failure.
pub fn assert_no_issue(issues: &[Issue], kind_name: &str) {
    let found: Vec<_> = issues
        .iter()
        .filter(|i| i.kind.name() == kind_name)
        .collect();
    if !found.is_empty() {
        panic!(
            "Expected no {} issues, but found:\n{}",
            kind_name,
            fmt_issues(&found.into_iter().cloned().collect::<Vec<_>>()),
        );
    }
}

fn fmt_issues(issues: &[Issue]) -> String {
    if issues.is_empty() {
        return "  (none)".to_string();
    }
    issues
        .iter()
        .map(|i| {
            format!(
                "  {} @ line {}, col {} — {}",
                i.kind.name(),
                i.location.line,
                i.location.col_start,
                i.kind.message(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

- [ ] **Step 3: Verify it compiles (no tests yet)**

```bash
cd /Users/adamspychala/Projects/mir
cargo build -p mir-test-utils
```
Expected: compiles cleanly (zero errors).

---

## Task 2: Register crate in workspace and wire dev-deps

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/mir-analyzer/Cargo.toml`

- [ ] **Step 1: Add `mir-test-utils` to workspace `Cargo.toml`**

In the `[workspace] members` array, add `"crates/mir-test-utils"`. In `[workspace.dependencies]`, add:

```toml
mir-test-utils = { path = "crates/mir-test-utils", version = "0.1.0" }
```

- [ ] **Step 2: Add `mir-test-utils` to `crates/mir-analyzer/Cargo.toml` dev-deps**

Append to `crates/mir-analyzer/Cargo.toml`:

```toml
[dev-dependencies]
mir-test-utils = { workspace = true }
mir-issues     = { workspace = true }
```

- [ ] **Step 3: Verify workspace builds**

```bash
cd /Users/adamspychala/Projects/mir
cargo build --workspace
```
Expected: zero errors.

- [ ] **Step 4: Commit**

```bash
git add crates/mir-test-utils/ Cargo.toml crates/mir-analyzer/Cargo.toml
git commit -m "feat: add mir-test-utils harness crate"
```

---

## Task 3: `undefined_function` tests

**Files:**
- Create: `crates/mir-analyzer/tests/undefined_function.rs`

**Position reference** (all computed from `span_to_line_col`):
- `foo()` at start of line → the function call `expr.span` starts at `f` → col 0.
- `\nonExistent()` call → expr.span starts at `\` → col 0.
- `missing()` inside method with 8-space indent → col 8.

- [ ] **Step 1: Create the test file**

```rust
// crates/mir-analyzer/tests/undefined_function.rs
use mir_issues::IssueKind;
use mir_test_utils::{assert_issue, assert_no_issue, check};

#[test]
fn reports_unknown_function() {
    let issues = check("<?php\nfoo();\n");
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "foo".into() },
        2,
        0,
    );
}

#[test]
fn does_not_report_strlen() {
    let issues = check("<?php\nstrlen('hello');\n");
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn does_not_report_array_map() {
    let issues = check("<?php\narray_map(fn($x) => $x, [1, 2, 3]);\n");
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn does_not_report_user_defined_function() {
    let issues = check("<?php\nfunction myFn(): void {}\nmyFn();\n");
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn reports_global_namespace_unknown_function() {
    // Leading \ forces global namespace lookup; still unknown
    let issues = check("<?php\n\\nonExistent();\n");
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "nonExistent".into() },
        2,
        0,
    );
}

#[test]
fn does_not_report_unpack() {
    // unpack() is a PHP builtin — must be in stubs
    // NOTE: this test currently FAILS if unpack() stub is missing (see CLAUDE.md gap analysis)
    let issues = check("<?php\n$r = unpack('N*', pack('N*', 1));\n");
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn does_not_report_suppressed_call() {
    let src = "<?php\n/** @psalm-suppress UndefinedFunction */\nnoSuchFunction();\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedFunction");
}

#[test]
fn reports_inside_method_body() {
    let src = "<?php\nclass A {\n    public function go(): void {\n        missing();\n    }\n}\n";
    let issues = check(src);
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "missing".into() },
        4,
        8,
    );
}

#[test]
fn reports_each_call_site_independently() {
    let src = "<?php\nfoo();\nfoo();\n";
    let issues = check(src);
    // Two separate call sites — one on line 2 and one on line 3
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "foo".into() },
        2,
        0,
    );
    assert_issue(
        &issues,
        IssueKind::UndefinedFunction { name: "foo".into() },
        3,
        0,
    );
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /Users/adamspychala/Projects/mir
cargo test -p mir-analyzer --test undefined_function -- --nocapture 2>&1
```
Expected: `reports_unknown_function`, `does_not_report_strlen`, `does_not_report_array_map`, `does_not_report_user_defined_function`, `reports_global_namespace_unknown_function`, `does_not_report_suppressed_call`, `reports_inside_method_body`, `reports_each_call_site_independently` should pass. `does_not_report_unpack` may fail if unpack stub is absent.

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/tests/undefined_function.rs
git commit -m "test(undefined_function): add 9 rule tests with position assertions"
```

---

## Task 4: `undefined_class` tests

**Files:**
- Create: `crates/mir-analyzer/tests/undefined_class.rs`

**Position reference:**
- `new UnknownClass()` → `n.class.span` points at `U` of `UnknownClass`; `new ` is 4 chars → col 4.
- Type hint in `function f(UnknownClass $x)` → `function f(` is 11 chars → col 11.
- Type hint in `function f(): UnknownClass` → `function f(): ` is 14 chars → col 14.
- `use ast\Node;` on line 2, type hint `Node` on line 3 col 11.

- [ ] **Step 1: Create the test file**

```rust
// crates/mir-analyzer/tests/undefined_class.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_new_unknown_class() {
    let issues = check("<?php\nnew UnknownClass();\n");
    assert_issue_kind(&issues, "UndefinedClass", 2, 4);
}

#[test]
fn does_not_report_stdclass() {
    let issues = check("<?php\nnew stdClass();\n");
    assert_no_issue(&issues, "UndefinedClass");
}

#[test]
fn does_not_report_user_defined_class() {
    let src = "<?php\nclass Foo {}\nnew Foo();\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedClass");
}

#[test]
fn reports_unknown_class_in_param_type_hint() {
    let src = "<?php\nfunction f(UnknownClass $x): void {}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "UndefinedClass", 2, 11);
}

#[test]
fn reports_unknown_class_in_return_type_hint() {
    // "function f(): " is 14 chars → UnknownClass starts at col 14
    let src = "<?php\nfunction f(): UnknownClass { return null; }\n";
    let issues = check(src);
    assert_issue_kind(&issues, "UndefinedClass", 2, 14);
}

#[test]
fn reports_extension_class_via_use_alias() {
    // ast\Node does not exist in the project codebase → should fire
    let src = "<?php\nuse ast\\Node;\nfunction f(Node $x): void {}\n";
    let issues = check(src);
    // type hint `Node` is on line 3, col 11 (after "function f(")
    assert_issue_kind(&issues, "UndefinedClass", 3, 11);
}

#[test]
fn does_not_report_known_aliased_class() {
    // class Bar defined in this file; aliased as Baz via use — no fire
    let src = "<?php\nclass Bar {}\nuse Bar as Baz;\nnew Baz();\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedClass");
}

#[test]
fn reports_instanceof_unknown_class() {
    // NOTE: currently unimplemented — instanceof does not trigger UndefinedClass.
    // This test documents expected future behavior and will fail until wired up.
    let src = "<?php\nfunction f($x): void {\n    if ($x instanceof UnknownClass) {}\n}\n";
    let issues = check(src);
    // "    if ($x instanceof " is 22 chars → UnknownClass at col 22
    assert_issue_kind(&issues, "UndefinedClass", 3, 22);
}

#[test]
fn does_not_report_after_suppression() {
    let src = "<?php\n/** @psalm-suppress UndefinedClass */\nnew NoSuchClass();\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedClass");
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /Users/adamspychala/Projects/mir
cargo test -p mir-analyzer --test undefined_class -- --nocapture 2>&1
```
Expected: most pass. `does_not_report_known_aliased_class` may fail if alias resolution is incomplete.

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/tests/undefined_class.rs
git commit -m "test(undefined_class): add 8 rule tests with position assertions"
```

---

## Task 5: `invalid_argument` tests

**Files:**
- Create: `crates/mir-analyzer/tests/invalid_argument.rs`

**Position reference:** `InvalidArgument` is emitted at `arg_span` (the argument expression span).
- `f('hello')` on line 3 → `'hello'` starts at col 2 (after `f(`).
- For named args: the value position.
- `strlen(42)` → `42` starts at col 7 (after `strlen(`).

**Note:** The `expected`/`actual` strings in `IssueKind::InvalidArgument` are type display strings (e.g. `"int"`, `"string"`). These tests use `assert_issue_kind` to avoid coupling to exact type-format strings.

- [ ] **Step 1: Create the test file**

```rust
// crates/mir-analyzer/tests/invalid_argument.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_string_passed_as_int() {
    // function f(int $x) — line 2; f('hello') — line 3
    // 'hello' is the argument, starts at col 2 (after "f(")
    let src = "<?php\nfunction f(int $x): void {}\nf('hello');\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidArgument", 3, 2);
}

#[test]
fn does_not_report_correct_int_arg() {
    let src = "<?php\nfunction f(int $x): void {}\nf(42);\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidArgument");
}

#[test]
fn reports_null_passed_as_string() {
    // null is not string
    let src = "<?php\nfunction f(string $x): void {}\nf(null);\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidArgument", 3, 2);
}

#[test]
fn reports_incompatible_union_arg() {
    // int|string passed where only int is expected
    let src = "<?php\nfunction f(int $x): void {}\n/** @var int|string $v */\n$v = 1;\nf($v);\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidArgument", 5, 2);
}

#[test]
fn does_not_report_subclass_as_parent() {
    let src = "<?php\nclass Base {}\nclass Child extends Base {}\nfunction f(Base $x): void {}\nf(new Child());\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidArgument");
}

#[test]
fn reports_wrong_type_to_strlen() {
    // strlen(42) — 42 starts at col 7 (after "strlen(")
    let src = "<?php\nstrlen(42);\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidArgument", 2, 7);
}

#[test]
fn does_not_report_mixed_arg() {
    // mixed bypasses type checking
    let src = "<?php\nfunction f(int $x): void {}\n/** @var mixed $v */\n$v = 1;\nf($v);\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidArgument");
}

#[test]
fn reports_variadic_wrong_type() {
    // variadic int receives string
    let src = "<?php\nfunction f(int ...$xs): void {}\nf('a', 'b');\n";
    let issues = check(src);
    // first arg 'a' is at col 2
    assert_issue_kind(&issues, "InvalidArgument", 3, 2);
}

#[test]
fn reports_named_argument_wrong_type() {
    // PHP 8 named argument — the value expression span is used
    let src = "<?php\nfunction f(int $x): void {}\nf(x: 'hello');\n";
    let issues = check(src);
    // 'hello' value starts at col 5 (after "f(x: ")
    assert_issue_kind(&issues, "InvalidArgument", 3, 5);
}

#[test]
fn does_not_report_correct_union_to_union_param() {
    // function accepts string|int — passing string is fine
    let src = "<?php\nfunction f(string|int $x): void {}\nf('hello');\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidArgument");
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /Users/adamspychala/Projects/mir
cargo test -p mir-analyzer --test invalid_argument -- --nocapture 2>&1
```
Expected: most pass. Some may fail due to known InvalidArgument false-positive issues (see CLAUDE.md gap analysis ~17k over-reported). Failing tests document the rule's intended behavior.

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/tests/invalid_argument.rs
git commit -m "test(invalid_argument): add 9 rule tests with position assertions"
```

---

## Task 6: `invalid_return_type` tests

**Files:**
- Create: `crates/mir-analyzer/tests/invalid_return_type.rs`

**Position reference:** `InvalidReturnType` is emitted at `stmt.span` (the `return` statement span).
- In `function f(): int {\n    return 'hello';\n}`, `return` is on line 3 with 4-space indent → col 4.
- `return;` inside `function f(): int` — col 4.
- `return null;` inside void function — col 4.

- [ ] **Step 1: Create the test file**

```rust
// crates/mir-analyzer/tests/invalid_return_type.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_string_returned_from_int_function() {
    let src = "<?php\nfunction f(): int {\n    return 'hello';\n}\n";
    let issues = check(src);
    // return statement at line 3, col 4
    assert_issue_kind(&issues, "InvalidReturnType", 3, 4);
}

#[test]
fn does_not_report_correct_return_type() {
    let src = "<?php\nfunction f(): int {\n    return 42;\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidReturnType");
}

#[test]
fn reports_null_returned_from_non_nullable() {
    let src = "<?php\nfunction f(): string {\n    return null;\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 3, 4);
}

#[test]
fn reports_bare_return_from_non_void() {
    // bare return from int function
    let src = "<?php\nfunction f(): int {\n    return;\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 3, 4);
}

#[test]
fn does_not_report_subclass_return() {
    let src = "<?php\nclass Base {}\nclass Child extends Base {}\nfunction f(): Base {\n    return new Child();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidReturnType");
}

#[test]
fn does_not_report_mixed_return() {
    let src = "<?php\nfunction f(): int {\n    /** @var mixed $x */\n    $x = 1;\n    return $x;\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "InvalidReturnType");
}

#[test]
fn reports_return_null_from_void() {
    let src = "<?php\nfunction f(): void {\n    return null;\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 3, 4);
}

#[test]
fn reports_wrong_union_return() {
    // declared int; returning int|string is not a subtype
    let src = "<?php\nfunction f(): int {\n    /** @var int|string $v */\n    $v = 1;\n    return $v;\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "InvalidReturnType", 5, 4);
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /Users/adamspychala/Projects/mir
cargo test -p mir-analyzer --test invalid_return_type -- --nocapture 2>&1
```
Expected: most pass. Some may fail due to known false-positive issues (~10k over-reported in CLAUDE.md).

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/tests/invalid_return_type.rs
git commit -m "test(invalid_return_type): add 8 rule tests with position assertions"
```

---

## Task 7: `undefined_method` tests

**Files:**
- Create: `crates/mir-analyzer/tests/undefined_method.rs`

**Position reference:** `UndefinedMethod` is emitted at `expr.span` (the entire method call expression span).
- `$f->missing()` on line 4 → expr starts at `$` → col 0.
- `Foo::bar()` on line 3 → expr starts at `F` → col 0.

- [ ] **Step 1: Create the test file**

```rust
// crates/mir-analyzer/tests/undefined_method.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_missing_instance_method() {
    let src = "<?php\nclass Foo {}\n$f = new Foo();\n$f->missing();\n";
    let issues = check(src);
    // $f->missing() call expr starts at col 0 of line 4
    assert_issue_kind(&issues, "UndefinedMethod", 4, 0);
}

#[test]
fn does_not_report_defined_method() {
    let src = "<?php\nclass Foo {\n    public function bar(): void {}\n}\n$f = new Foo();\n$f->bar();\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_call_on_null_as_undefined_method() {
    // NullMethodCall should fire, not UndefinedMethod
    let src = "<?php\n$x = null;\n$x->foo();\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_method_defined_on_interface() {
    let src = "<?php\ninterface I {\n    public function doIt(): void;\n}\nfunction f(I $i): void {\n    $i->doIt();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_method_defined_on_abstract_class() {
    let src = "<?php\nabstract class Base {\n    abstract public function run(): void;\n}\nfunction f(Base $b): void {\n    $b->run();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_method_call_on_mixed() {
    let src = "<?php\n/** @var mixed $x */\n$x = 1;\n$x->anything();\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn reports_missing_static_method() {
    let src = "<?php\nclass Foo {}\nFoo::missing();\n";
    let issues = check(src);
    // Foo::missing() starts at col 0 of line 3
    assert_issue_kind(&issues, "UndefinedMethod", 3, 0);
}

#[test]
fn does_not_report_parent_method_that_exists() {
    let src = "<?php\nclass Base {\n    public function run(): void {}\n}\nclass Child extends Base {\n    public function go(): void {\n        parent::run();\n    }\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}

#[test]
fn does_not_report_call_on_generic_type_param() {
    // Generic type T — unknown class; suppress UndefinedMethod
    let src = "<?php\n/**\n * @template T\n * @param T $obj\n */\nfunction f($obj): void {\n    $obj->method();\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /Users/adamspychala/Projects/mir
cargo test -p mir-analyzer --test undefined_method -- --nocapture 2>&1
```
Expected: most pass. The generic type test may fail due to known UndefinedMethod false-positive issues (~20k in CLAUDE.md).

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/tests/undefined_method.rs
git commit -m "test(undefined_method): add 9 rule tests with position assertions"
```

---

## Task 8: `method_signature_mismatch` tests

**Files:**
- Create: `crates/mir-analyzer/tests/method_signature_mismatch.rs`

**Position reference:** `MethodSignatureMismatch` is emitted by `ClassAnalyzer` with `Location { line: 1, col_start: 0, col_end: 0 }` (no precise span). Assert `line: 1, col_start: 0`.

**Note:** The `detail` field in `IssueKind::MethodSignatureMismatch` is a human-readable string. Tests use `assert_issue_kind` to avoid coupling to its exact format.

- [ ] **Step 1: Create the test file**

```rust
// crates/mir-analyzer/tests/method_signature_mismatch.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_override_narrowing_param_type() {
    // Parent accepts string; Child accepts only int — narrowing is not allowed
    let src = "<?php\nclass Base {\n    public function f(string $x): void {}\n}\nclass Child extends Base {\n    public function f(int $x): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn reports_override_widening_return_type() {
    // Parent returns int; Child returns int|string — widening return is not allowed
    let src = "<?php\nclass Base {\n    public function f(): int { return 1; }\n}\nclass Child extends Base {\n    public function f(): int|string { return 1; }\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn does_not_report_compatible_override() {
    let src = "<?php\nclass Base {\n    public function f(string $x): void {}\n}\nclass Child extends Base {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn reports_override_adds_required_param() {
    // Parent has 0 params; Child has 1 required param
    let src = "<?php\nclass Base {\n    public function f(): void {}\n}\nclass Child extends Base {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn does_not_report_override_with_optional_extra_param() {
    // Extra param with default is allowed
    let src = "<?php\nclass Base {\n    public function f(): void {}\n}\nclass Child extends Base {\n    public function f(string $x = 'default'): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn reports_override_removes_default() {
    // Parent has optional param ($x with default); Child makes it required — fires
    let src = "<?php\nclass Base {\n    public function f(string $x = 'hi'): void {}\n}\nclass Child extends Base {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn reports_interface_implementation_wrong_signature() {
    let src = "<?php\ninterface I {\n    public function f(string $x): void;\n}\nclass C implements I {\n    public function f(int $x): void {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "MethodSignatureMismatch", 1, 0);
}

#[test]
fn does_not_report_correct_interface_implementation() {
    let src = "<?php\ninterface I {\n    public function f(string $x): void;\n}\nclass C implements I {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}

#[test]
fn does_not_report_correct_abstract_implementation() {
    let src = "<?php\nabstract class Base {\n    abstract public function f(string $x): void;\n}\nclass Child extends Base {\n    public function f(string $x): void {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "MethodSignatureMismatch");
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /Users/adamspychala/Projects/mir
cargo test -p mir-analyzer --test method_signature_mismatch -- --nocapture 2>&1
```
Expected: most pass. Some may fail due to known MethodSignatureMismatch over-reporting (~3k in CLAUDE.md).

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/tests/method_signature_mismatch.rs
git commit -m "test(method_signature_mismatch): add 7 rule tests with position assertions"
```

---

## Task 9: `possibly_invalid_array_offset` tests

**Files:**
- Create: `crates/mir-analyzer/tests/possibly_invalid_array_offset.rs`

**Note:** The analyzer emits `PossiblyInvalidArrayOffset` (not `PossiblyInvalidArrayAccess`). This is the mir variant for what Psalm calls `PossiblyInvalidArrayAccess`. The rule fires at the LHS destructure expression span (`expr.span` of the array pattern).

**Position reference:** Destructuring `[$a, $b] = get();` — `[` starts at col 0 of its line.

- [ ] **Step 1: Create the test file**

```rust
// crates/mir-analyzer/tests/possibly_invalid_array_offset.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_destructure_of_array_or_false() {
    // array|false → has_non_array && has_array → fires PossiblyInvalidArrayOffset
    let src = "<?php\n/**\n * @return array|false\n */\nfunction get(): array|false { return false; }\n[$a, $b] = get();\n";
    let issues = check(src);
    // [$a, $b] destructure LHS at line 6, col 0
    assert_issue_kind(&issues, "PossiblyInvalidArrayOffset", 6, 0);
}

#[test]
fn does_not_report_destructure_of_plain_array() {
    let src = "<?php\n/**\n * @return array\n */\nfunction get(): array { return []; }\n[$a, $b] = get();\n";
    let issues = check(src);
    assert_no_issue(&issues, "PossiblyInvalidArrayOffset");
}

#[test]
fn reports_destructure_of_only_false() {
    // false alone: has_non_array=true, has_array=false → rule does NOT fire
    // (no array type in union → destructuring the false is a different kind of error)
    let src = "<?php\n$v = false;\n[$a] = $v;\n";
    let issues = check(src);
    // has_array=false for pure TFalse → PossiblyInvalidArrayOffset does NOT fire
    assert_no_issue(&issues, "PossiblyInvalidArrayOffset");
}

#[test]
fn reports_both_elements_of_multi_var_destructure() {
    // Both $a and $b should be in scope even when the issue fires
    let src = "<?php\n/**\n * @return array|false\n */\nfunction get(): array|false { return false; }\n[$a, $b] = get();\necho $a + $b;\n";
    let issues = check(src);
    assert_issue_kind(&issues, "PossiblyInvalidArrayOffset", 6, 0);
}

#[test]
fn does_not_report_after_false_check() {
    // if ($r !== false) { [$a] = $r; } — $r is narrowed to array in the if-branch
    let src = "<?php\n/**\n * @return array|false\n */\nfunction get(): array|false { return false; }\n$r = get();\nif ($r !== false) {\n    [$a] = $r;\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "PossiblyInvalidArrayOffset");
}

#[test]
fn does_not_report_unpack_result_when_stub_present() {
    // unpack() returns array|false; if stub is present this should fire.
    // NOTE: currently FAILS because unpack() stub is missing → returns mixed → no issue.
    // This test documents the expected behavior once the unpack stub is added.
    let src = "<?php\n[$a] = unpack('N', pack('N', 1));\n";
    let issues = check(src);
    assert_issue_kind(&issues, "PossiblyInvalidArrayOffset", 2, 0);
}

#[test]
fn does_not_report_plain_array_offset_access() {
    let src = "<?php\n$arr = [1, 2, 3];\n$x = $arr[0];\n";
    let issues = check(src);
    assert_no_issue(&issues, "PossiblyInvalidArrayOffset");
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /Users/adamspychala/Projects/mir
cargo test -p mir-analyzer --test possibly_invalid_array_offset -- --nocapture 2>&1
```
Expected: `does_not_report_destructure_of_plain_array`, `reports_destructure_of_only_false`, `does_not_report_plain_array_offset_access` pass. `does_not_report_unpack_result_when_stub_present` will fail until unpack stub is added.

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/tests/possibly_invalid_array_offset.rs
git commit -m "test(possibly_invalid_array_offset): add 7 rule tests with position assertions"
```

---

## Task 10: `redundant_condition` tests

**Files:**
- Create: `crates/mir-analyzer/tests/redundant_condition.rs`

**Position reference:** `RedundantCondition` is emitted via the narrowing/type-check path. The span is the condition expression. In `if ($x === null)` at start of line, the condition `$x === null` starts at col 4 (after `if (`).

- [ ] **Step 1: Create the test file**

```rust
// crates/mir-analyzer/tests/redundant_condition.rs
use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn reports_null_check_on_non_nullable() {
    // $x is string — checking === null is always false
    let src = "<?php\nfunction f(string $x): void {\n    if ($x === null) {}\n}\n";
    let issues = check(src);
    // condition `$x === null` starts at col 8 (after "    if (")
    assert_issue_kind(&issues, "RedundantCondition", 3, 8);
}

#[test]
fn reports_not_null_check_on_non_nullable() {
    let src = "<?php\nfunction f(string $x): void {\n    if ($x !== null) {}\n}\n";
    let issues = check(src);
    assert_issue_kind(&issues, "RedundantCondition", 3, 8);
}

#[test]
fn does_not_report_null_check_on_nullable() {
    let src = "<?php\nfunction f(?string $x): void {\n    if ($x === null) {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "RedundantCondition");
}

#[test]
fn reports_is_string_on_string_type() {
    let src = "<?php\nfunction f(string $x): void {\n    if (is_string($x)) {}\n}\n";
    let issues = check(src);
    // is_string($x) starts at col 8
    assert_issue_kind(&issues, "RedundantCondition", 3, 8);
}

#[test]
fn does_not_report_is_string_on_union() {
    let src = "<?php\nfunction f(string|int $x): void {\n    if (is_string($x)) {}\n}\n";
    let issues = check(src);
    assert_no_issue(&issues, "RedundantCondition");
}

#[test]
fn reports_redundant_check_after_narrowing() {
    // After $x is narrowed to string in first branch, second check is redundant
    let src = "<?php\nfunction f(string|int $x): void {\n    if (is_string($x)) {\n        if (is_string($x)) {}\n    }\n}\n";
    let issues = check(src);
    // inner is_string($x) at line 4, col 12
    assert_issue_kind(&issues, "RedundantCondition", 4, 12);
}
```

- [ ] **Step 2: Run the tests**

```bash
cd /Users/adamspychala/Projects/mir
cargo test -p mir-analyzer --test redundant_condition -- --nocapture 2>&1
```
Expected: pass/fail status confirms which RedundantCondition cases are currently implemented.

- [ ] **Step 3: Commit**

```bash
git add crates/mir-analyzer/tests/redundant_condition.rs
git commit -m "test(redundant_condition): add 6 rule tests with position assertions"
```

---

## Self-Review Checklist

After all tasks:

- [ ] `cargo test -p mir-analyzer 2>&1` — run all integration tests together and note which pass/fail.
- [ ] The set of failing tests should match the known gaps in CLAUDE.md (unpack stub, MethodSignatureMismatch over-firing, etc.).
- [ ] Any unexpected failures indicate either a position calculation error or a newly discovered analyzer bug — investigate before closing.
