===description===
`negative-int` compared with `> 0` is a docblock contradiction.
===file===
<?php
/** @param negative-int $n */
function test(int $n): void {
    assert($n > 0);
//         ^^^^^^ DocblockTypeContradiction: Type 'negative-int' makes '$n > 0' impossible — this can never hold
}
===expect===
