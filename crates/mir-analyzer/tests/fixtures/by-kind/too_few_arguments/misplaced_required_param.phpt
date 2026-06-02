===description===
Misplaced required param
===file===
<?php
function foo(string $bar = null, int $bat): void {}
foo();
===expect===
TooFewArguments@3:1-3:6: Too few arguments for foo(): expected 1, got 0
