===description===
PossiblyNullOperand fires when the divisor in a division might be null.
===file===
<?php
function ratio(int $a, ?int $b): float {
    return $a / $b;
}
===expect===
PossiblyNullOperand@3:11-3:18: Operator '/' operand 'int|null' might be null
