===description===
Invalid generic clone
===file===
<?php
/**
 * @template T as int|string
 * @param T $a
 */
function foo($a): void {
    clone $a;
//  ^^^^^^^^ InvalidClone: cannot clone non-object int|string
}
===expect===
