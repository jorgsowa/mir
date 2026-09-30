===description===
reports too few function arguments
===file===
<?php
function takes_two(int $a, string $b): void {}
//                 ^^^^^^ UnusedParam: Parameter $a is never used
//                         ^^^^^^^^^ UnusedParam: Parameter $b is never used
takes_two(1);
//<^^^^^^^^^^^^ TooFewArguments: Too few arguments for takes_two(): expected 2, got 1
===expect===
