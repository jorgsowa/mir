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
