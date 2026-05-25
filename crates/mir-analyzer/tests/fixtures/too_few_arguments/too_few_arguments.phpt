===description===
Too few arguments
===file===
<?php
function fooFoo(int $a): void {}
fooFoo();
===expect===
TooFewArguments
===ignore===
TODO
