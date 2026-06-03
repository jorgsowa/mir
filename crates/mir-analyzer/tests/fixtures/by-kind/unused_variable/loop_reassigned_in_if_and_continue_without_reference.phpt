===description===
Loop reassigned in if and continue without reference
===file===
<?php
$a = 3;

echo $a;

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = 5;
        continue;
    }

    $a = 3;
}
===expect===
UnusedVariable@12:5-12:7: Variable $a is never read
