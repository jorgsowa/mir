===description===
SKIPPED-byrefInForeachLoopWithoutReference
===file===
<?php
$a = [1, 2, 3];
foreach ($a as &$b) {
    $b = $b + 1;
}
===expect===
UnusedVariable@4:5-4:7: Variable $b is never read
