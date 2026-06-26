===description===
floor()/ceil()/round() return TIntegralFloat (always a whole-valued float), so passing
their result to an int-typed parameter is lossless — no ImplicitFloatToIntCast fires.
round() with an explicit precision of 0 is also covered.

===file===
<?php
function takes_int(int $n): void { echo $n; }

takes_int(floor(3.7));
takes_int(ceil(3.1));
takes_int(round(3.5));
takes_int(round(3.5, 0));

===expect===
