===description===
Same as closure_union_bad_arity_first_branch, but with the ternary branches
swapped so the over-arity closure is the *second* union member. Regression test
for a bug where the arity check only inspected the first closure in a union,
silently missing the second — this must emit the same diagnostic either way.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingClosureReturnType errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//      ^^^ InvalidArgument: Argument $c of process() expects 'callable with 1 required parameter(s)', got 'callable with 2 required parameter(s)'
