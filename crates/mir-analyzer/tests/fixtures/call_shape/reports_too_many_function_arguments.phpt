===file===
<?php
function takes_one(int $a): void {}
takes_one(1, 2);
===expect===
UnusedParam: Parameter $a is never used
TooManyArguments: Too many arguments for takes_one(): expected 1, got 2
