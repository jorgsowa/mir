===description===
In non-strict mode, PHP coerces float to int at call sites without a TypeError.
mir must not emit InvalidArgument (Error) here — only ImplicitFloatToIntCast (Warning)
is the appropriate diagnostic. Previously both fired, making the Error a false positive.

===file===
<?php
function process(int $id): void {
    echo $id;
}

$score = 9.8;
process($score);
//      ^^^^^^ ImplicitFloatToIntCast: Implicit cast from 9.8 to int truncates the fractional part

process(7.3);
//      ^^^ ImplicitFloatToIntCast: Implicit cast from 7.3 to int truncates the fractional part
