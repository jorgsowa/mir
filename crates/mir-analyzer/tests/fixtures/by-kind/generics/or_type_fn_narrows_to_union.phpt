===description===
`is_int($x) || is_string($x)` in an if-condition's true branch must narrow
$x to int|string — the scalar-type-check counterpart of instanceof-OR
narrowing, which only recognized `instanceof` conditions.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param int|string|bool $x
 */
function bar($x): void {
    if (is_int($x) || is_string($x)) {
        /** @mir-check $x is int|string */
        $_ = 1;
    }
}
===expect===
