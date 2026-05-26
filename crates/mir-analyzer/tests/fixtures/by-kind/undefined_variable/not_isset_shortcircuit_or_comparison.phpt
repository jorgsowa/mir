===description===
!isset short-circuit with || — numeric comparison on RHS
!isset($x) || $x < $y: no UndefinedVariable on $x in the comparison operand
===file===
<?php
/** @param int|null $min */
function test(?int $min, int $value): bool {
    return !isset($min) || $value < $min;
}
===expect===
