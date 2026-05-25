===description===
Callable multiple params with optional
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
PossiblyInvalidArgument
===ignore===
TODO
