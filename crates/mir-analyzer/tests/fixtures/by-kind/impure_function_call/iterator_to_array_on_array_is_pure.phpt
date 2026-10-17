===description===
iterator_to_array() on an array runs no iterator code, so it is pure there
(webmozart/assert's @psalm-pure `isAnyOf()` converts its
`array<class-string>` argument). A Traversable or iterable argument can
still run arbitrary `current()`/`next()` code and stays flagged.
===file===
<?php
/**
 * @pure
 * @param array<string> $classes
 */
function joinArray(array $classes): string {
    return implode(', ', iterator_to_array($classes));
}

/**
 * @pure
 * @param Traversable<string> $classes
 */
function countTraversable(Traversable $classes): int {
    return count(iterator_to_array($classes));
//               ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function iterator_to_array() in a @pure function
}

/**
 * @pure
 * @param iterable<string> $classes
 */
function countIterable(iterable $classes): int {
    return count(iterator_to_array($classes));
//               ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function iterator_to_array() in a @pure function
}
