===description===
`positive-int === 0` and `negative-int === 1` are statically impossible;
`can_equal` knows the named int subtypes' bounds.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param positive-int $n */
function test_pos_eq_zero(int $n): void {
    assert($n === 0);
//         ^^^^^^^^ DocblockTypeContradiction: Type 'positive-int' makes '$n === 0' impossible — this can never hold
}

/** @param negative-int $n */
function test_neg_eq_one(int $n): void {
    assert($n === 1);
//         ^^^^^^^^ DocblockTypeContradiction: Type 'negative-int' makes '$n === 1' impossible — this can never hold
}

/** @param non-negative-int $n */
function test_nonneg_eq_minus_one(int $n): void {
    assert($n === -1);
//         ^^^^^^^^^ DocblockTypeContradiction: Type 'non-negative-int' makes '$n === -1' impossible — this can never hold
}
