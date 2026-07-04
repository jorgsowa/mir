===description===
A `callable(TValue): bool` docblock using an unsubstituted template parameter
(the common Laravel-collection style) must not be compared structurally against
a concrete closure signature — TValue is a placeholder, not a real type, so
flagging a mismatch here would be a false positive.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template TValue
 * @param TValue $item
 * @param callable(TValue): bool $callback
 */
function filterOne($item, callable $callback): bool {
    return $callback($item);
}

filterOne(42, function (int $x): bool { return $x > 0; });
===expect===
