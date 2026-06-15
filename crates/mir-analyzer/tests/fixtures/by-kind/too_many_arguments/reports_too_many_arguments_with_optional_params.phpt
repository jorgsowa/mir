===description===
reports too many arguments with optional params
===file===
<?php
function greet(string $name, string $suffix = ''): void {}
greet('Ada', 'Mrs.', 'extra');
===expect===
UnusedParam@2:15-2:27: Parameter $name is never used
UnusedParam@2:29-2:48: Parameter $suffix is never used
TooManyArguments@3:21-3:28: Too many arguments for greet(): expected 2, got 3
