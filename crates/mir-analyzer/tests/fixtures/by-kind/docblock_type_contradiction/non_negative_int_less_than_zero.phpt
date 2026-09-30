===description===
`non-negative-int` compared with `< 0` is a docblock contradiction.
===file===
<?php
/** @param non-negative-int $n */
function test(int $n): void {
    assert($n < 0);
//         ^^^^^^ DocblockTypeContradiction: Type 'non-negative-int' makes '$n < 0' impossible — this can never hold
}
===expect===
