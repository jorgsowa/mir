===description===
A `@template TValue` whose name happens to collide with an unrelated, real
class in the same file must still be treated as a template within this
function's own scope, not compared against the coincidentally-named class.
A plain "does this class exist" check alone is fooled by the collision.
===file===
<?php
class TValue {}

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
