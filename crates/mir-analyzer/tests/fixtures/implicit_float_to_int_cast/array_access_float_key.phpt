===description===
Using float value as array offset - silently truncated to int

===file===
<?php
$arr = [];
$x = 3.7;
$val = $arr[$x];

===expect===
ImplicitFloatToIntCast@4:12: Implicit cast from 3.7 to int truncates the fractional part
