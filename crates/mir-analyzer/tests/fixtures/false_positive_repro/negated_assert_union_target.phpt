===description===
A negated assertion whose target is a union of 2+ types (`@psalm-assert
!A|B $x`) was a total no-op — `negate_assertion_type` bailed out entirely
whenever the asserted type had more than one atom, instead of subtracting
each atom in turn the way a single-atom negated target already does.
===config===
suppress=UnusedParam
===file===
<?php
class A {}
class B {}

/**
 * @param mixed $v
 * @psalm-assert !A|B $v
 */
function assertNotAOrB($v): void {
    if ($v instanceof A || $v instanceof B) {
        throw new \InvalidArgumentException("must not be A or B");
    }
}

/** @param A|B|int $v */
function test($v): void {
    assertNotAOrB($v);
    /** @mir-check $v is int */
    echo "ok";
}
===expect===
