===file===
<?php
function takes_two(int $a, string $b): void {}
takes_two(1);
===expect===
UnusedParam: Parameter $a is never used
UnusedParam: Parameter $b is never used
TooFewArguments: Too few arguments for takes_two(): expected 2, got 1
