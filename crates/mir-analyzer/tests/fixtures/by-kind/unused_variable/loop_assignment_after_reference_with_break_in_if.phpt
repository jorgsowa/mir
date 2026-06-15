===description===
Loop assignment after reference with break in if
===file===
<?php
$a = 0;
while (rand(0, 1)) {
    echo $a;

    if (rand(0, 1)) {
        $a = 1;
        break;
    }
}
===expect===
UnusedVariable@7:8-7:10: Variable $a is never read
