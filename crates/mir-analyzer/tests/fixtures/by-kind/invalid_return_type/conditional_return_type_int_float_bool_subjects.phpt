===description===
`resolve_conditional_branch` only recognized `null`/`true`/`false`/`string`/
`array`/`list` subjects — an `int`, `float`, or `bool` discriminant (e.g.
`($value is int ? A : B)`) always fell through to `_ => return None`, so the
conditional could never resolve at a call site and silently widened to the
union of both branches regardless of the actual argument type.
===file===
<?php
/**
 * @param mixed $value
 * @return ($value is int ? string : int)
 */
function intCheck($value) {
    return is_int($value) ? "s" : 1;
}
$a = intCheck(5);
/** @mir-check $a is string */
echo $a;
$b = intCheck("x");
/** @mir-check $b is int */
echo $b;

/**
 * @param mixed $value
 * @return ($value is float ? int : float)
 */
function floatCheck($value) {
    return is_float($value) ? 1 : 1.5;
}
$c = floatCheck(1.5);
/** @mir-check $c is int */
echo $c;
$d = floatCheck(1);
/** @mir-check $d is float */
echo $d;

/**
 * @param mixed $value
 * @return ($value is bool ? int : bool)
 */
function boolCheck($value) {
    return is_bool($value) ? 1 : false;
}
$e = boolCheck(true);
/** @mir-check $e is int */
echo $e;
$f = boolCheck(1);
/** @mir-check $f is bool */
echo $f;
===expect===
