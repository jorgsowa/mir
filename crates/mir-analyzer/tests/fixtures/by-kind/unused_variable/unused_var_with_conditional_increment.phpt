===description===
Unused var with conditional increment
===file===
<?php
$a = 5;
if (rand(0, 1)) {
    $a++;
}
===expect===
