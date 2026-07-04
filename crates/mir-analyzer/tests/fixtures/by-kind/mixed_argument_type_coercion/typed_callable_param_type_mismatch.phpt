===description===
A closure passed to a `callable(string):void` typed param must itself accept a
string. Previously `check_typed_callable_arg` only compared parameter *counts*,
so a closure declaring an incompatible parameter type (`int`) passed silently
even though calling it with the promised string would throw at runtime.
===config===
suppress=UnusedParam,MissingClosureReturnType
===file===
<?php
/** @param callable(string):void $c */
function process(callable $c): void {
    $c("hello");
}

process(function (int $a): void {});
===expect===
InvalidArgument@7:8-7:34: Argument $callback of typed_callable() expects 'callable whose parameter #1 accepts 'string'', got 'callable whose parameter #1 only accepts 'int''
