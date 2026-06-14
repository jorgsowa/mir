===description===
Correct case in first-class callable syntax is not reported.
===config===
suppress=UnusedVariable
===file===
<?php
function myFunc(int $x): int { return $x; }

$fn = myFunc(...);
===expect===
