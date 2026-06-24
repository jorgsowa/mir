===description===
N5 (false-branch fix): TCallableString must be recognised as callable so the
!is_callable false branch correctly removes it. A callable-string atom is
definitionally callable, so is_callable() is always true for it.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function needs_int(int $i): void {}
function needs_string(string $s): void {}

/** @param callable-string $fn */
function test_not_callable_of_callable_string_is_redundant(mixed $fn): void {
    if (!is_callable($fn)) {}
}

/** @param callable-string $fn */
function test_callable_of_callable_string_is_redundant(mixed $fn): void {
    if (is_callable($fn)) {
        needs_int($fn);
    }
}

/** @param callable-string|int $x */
function test_union_true_branch_narrows_out_int(mixed $x): void {
    if (is_callable($x)) {
        needs_int($x);
    }
}

/** @param callable-string|int $x */
function test_union_false_branch_narrows_out_callable_string(mixed $x): void {
    if (!is_callable($x)) {
        needs_string($x);
    }
}
===expect===
RedundantCondition@8:8-8:25: Condition is always true/false for type 'bool'
RedundantCondition@13:8-13:24: Condition is always true/false for type 'bool'
InvalidArgument@14:18-14:21: Argument $i of needs_int() expects 'int', got 'callable-string'
InvalidArgument@21:18-21:20: Argument $i of needs_int() expects 'int', got 'callable-string'
ArgumentTypeCoercion@28:21-28:23: Argument $s of needs_string() expects 'string', got 'int' — coercion may fail at runtime
