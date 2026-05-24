===description===
undefinedCallableFunction
===file===
<?php
function foo(callable $c): void {}

foo("trime");
===expect===
UndefinedFunction
===ignore===
TODO
