===description===
`positive-int` compared with `< 0` or `=== 0` is a docblock contradiction.
===file===
<?php
/** @param positive-int $n */
function test_lt(int $n): void {
    assert($n < 0);
}

/** @param positive-int $n */
function test_identical(int $n): void {
    assert($n === 0);
}
===expect===
DocblockTypeContradiction@4:11-4:17: Type 'positive-int' makes '$n < 0' impossible — this can never hold
DocblockTypeContradiction@9:11-9:19: Type 'positive-int' makes '$n === 0' impossible — this can never hold
