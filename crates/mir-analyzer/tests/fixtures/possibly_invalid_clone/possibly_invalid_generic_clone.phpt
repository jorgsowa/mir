===description===
possiblyInvalidGenericClone
===file===
<?php
/**
 * @template T as int|Exception
 * @param T $a
 */
function foo($a): void {
    clone $a;
}
===expect===
PossiblyInvalidClone
===ignore===
TODO
