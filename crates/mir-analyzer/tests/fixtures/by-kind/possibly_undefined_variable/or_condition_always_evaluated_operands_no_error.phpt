===description===
When `A || B` is false every operand of B that PHP always evaluates ran: a ternary
condition, a cast operand, the first `isset` operand and array literal elements.
Conditional parts (ternary branches, later `isset` operands) are not promoted.
===file===
<?php
function ternaryCondition(bool $f): int {
    if ($f || (($x = rand()) ? true : false)) {
        return 0;
    }
    /** @mir-check $x is int */
    return $x;
}

function castOperand(bool $f): int {
    if ($f || (bool) ($y = rand())) {
        return 0;
    }
    return $y;
}

function issetFirstOperand(bool $f, array $arr): int {
    if ($f || isset($arr[$z = 1])) {
        return 0;
    }
    return $z;
}

function arrayLiteralElement(bool $f): int {
    if ($f || [$q = rand()] === []) {
        return 0;
    }
    return $q;
}

function ternaryBranchNotPromoted(bool $f, bool $g): int {
    if ($f || ($g ? ($b = rand()) : false)) {
        return 0;
    }
    return $b;
}

function laterIssetOperandNotPromoted(bool $f, array $arr): int {
    if ($f || isset($arr[0], $arr[$l = 1])) {
        return 0;
    }
    return $l;
}
===expect===
PossiblyUndefinedVariable@35:11-35:13: Variable $b might not be defined
PossiblyUndefinedVariable@42:11-42:13: Variable $l might not be defined
