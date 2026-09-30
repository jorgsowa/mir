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
//     ^^ MixedArgument: Argument $a of fooFoo() is mixed
===expect===
