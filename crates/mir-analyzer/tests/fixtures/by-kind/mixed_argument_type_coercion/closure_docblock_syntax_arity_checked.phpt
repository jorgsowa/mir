===description===
`@param Closure(string):void $c` must be arity-checked exactly like the
equivalent `@param callable(string):void $c` — previously the dispatch that
wires up the typed-callable check only matched `Atomic::TCallable`, so the
`Closure(...)` docblock spelling silently skipped arity checking entirely.
===config===
suppress=UnusedParam,MissingClosureReturnType
===file===
<?php
/** @param Closure(string):void $c */
function process(Closure $c): void {
    $c("hello");
}

process(function (string $a, string $b): void {});
===expect===
InvalidArgument@7:8-7:48: Argument $c of process() expects 'callable with 1 required parameter(s)', got 'callable with 2 required parameter(s)'
