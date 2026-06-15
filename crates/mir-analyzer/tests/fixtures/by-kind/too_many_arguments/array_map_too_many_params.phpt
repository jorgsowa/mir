===description===
array_map callback accepts too few arguments for arrays passed
===file===
<?php

function foo(int $a) : void {}

// 2 arrays but foo only accepts 1 parameter
array_map("foo", [1, 2, 3], [4, 5, 6]);

===expect===
UnusedParam@3:13-3:19: Parameter $a is never used
TooManyArguments@6:10-6:15: Too many arguments for foo(): expected 1, got 2
