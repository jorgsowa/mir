===description===
reports duplicate named argument
===file===
<?php
function greet(string $name): void {}
greet(name: 'Ada', name: 'Grace');
===expect===
UnusedParam@2:15: Parameter $name is never used
InvalidNamedArgument@3:19: greet() has no parameter named $name
