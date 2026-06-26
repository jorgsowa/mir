===description===
PossiblyInvalidOperand does NOT fire for int|float; both types support arithmetic.
===file===
<?php
function compute(int|float $x, int $y): float {
    return $x * $y;
}
===expect===
