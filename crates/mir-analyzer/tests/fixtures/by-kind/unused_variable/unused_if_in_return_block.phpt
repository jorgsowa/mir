===description===
Unused if in return block
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
UnusedVariable@2:1-2:3: Variable $i is never read
