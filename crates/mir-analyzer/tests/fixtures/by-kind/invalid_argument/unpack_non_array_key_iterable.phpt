===description===
Unpack non array key iterable
===file===
<?php
/** @suppress UnusedParam */
function foo(string ...$args): void {}

/** @var Iterator<float, string> */
$test = null;
foo(...$test);

===expect===
MixedArgument@7:5-7:13: Argument $args of foo() is mixed
