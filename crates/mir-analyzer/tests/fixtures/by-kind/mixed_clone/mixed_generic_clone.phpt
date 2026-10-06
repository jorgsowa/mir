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
//  ^^^^^^^^ MixedClone: cannot clone mixed
}
