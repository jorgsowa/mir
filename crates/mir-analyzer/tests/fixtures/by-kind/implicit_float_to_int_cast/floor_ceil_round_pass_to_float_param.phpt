===description===
TIntegralFloat is a subtype of float, so passing floor/ceil/round results to a float-typed
parameter is silently accepted — no diagnostic fires.

===file===
<?php
function takes_float(float $x): void { echo $x; }

takes_float(floor(3.7));
takes_float(ceil(3.1));
takes_float(round(3.5));

===expect===
