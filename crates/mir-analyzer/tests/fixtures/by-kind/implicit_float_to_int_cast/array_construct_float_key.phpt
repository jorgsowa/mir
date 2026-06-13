===description===
Using float value as array key in array construction - silently truncated to int

===config===
suppress=UnusedVariable
===file===
<?php
$arr = [1.5 => "value"];

===expect===
ImplicitFloatToIntCast@2:9-2:12: Implicit cast from 1.5 to int truncates the fractional part
