===description===
Null argument
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(null);
===expect===
NullArgument@3:7-3:11: Argument $a of fooFoo() cannot be null
