===description===
A union of closures with different arities (e.g. from a ternary) must have every
branch checked against a typed callable param, not just the first — here the
over-arity closure is the first ternary branch.
===config===
suppress=UnusedParam,MissingClosureReturnType,MixedAssignment
===file===
<?php
/** @param callable(string):void $c */
function process(callable $c): void {
    $c("hello");
}

$flag = true;
$cb = $flag
    ? function (string $a, string $b): void {}
    : function (string $a): void {};

process($cb);
===expect===
InvalidArgument@12:8-12:11: Argument $callback of typed_callable() expects 'callable with 1 required parameter(s)', got 'callable with 2 required parameter(s)'
