===description===
Null argument
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(null);
===expect===
NullArgument@3:8-3:12: Argument $a of fooFoo() cannot be null
