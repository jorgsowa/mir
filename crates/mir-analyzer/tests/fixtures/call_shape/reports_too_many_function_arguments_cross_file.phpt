===file:Helper.php===
<?php
function greet(string $name): void {}
===file:App.php===
<?php
greet('Ada', 'Grace');
===expect===
Helper.php: UnusedParam: Parameter $name is never used
App.php: TooManyArguments: Too many arguments for greet(): expected 1, got 2
