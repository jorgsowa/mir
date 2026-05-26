===description===
Detect redundancy after loop with continue
===file===
<?php
$gap = null;

foreach ([1, 2, 3] as $_) {
    if (rand(0, 1)) {
        continue;
    }

    $gap = "asa";
    throw new Exception($gap);
}
===expect===
UnusedVariable
===ignore===
TODO
