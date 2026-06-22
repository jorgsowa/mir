===description===
N5: is_callable($x) true branch must preserve string and array atoms — PHP
accepts strings (function names) and arrays (['class', 'method']) as callables.
Previously, narrow_to_callable() dropped string/array atoms, giving a narrower
type than PHP's actual runtime behaviour.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_string_kept_in_true_branch(string $fn): void {
    if (is_callable($fn)) {
        // String that passed is_callable IS callable — must not be dropped.
        /** @mir-check $fn is string */
        $_ = $fn;
    }
}

function test_array_kept_in_true_branch(array $pair): void {
    if (is_callable($pair)) {
        // Array that passed is_callable IS callable — must not be dropped.
        /** @mir-check $pair is array */
        $_ = $pair;
    }
}

function test_closure_kept_in_true_branch(\Closure $fn): void {
    if (is_callable($fn)) {
        /** @mir-check $fn is Closure */
        $_ = $fn;
    }
}

/** @param string|int $x */
function test_non_callable_dropped_in_true_branch(mixed $x): void {
    if (is_callable($x)) {
        // int is not a valid callable — it must be removed from the true branch,
        // leaving only string (which can be a callable function name).
        /** @mir-check $x is string */
        $_ = $x;
    }
}

/** @param string|callable $x */
function test_false_branch_keeps_string(mixed $x): void {
    if (!is_callable($x)) {
        // TCallable is definitely callable — removed from false branch.
        // String might be non-callable — kept.
        /** @mir-check $x is string */
        $_ = $x;
    }
}
===expect===
