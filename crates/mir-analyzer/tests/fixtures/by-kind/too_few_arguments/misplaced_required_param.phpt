===description===
Misplaced required param
===file===
<?php
function foo(string $bar = null, int $bat): void {}
foo();
===expect===
TooFewArguments
===ignore===
TODO
