===description===
reports too many arguments with optional params
===file===
<?php
function greet(string $name, string $suffix = ''): void {}
//             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
//                           ^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $suffix is never used
greet('Ada', 'Mrs.', 'extra');
//                   ^^^^^^^ TooManyArguments: Too many arguments for greet(): expected 2, got 3
===expect===
