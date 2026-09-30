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
//      ^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        $_ = $n; // always reached
    }
}

/** @param int<5, 5> $n */
function test_eq_out_of_range(int $n): void {
    if ($n === 0) {
//      ^^^^^^^^ DocblockTypeContradiction: Type 'int<5, 5>' makes '$n === 0' impossible — this can never hold
//      ^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        $_ = $n; // never reached
    }
}
===expect===
