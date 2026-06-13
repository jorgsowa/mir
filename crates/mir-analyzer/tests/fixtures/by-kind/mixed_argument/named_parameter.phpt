===description===
MixedArgument fires for named arguments too when the value is mixed.
===file===
<?php
function foo(int $a, string $b): void {}
/** @var mixed $x */
$x = null;
foo(a: $x, b: "hello");

===expect===
MixedArgument@5:5-5:10: Argument $a of foo() is mixed
