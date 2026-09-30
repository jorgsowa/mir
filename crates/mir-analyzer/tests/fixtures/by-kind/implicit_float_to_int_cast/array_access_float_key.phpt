===description===
Using float value as array offset - silently truncated to int

===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
$arr = [];
$x = 3.7;
$val = $arr[$x];
//          ^^ ImplicitFloatToIntCast: Implicit cast from 3.7 to int truncates the fractional part

===expect===
