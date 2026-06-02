===description===
Invalid scalar argument
===file===
<?php
function fooFoo(int $a): void {}
fooFoo("string");
===expect===
InvalidArgument@3:8-3:16: Argument $a of fooFoo() expects 'int', got '"string"'
