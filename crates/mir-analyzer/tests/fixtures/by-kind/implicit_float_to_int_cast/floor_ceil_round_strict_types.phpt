===description===
With strict_types=1, even TIntegralFloat (floor/ceil/round) cannot be passed to int without
an explicit cast. PHP throws TypeError, so InvalidArgument fires instead of being silently
accepted or emitting ImplicitFloatToIntCast.

===file===
<?php
declare(strict_types=1);
function takes_int(int $n): void { echo $n; }

takes_int(floor(3.7));
takes_int(ceil(3.1));
takes_int(round(3.5));

===expect===
InvalidArgument@5:10-5:20: Argument $n of takes_int() expects 'int', got 'float'
InvalidArgument@6:10-6:19: Argument $n of takes_int() expects 'int', got 'float'
InvalidArgument@7:10-7:20: Argument $n of takes_int() expects 'int', got 'float'
