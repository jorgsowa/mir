===description===
Callable multiple params with optional
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param callable(string, string, string=):bool $arg
 * @return void
 */
function foo($arg) {}

function bar(string $a, string $b, string $c): bool {}

foo("bar");
===expect===
InvalidReturnType@8:52-8:54: Return type 'void' is not compatible with declared 'bool'
InvalidArgument@10:4-10:9: Argument $arg of foo() expects 'callable with 2 required parameter(s)', got 'callable with 3 required parameter(s)'
