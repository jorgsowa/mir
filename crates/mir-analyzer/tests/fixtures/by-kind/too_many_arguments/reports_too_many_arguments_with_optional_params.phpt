===description===
reports too many arguments with optional params
===file===
<?php
function greet(string $name, string $suffix = ''): void {}
greet('Ada', 'Mrs.', 'extra');
===expect===
UnusedParam@2:16-2:28: Parameter $name is never used
UnusedParam@2:30-2:49: Parameter $suffix is never used
TooManyArguments@3:22-3:29: Too many arguments for greet(): expected 2, got 3
