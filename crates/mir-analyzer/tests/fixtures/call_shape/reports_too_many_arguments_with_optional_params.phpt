===file===
<?php
function greet(string $name, string $suffix = ''): void {}
greet('Ada', 'Mrs.', 'extra');
===expect===
UnusedParam: Parameter $name is never used
UnusedParam: Parameter $suffix is never used
TooManyArguments: Too many arguments for greet(): expected 2, got 3
