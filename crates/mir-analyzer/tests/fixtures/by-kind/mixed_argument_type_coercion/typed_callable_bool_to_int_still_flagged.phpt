===description===
`bool -> int` between a typed-callable's documented parameter and the actual
closure's declared parameter is *not* given the same leniency as `float <->
int` — this matches how a direct `bool` argument to an `int`-typed parameter
is already treated as a hard `InvalidArgument` elsewhere in the analyzer, so
the typed-callable check stays consistent rather than inventing a more
permissive policy just for closures.
===config===
suppress=UnusedParam,MissingClosureReturnType
===file===
<?php
/** @param callable(bool):void $c */
function process(callable $c): void {
    $c(true);
}
process(function (int $a): void {});
===expect===
InvalidArgument@6:8-6:34: Argument $callback of typed_callable() expects 'callable whose parameter #1 accepts 'bool'', got 'callable whose parameter #1 only accepts 'int''
