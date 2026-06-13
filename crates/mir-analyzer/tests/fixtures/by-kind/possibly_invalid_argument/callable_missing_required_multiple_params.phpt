===description===
Callable missing required multiple params
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param callable(string, string, string, string):bool $arg
 * @return void
 */
function foo($arg) {}

function bar(string $a, string $b, string $c): bool {}

foo("bar");
===expect===
InvalidReturnType@8:53-8:55: Return type 'void' is not compatible with declared 'bool'
