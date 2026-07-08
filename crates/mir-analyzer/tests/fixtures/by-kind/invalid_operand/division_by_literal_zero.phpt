===description===
Dividing or taking the modulo of a value by a literal `0` is an
unconditional runtime DivisionByZeroError — the constant-folder already
detects a zero divisor (to skip folding) but never reported it.
===config===
suppress=UnusedParam
===file===
<?php
function div_by_zero(int $x): float {
    return $x / 0;
}

function mod_by_zero(int $x): int {
    return $x % 0;
}

function div_by_nonzero(int $x): float {
    return $x / 5;
}
===expect===
DivisionByZero@3:11-3:17: Division by zero: right operand of '/' is always 0
DivisionByZero@7:11-7:17: Division by zero: right operand of '%' is always 0
