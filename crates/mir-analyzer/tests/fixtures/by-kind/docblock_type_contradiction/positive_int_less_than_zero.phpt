===description===
`positive-int` compared with `< 0` or `=== 0` is a docblock contradiction.
===file===
<?php
/** @param positive-int $n */
function test_lt(int $n): void {
    assert($n < 0);
//         ^^^^^^ DocblockTypeContradiction: Type 'positive-int' makes '$n < 0' impossible — this can never hold
}

/** @param positive-int $n */
function test_identical(int $n): void {
    assert($n === 0);
//         ^^^^^^^^ DocblockTypeContradiction: Type 'positive-int' makes '$n === 0' impossible — this can never hold
}
