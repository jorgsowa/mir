===description===
When a variable has a single-point int range, a !== check against a value
outside that range is always true (the false branch is unreachable).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<5, 5> $n */
function test_ne_out_of_range(int $n): void {
    if ($n !== 0) {
        $_ = $n; // always reached
    }
}

/** @param int<5, 5> $n */
function test_eq_out_of_range(int $n): void {
    if ($n === 0) {
        $_ = $n; // never reached
    }
}
===expect===
RedundantCondition@4:8-4:16: Condition is always true/false for type 'bool'
DocblockTypeContradiction@11:8-11:16: Type 'int<5, 5>' makes '$n === 0' impossible — this can never hold
RedundantCondition@11:8-11:16: Condition is always true/false for type 'bool'
