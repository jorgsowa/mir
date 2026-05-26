===description===
Too many arguments
===file===
<?php
function fooFoo(int $a): void {}
fooFoo(5, "dfd");
===expect===
TooManyArguments
===ignore===
TODO
