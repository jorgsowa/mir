===description===
`negative-int` compared with `> 0` is a docblock contradiction.
===file===
<?php
/** @param negative-int $n */
function test(int $n): void {
    assert($n > 0);
}
===expect===
DocblockTypeContradiction@4:11-4:17: Type 'negative-int' makes '$n > 0' impossible — this can never hold
