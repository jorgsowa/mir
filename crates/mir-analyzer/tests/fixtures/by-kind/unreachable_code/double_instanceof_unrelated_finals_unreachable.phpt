===description===
`$x instanceof A && $x instanceof B` for two classes that can never both be
true (unrelated finals, or unrelated concrete classes under single
inheritance) must be flagged as an unreachable/redundant condition.
narrow_instanceof_preserving_subtypes's final fallback reset an empty
result (every atom proven incompatible) back to a bare `narrowed_ty`,
indistinguishable from "nothing was known yet" — masking the existing
RedundantCondition/divergence machinery. Also covers the companion gap the
fix would otherwise expose: a `Closure(...): R`-typed value (its own
TClosure atomic, not a TNamedObject) is a real `Closure` instance and must
survive `instanceof Closure` narrowing instead of being dropped as
"incompatible".
===file===
<?php

final class Cat {}
final class Dog {}

function bothFinals(Cat|Dog $animal): void {
    if ($animal instanceof Cat && $animal instanceof Dog) {
        echo "unreachable";
    }
}

class PdoLike {}

/**
 * @param PdoLike|Closure(): PdoLike $conn
 */
function connect(PdoLike|Closure $conn): PdoLike {
    if ($conn instanceof Closure) {
        return $conn();
    }
    return $conn;
}
===expect===
RedundantCondition@7:8-7:56: Condition is always true/false for type 'bool'
