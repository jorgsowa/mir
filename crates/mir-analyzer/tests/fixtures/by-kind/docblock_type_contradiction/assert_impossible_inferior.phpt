===description===
Assert impossible inferior
===file===
<?php
/**
 * @param int<5, max> $a
 */
function scope(int $a): void{
    assert($a < 4);
//         ^^^^^^ DocblockTypeContradiction: Type 'int<5, max>' makes '$a < 4' impossible — this can never hold
}
===expect===
