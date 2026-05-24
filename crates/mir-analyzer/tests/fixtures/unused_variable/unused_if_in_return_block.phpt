===description===
unusedIfInReturnBlock
===file===
<?php
$i = rand(0, 1);

foreach ([1, 2, 3] as $a) {
    if ($a % 2) {
        $i = 7;
        return;
    }
}

if ($i) {}
===expect===
UnusedVariable
===ignore===
TODO
