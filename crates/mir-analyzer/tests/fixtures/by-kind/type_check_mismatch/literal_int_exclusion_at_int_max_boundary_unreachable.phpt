===description===
Excluding PHP_INT_MAX via `!== PHP_INT_MAX` from a range pinned exactly to
PHP_INT_MAX must empty the range (no larger int exists) — `value.checked_add(1)`
overflowing used to silently produce an unbounded range instead, so the
provably-impossible branch was wrongly treated as reachable. Mirrors the
existing `int_comparison_php_int_max_boundary_unreachable.phpt` reachability
probe but for the literal `!==`/`===` exclusion path, not `>`/`<=`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<9223372036854775807, 9223372036854775807> $x */
function test_excluding_int_max_from_pinned_max_is_unreachable(int $x): void {
    if ($x !== 9223372036854775807) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

/** @param int<1,1> $x */
function test_ordinary_single_point_exclusion_still_unreachable(int $x): void {
    if ($x !== 1) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}
===expect===
RedundantCondition@4:8-4:34: Condition is always true/false for type 'bool'
RedundantCondition@12:8-12:16: Condition is always true/false for type 'bool'
