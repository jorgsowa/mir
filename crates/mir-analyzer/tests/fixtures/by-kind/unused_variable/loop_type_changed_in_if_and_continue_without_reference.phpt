===description===
Loop type changed in if and continue without reference
===file===
<?php
$a = false;

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = true;
        continue;
    }

    $a = false;
}
===expect===
UnusedVariable@2:1-2:3: Variable $a is never read
