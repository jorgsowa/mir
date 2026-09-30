===description===
Null argument
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(null);
//     ^^^^ NullArgument: Argument $a of fooFoo() cannot be null
===expect===
