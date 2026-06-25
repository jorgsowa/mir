===description===
In strict_types=1 mode, passing float to int-typed parameter is a TypeError.
Should emit InvalidArgument (Error), not ImplicitFloatToIntCast.

===file===
<?php
declare(strict_types=1);

function foo(int $n): void {
    echo $n;
}

$x = 3.7;
foo($x);

===expect===
InvalidArgument@9:4-9:6: Argument $n of foo() expects 'int', got '3.7'
