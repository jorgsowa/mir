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
}
===expect===
PossiblyInvalidClone@7:5-7:13: cannot clone possibly non-object int|Exception
