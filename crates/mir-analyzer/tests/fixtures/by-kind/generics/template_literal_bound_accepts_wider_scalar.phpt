===description===
A wider scalar may hold a value of a literal-only template bound; a literal outside it may not
===file===
<?php
/**
 * @template B of 1|2
 * @param B $b
 */
function f(int $b): int { return $b; }

function g(int $i): int {
    f(3);
//  ^^^^ InvalidArgument: Argument $b of f() expects '1|2', got '3'
    return f($i);
}
