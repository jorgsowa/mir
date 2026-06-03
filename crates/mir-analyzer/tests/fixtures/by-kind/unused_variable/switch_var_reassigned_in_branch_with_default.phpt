===description===
Switch var reassigned in branch with default
===file===
<?php
$a = false;

switch (rand(0, 2)) {
    case 0:
        $a = true;
        break;

    default:
        $a = false;
}
===expect===
UnusedVariable@2:1-2:3: Variable $a is never read
