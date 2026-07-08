===description===
Keyed destructuring (`['a' => $a] = $arr`) against a shape-typed source
must resolve each target's type from the matching property, not fall back
to mixed.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{a: int, b: string} $arr
 */
function test(array $arr): void {
    ['a' => $a, 'b' => $b] = $arr;
    /** @mir-check $a is int */
    echo 1;
    /** @mir-check $b is string */
    echo 2;
}
===expect===
