===description===
loopTypeChangedInIfWithoutReference
===file===
<?php
$a = false;

while (rand(0, 1)) {
    if (rand(0, 1)) {
        $a = true;
    }
}
===expect===
UnusedVariable
===ignore===
TODO
