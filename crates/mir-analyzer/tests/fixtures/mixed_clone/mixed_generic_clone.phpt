===description===
mixedGenericClone
===file===
<?php
/**
 * @template T
 * @param T $a
 */
function foo($a): void {
    clone $a;
}
===expect===
MixedClone@7:4: cannot clone mixed
