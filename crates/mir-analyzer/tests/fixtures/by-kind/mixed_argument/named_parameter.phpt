===description===
MixedArgument fires for named arguments too when the value is mixed.
===config===
suppress=UnusedParam
===file===
<?php
function foo(int $a, string $b): void {}
/** @var mixed $x */
$x = null;
foo(a: $x, b: "hello");

===expect===
MixedArgument@5:4-5:9: Argument $a of foo() is mixed
