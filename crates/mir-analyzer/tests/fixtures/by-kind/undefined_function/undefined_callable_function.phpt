===description===
Undefined callable function
===config===
suppress=UnusedParam
===file===
<?php
function foo(callable $c): void {}

foo("trime");
===expect===
UndefinedFunction@4:4-4:11: Function trime() is not defined
