===description===
Arithmetic on TIntegralFloat results produces regular float, so the result of
floor(x) + 1.5 is float (not TIntegralFloat). Passing that to an int param still
fires ImplicitFloatToIntCast.

===file===
<?php
function takes_int(int $n): void { echo $n; }

$x = floor(3.7) + 1.5;
takes_int($x);

===expect===
ImplicitFloatToIntCast@5:10-5:12: Implicit cast from float to int truncates the fractional part
