===description===
Using floor/ceil/round result as an array key: TIntegralFloat keys are always integral
so the float→int truncation is lossless — no ImplicitFloatToIntCast fires.
Compare with a regular float key which does fire.

===file===
<?php
$arr = [1, 2, 3];

// TIntegralFloat key — no warning
echo $arr[floor(1.7)];
echo $arr[ceil(0.3)];
echo $arr[round(1.5)];

// Regular float key — warning fires
$d = 1.7;
echo $arr[$d];

===expect===
ImplicitFloatToIntCast@11:10-11:12: Implicit cast from 1.7 to int truncates the fractional part
