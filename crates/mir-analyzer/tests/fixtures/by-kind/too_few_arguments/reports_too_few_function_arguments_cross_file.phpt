===description===
reports too few function arguments cross file
===file:Helper.php===
<?php
function greet(string $name, string $suffix): void {}
===file:App.php===
<?php
greet('Ada');
===expect===
App.php: TooFewArguments@2:1-2:13: Too few arguments for greet(): expected 2, got 1
Helper.php: UnusedParam@2:16-2:28: Parameter $name is never used
Helper.php: UnusedParam@2:30-2:44: Parameter $suffix is never used
