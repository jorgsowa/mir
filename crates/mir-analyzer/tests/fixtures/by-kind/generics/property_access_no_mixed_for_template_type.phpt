===description===
G1: property access on template-typed variables must not emit MixedPropertyFetch —
template params (both unconstrained and bounded) are intentionally parameterised, not mixed.
===config===
suppress=UnusedVariable,MissingPropertyType,MissingReturnType
===file===
<?php
/**
 * @template T
 * @param T $a
 */
function unconstrained_prop($a): void {
    $a->name;     // T — must not fire MixedPropertyFetch
    $a->count;    // T — must not fire MixedPropertyFetch
}

/**
 * @template T of object
 * @param T $obj
 */
function bounded_prop($obj): void {
    $obj->id;     // T of object — must not fire MixedPropertyFetch
}

/**
 * @template K
 * @template V
 * @param array<K, V> $arr
 */
function template_value_prop(array $arr): void {
    foreach ($arr as $k => $v) {
        $v->data; // V is a template param — must not fire MixedPropertyFetch
    }
}
===expect===
