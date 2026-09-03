===description===
FP-M (part 1): PHP coerces bool to int for bitwise ops, so bool operands must not
emit InvalidOperand for &, |, ^, ~, <<, >> operators.
===file===
<?php

function bitwiseBool(bool $x, int $y): int {
    return $x & $y;
}

function bitwiseBoolOr(bool $a, bool $b): int {
    return $a | $b;
}

function bitwiseBoolXor(bool $a, bool $b): int {
    return $a ^ $b;
}

function bitwiseTrueConst(int $n): int {
    return true & $n;
}

function bitwiseFalseConst(int $n): int {
    return false | $n;
}
===expect===
