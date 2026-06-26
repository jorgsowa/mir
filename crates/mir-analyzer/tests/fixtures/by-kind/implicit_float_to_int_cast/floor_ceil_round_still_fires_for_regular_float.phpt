===description===
Regular float variables (not from floor/ceil/round) still trigger ImplicitFloatToIntCast.
Confirms TIntegralFloat suppression is specific to those functions, not all floats.

===file===
<?php
function takes_int(int $n): void { echo $n; }

$x = 3.7;
takes_int($x);

===expect===
ImplicitFloatToIntCast@5:10-5:12: Implicit cast from 3.7 to int truncates the fractional part
