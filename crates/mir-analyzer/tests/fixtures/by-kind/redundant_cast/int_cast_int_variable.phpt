===description===
Redundant cast from int variable to int

===config===
suppress=UnusedVariable
===file===
<?php
$x = 3;
$y = (int)$x;
//        ^^ RedundantCast: Casting '3' to 'int' is redundant

===expect===
