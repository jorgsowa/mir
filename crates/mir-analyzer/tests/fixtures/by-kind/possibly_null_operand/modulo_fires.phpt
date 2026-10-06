===description===
PossiblyNullOperand fires for the modulo operator when the divisor might be null.
===file===
<?php
function remainder(int $a, ?int $b): int {
    return $a % $b;
//         ^^^^^^^ PossiblyNullOperand: Operator '%' operand 'int|null' might be null
}
