===description===
Byref var sets possible
===file===
<?php
/**
 * @param mixed $a
 * @param-out int $a
 */
function takesByRef(&$a) : void {
    $a = 5;
}

if (rand(0, 1)) {
    takesByRef($b);
}

echo $b;
===expect===
PossiblyUndefinedVariable@14:6-14:8: Variable $b might not be defined
