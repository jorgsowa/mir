===description===
G1: foreach over array<K,V> inside a generic function must not emit MixedAssignment for the
value variable — V is a template param (an intentionally parameterised slot), not truly mixed.
===config===
suppress=UnusedVariable,MissingPropertyType,MissingReturnType
===file===
<?php
/**
 * @template K
 * @template V
 * @param array<K, V> $arr
 */
function process_all(array $arr): void {
    foreach ($arr as $k => $v) {
        // $v has type V (template param), not mixed — no MixedAssignment expected
        $_ = $v;
    }
}

/**
 * @template T
 * @param list<T> $items
 */
function process_list(array $items): void {
    foreach ($items as $item) {
        // $item has type T (template param), not mixed
        $_ = $item;
    }
}

/**
 * @template V
 * @param array<string, V> $map
 * @return array<V, string>
 */
function invert(array $map): array {
    $result = [];
    foreach ($map as $k => $v) {
        $result[$v] = $k;
    }
    return $result;
}
===expect===
