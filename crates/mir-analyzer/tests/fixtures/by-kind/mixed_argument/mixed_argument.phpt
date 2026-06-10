===description===
Mixed argument
===file===
<?php
function fooFoo(int $a): void {}
/** @var mixed */
$a = "hello";
fooFoo($a);
===expect===
MixedArgument@5:8-5:10: Argument $a of fooFoo() is mixed
