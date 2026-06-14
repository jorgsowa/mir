===description===
Assert out of range
===file===
<?php
/**
 * @param int<1, 5> $a
 */
function scope(int $a): void{
    assert($a === 0);
}
===expect===
DocblockTypeContradiction@6:12-6:20: Type 'int<1, 5>' makes '$a === 0' impossible — this can never hold
