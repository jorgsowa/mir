===description===
Null argument
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(null);
===expect===
NullArgument
===ignore===
TODO
