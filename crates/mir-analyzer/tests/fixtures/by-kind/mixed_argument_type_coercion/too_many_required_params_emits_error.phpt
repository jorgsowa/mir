===description===
A closure with more required parameters than the typed callable param expects
must emit InvalidArgument. The caller will pass only the declared number of
arguments, so a callable demanding more is incompatible.
===config===
suppress=UnusedParam,MissingClosureReturnType
===file===
<?php
/** @param callable(string):void $c */
function process(callable $c): void {
    $c("hello");
}

process(function (string $a, string $b): void {});
===expect===
InvalidArgument@7:8-7:48: Argument $callback of typed_callable() expects 'callable with 1 required parameter(s)', got 'callable with 2 required parameter(s)'
