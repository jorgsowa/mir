===description===
Same as closure_union_bad_arity_first_branch, but with the ternary branches
swapped so the over-arity closure is the *second* union member. Regression test
for a bug where the arity check only inspected the first closure in a union,
silently missing the second — this must emit the same diagnostic either way.
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
    ? function (string $a): void {}
    : function (string $a, string $b): void {};

process($cb);
===expect===
InvalidArgument@12:8-12:11: Argument $c of process() expects 'callable with 1 required parameter(s)', got 'callable with 2 required parameter(s)'
