===description===
Switch var conditional assignment without reference
===file===
<?php
switch (rand(0, 4)) {
    case 0:
        if (rand(0, 1)) {
            $a = 0;
            break;
        }

    default:
        $a = 1;
}
===expect===
UnusedVariable@10:9-10:11: Variable $a is never read
