===description===
Redundant cast from int literal to int

===config===
suppress=UnusedVariable
===file===
<?php
$x = (int)3;
//        ^ RedundantCast: Casting '3' to 'int' is redundant

===expect===
