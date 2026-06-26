===description===
PossiblyNullOperand does NOT fire for addition; PHP coerces null to 0 in additive expressions.
===file===
<?php
function add(int $a, ?int $b): int {
    return $a + $b;
}
===expect===
