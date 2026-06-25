===description===
G1: identity-style functions return the same type as their template argument.
@mir-check verifies the inferred return type at each call site.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @template T
 * @param T $x
 * @return T
 */
function identity($x) { return $x; }

$s = identity("hello");
/** @mir-check $s is string */

$i = identity(42);
/** @mir-check $i is 42 */

$b = identity(true);
/** @mir-check $b is true */

/** @var array<string, int> $arr */
$arr = ['a' => 1];
$a = identity($arr);
/** @mir-check $a is array<string, int> */
===expect===
