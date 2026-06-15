===description===
Mixed argument
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
/** @var mixed */
$a = "hello";
fooFoo($a);
===expect===
MixedArgument@5:7-5:9: Argument $a of fooFoo() is mixed
