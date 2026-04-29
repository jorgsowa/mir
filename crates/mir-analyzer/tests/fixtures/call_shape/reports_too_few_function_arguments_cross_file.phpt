===file:Helper.php===
<?php
function greet(string $name, string $suffix): void {}
===file:App.php===
<?php
greet('Ada');
===expect===
Helper.php: UnusedParam: Parameter $name is never used
Helper.php: UnusedParam: Parameter $suffix is never used
App.php: TooFewArguments: Too few arguments for greet(): expected 2, got 1
