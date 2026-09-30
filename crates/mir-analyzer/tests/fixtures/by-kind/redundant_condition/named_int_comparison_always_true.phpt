===description===
Comparisons on named int subtypes that are always true are reported as
RedundantCondition: positive-int > 0, non-negative-int >= 0, negative-int < 0.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test_pos_gt_zero(int $n): void {
    if ($n > 0) {
//      ^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
        $_ = $n;
    }
}

/** @param non-negative-int $n */
function test_nonneg_ge_zero(int $n): void {
    if ($n >= 0) {
//      ^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
        $_ = $n;
    }
}

/** @param negative-int $n */
function test_neg_lt_zero(int $n): void {
    if ($n < 0) {
//      ^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
        $_ = $n;
    }
}
===expect===
