===description===
reports too few function arguments cross file
===file:Helper.php===
<?php
function greet(string $name, string $suffix): void {}
===file:App.php===
<?php
greet('Ada');
===expect===
App.php: TooFewArguments@2:1: Too few arguments for greet(): expected 2, got 1
Helper.php: UnusedParam@2:16: Parameter $name is never used
Helper.php: UnusedParam@2:30: Parameter $suffix is never used
