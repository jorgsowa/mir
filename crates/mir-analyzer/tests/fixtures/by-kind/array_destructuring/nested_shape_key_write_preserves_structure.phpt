===description===
A nested literal-key write (`$arr['a']['b'] = $v`) must update just that
one inner property, leaving the rest of the outer shape's structure intact
instead of collapsing the whole outer shape to a generic array.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param array{a: array{b: int}} $arr
 */
function two_levels(array $arr, int $n): void {
    $arr['a']['b'] = $n;
    /** @mir-check $arr is array{a: array{b: int}} */
    echo 1;
}

/**
 * @param array{a: array{b: array{c: int}}} $arr
 */
function three_levels(array $arr, int $n): void {
    $arr['a']['b']['c'] = $n;
    /** @mir-check $arr is array{a: array{b: array{c: int}}} */
    echo 1;
}
===expect===
