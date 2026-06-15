===description===
foreach by-reference iteration: writes to the reference variable update the source array and must not be flagged as unused
===file===
<?php
$a = [1, 2, 3];
foreach ($a as &$b) {
    $b = $b + 1;
}
===expect===
