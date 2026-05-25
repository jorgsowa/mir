===description===
reports duplicate named argument
===file===
<?php
function greet(string $name): void {}
greet(name: 'Ada', name: 'Grace');
===expect===
UnusedParam@2:16: Parameter $name is never used
InvalidNamedArgument@3:20: greet() has no parameter named $name
