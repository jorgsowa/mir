===description===
PossiblyNullOperand fires when the divisor in a division might be null.
===file===
<?php
function ratio(int $a, ?int $b): float {
    return $a / $b;
//         ^^^^^^^ PossiblyNullOperand: Operator '/' operand 'int|null' might be null
}
===expect===
