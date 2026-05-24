===description===
switchInIf
===file===
<?php
$a = 0;

if (rand(0, 1)) {
    switch (rand(0, 4)) {
        case 0:
            $a = 3;
            break;

        default:
            $a = 3;
    }
} else {
    $a = 6;
}

echo $a;
===expect===
UnusedVariable
===ignore===
TODO
