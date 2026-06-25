===description===
In strict_types=1 mode, a literal float passed to an int parameter is a TypeError.
Should emit InvalidArgument (Error), not ImplicitFloatToIntCast.

===file===
<?php
declare(strict_types=1);

function foo(int $n): void {
    echo $n;
}

foo(3.7);

===expect===
InvalidArgument@8:4-8:7: Argument $n of foo() expects 'int', got '3.7'
