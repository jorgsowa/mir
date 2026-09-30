===description===
Redundant cast from float literal to float

===config===
suppress=UnusedVariable
===file===
<?php
$x = (float)3.0;
//          ^^^ RedundantCast: Casting '3' to 'float' is redundant

===expect===
