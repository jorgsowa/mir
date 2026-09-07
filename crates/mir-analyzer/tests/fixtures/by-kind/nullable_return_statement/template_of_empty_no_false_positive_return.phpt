===description===
A @template bound on the falsy pseudo-type `empty` (whose expansion includes
null) must not trigger a false NullableReturnStatement on a trivial identity
function — the bare template atom itself is never TNull even though its
bound's expansion is.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template T of empty
 * @param T $x
 * @return T
 */
function identity($x) {
    return $x;
}
===expect===
