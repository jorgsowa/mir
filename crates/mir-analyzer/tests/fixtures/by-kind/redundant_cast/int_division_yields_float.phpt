===description===
`int / int` yields `int|float` in PHP (division may not be exact). The (int) cast
on a division result must not be flagged as RedundantCast, and passing the division
result to a float parameter must not emit InvalidArgument. `/=` must also yield
`int|float`, not `int`. Exact literal division (6 / 2 = 3) still folds to int.
===config===
php_version=8.1
suppress=UnusedParam,UnusedVariable
===file===
<?php
declare(strict_types=1);

function takes_float(float $f): void {}

// Non-exact literal division folds to float; `(int)` cast is valid, not redundant.
$a = 5 / 2;
/** @mir-check $a is float */
$a;

// int/int via variables — result is int|float; passing to float param is valid.
function ratio(int $x, int $y): void {
    takes_float($x / $y);
}

// (int) cast of int/int is not a redundant cast.
function midpoint(int $lo, int $hi): int {
    return (int)(($lo + $hi) / 2);
}

// /= on an int variable also yields int|float.
function apply_scale(int $n): void {
    $n /= 3;
    takes_float($n);
}
===expect===
