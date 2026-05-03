===description===
reports too many function arguments cross file
===file:Helper.php===
<?php
function greet(string $name): void {}
===file:App.php===
<?php
greet('Ada', 'Grace');
===expect===
App.php: TooManyArguments@2:13: Too many arguments for greet(): expected 1, got 2
Helper.php: UnusedParam@2:15: Parameter $name is never used
===ignore===
TODO
