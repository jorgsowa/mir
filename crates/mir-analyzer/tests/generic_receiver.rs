use mir_test_utils::{assert_issue_kind, assert_no_issue, check};

#[test]
fn class_level_template_resolves_to_concrete_type() {
    // In the broken state, first() returns mixed → MixedMethodCall (not UndefinedMethod)
    // when calling a non-existent method. In the fixed state, first() returns User →
    // UndefinedMethod. This test gates the fix.
    let src = r#"<?php
/** @template T */
class Collection {
    /** @return T */
    public function first(): mixed { return null; }
}
class User {
    public function getName(): string { return 'Alice'; }
}
function test(): void {
    /** @var Collection<User> $items */
    $items = new Collection();
    $first = $items->first();
    $first->nonExistentMethod();
}
"#;
    let issues = check(src);
    assert_issue_kind(&issues, "UndefinedMethod", 14, 4);
}

#[test]
fn class_level_template_allows_valid_user_methods() {
    // After the fix, calling an existing User method must not emit UndefinedMethod.
    let src = r#"<?php
/** @template T */
class Collection {
    /** @return T */
    public function first(): mixed { return null; }
}
class User {
    public function getName(): string { return 'Alice'; }
}
function test(): void {
    /** @var Collection<User> $items */
    $items = new Collection();
    $first = $items->first();
    $first->getName();
}
"#;
    let issues = check(src);
    assert_no_issue(&issues, "UndefinedMethod");
}
