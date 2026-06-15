===description===
Redundant cast from bool to bool

===config===
suppress=UnusedVariable
===file===
<?php
$x = true;
$y = (bool)$x;

===expect===
RedundantCast@3:11-3:13: Casting 'true' to 'bool' is redundant
