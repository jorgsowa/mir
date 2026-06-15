===description===
Redundant cast from int variable to int

===config===
suppress=UnusedVariable
===file===
<?php
$x = 3;
$y = (int)$x;

===expect===
RedundantCast@3:10-3:12: Casting '3' to 'int' is redundant
