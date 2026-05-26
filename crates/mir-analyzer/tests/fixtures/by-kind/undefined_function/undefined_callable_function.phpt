===description===
Undefined callable function
===file===
<?php
function foo(callable $c): void {}

foo("trime");
===expect===
UndefinedFunction
===ignore===
TODO
