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
PossiblyUndefinedGlobalVariable
===ignore===
TODO
