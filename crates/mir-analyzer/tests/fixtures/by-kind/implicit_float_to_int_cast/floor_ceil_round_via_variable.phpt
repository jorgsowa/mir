===description===
TIntegralFloat assigned to a variable then passed to int param: no ImplicitFloatToIntCast.
Covers the intermediate-variable path to ensure TIntegralFloat is preserved through assignment.

===file===
<?php
function takes_int(int $n): void { echo $n; }

$a = floor(3.7);
$b = ceil(3.1);
$c = round(3.5);
takes_int($a);
takes_int($b);
takes_int($c);

===expect===
