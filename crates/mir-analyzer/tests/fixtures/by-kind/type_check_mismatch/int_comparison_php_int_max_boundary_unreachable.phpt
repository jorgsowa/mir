===description===
`$x > PHP_INT_MAX` (and its `<=`-negation) is a genuine contradiction — no
`int` can exceed `i64::MAX` — but `n.checked_add(1)` overflowing used to
silently fall back to an unconstrained upper bound instead of an empty
range, so the impossible branch was wrongly treated as reachable. Uses the
`@mir-check $_ is never` reachability-probe pattern: a branch the analyzer
marks divergent skips analysis of its body entirely, so the assertion
silently produces no diagnostic; a reachable branch produces a mismatch.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<0,10> $x */
function test_greater_than_int_max_is_unreachable(int $x): void {
    if ($x > 9223372036854775807) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^ DocblockTypeContradiction: Type 'int<0, 10>' makes '$x > 9223372036854775807' impossible — this can never hold
//      ^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

/** @param int<0,10> $x */
function test_not_less_or_equal_int_max_is_unreachable(int $x): void {
    if ($x <= 9223372036854775807) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        return;
    }
    /** @mir-check $_ is never */
    $_ = 1;
//  ^^^^^^^ UnreachableCode: Unreachable code detected
}

/** @param int<0,10> $x */
function test_ordinary_comparison_still_narrows(int $x): void {
    if ($x > 5) {
        /** @mir-check $x is int<6, 10> */
        $_ = 1;
    }
}
===expect===
