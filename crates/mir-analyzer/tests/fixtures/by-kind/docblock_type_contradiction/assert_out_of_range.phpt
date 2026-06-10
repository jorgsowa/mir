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
