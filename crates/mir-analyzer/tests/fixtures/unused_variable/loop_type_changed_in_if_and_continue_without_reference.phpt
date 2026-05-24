===description===
loopTypeChangedInIfAndContinueWithoutReference
===file===
<?php
$a = false;

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = true;
        continue;
    }

    $a = false;
}
===expect===
UnusedVariable
===ignore===
TODO
