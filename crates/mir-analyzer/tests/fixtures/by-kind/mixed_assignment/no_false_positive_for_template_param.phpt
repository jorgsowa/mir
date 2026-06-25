===description===
FP: MixedAssignment must not fire for template-param-typed variables — T is an
intentionally parameterised slot, not a lost-type-information mixed.
===config===
suppress=UnusedVariable,MissingReturnType,MissingPropertyType
===file===
<?php
/**
 * @template T
 * @param T $x
 */
function accept_template($x): void {
    $copy = $x;  // no MixedAssignment
}

/**
 * @template K
 * @template V
 * @param array<K, V> $arr
 */
function iterate_template(array $arr): void {
    foreach ($arr as $k => $v) {
        $vv = $v;  // no MixedAssignment even though V has no declared bound
    }
}

/**
 * @template T of object
 * @param list<T> $items
 */
function copy_list(array $items): void {
    foreach ($items as $item) {
        $local = $item;  // no MixedAssignment; T has a bound but is still a param
    }
}
===expect===
