===description===
A closure with one required param and one optional param satisfies callable(string):void
— optional params do not count toward the required-arity comparison, so no error
is emitted.
===config===
suppress=UnusedParam,MissingClosureReturnType
===file===
<?php
/** @param callable(string):void $c */
function process(callable $c): void {
    $c("hello");
}

process(function (string $s, int $extra = 0): void {});
===expect===
