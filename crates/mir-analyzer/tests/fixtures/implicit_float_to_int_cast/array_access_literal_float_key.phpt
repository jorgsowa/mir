===description===
Using literal float value as array offset - silently truncated to int

===file===
<?php
$arr = [];
$val = $arr[3.7];

===expect===
ImplicitFloatToIntCast@3:13: Implicit cast from 3.7 to int truncates the fractional part
