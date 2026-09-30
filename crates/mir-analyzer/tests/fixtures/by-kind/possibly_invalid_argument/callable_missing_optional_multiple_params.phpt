===description===
Callable missing optional multiple params
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param callable(string, string, string, string=):bool $arg
 * @return void
 */
function foo($arg) {}

function bar(string $a, string $b, string $c): bool {}
//                                                  ^^ InvalidReturnType: Return type 'void' is not compatible with declared 'bool'

foo("bar");
===expect===
