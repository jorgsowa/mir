===description===
Unused var with conditional increment
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $a++;
}
===expect===
UnusedVariable@4:5-4:7: Variable $a is never read
