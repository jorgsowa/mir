===description===
(divergence from Psalm: `$a = 5` is read by the next iteration's
`$a !== null` condition; `$a = 4` before break is the dead write)
Loop set if null with break without reference
===file===
<?php
$a = null;

while (rand(0, 1)) {
    if ($a !== null) {
        $a = 4;
        break;
    }

    $a = 5;
}
===expect===
UnusedVariable@6:8-6:10: Variable $a is never read
