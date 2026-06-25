===description===
G1: a bounded template param (T of Countable) must not trigger MixedAssignment when
the function iterates over or assigns from a template-typed parameter.
===config===
suppress=UnusedVariable,MissingReturnType,MixedMethodCall
===file===
<?php
/**
 * @template T of Countable
 * @param T $collection
 * @return T
 */
function first_of(Countable $collection) {
    $local = $collection;  // type T of Countable — not truly mixed
    return $local;
}

/**
 * @template T
 * @param array<T> $items
 * @return T|null
 */
function first_item(array $items) {
    foreach ($items as $item) {
        // $item is T — must not emit MixedAssignment
        $copy = $item;
        return $copy;
    }
    return null;
}
===expect===
