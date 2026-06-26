===description===
PossiblyInvalidOperand does NOT fire for nullable int; PHP coerces null to 0 in arithmetic.
===file===
<?php
function add(?int $x, int $y): int {
    return $x + $y;
}
===expect===
