===description===
loopSetIfNullWithBreakWithoutReference2
===file===
<?php
$a = null;

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = 4;
        break;
    }

    $a = 5;
}
===expect===
UnusedVariable
===ignore===
TODO
