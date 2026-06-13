===description===
ImpureFunctionCall does NOT fire when a @pure function calls another @pure function.
===file===
<?php
/** @pure */
function double(int $n): int {
    return $n * 2;
}

/** @pure */
function quadruple(int $n): int {
    return double(double($n));
}

===expect===
