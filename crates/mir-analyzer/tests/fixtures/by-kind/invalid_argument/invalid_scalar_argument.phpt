===description===
Invalid scalar argument
===config===
suppress=UnusedParam
===file===
<?php
function fooFoo(int $a): void {}
fooFoo("string");
===expect===
InvalidArgument@3:7-3:15: Argument $a of fooFoo() expects 'int', got '"string"'
