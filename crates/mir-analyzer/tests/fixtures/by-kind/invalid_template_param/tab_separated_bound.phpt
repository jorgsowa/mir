===description===
A tab-separated `@template T of Bound` line binds the bound instead of dropping it
===file===
<?php
/**
 * @template	T	of	ArrayAccess
 * @param T $x
 */
function f($x): void {}
//         ^^ UnusedParam: Parameter $x is never used

f(5);
//<^^^^ InvalidArgument: Argument $x of f() expects 'ArrayAccess', got '5'
