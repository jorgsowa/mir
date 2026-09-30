===description===
Undefined callable function
===config===
suppress=UnusedParam
===file===
<?php
function foo(callable $c): void {}

foo("trime");
//  ^^^^^^^ UndefinedFunction: Function trime() is not defined
===expect===
