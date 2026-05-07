===description===
reports too many function arguments
===file===
<?php
function takes_one(int $a): void {}
takes_one(1, 2);
===expect===
UnusedParam@2:19: Parameter $a is never used
TooManyArguments@3:13: Too many arguments for takes_one(): expected 1, got 2
