===description===
Loop set if null with break without reference2
===file===
<?php
$a = null;

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = 4;
        break;
    }

    $a = 5;
}
===expect===
UnusedVariable@2:0-2:2: Variable $a is never read
