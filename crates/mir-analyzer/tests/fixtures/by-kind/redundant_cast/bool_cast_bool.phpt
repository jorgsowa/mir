===description===
Redundant cast from bool to bool

===config===
suppress=UnusedVariable
===file===
<?php
$x = true;
$y = (bool)$x;
//         ^^ RedundantCast: Casting 'true' to 'bool' is redundant

===expect===
