===description===
PossiblyNullOperand does NOT fire when the operand cannot be null.
===file===
<?php
function ratio(int $a, int $b): float {
    return $a / $b;
}
===expect===
