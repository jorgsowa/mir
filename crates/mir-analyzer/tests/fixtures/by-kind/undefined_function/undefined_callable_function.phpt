===description===
Undefined callable function
===file===
<?php
function foo(callable $c): void {}

foo("trime");
===expect===
UndefinedFunction@4:5-4:12: Function trime() is not defined
