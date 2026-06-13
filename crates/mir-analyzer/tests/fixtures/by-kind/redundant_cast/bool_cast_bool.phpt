===description===
Redundant cast from bool to bool

===config===
suppress=UnusedVariable
===file===
<?php
$x = true;
$y = (bool)$x;

===expect===
RedundantCast@3:12-3:14: Casting 'true' to 'bool' is redundant
