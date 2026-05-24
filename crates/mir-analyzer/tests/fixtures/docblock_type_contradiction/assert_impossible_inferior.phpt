===description===
assertImpossibleInferior
===file===
<?php
/**
 * @param int<5, max> $a
 */
function scope(int $a): void{
    assert($a < 4);
}
===expect===
DocblockTypeContradiction
===ignore===
TODO
