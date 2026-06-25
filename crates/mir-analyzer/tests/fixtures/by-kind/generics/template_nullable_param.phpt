===description===
G1: template combined with null — T|null param and return type correctly propagates
the concrete type without emitting MixedAssignment.
===config===
suppress=UnusedVariable,MissingReturnType
===file===
<?php
/**
 * @template T
 * @param T|null $x
 * @return T|null
 */
function maybe($x) { return $x; }

$r = maybe("hello");
/** @mir-check $r is string|null */

$r2 = maybe(null);
/** @mir-check $r2 is null */
===expect===
