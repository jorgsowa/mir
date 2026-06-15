===description===
Misplaced required param
===config===
suppress=UnusedParam
===file===
<?php
function foo(string $bar = null, int $bat): void {}
foo();
===expect===
TooFewArguments@3:0-3:5: Too few arguments for foo(): expected 1, got 0
