===description===
negative-int and non-negative-int must be accepted where numeric/scalar/float
are declared, just like positive-int and plain int already are.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function returns_negative_int_for_numeric(): float {
    /** @var negative-int $x */
    $x = -5;
    return $x;
}

function returns_nonneg_int_for_numeric(): float {
    /** @var non-negative-int $x */
    $x = 3;
    return $x;
}

function returns_negative_int_for_scalar(): string|int|float|bool {
    /** @var negative-int $x */
    $x = -5;
    return $x;
}
===expect===
