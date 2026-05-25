===description===
Assert redundant inferior
===file===
<?php
/**
 * @param int<min, 5> $a
 */
function scope(int $a): void{
    assert($a < 10);
}
===expect===
RedundantConditionGivenDocblockType
===ignore===
TODO
