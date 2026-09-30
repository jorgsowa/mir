===description===
Misplaced required param
===config===
suppress=UnusedParam
===file===
<?php
function foo(string $bar = null, int $bat): void {}
foo();
//<^^^^^ TooFewArguments: Too few arguments for foo(): expected 1, got 0
===expect===
