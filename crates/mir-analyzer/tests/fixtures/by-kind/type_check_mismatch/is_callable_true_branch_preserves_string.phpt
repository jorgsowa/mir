===description===
N5: is_callable() true branch must NOT diverge for string, array, or object atoms.
Before the fix, narrow_to_callable() dropped those atoms and mark_diverges=true made the
branch unreachable — diagnostics inside were silently suppressed.

These tests detect wrong diverge by putting an ArgumentTypeCoercion inside the
is_callable true branch. The diagnostic fires only when the branch is alive; if
the branch is wrongly diverged it is suppressed and the test fails.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function needs_int(int $i): void {}

function test_string_branch_alive(string $fn): void {
    if (is_callable($fn)) {
        // Branch must be reachable: $fn is string so needs_int fires.
        // Before fix: branch diverges → suppressed → test fails.
        needs_int($fn);
    }
}

function test_array_branch_alive(array $pair): void {
    if (is_callable($pair)) {
        // Same: array must survive into the true branch.
        needs_int($pair);
    }
}

/** @param string|int $x */
function test_int_dropped_string_kept(mixed $x): void {
    if (is_callable($x)) {
        // After fix: $x is string (int dropped). needs_int(string) fires.
        // Before fix: branch diverges → suppressed → test fails.
        needs_int($x);
    }
}

/** @param string|callable $x */
function test_false_branch_keeps_string(mixed $x): void {
    if (!is_callable($x)) {
        // callable removed, string kept. Unchanged by this fix; sanity guard.
        /** @mir-check $x is string */
        $_ = $x;
    }
}
===expect===
InvalidArgument@9:18-9:21: Argument $i of needs_int() expects 'int', got 'string'
InvalidArgument@16:18-16:23: Argument $i of needs_int() expects 'int', got 'array'
InvalidArgument@25:18-25:20: Argument $i of needs_int() expects 'int', got 'string'
