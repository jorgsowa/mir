===description===
PossiblyNullOperand fires for the modulo operator when the divisor might be null.
===file===
<?php
function remainder(int $a, ?int $b): int {
    return $a % $b;
}
===expect===
PossiblyNullOperand@3:11-3:18: Operator '%' operand 'int|null' might be null
