===description===
Switch var reassigned in branch
===file===
<?php
$a = false;

switch (rand(0, 2)) {
    case 0:
        $a = true;
}
===expect===
UnusedVariable@2:1-2:3: Variable $a is never read
