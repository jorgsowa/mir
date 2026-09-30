===description===
reports too many function arguments cross file
===file:Helper.php===
<?php
function greet(string $name): void {}
//             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used
===file:App.php===
<?php
greet('Ada', 'Grace');
//           ^^^^^^^ TooManyArguments: Too many arguments for greet(): expected 1, got 2
===expect===
