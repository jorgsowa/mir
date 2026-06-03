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
UnusedVariable@7:9-7:11: Variable $a is never read
