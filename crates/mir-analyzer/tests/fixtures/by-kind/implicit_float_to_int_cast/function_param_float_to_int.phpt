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
ImplicitFloatToIntCast@7:4-7:6: Implicit cast from 3.7 to int truncates the fractional part
