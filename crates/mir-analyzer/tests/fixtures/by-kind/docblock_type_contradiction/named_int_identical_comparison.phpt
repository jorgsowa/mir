===description===
`positive-int === 0` and `negative-int === 1` are statically impossible;
`can_equal` knows the named int subtypes' bounds.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test_pos_eq_zero(int $n): void {
    assert($n === 0);
}

/** @param negative-int $n */
function test_neg_eq_one(int $n): void {
    assert($n === 1);
}

/** @param non-negative-int $n */
function test_nonneg_eq_minus_one(int $n): void {
    assert($n === -1);
}
===expect===
DocblockTypeContradiction@4:11-4:19: Type 'positive-int' makes '$n === 0' impossible — this can never hold
DocblockTypeContradiction@9:11-9:19: Type 'negative-int' makes '$n === 1' impossible — this can never hold
DocblockTypeContradiction@14:11-14:20: Type 'non-negative-int' makes '$n === -1' impossible — this can never hold
