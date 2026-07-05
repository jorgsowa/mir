===description===
G1: a `list<T>` docblock param must bind T from a literal array argument
(which types as a list-shaped TKeyedArray, not TList) and, symmetrically, an
`array<K,V>` param must bind K/V from a genuine list-typed argument.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @template T
 * @param list<T> $items
 * @return T
 */
function first(array $items) {
    return $items[0];
}

$x = first(['a', 'b', 'c']);
/** @mir-check $x is string */

/**
 * @template T
 * @param T $a
 * @param T $b
 * @return list<T>
 */
function make_list($a, $b) {
    return [$a, $b];
}

/**
 * @template K
 * @template V
 * @param array<K, V> $arr
 * @return V
 */
function first_value(array $arr) {
    foreach ($arr as $v) {
        return $v;
    }
    throw new RuntimeException('empty');
}

$list = make_list(1, 2);
$y = first_value($list);
/** @mir-check $y is int */
===expect===
