===description===
Passing literal float value to int-typed parameter

===file===
<?php
function foo(int $n): void {
    echo $n;
}

foo(3.7);
//  ^^^ ImplicitFloatToIntCast: Implicit cast from 3.7 to int truncates the fractional part
