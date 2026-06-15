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
}
===expect===
InvalidClone@7:4-7:12: cannot clone non-object int|string
