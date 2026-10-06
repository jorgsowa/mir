===description===
Possibly invalid generic clone
===file===
<?php
/**
 * @template T as int|Exception
 * @param T $a
 */
function foo($a): void {
    clone $a;
//  ^^^^^^^^ PossiblyInvalidClone: cannot clone possibly non-object int|Exception
}
