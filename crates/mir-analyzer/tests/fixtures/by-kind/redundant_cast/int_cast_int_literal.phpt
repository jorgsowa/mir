===description===
Redundant cast from int literal to int

===config===
suppress=UnusedVariable
===file===
<?php
$x = (int)3;

===expect===
RedundantCast@2:10-2:11: Casting '3' to 'int' is redundant
