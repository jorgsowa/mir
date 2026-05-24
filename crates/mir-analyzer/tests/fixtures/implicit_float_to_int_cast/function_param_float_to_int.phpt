===description===
Passing float value to int-typed parameter

===file===
<?php
function foo(int $n): void {
    echo $n;
}

$x = 3.7;
foo($x);

===expect===
ImplicitFloatToIntCast@7:5: Implicit cast from 3.7 to int truncates the fractional part
InvalidArgument@7:5: Argument $n of foo() expects 'int', got '3.7'
