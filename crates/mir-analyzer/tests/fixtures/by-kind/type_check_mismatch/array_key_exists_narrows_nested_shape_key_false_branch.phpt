===description===
`!array_key_exists('b', $arr['a'])` proves `$arr['a']` isn't the shape arm
that guarantees `b`'s presence, excluding it from the union — the
false-branch counterpart of `array_key_exists_narrows_nested_shape_key`,
which only proved the true-branch (`$arr['a']['b']` present). Previously the
false branch only handled a plain-variable or single-level-property array
argument, not a nested-path one (`$arr['a']`).
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/** @param array{a: array{b: int}|array{c: string}} $arr */
function f(array $arr): void {
    if (!array_key_exists('b', $arr['a'])) {
        $val = $arr['a'];
        /** @mir-check $val is array{c: string} */
        echo 1;
    }
}
===expect===
