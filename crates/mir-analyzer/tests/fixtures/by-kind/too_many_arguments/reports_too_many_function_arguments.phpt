===description===
reports too many function arguments
===file===
<?php
function takes_one(int $a): void {}
//                 ^^^^^^ UnusedParam: Parameter $a is never used
takes_one(1, 2);
//           ^ TooManyArguments: Too many arguments for takes_one(): expected 1, got 2
===expect===
