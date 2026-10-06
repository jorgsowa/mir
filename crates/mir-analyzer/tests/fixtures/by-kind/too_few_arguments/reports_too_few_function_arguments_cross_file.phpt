===description===
reports too few function arguments cross file
===file:Helper.php===
<?php
function greet(string $name, string $suffix): void {}
//             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
//                           ^^^^^^^^^^^^^^ UnusedParam: Parameter $suffix is never used
===file:App.php===
<?php
greet('Ada');
//<^^^^^^^^^^^^ TooFewArguments: Too few arguments for greet(): expected 2, got 1
