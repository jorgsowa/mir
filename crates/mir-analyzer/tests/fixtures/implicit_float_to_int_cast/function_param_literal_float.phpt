===description===
Passing literal float value to int-typed parameter

===file===
<?php
function foo(int $n): void {
    echo $n;
}

foo(3.7);

===expect===
ImplicitFloatToIntCast@6:4: Implicit cast from 3.7 to int truncates the fractional part
InvalidArgument@6:4: Argument $n of foo() expects 'int', got '3.7'
