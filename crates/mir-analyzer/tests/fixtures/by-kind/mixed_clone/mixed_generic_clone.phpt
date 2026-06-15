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
MixedClone@7:4-7:12: cannot clone mixed
