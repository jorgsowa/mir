===description===
Loop assignment after reference with break
===file===
<?php
$a = 0;
while (rand(0, 1)) {
    echo $a;
    $a = 1;
    break;
}
===expect===
UnusedVariable@5:5-5:7: Variable $a is never read
