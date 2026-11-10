===description===
A property read through a union receiver has the union of each class's
declared type, so a type check on it isn't `RedundantCondition`.
===file===
<?php
final class StrBox { public string $v = ''; }
final class IntBox { public int $v = 0; }

function pick(StrBox|IntBox $b): string {
    /** @mir-check $b->v is string|int */
    $v = $b->v;
    if (is_string($b->v)) {
        return $b->v;
    }
    return (string) $v;
}

function sameType(StrBox|IntBox $b): void {
    if (is_object($b->v)) {
//      ^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo 'never';
    }
}
