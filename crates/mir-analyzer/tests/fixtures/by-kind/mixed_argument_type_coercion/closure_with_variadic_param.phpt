===description===
A closure with a variadic parameter satisfies callable(string, int):void — the
variadic flag prevents it from being counted as an extra required parameter, so
no InvalidArgument is emitted.
===config===
suppress=UnusedParam,MissingClosureReturnType
===file===
<?php
/** @param callable(string, int):void $c */
function dispatch(callable $c): void {
    $c("hello", 42);
}

dispatch(function (string $a, int ...$rest): void {});
===expect===
