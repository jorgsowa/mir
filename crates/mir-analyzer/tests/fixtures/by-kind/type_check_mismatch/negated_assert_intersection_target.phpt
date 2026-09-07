===description===
`negate_assertion_type` had no arm for a `TIntersection` target — a
`@psalm-assert !(A&B) $v` on a `(A&B)|C` value fell to the catch-all and
left the type unchanged instead of excluding the intersection-satisfying
atom and narrowing to `C`.
===config===
suppress=UnusedParam
===file===
<?php
interface A {}
interface B {}
class C {}

/**
 * @param mixed $v
 * @psalm-assert !(A&B) $v
 */
function assertNotBoth($v): void {}

/** @param (A&B)|C $v */
function test($v): void {
    assertNotBoth($v);
    /** @mir-check $v is C */
    echo "ok";
}
===expect===
