===description===
Mixed generic clone
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
MixedClone@7:5: cannot clone mixed
