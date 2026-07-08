===description===
FN: a @param docblock that partially conflicts with the native hint (shares
one family, contradicts on another — `int|string` vs native `int`) made
mir accept the WHOLE raw docblock union as $x's body type, silently
widening $x to include the foreign `string` atom. That hid a real
RedundantCast: `(int) $x` on a native `int $x` is redundant regardless of
what the docblock (wrongly) also claims.
===config===
suppress=UnusedVariable,MismatchingDocblockParamType
===file===
<?php
/**
 * @param int|string $x
 */
function f(int $x): void {
    $y = (int) $x;
}
===expect===
RedundantCast@6:15-6:17: Casting 'int' to 'int' is redundant
