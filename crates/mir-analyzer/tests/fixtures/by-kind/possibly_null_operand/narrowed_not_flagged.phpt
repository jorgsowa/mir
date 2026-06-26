===description===
PossiblyNullOperand does NOT fire after null is excluded by a type narrowing guard.
===file===
<?php
function ratio(int $a, ?int $b): float {
    if ($b === null) {
        return 0.0;
    }
    return $a / $b;
}
===expect===
