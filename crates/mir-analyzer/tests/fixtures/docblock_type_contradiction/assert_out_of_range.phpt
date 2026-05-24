===description===
assertOutOfRange
===file===
<?php
/**
 * @param int<1, 5> $a
 */
function scope(int $a): void{
    assert($a === 0);
}
===expect===
DocblockTypeContradiction
===ignore===
TODO
