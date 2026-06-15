===description===
reports too few function arguments
===file===
<?php
function takes_two(int $a, string $b): void {}
takes_two(1);
===expect===
UnusedParam@2:19-2:25: Parameter $a is never used
UnusedParam@2:27-2:36: Parameter $b is never used
TooFewArguments@3:0-3:12: Too few arguments for takes_two(): expected 2, got 1
