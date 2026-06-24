===description===
N5 (false-branch fix): @mir-check verifies that callable-string is correctly
removed from the !is_callable false branch and kept in the is_callable true
branch. Uses type-check assertions instead of argument diagnostics so the
narrowing result is directly observable without side-effect calls.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

/** @param callable-string|int $x */
function test_is_callable_true_branch(mixed $x): void {
    if (is_callable($x)) {
        /** @mir-check $x is callable-string */
        $_ = $x;
    }
}

/** @param callable-string|int $x */
function test_is_callable_false_branch(mixed $x): void {
    if (!is_callable($x)) {
        /** @mir-check $x is int */
        $_ = $x;
    }
}

/** @param callable-string|string $x */
function test_mixed_string_types_false_branch(mixed $x): void {
    if (!is_callable($x)) {
        /** @mir-check $x is string */
        $_ = $x;
    }
}
===expect===
