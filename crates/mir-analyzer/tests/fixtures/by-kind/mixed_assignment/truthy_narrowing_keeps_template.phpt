===description===
FALSE POSITIVE: narrowing a bare @template-typed variable through a
truthiness check (`if ($x)`) must preserve the template, not widen it to
real `mixed`. Type::narrow_to_truthy/narrow_to_falsy bailed out via
`is_mixed()`, which (like the other instances of this bug class) also
returns true for an unconstrained template — discarding the `T` identity
and replacing it with a literal `TMixed` atom, which then trips a real
MixedAssignment since `is_mixed_not_template()` no longer sees any
template atom to protect.
===config===
suppress=MissingReturnType
===file===
<?php
/**
 * @template T
 * @param T $x
 * @return T
 */
function f($x) {
    if ($x) {
        $y = $x;
        /** @mir-check $y is T */
        return $y;
    }
    return $x;
}
===expect===
