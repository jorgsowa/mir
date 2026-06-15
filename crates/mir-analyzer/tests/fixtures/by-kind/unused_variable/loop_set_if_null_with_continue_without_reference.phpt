===description===
Loop set if null with continue without reference
===file===
<?php
$a = null;

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = 4;
        continue;
    }

    $a = 5;
}
===expect===
UnusedVariable@2:0-2:2: Variable $a is never read
