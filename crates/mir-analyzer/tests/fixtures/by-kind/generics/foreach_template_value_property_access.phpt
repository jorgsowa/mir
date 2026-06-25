===description===
G1: iterating over an array whose value type is a template param and accessing properties
on the values must not emit MixedPropertyFetch or MixedAssignment.
===config===
suppress=UnusedVariable,MissingPropertyType,MissingReturnType
===file===
<?php
/**
 * @template T
 * @param list<T> $items
 */
function pluck_names(array $items): void {
    foreach ($items as $item) {
        // $item is T — neither MixedAssignment nor MixedPropertyFetch
        $item->name;
    }
}

/**
 * @template K
 * @template V
 * @param array<K, V> $map
 */
function inspect_values(array $map): void {
    foreach ($map as $key => $val) {
        // $val is V — no MixedPropertyFetch on $val->id
        $val->id;
    }
}
===expect===
