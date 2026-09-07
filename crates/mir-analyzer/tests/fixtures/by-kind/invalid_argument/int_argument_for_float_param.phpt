===description===
FP-M (part 2): PHP implicitly coerces int to float. Passing an int where float is
expected must not emit InvalidArgument.
===file===
<?php

function takesFloat(float $x): float {
    return $x * 2.0;
}

$_ = takesFloat(42);
$_ = takesFloat(-1);

function roundTrip(float $x): void {
    echo $x;
}
roundTrip(0);
roundTrip(100);
===expect===
