===description===
Invalid scalar argument
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
fooFoo("string");
//     ^^^^^^^^ InvalidArgument: Argument $a of fooFoo() expects 'int', got '"string"'
===expect===
