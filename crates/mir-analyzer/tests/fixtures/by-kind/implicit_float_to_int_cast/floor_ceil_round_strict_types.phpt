===description===
With strict_types=1, even TIntegralFloat (floor/ceil/round) cannot be passed to int without
an explicit cast. PHP throws TypeError, so InvalidArgument fires instead of being silently
accepted or emitting ImplicitFloatToIntCast.

===file===
<?php
declare(strict_types=1);
function takes_int(int $n): void { echo $n; }

takes_int(floor(3.7));
//        ^^^^^^^^^^ InvalidArgument: Argument $n of takes_int() expects 'int', got 'float'
takes_int(ceil(3.1));
//        ^^^^^^^^^ InvalidArgument: Argument $n of takes_int() expects 'int', got 'float'
takes_int(round(3.5));
//        ^^^^^^^^^^ InvalidArgument: Argument $n of takes_int() expects 'int', got 'float'

===expect===
