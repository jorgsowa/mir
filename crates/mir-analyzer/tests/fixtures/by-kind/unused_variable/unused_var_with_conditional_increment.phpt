===description===
Unused var with conditional increment
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $a++;
}
===expect===
UnusedVariable@4:4-4:6: Variable $a is never read
