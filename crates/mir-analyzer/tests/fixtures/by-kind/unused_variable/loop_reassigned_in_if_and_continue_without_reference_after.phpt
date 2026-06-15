===description===
Loop reassigned in if and continue without reference after
===file===
<?php
$a = 5;

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = 7;
        continue;
    }

    $a = 3;
}
===expect===
UnusedVariable@2:0-2:2: Variable $a is never read
