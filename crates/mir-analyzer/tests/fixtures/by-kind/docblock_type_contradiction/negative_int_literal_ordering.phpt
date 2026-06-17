===description===
Ordering comparisons with negative int literals: `int<5, 10> < -1` is
impossible since min=5 > -1; these also require extract_lit to handle
negated literals.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<5, 10> $n */
function test_range_less_than_neg(int $n): void {
    assert($n < -1);
}

/** @param positive-int $n */
function test_pos_less_than_neg(int $n): void {
    assert($n < -1);
}

/** @param positive-int $n */
function test_pos_lte_neg(int $n): void {
    assert($n <= -1);
}
===expect===
DocblockTypeContradiction@4:11-4:18: Type 'int<5, 10>' makes '$n < -1' impossible — this can never hold
DocblockTypeContradiction@9:11-9:18: Type 'positive-int' makes '$n < -1' impossible — this can never hold
DocblockTypeContradiction@14:11-14:19: Type 'positive-int' makes '$n <= -1' impossible — this can never hold
