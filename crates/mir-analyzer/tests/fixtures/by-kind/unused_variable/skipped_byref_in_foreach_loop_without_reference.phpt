===description===
SKIPPED-byrefInForeachLoopWithoutReference
===file===
<?php
$a = [1, 2, 3];
foreach ($a as &$b) {
    $b = $b + 1;
}
===expect===
UnusedForeachValue@4:4-4:6: Foreach value $b is never read
