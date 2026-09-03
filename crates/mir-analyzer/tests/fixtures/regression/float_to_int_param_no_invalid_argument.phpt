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

process(7.3);

===expect===
ImplicitFloatToIntCast@7:8-7:14: Implicit cast from 9.8 to int truncates the fractional part
ImplicitFloatToIntCast@9:8-9:11: Implicit cast from 7.3 to int truncates the fractional part
