===description===
Detect redundancy after loop with continue
===config===
suppress=MissingThrowsDocblock
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
UnusedVariable@2:0-2:4: Variable $gap is never read
