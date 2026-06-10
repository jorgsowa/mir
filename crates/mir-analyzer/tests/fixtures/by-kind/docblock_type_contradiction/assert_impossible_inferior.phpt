===description===
Assert impossible inferior
===file===
<?php
/**
 * @param int<5, max> $a
 */
function scope(int $a): void{
    assert($a < 4);
}
===expect===
