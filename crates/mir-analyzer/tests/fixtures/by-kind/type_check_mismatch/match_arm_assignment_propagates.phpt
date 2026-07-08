===description===
A variable assigned inside every match arm's body is defined afterward —
exactly one arm runs (or PHP throws), so the assignment is real and
permanent, not confined to a discarded branch context
===config===
suppress=UnusedVariable
===file===
<?php
function f(int $x): void {
    $y = match ($x) {
        1 => $z = "one",
        2 => $z = "two",
        default => $z = "other",
    };
    /** @mir-check $z is string */
    echo $z;
}
===expect===
