===description===
Ordering comparisons with negative int literals: `int<5, 10> < -1` is
impossible since min=5 > -1; these also require extract_lit to handle
negated literals.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param int<5, 10> $n */
function test_range_less_than_neg(int $n): void {
    assert($n < -1);
//         ^^^^^^^ DocblockTypeContradiction: Type 'int<5, 10>' makes '$n < -1' impossible — this can never hold
}

/** @param positive-int $n */
function test_pos_less_than_neg(int $n): void {
    assert($n < -1);
//         ^^^^^^^ DocblockTypeContradiction: Type 'positive-int' makes '$n < -1' impossible — this can never hold
}

/** @param positive-int $n */
function test_pos_lte_neg(int $n): void {
    assert($n <= -1);
//         ^^^^^^^^ DocblockTypeContradiction: Type 'positive-int' makes '$n <= -1' impossible — this can never hold
}
